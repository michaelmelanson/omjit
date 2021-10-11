use super::{basic_block::BasicBlockId, scope::Id};

#[derive(Debug, Clone)]
pub enum Value {
    Function {
        id: Option<Id>,
        params: Vec<Id>,
        body: BasicBlockId,
    },

    FunctionParameter(usize),
}
