use std::{cell::RefCell, rc::Rc};

use crate::{source_location::SourceLocation, Id, Value};

use super::{flow_instruction::FlowInstruction, scope::Scope, tail_instruction::TailInstruction};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BasicBlockId(pub usize);

#[derive(Debug)]
pub struct BasicBlock<'a> {
    pub id: BasicBlockId,
    pub scope: Rc<RefCell<Scope>>,
    pub instructions: Vec<FlowInstruction>,
    pub location: SourceLocation<'a>,
    pub tails: Vec<TailInstruction>,
}

impl<'a> BasicBlock<'a> {
    pub fn new(
        id: BasicBlockId,
        scope: Rc<RefCell<Scope>>,
        span: SourceLocation<'a>,
        next_basic_block_id: Option<BasicBlockId>,
    ) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            scope,
            location: span,
            tails: Vec::new(),
        }
    }

    pub fn push(&mut self, instruction: FlowInstruction) {
        self.instructions.push(instruction);
    }

    pub fn lookup(&self, id: &Id) -> Option<Value> {
        self.scope.borrow().lookup(id)
    }

    pub fn instructions(&self) -> Vec<FlowInstruction> {
        self.instructions.clone()
    }

    pub fn stack_allocation(&self) -> usize {
        self.scope.borrow().stack_allocation
    }
}
