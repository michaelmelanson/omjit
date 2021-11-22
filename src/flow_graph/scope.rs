use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

use super::value::Value;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Default, Debug)]
pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,
    bindings: HashMap<Id, Value>,
    stack_offset: usize,
    pub stack_allocation: usize
}

impl Scope {
    pub fn new(parent: Option<Rc<RefCell<Scope>>>) -> Self {
        let stack_offset = if let Some(ref parent) = parent {
            let parent = parent.borrow();
            parent.stack_offset + parent.stack_allocation
        } else {
            0
        };

        Scope {
            parent,
            bindings: HashMap::new(),
            stack_offset,
            stack_allocation: 0
        }
    }

    pub fn insert(&mut self, name: Id, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn allocate_stack(&mut self, size: usize) -> usize {
        self.stack_offset += size;
        self.stack_allocation += size;
        self.stack_offset
    }

    pub fn lookup(&self, name: &Id) -> Option<Value> {
        if let Some(value) = self.bindings.get(name) {
            Some(value.clone())
        } else if let Some(parent) = self.parent.as_ref() {
            parent.borrow().lookup(name)
        } else {
            None
        }
    }
}

impl Id {
    pub(crate) fn new(name: &str) -> Id {
        Id(name.to_owned())
    }
}
