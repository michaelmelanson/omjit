use std::{cell::RefCell, rc::Rc};

use crate::source_location::SourceLocation;

use super::{flow_instruction::FlowInstruction, scope::Scope};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BasicBlockId(pub usize);

#[derive(Debug)]
pub struct BasicBlock<'a> {
    pub id: BasicBlockId,
    pub scope: Rc<RefCell<Scope>>,
    pub instructions: Vec<FlowInstruction>,
    pub location: SourceLocation<'a>,
}

impl<'a> BasicBlock<'a> {
    pub fn new(id: BasicBlockId, scope: Rc<RefCell<Scope>>, span: SourceLocation<'a>) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            scope,
            location: span,
        }
    }

    pub fn push(&mut self, instruction: FlowInstruction) {
        self.instructions.push(instruction);
    }
}
