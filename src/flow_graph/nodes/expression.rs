use almond::ast::{
    LiteralValue, Node,
    NodeKind::{self, BinaryExpression, Identifier},
};

use crate::flow_graph::{
    basic_block::BasicBlock, flow_instruction::FlowInstruction, scope::Id, value::Value,
};

pub fn evaluate_expression(parent_block: &mut BasicBlock, node: &Node) {
    match &node.kind {
        NodeKind::CallExpression { arguments, callee } => {
            let callee_id = match &callee.kind {
                NodeKind::Identifier { name } => Id::new(name),
                kind => todo!("callee {:?}", kind),
            };

            let callee = parent_block.scope.borrow().lookup(&callee_id);
            if let Some(callee) = callee {
                match callee {
                    Value::Function {
                        id: _,
                        body,
                        params,
                    } => {
                        if params.len() != arguments.len() {
                            todo!(
                                "arity mismatch: {} params vs {} arguments",
                                params.len(),
                                arguments.len()
                            );
                        }

                        for argument in arguments {
                            evaluate_expression(parent_block, argument);
                        }

                        parent_block.push(FlowInstruction::CallFunction {
                            basic_block_id: body,
                            argument_count: arguments.len(),
                        });
                    }

                    Value::SystemFunction(function) => {
                        if function.arity() != arguments.len() {
                            todo!(
                                "arity mismatch: {} params vs {} arguments",
                                function.arity(),
                                arguments.len()
                            );
                        }

                        for argument in arguments {
                            evaluate_expression(parent_block, argument);
                        }

                        parent_block.push(FlowInstruction::CallSystemFunction(function));
                    }

                    other => todo!("call expression to callee {:?}", other),
                }
            } else {
                todo!("undefined callee {:?}", callee_id);
            }
        }

        BinaryExpression {
            operator,
            left,
            right,
        } => {
            evaluate_expression(parent_block, left);
            evaluate_expression(parent_block, right);
            parent_block.push(FlowInstruction::ApplyBinaryOperator(*operator))
        }

        Identifier { name } => {
            let id = Id::new(name);

            let value = parent_block.scope.borrow().lookup(&id);
            if let Some(value) = value {
                parent_block.push(match value {
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
                parent_block.push(FlowInstruction::PushLiteralString(literal.clone()))
            }
            LiteralValue::Boolean(literal) => {
                parent_block.push(FlowInstruction::PushLiteralBoolean(*literal))
            }
            LiteralValue::Null => parent_block.push(FlowInstruction::PushLiteralNull),
            LiteralValue::Number(literal) => {
                parent_block.push(FlowInstruction::PushLiteralNumber(*literal))
            }
            LiteralValue::RegExp(_literal) => todo!("literal regexp"),
        },

        kind => todo!("expression node {:?}", kind),
    }
}
