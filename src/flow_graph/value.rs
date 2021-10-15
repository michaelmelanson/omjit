use super::{basic_block::BasicBlockId, scope::Id};

#[derive(Debug, Clone)]
pub enum Value {
    Function {
        id: Option<Id>,
        params: Vec<Id>,
        body: BasicBlockId,
    },

    FunctionParameter(usize),
    SystemFunction(SystemFunction),
}

#[derive(Debug, Clone)]
pub enum SystemFunction {
    ConsoleLog,
}

impl SystemFunction {
    pub fn arity(&self) -> usize {
        match self {
            SystemFunction::ConsoleLog => 1,
        }
    }
}
