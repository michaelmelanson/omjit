use std::{collections::HashMap, mem};

use anyhow::Result;
use memmap::Mmap;

use crate::{
    codegen::{codegen_basic_block, codegen_trampoline, print_disassembled_code, UnaryFunction},
    flow_graph::{BasicBlockId, FlowGraph},
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TypeInfo;

pub struct Environment<'a> {
    dump_disassembly: bool,
    block_versions: HashMap<(BasicBlockId, TypeInfo), (Mmap, UnaryFunction)>,
    trampolines: HashMap<(BasicBlockId, TypeInfo), (Mmap, UnaryFunction)>,
    pub flow_graph: FlowGraph<'a>,
}

impl<'a> Environment<'a> {
    pub fn new(flow_graph: FlowGraph<'a>, dump_disassembly: bool) -> Self {
        Self {
            dump_disassembly,
            block_versions: HashMap::new(),
            trampolines: HashMap::new(),
            flow_graph,
        }
    }

    pub fn basic_block_fn(
        &mut self,
        basic_block_id: BasicBlockId,
        type_info: TypeInfo,
    ) -> UnaryFunction {
        let key = (basic_block_id, type_info);

        if let Some((_mmap, block_fn)) = self.block_versions.get(&key) {
            return *block_fn;
        }

        let trampoline_result = codegen_trampoline(self, &basic_block_id, self.dump_disassembly)
            .expect("codegen failed");
        self.trampolines.insert(key.clone(), trampoline_result);
        self.trampolines.get(&key).unwrap().1
    }

    pub fn run(mut self) -> Result<()> {
        let type_info = TypeInfo;
        let basic_block_id = self.flow_graph.root_block_id.expect("no root block");

        let block_fn = self.basic_block_fn(basic_block_id, type_info);
        block_fn();

        Ok(())
    }

    pub fn compile_basic_block(
        &mut self,
        basic_block_id: &BasicBlockId,
        type_info: &TypeInfo,
    ) -> UnaryFunction {
        let mmap =
            codegen_basic_block(self, basic_block_id, self.dump_disassembly).expect("codegen");
        let entry_fn: extern "win64" fn() = unsafe { mem::transmute(mmap.as_ptr()) };

        self.block_versions
            .insert((*basic_block_id, type_info.clone()), (mmap, entry_fn));

        entry_fn
    }
}
