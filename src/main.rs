mod codegen;
mod environment;
mod flow_graph;
mod source_location;

use structopt::StructOpt;

use crate::{environment::Environment, flow_graph::FlowGraph};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long)]
    disassemble: bool,

    #[structopt(short, long)]
    show_flowgraph: bool,

    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();

    let code = std::fs::read_to_string(args.path).expect("read source file");
    let (_, node) = almond::parse_program(code.as_str().into()).expect("parse");
    let flow_graph = FlowGraph::from_root_node(&node);

    if args.show_flowgraph {
        println!("Flow graph: {:#?}", flow_graph);
    }

    let environment = Environment::new(flow_graph, args.disassemble);
    environment.run().expect("run failed");
}
