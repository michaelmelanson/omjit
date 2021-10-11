use almond::ast::{
    LiteralValue, Node,
    NodeKind::{self, BinaryExpression, Identifier},
};

use crate::flow_graph::{
    basic_block::BasicBlock, flow_instruction::FlowInstruction, scope::Id, value::Value,
};

pub fn evaluate_expression(block: &mut BasicBlock, node: &Node) {
    match &node.kind {
        BinaryExpression {
            operator,
            left,
            right,
        } => {
            evaluate_expression(block, left);
            evaluate_expression(block, right);
            block.push(FlowInstruction::ApplyBinaryOperator(*operator))
        }

        Identifier { name } => {
            let id = Id::new(name);

            let value = block.scope.borrow().lookup(&id);
            if let Some(value) = value {
                block.push(match value {
                    Value::FunctionParameter(index) => {
                        FlowInstruction::PushFunctionParameter(index)
                    }
                    value => todo!("evaluate identifier value {:?}", value),
                })
            } else {
                todo!("undefined identifier {:?}", id);
            }
        }

        NodeKind::Literal { value } => match value {
            LiteralValue::String(literal) => {
                block.push(FlowInstruction::PushLiteralString(literal.clone()))
            }
            LiteralValue::Boolean(literal) => {
                block.push(FlowInstruction::PushLiteralBoolean(*literal))
            }
            LiteralValue::Null => block.push(FlowInstruction::PushLiteralNull),
            LiteralValue::Number(literal) => {
                block.push(FlowInstruction::PushLiteralNumber(*literal))
            }
            LiteralValue::RegExp(_literal) => todo!("literal regexp"),
        },

        kind => todo!("expression node {:?}", kind),
    }
}
