use almond::ast::Node;

use crate::flow_graph::{basic_block::BasicBlock, flow_instruction::FlowInstruction};

use super::expression::evaluate_expression;

pub fn handle_return_statement(block: &mut BasicBlock, argument: &Option<Node>) {
    if let Some(argument) = argument {
        evaluate_expression(block, argument);
        block.push(FlowInstruction::ReturnValue);
    } else {
        todo!("return with no argument");
    }
}
