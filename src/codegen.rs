use std::io::Write;

use almond::ast::BinaryOperator;
use anyhow::Result;
use iced_x86::code_asm::AsmRegister64;
use iced_x86::Formatter;
use iced_x86::{code_asm::*, Decoder, DecoderOptions, Instruction, NasmFormatter};
use memmap::Mmap;

use crate::flow_graph::{FlowInstruction, SystemFunction};
use crate::{
    environment::{Environment, TypeInfo},
    flow_graph::BasicBlockId,
};

pub type UnaryFunction = extern "win64" fn() -> ();

pub struct CodegenContext {
    pub stack: Vec<CodegenStackEntry>,
}

impl CodegenContext {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn current_stack_register(&self) -> AsmRegister64 {
        const ALL_REGISTERS: [AsmRegister64; 7] = [
            // RAX is reserved for return values
            // RCX is reserved for function argument
            // RDX is reserved for function argument
            rbx,
            // RSP is reserved for stack pointer
            // RBP is reserved for stack frame
            // RSI is reserved for string operations
            // RDI is reserved for string operations
            // R8 is reserved for function argument
            // R9 is reserved for function argument
            r10, r11, r12, r13, r14, r15,
        ];

        let index = self.stack.len();
        *ALL_REGISTERS.get(index).expect("register stack overflow")
    }

    pub fn push(&mut self, entry: CodegenStackEntry) -> AsmRegister64 {
        let register = self.current_stack_register();
        self.stack.push(entry);
        register
    }

    pub fn argument_register(&self, index: usize) -> AsmRegister64 {
        const ALL_REGISTERS: [AsmRegister64; 4] = [rcx, rdx, r8, r9];

        *ALL_REGISTERS
            .get(index)
            .expect("argument register overflow")
    }

    pub(crate) fn pop(&mut self) -> (CodegenStackEntry, AsmRegister64) {
        let entry = self.stack.pop().expect("stack underflow");
        let register = self.current_stack_register();
        (entry, register)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CodegenStackEntry {
    Boolean,
    String,
    Number,
    Null,
}

pub fn print_disassembled_code(bytes: &[u8], rip: u64) -> String {
    const HEXBYTES_COLUMN_BYTE_LENGTH: usize = 10;
    let mut decoder = Decoder::with_ip(64, bytes, rip, DecoderOptions::NONE);

    // Formatters: Masm*, Nasm*, Gas* (AT&T) and Intel* (XED).
    // For fastest code, see `SpecializedFormatter` which is ~3.3x faster. Use it if formatting
    // speed is more important than being able to re-assemble formatted instructions.
    let mut formatter = NasmFormatter::new();

    // Change some options, there are many more
    formatter.options_mut().set_digit_separator("`");
    formatter.options_mut().set_first_operand_char_index(10);

    // String implements FormatterOutput
    let mut output = String::new();

    // Initialize this outside the loop because decode_out() writes to every field
    let mut instruction = Instruction::default();

    // The decoder also implements Iterator/IntoIterator so you could use a for loop:
    //      for instruction in &mut decoder { /* ... */ }
    // or collect():
    //      let instructions: Vec<_> = decoder.into_iter().collect();
    // but can_decode()/decode_out() is a little faster:
    while decoder.can_decode() {
        // There's also a decode() method that returns an instruction but that also
        // means it copies an instruction (40 bytes):
        //     instruction = decoder.decode();
        decoder.decode_out(&mut instruction);

        // Format the instruction ("disassemble" it)
        output.clear();
        formatter.format(&instruction, &mut output);

        // Eg. "00007FFAC46ACDB2 488DAC2400FFFFFF     lea       rbp,[rsp-100h]"
        print!("{:016X} ", instruction.ip());
        let start_index = (instruction.ip() - rip) as usize;
        let instr_bytes = &bytes[start_index..start_index + instruction.len()];
        for b in instr_bytes.iter() {
            print!("{:02X}", b);
        }
        if instr_bytes.len() < HEXBYTES_COLUMN_BYTE_LENGTH {
            for _ in 0..HEXBYTES_COLUMN_BYTE_LENGTH - instr_bytes.len() {
                print!("  ");
            }
        }
        println!(" {}", output);
    }

    output
}

pub fn codegen_trampoline(
    environment: &mut Environment,
    basic_block_id: &BasicBlockId,
    dump_disassembly: bool
) -> Result<(Mmap, UnaryFunction)> {
    let environment_ptr = environment as *mut Environment;
    let mut asm = CodeAssembler::new(64)?;
    let mut callsite = asm.create_label();

    asm.push(rcx)?;
    asm.push(rdx)?;
    asm.push(r8)?;
    asm.push(r9)?;
    asm.mov(rax, basic_block_trampoline as u64)?;
    asm.mov(rcx, environment_ptr as u64)?;
    asm.mov(rdx, basic_block_id.0 as u64)?;
    asm.lea(r8, ptr(callsite))?;

    asm.sub(rsp, 0x28)?;
    asm.set_label(&mut callsite)?;
    asm.call(rax)?;
    asm.add(rsp, 0x28)?;

    asm.pop(r9)?;
    asm.pop(r8)?;
    asm.pop(rdx)?;
    asm.pop(rcx)?;
    asm.jmp(rax)?;
    asm.ret()?;

    let mut mmap = memmap::MmapMut::map_anon(4096)?;
    let rip = mmap.as_ptr() as u64;
    let instructions = asm.assemble(rip)?;
    (&mut mmap[..]).write(&instructions)?;
    let mmap = mmap.make_exec()?;

    if dump_disassembly {
        println!("Trampoline function for {:?}:", basic_block_id);
        print_disassembled_code(&mmap[0..instructions.len()], rip);
    }

    let entry_fn: UnaryFunction = unsafe { std::mem::transmute(mmap.as_ptr()) };

    Ok((mmap, entry_fn))
}

pub fn codegen_basic_block(
    environment: &mut Environment,
    basic_block_id: &BasicBlockId,
    dump_disassembly: bool
) -> Result<Mmap> {
    let basic_block = environment
        .flow_graph
        .get_basic_block(&basic_block_id)
        .expect("invalid basic block id");

    let mut asm = CodeAssembler::new(64)?;
    let mut context = CodegenContext::new();

    let instructions = basic_block.instructions.clone();

    for instruction in instructions {
        match instruction {
            FlowInstruction::PushLiteralBoolean(_literal) => todo!(),
            FlowInstruction::PushLiteralString(_literal) => todo!(),
            FlowInstruction::PushLiteralNumber(literal) => {
                let register = context.push(CodegenStackEntry::Number);
                asm.mov(register, literal as u64)?;
            }
            FlowInstruction::PushLiteralNull => todo!(),
            FlowInstruction::PushFunctionParameter(index) => {
                // TODO what type?!?
                let register = context.push(CodegenStackEntry::Number);
                asm.mov(register, context.argument_register(index))?;
            }
            FlowInstruction::ApplyBinaryOperator(operator) => {
                let (right_entry, right) = context.pop();
                let (left_entry, left) = context.pop();

                if left_entry != CodegenStackEntry::Number {
                    todo!("binary operator on left type {:?}", left_entry);
                }

                if right_entry != CodegenStackEntry::Number {
                    todo!("binary operator on right type {:?}", right_entry);
                }

                let _destination = context.push(CodegenStackEntry::Number);

                match operator {
                    BinaryOperator::Plus => asm.add(left, right)?,
                    op => todo!("codegen for binary operator {:?}", op),
                };
            }

            FlowInstruction::CallFunction {
                basic_block_id,
                argument_count,
            } => {
                for argument_index in 0..argument_count {
                    let (stack_entry, stack_register) = context.pop();
                    
                    if stack_entry != CodegenStackEntry::Number { 
                        todo!("non-number argument");
                    }

                    let argument_register = context.argument_register(argument_index);
                    asm.mov(argument_register, stack_register)?;
                }

                let type_info = TypeInfo;
                let block_fn = environment.basic_block_fn(basic_block_id, type_info);

                asm.sub(rsp, 0x28)?;
                asm.call(block_fn as u64)?;
                asm.add(rsp, 0x28)?;

                let return_value = context.push(CodegenStackEntry::Number);
                asm.mov(return_value, rax)?;
            }

            FlowInstruction::CallSystemFunction(function) => {
                let (entry, argument) = context.pop();

                let callee = match (function, entry) {
                    (SystemFunction::ConsoleLog, CodegenStackEntry::Number) => {
                        console_log_integer_fn
                    }
                    (function, entry) => {
                        todo!("call system function {:?} with arg {:?}", function, entry)
                    }
                };

                asm.mov(context.argument_register(0), argument)?;
                asm.sub(rsp, 0x28)?;
                asm.call(callee as *const u8 as u64)?;
                asm.add(rsp, 0x28)?;

                let return_value = context.push(CodegenStackEntry::Number);
                asm.mov(return_value, rax)?;
            }

            FlowInstruction::ReturnValue => {
                let (_entry, return_value) = context.pop();

                asm.mov(rax, return_value)?;
                asm.ret()?;
            }

            FlowInstruction::Return => {
                asm.ret()?;
            },


            FlowInstruction::GoToBlock(_basic_block_id) => todo!("GoToBlock instruction"),

            FlowInstruction::DiscardValue => {
                context.pop();
            },
        }
    }

    let mut mmap = memmap::MmapMut::map_anon(4096)?;
    let rip = mmap.as_ptr() as u64;
    let instructions = asm.assemble(rip)?;
    (&mut mmap[..]).write(&instructions)?;
    let mmap = mmap.make_exec()?;

    if dump_disassembly {
        println!("Code for block {:?}:", basic_block_id);
        print_disassembled_code(&mmap[0..instructions.len()], rip);
    }

    Ok(mmap)
}

extern "win64" fn basic_block_trampoline(
    environment: *mut Environment,
    basic_block_id: usize,
    call_site: u64,
) -> u64 {
    let environment = unsafe { &mut *environment as &mut Environment };
    let basic_block_id = BasicBlockId(basic_block_id);
    let type_info = TypeInfo;

    // TODO patch up the call site instead of calling the fn here
    let basic_block_fn = environment.compile_basic_block(&basic_block_id, &type_info);
    return basic_block_fn as u64;
}

extern "win64" fn console_log_integer_fn(value: u64) {
    println!("{:?}", value);
}
