mod system_function;

use std::fmt::Debug;

use super::{basic_block::BasicBlockId, scope::Id};

pub use self::system_function::{
    SystemFunction, SystemFunctionGeneratorFn, SystemFunctionHandlerFn,
};

#[derive(Debug, Clone)]
pub enum Value {
    StackVariable { offset: usize },
    
    Function {
        id: Option<Id>,
        params: Vec<Id>,
        body: BasicBlockId,
    },

    FunctionParameter(usize),
    SystemFunction(SystemFunction),
}
