use almond::ast::BinaryOperator;

use crate::Id;

use super::{basic_block::BasicBlockId, value::SystemFunction};

#[derive(Clone, Debug)]
pub enum FlowInstruction {
    FunctionPrologue,
    FunctionEpilogue,
    Assign,
    PushLiteralBoolean(bool),
    PushLiteralString(String),
    PushLiteralNumber(f64),
    PushLiteralNull,
    PushFunctionParameter(usize),
    PushStackVariable(usize),
    ApplyBinaryOperator(BinaryOperator),
    CallFunction {
        basic_block_id: BasicBlockId,
        argument_count: usize,
    },
    CallSystemFunction(SystemFunction),
    ReturnValue,
    Return,
    GoToBlock(BasicBlockId),
    DiscardValue,
}
