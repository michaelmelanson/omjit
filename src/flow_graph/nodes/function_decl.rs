use std::{cell::RefCell, rc::Rc};

use almond::ast::{Function, NodeKind};

use crate::flow_graph::{
    basic_block::BasicBlock,
    scope::{Id, Scope},
    value::Value,
    FlowGraph,
};

pub fn handle_function_decl<'a>(
    flow_graph: &mut FlowGraph<'a>,
    parent_block: &mut BasicBlock,
    function_node: &Function<'a>,
) {
    if let NodeKind::BlockStatement { body } = &function_node.body.kind {
        let id = function_node.id.as_ref();
        let id = &id.as_ref().map(|id| match &id.kind {
            NodeKind::Identifier { name } => Id::new(name),
            kind => todo!("function identifier {:?}", kind),
        });

        let mut scope = Scope::new(Some(parent_block.scope.clone()));

        let params = function_node
            .params
            .iter()
            .map(|node| match &node.kind {
                NodeKind::Identifier { name } => Id::new(name),
                kind => todo!("function parameter {:?}", kind),
            })
            .collect::<Vec<Id>>();

        for (index, param) in params.iter().enumerate() {
            scope.insert(param.clone(), Value::FunctionParameter(index));
        }

        let scope = Rc::new(RefCell::new(scope));

        let body = flow_graph.create_basic_block(&function_node.body, scope, body);

        let value = Value::Function {
            id: id.clone(),
            params,
            body,
        };

        if let Some(id) = id {
            parent_block.scope.borrow_mut().insert(id.clone(), value);
        }
    } else {
        todo!("function body {:?}", &function_node.body.kind);
    }
}
