use almond::ast::BinaryOperator;

use super::basic_block::BasicBlockId;

#[derive(Clone, Debug)]
pub enum FlowInstruction {
    PushLiteralBoolean(bool),
    PushLiteralString(String),
    PushLiteralNumber(f64),
    PushLiteralNull,
    PushFunctionParameter(usize),
    ApplyBinaryOperator(BinaryOperator),
    CallFunction {
        basic_block_id: BasicBlockId,
        argument_count: usize,
    },
    ReturnValue,
}
