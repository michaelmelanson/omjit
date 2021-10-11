use almond::ast::{Node, NodeKind};

use crate::flow_graph::{
    basic_block::BasicBlock, flow_instruction::FlowInstruction,
    nodes::expression::evaluate_expression, scope::Id, value::Value, FlowGraph,
};

pub fn handle_expression_statement<'a>(
    _flow_graph: &mut FlowGraph<'a>,
    parent_block: &mut BasicBlock,
    expression: &Node<'a>,
    _directive: &Option<String>,
) {
    match &expression.kind {
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

                    other => todo!("call expression to callee {:?}", other),
                }
            } else {
                todo!("undefined callee {:?}", callee_id);
            }
        }

        kind => todo!("expression statement {:?}", kind),
    }
}
