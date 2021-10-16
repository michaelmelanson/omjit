use almond::ast::Node;

use crate::flow_graph::{
    basic_block::BasicBlock, flow_instruction::FlowInstruction,
    nodes::expression::evaluate_expression,
};

pub fn handle_expression_statement<'a>(parent_block: &mut BasicBlock, expression: &Node<'a>) {
    evaluate_expression(parent_block, expression);
    parent_block
        .instructions
        .push(FlowInstruction::DiscardValue);
}
