#![feature(once_cell)]
use std::{lazy::SyncLazy, sync::{Mutex, RwLock}};
use lazy_static::lazy_static;

use omjit::{
    CodegenStackEntry, Environment, FlowGraph, Id, Scope, SystemFunction, SystemFunctionHandlerFn,
    Value,
};

// static EMITTED: SyncLazy<Mutex<Vec<u64>>> = SyncLazy::new(|| Mutex::new(Vec::with_capacity(1)));

extern "win64" fn emit_number_fn(number: u64) {
    // EMITTED.lock().unwrap().push(number);
}

#[test]
fn test_add() {
    let code = "function add(a, b) { return a + b; } add(2, 3);";

    let mut scope = Scope::default();
    // scope.insert(
    //     Id("__emit".to_string()),
    //     Value::SystemFunction(SystemFunction::new(
    //         "emit".to_string(),
    //         1,
    //         Box::new(|args| match args {
    //             [CodegenStackEntry::Number] => Some(emit_number_fn as SystemFunctionHandlerFn),
    //             _ => None,
    //         }),
    //     )),
    // );

    let (_, node) = almond::parse_program(code.into()).expect("parse");
    let flow_graph = FlowGraph::from_root_node(&node, scope);
    let environment = Environment::new(flow_graph, true);
    environment.run().expect("run failed");

    // assert_eq!(&*EMITTED.lock().unwrap(), &[5]);
    // EMITTED.lock().unwrap().clear();
    println!("Done");
}
