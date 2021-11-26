pub mod gdb_jit;

use std::io::Write;

use almond::ast::BinaryOperator;
use anyhow::Result;
use iced_x86::{
    code_asm::{AsmRegister64, *},
    Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter,
};
use memmap::Mmap;

use crate::{
    environment::{Environment, TypeInfo},
    flow_graph::BasicBlockId,
};
use crate::{flow_graph::{FlowInstruction, TailInstruction}, Id, Value};

use self::gdb_jit::GdbJitImageRegistration;

pub type UnaryFunction = extern "win64" fn() -> ();

pub struct CodegenContext {
    pub stack: Vec<CodegenStackEntry>,
}

const ARGUMENT_REGISTERS: [AsmRegister64; 4] = [rcx, rdx, r8, r9];
const VOLATILE_REGISTERS: [AsmRegister64; 7] = [rbx, r10, r11, r12, r13, r14, r15];

impl CodegenContext {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn current_stack_register(&self) -> AsmRegister64 {
        let index = self.stack.len();
        *VOLATILE_REGISTERS
            .get(index)
            .expect("register stack overflow")
    }

    pub fn push(&mut self, entry: CodegenStackEntry) -> AsmRegister64 {
        let register = self.current_stack_register();
        self.stack.push(entry);
        register
    }

    pub fn argument_register(&self, index: usize) -> AsmRegister64 {
        *ARGUMENT_REGISTERS
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

    StackVariable(usize),

    Id(Id),
}

pub fn print_disassembled_code(bytes: &[u8], base_address: u64) -> String {
    const HEXBYTES_COLUMN_BYTE_LENGTH: usize = 10;
    let mut decoder = Decoder::with_ip(64, bytes, base_address, DecoderOptions::NONE);

    // Formatters: Masm*, Nasm*, Gas* (AT&T) and Intel* (XED).
    // For fastest code, see `SpecializedFormatter` which is ~3.3x faster. Use it if formatting
    // speed is more important than being able to re-assemble formatted instructions.
    let mut formatter = NasmFormatter::new();

    // Change some options, there are many more
    formatter.options_mut().set_first_operand_char_index(10);
    formatter.options_mut().set_rip_relative_addresses(true);
    formatter
        .options_mut()
        .set_space_after_operand_separator(true);

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
        let start_index = (instruction.ip() - base_address) as usize;
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
    dump_disassembly: bool,
) -> Result<(GdbJitImageRegistration, UnaryFunction)> {
    let environment_ptr = environment as *mut Environment;
    let mut asm = CodeAssembler::new(64)?;

    let stack_size = 3 * 8;

    // asm.int3()?;
    asm.sub(rsp, stack_size)?;

    let arg0 = rsp + 0i8;
    let arg1 = rsp + 8i8;

    asm.mov(arg0, rcx)?;
    asm.mov(arg1, rdx)?;

    // get a pointer to the actual code
    asm.mov(rcx, environment_ptr as u64)?;
    asm.mov(rdx, basic_block_id.0 as u64)?;
    asm.call(basic_block_trampoline as u64)?;

    // restore arguments and call generated function
    asm.mov(rcx, arg0)?;
    asm.mov(rdx, arg1)?;
    asm.call(rax)?;

    asm.add(rsp, stack_size)?;
    asm.ret()?;

    let (base_address, registration, size) = assemble_code(asm)?;

    if dump_disassembly {
        println!("Trampoline function for {:?}:", basic_block_id);
        print_disassembled_code(&registration.file()[0..size], base_address);
    }

    let entry_fn: UnaryFunction = unsafe { std::mem::transmute(registration.file().as_ptr()) };

    Ok((registration, entry_fn))
}

fn assemble_code(mut asm: CodeAssembler) -> Result<(u64, GdbJitImageRegistration, usize)> {
    let mut mmap = memmap::MmapMut::map_anon(4096)?;
    let base_address = mmap.as_ptr() as u64;
    let instructions = asm.assemble(base_address)?;
    let size = instructions.len();
    (&mut mmap[..]).write_all(&instructions)?;
    let mmap = mmap.make_exec()?;

    let registration = GdbJitImageRegistration::register(mmap);

    Ok((base_address, registration, size))
}

pub fn codegen_basic_block(
    environment: &mut Environment,
    basic_block_id: &BasicBlockId,
    dump_disassembly: bool,
) -> Result<Mmap> {
    let mut asm = CodeAssembler::new(64)?;
    let mut context = CodegenContext::new();

    let instructions = environment
        .get_basic_block(basic_block_id)
        .expect("invalid basic block id")
        .instructions();

    let stack_allocation = environment
        .get_basic_block(basic_block_id)
        .expect("invalid basic block id")
        .stack_allocation();

    let shadow_space = 8 * VOLATILE_REGISTERS.len();

    let stack_size = shadow_space + stack_allocation;
    let extra_stack = 8 + (stack_size.checked_next_multiple_of(16).unwrap());

    for instruction in instructions {
        match instruction {
            FlowInstruction::FunctionPrologue => {
                // asm.int3()?;

                if extra_stack > 0 {
                    asm.sub(rsp, extra_stack as i32)?;
                }

                for (index, register) in VOLATILE_REGISTERS.iter().enumerate() {
                    asm.mov(rsp + (((index + 0) * 8) as i8), *register)?;
                }
            }

            FlowInstruction::FunctionEpilogue => {
                for (index, register) in VOLATILE_REGISTERS.iter().enumerate() {
                    asm.mov(*register, rsp + (((index + 0) * 8) as i8))?;
                }

                if extra_stack > 0 {
                    asm.add(rsp, extra_stack as i32)?;
                }

                // asm.pop(rbp)?;
                asm.ret()?;
            }

            FlowInstruction::Assign => {
                let (left_entry, _left) = context.pop();
                let (_right_entry, right) = context.pop();

                let (left, id) = match &left_entry {
                    CodegenStackEntry::Id(id) => {
                        let value = environment
                            .get_basic_block(basic_block_id)
                            .clone()
                            .expect("invalid basic block id")
                            .lookup(&id);

                        (value, Some(id))
                    }
                    CodegenStackEntry::StackVariable(offset) => {
                        (Some(Value::StackVariable { offset: *offset }), None)
                    }
                    other => unimplemented!("assignment to left hand side {:?}", other),
                };

                match left {
                    Some(Value::StackVariable { offset }) => {
                        asm.mov(rsp + shadow_space + (8 * offset), right)?;
                    }
                    Some(Value::FunctionParameter(_index)) => {
                        todo!("assignment to function parameter")
                    }

                    Some(Value::Function {
                        body: _,
                        id,
                        params: _,
                    }) => unimplemented!("assignment to function {:?}", id),
                    Some(Value::SystemFunction(_)) => {
                        unimplemented!("assignment to system function {:?}", id)
                    }

                    None => unimplemented!("assignment left hand side {:?} not defined", id),
                }
            }
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
            FlowInstruction::PushStackVariable(offset) => {
                let register = context.push(CodegenStackEntry::StackVariable(offset));
                asm.mov(register, rsp + shadow_space + (8 * offset))?;
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

                // asm.sub(rsp, 0x28)?;
                asm.call(block_fn as u64)?;
                // asm.add(rsp, 0x28)?;

                let return_value = context.push(CodegenStackEntry::Number);
                asm.mov(return_value, rax)?;
            }

            FlowInstruction::CallSystemFunction(function) => {
                let mut argument_entries = Vec::new();

                for index in 0..function.arity() {
                    let (entry, argument) = context.pop();
                    argument_entries.push(entry);

                    asm.mov(context.argument_register(index), argument)?;
                }

                let callee = function.handler_fn(&argument_entries);
                let callee = callee.unwrap_or_else(|| {
                    todo!(
                        "handler not implemented for system function {} with args {:?}",
                        function.name(),
                        argument_entries
                    )
                });

                asm.call(callee as *const u8 as u64)?;

                let return_value = context.push(CodegenStackEntry::Number);
                asm.mov(return_value, rax)?;
            }

            FlowInstruction::ReturnValue => {
                let (_entry, return_value) = context.pop();

                asm.mov(rax, return_value)?;
                // asm.ret()?;
            }

            FlowInstruction::Return => {
                asm.mov(rax, 0u64)?;
                // asm.ret()?;
            }

            FlowInstruction::GoToBlock(_basic_block_id) => todo!("GoToBlock instruction"),

            FlowInstruction::DiscardValue => {
                context.pop();
            }
        }
    }

    let tail_instructions = environment
        .get_basic_block(basic_block_id)
        .expect("invalid basic block id")
        .tails
        .clone();

    for instruction in tail_instructions {
        match instruction {
            TailInstruction::Jump(target_block_id) => {
                let target = environment.basic_block_fn(target_block_id, TypeInfo) as u64;
                asm.jmp(target)?;
            }
            TailInstruction::ConditionalJump(_target_block_id) => todo!("conditional jump tail"),
        }
    }

    let mut mmap = memmap::MmapMut::map_anon(4096)?;
    let rip = mmap.as_ptr() as u64;
    let instructions = asm.assemble(rip)?;
    (&mut mmap[..]).write_all(&instructions)?;
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
) -> u64 {
    let environment = unsafe { &mut *environment as &mut Environment };
    let basic_block_id = BasicBlockId(basic_block_id);
    let type_info = TypeInfo;

    println!("Entered trampoline for basic block {:?}", basic_block_id);

    let basic_block_fn = environment.compile_basic_block(&basic_block_id, &type_info);
    basic_block_fn as u64
}
