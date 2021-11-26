use std::{cell::RefCell, rc::Rc};

use almond::ast::{Node, NodeKind};

use crate::{
    flow_graph::{
        nodes::variable_declaration::handle_variable_declarations,
        tail_instruction::TailInstruction, BasicBlock,
    },
    FlowGraph, Scope,
};

pub fn handle_for_statement<'a>(
    flow_graph: &mut FlowGraph<'a>,
    parent_node: &Node<'a>,
    block: &mut BasicBlock,
    init: &Option<Node<'a>>,
    _test: &Option<Node<'a>>,
    _update: &Option<Node<'a>>,
    body: &Node<'a>,
) {
    if let Some(init) = init {
        match &init.kind {
            NodeKind::VariableDeclaration { declarations, kind: _ } => {
                handle_variable_declarations(block, &declarations)
            }
            _ => unimplemented!("for-loop init node {:?}", init),
        }
    }

    let body_block = match &body.kind {
        NodeKind::BlockStatement { body } => flow_graph.create_basic_block(
            parent_node,
            Rc::new(RefCell::new(Scope::new(Some(block.scope.clone())))),
            &body,
            false,
            
        ),
        other => unimplemented!("for loop body node {:?}", other),
    };

    block.tails.push(TailInstruction::Jump(body_block));

    // todo!("what do I do with for loop block {:?}?", body_block)
}
