use omjit::{
    CodegenStackEntry, Environment, FlowGraph, Id, Scope, SystemFunction, SystemFunctionHandlerFn,
    Value,
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    /// Print generated code during execution
    #[structopt(short, long)]
    disassemble: bool,

    /// Print the flow graph before executing program
    #[structopt(short, long)]
    show_flowgraph: bool,

    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();

    let mut scope = Scope::default();
    scope.insert(
        Id("__console_log".to_string()),
        Value::SystemFunction(SystemFunction::new(
            "console_log".to_string(),
            1,
            Box::new(console_log_generator_fn),
        )),
    );

    let code = std::fs::read_to_string(args.path).expect("read source file");
    let (_, node) = almond::parse_program(code.as_str().into()).expect("parse");
    let flow_graph = FlowGraph::from_root_node(&node, scope);

    if args.show_flowgraph {
        println!("Flow graph: {:#?}", flow_graph);
    }

    let environment = Environment::new(flow_graph, args.disassemble);
    environment.run().expect("run failed");
}

fn console_log_generator_fn(arguments: &[CodegenStackEntry]) -> Option<SystemFunctionHandlerFn> {
    match arguments {
        [CodegenStackEntry::Number] => Some(console_log_integer_fn as SystemFunctionHandlerFn),
        _ => None,
    }
}

extern "win64" fn console_log_integer_fn(value: u64) {
    println!("{:?}", value);
}
