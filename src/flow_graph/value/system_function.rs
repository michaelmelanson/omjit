use std::{fmt::Debug, rc::Rc};

use crate::codegen::CodegenStackEntry;

pub type SystemFunctionHandlerFn = *const extern "win64" fn();
pub type SystemFunctionGeneratorFn = Box<dyn Fn(&[CodegenStackEntry]) -> Option<SystemFunctionHandlerFn>>;

#[derive(Clone)]
pub struct SystemFunction {
    name: String,
    arity: usize,
    generator: Rc<SystemFunctionGeneratorFn>,
}

impl SystemFunction {
    pub fn new(name: String, arity: usize, generator: SystemFunctionGeneratorFn) -> Self {
        Self {
            name,
            arity,
            generator: Rc::new(generator),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn handler_fn(&self, arguments: &[CodegenStackEntry]) -> Option<SystemFunctionHandlerFn> {
        (self.generator)(arguments)
    }
}

impl Debug for SystemFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemFunction")
            .field("name", &self.name)
            // .field("generator", &self.generator)
            .finish()
    }
}
