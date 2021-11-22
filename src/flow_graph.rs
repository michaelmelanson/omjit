mod basic_block;
mod flow_instruction;
mod nodes;
mod scope;
mod value;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use almond::ast::{Node, NodeKind};

use crate::{
    flow_graph::nodes::{
        expression_statement::handle_expression_statement, function_decl::handle_function_decl,
        return_statement::handle_return_statement,
        for_statement::handle_for_statement
    },
    source_location::SourceLocation,
};

pub use self::{
    basic_block::{BasicBlock, BasicBlockId},
    flow_instruction::FlowInstruction,
    scope::{Id, Scope},
    value::{SystemFunction, SystemFunctionGeneratorFn, SystemFunctionHandlerFn, Value},
};

#[derive(Default, Debug)]
pub struct FlowGraph<'a> {
    pub root_block_id: Option<BasicBlockId>,
    next_basic_block_id: BasicBlockId,
    basic_blocks: HashMap<BasicBlockId, BasicBlock<'a>>,
}

impl<'a> FlowGraph<'a> {
    pub fn from_root_node(node: &'a Node, scope: Scope) -> Self {
        match &node.kind {
            NodeKind::Program { body } => {
                let mut graph = FlowGraph::default();

                let scope = Rc::new(RefCell::new(scope));
                let root_block_id = graph.create_basic_block(node, scope, body, true, None);

                graph.root_block_id = Some(root_block_id);
                graph
            }

            node_kind => todo!("insert node into flow graph {:?}", node_kind),
        }
    }

    fn next_basic_block_id(&mut self) -> BasicBlockId {
        let id = self.next_basic_block_id;
        self.next_basic_block_id = BasicBlockId(id.0 + 1);
        id
    }

    pub(crate) fn insert_block(&mut self, block: BasicBlock<'a>) {
        self.basic_blocks.insert(block.id, block);
    }

    pub(crate) fn add_node_to_block(&mut self, block: &mut BasicBlock<'a>, node: &Node<'a>) {
        match &node.kind {
            NodeKind::FunctionDeclaration { function } => {
                handle_function_decl(self, block, function)
            }
            NodeKind::ExpressionStatement {
                expression,
                directive: _,
            } => handle_expression_statement(block, expression),

            NodeKind::ReturnStatement { argument } => handle_return_statement(block, argument),

            NodeKind::ForStatement { body, init, test, update } => handle_for_statement(self, node, block, init, test, update, body),

            kind => todo!("compile node {:?}", kind),
        }
    }

    pub fn create_basic_block(
        &mut self,
        parent: &Node<'a>,
        scope: Rc<RefCell<Scope>>,
        nodes: &[Node<'a>],
        include_prologue_epilogue: bool,
        next_basic_block_id: Option<BasicBlockId>
    ) -> BasicBlockId {
        let id = self.next_basic_block_id();
        let mut block = BasicBlock::new(
            id,
            scope,
            SourceLocation {
                start: parent.start,
                end: parent.end,
            },
            next_basic_block_id
        );

        if include_prologue_epilogue {
            block.instructions.push(FlowInstruction::FunctionPrologue);
        }

        for node in nodes {
            self.add_node_to_block(&mut block, node);
        }

        if include_prologue_epilogue {
            block.instructions.push(FlowInstruction::FunctionEpilogue);
        }

        self.insert_block(block);
        id
    }

    pub fn get_basic_block(&self, basic_block_id: &BasicBlockId) -> Option<&BasicBlock<'a>> {
        self.basic_blocks.get(basic_block_id)
    }
}
