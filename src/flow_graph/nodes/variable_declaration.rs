use almond::ast::{Node, NodeKind};

use crate::{Id, Value, flow_graph::{BasicBlock, FlowInstruction, nodes::expression::evaluate_expression}};


pub fn handle_variable_declarations<'a>(block: &mut BasicBlock, declarations: &Vec<Node<'a>>) {
    for declaration in declarations {
        match &declaration.kind {
            NodeKind::VariableDeclarator { id, init } => {
                let id = match &id.kind {
                    NodeKind::Identifier { name } => Id::new(name),
                    other => unimplemented!("identifier kind {:?}", other)
                };

                {
                    let mut scope = block.scope.borrow_mut();
                    let offset = scope.allocate_stack(8);
                    scope.insert(id.clone(), Value::StackVariable { offset });
                }

                if let Some(init) = init.as_ref() {
                    evaluate_expression(block, init);
                } else {
                    block.instructions.push(FlowInstruction::PushLiteralNull);
                }
                
                block.instructions.push(FlowInstruction::Assign(id.clone()));
            }
            _ => unimplemented!("variable declaration node {:?}", declaration)
        }
    }
}