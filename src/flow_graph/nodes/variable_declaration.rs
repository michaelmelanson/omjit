use almond::ast::{Node, NodeKind};

use crate::{
    flow_graph::{
        nodes::expression::evaluate_expression, BasicBlock, FlowInstruction,
    },
    Id, Value,
};

pub fn handle_variable_declarations<'a>(block: &mut BasicBlock, declarations: &Vec<Node<'a>>) {
    for declaration in declarations {
        match &declaration.kind {
            NodeKind::VariableDeclarator { id: node, init } => {
                let id = match &node.kind {
                    NodeKind::Identifier { name } => Id::new(name),
                    other => unimplemented!("identifier kind {:?}", other),
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

                lookup_identifier(block, &id);
                block.instructions.push(FlowInstruction::Assign);
            }
            _ => unimplemented!("variable declaration node {:?}", declaration),
        }
    }
}

fn lookup_identifier(block: &mut BasicBlock, id: &Id) {
    let value = block.scope.borrow().lookup(id);

    match value {
        Some(value) => match value {
            Value::StackVariable { offset } => {
                block.push(FlowInstruction::PushStackVariable(offset))
            }
            Value::Function { id: _, params: _, body: _ } => todo!("lookup function"),
            Value::FunctionParameter(index) => {
                block.push(FlowInstruction::PushFunctionParameter(index))
            }
            Value::SystemFunction(function) => todo!("lookup system function {:?}", function),
        },
        None => todo!("Identifier {:?} is not defined", id),
    }
}
