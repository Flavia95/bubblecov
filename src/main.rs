use std::env;
use std::path::PathBuf;
use bstr::BString;
use gfa::gfa::GFA;
use gfa::parser::GFAParser;
use handlegraph::hashgraph::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = PathBuf::from(args[1].clone());
    let parser = GFAParser::new();
    let gfa: GFA<BString, ()> = parser.parse_file(path).unwrap();

    HashGraph::from_gfa(&gfa);
    
}
