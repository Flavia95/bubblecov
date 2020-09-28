use handlegraph::handle::NodeId;
extern crate clap;
use clap::{Arg, App};
use handlegraph::hashgraph::HashGraph;
use std::path::PathBuf;
//use std::env;
//use gfa::gfa::GFA;
//use gfa::parser::GFAParser;
//use bstr::BString;
use handlegraph::hashgraph::PathId;
use std::collections::HashMap;
use handlegraph::handle::Handle;
use handlegraph::pathgraph::steps_iter;
use gfa::parser::parse_gfa;
use handlegraph::pathgraph::PathHandleGraph;
use std::collections::BTreeMap;
use handlegraph::handlegraph::HandleGraph;




/// Returns a step as a String with NodeId and Orientation
fn process_step(h: &Handle) -> String {
    let orient = if h.is_reverse() {
        "+".to_string()
    } else {
        "-".to_string()
    };
    println!("{}{}", h.id().to_string(), orient);
    format!("{}{}", h.id().to_string(), orient)
}


/// Returns all paths as a hashmap, having the path_name as key and a list of steps as values
fn create_into_hashmap(
    g: &HashGraph,
    path_to_steps: &mut HashMap<String, Vec<String>>,
    path: &PathId,
    step: &Handle,
) -> bool {
    let path_name = g.get_path(path).unwrap().name.clone();

    path_to_steps
        .entry(path_name.to_string())
        .or_default()
        .push(process_step(step));

    true
}



/// Converts paths into sequences of nodes
pub fn paths_to_steps(graph: &HashGraph) -> HashMap<String, Vec<String>> {
    let mut path_to_steps_map: HashMap<String, Vec<String>> = HashMap::new();

    for path_id in std::iter::from_fn(graph.paths_iter_impl()) {
        for step in steps_iter(graph, path_id) {
            let handle = graph.handle_of_step(&step).unwrap();
            create_into_hashmap(graph, &mut path_to_steps_map, path_id, &handle);
        }
    }

    path_to_steps_map
    
}


/// Returns paths as sequences
pub fn get_path_to_sequence(
    graph: &HashGraph,
    path_to_steps_map: &HashMap<String, Vec<String>>,
) -> HashMap<String, String> {
    let mut path_to_sequence_map: HashMap<String, String> = HashMap::new();

    for (path_name, steps_list) in path_to_steps_map {
        path_to_sequence_map.insert(path_name.to_string(), String::new());

        for node_id_rev in steps_list {
            path_to_sequence_map
                .get_mut(path_name)
                .unwrap()
                .push_str(graph.sequence(Handle::pack(
                    NodeId::from(node_id_rev.parse::<u64>().unwrap()),
                    false,
                )));
        }
    }

    path_to_sequence_map
}


//Return position
pub fn get_node_positions_in_paths(
    graph: &HashGraph,
    path_to_steps_map: &mut HashMap<String, Vec<String>>,
) -> BTreeMap<NodeId, HashMap<String, usize>> {
    let mut node_id_to_path_and_pos_map: BTreeMap<NodeId, HashMap<String, usize>> = BTreeMap::new();

    for (path_name, steps_list) in path_to_steps_map {
        let mut pos = 0;

        for node_id_is_rev in steps_list {
            // Get orientation
            let _is_rev = node_id_is_rev.pop().unwrap();
            // Get the id of the node string -> NodeId
            let node_id: NodeId = NodeId::from(node_id_is_rev.parse::<u64>().unwrap());

            let node_handle = Handle::pack(node_id, false);
            let seq = graph.sequence(node_handle);

            node_id_to_path_and_pos_map
                .entry(node_id)
                .or_insert_with(HashMap::new);

            if !node_id_to_path_and_pos_map[&node_id].contains_key(path_name) {
                node_id_to_path_and_pos_map
                    .get_mut(&node_id)
                    .unwrap()
                    .insert(String::from(path_name), pos);
            }

            pos += seq.len();
            println!("prov is: {:#?}",path_name);
        }
    }

    node_id_to_path_and_pos_map
}

//read and parse GFA
fn read_test_gfa(input: &str) -> HashGraph {
    HashGraph::from_gfa(&parse_gfa(&PathBuf::from(input)).unwrap())
}

//hashmap with step-id and count of node in a path
fn obtain_coverage(paths_to_step_map: &HashMap<String, Vec<String>>) -> HashMap<String, u64> {

    let mut coverage : HashMap<String, u64> = HashMap::new();

    for path in paths_to_step_map.keys() {
        for step in &paths_to_step_map[path] {
            let val = coverage.entry(step.to_string()).or_insert(0);
            *val = *val + 1;
            //println!("Coverage of {} is {:?}", step, val);
        }
    }


    coverage
}


 
 /// The function that runs the script
fn main() {
    let matches = App::new("handlegraph-cli")
        .version("0.1.0")
        .author("Erik Garrison <erik.garrison@gmail.com>")
        .about("DetectionBubble")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input file to use")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Sets the output file to use")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Shows debug messages during execution"),
        )
        .arg(
            Arg::with_name("json")
                .short("j")
                .long("json")
                .value_name("PATH")
                .takes_value(true)
                .help("Sets the path where to store the json of both the starting graph and its bfs-tree"),
        )
        .arg(
            Arg::with_name("max-edges")
                .short("m")
                .long("max-edges")
                .value_name("NUMBER")
                .takes_value(true)
                .default_value("100")
                .help("Sets the maximum amount of edges to be used to find paths between nodes"),
        )
        .arg(
            Arg::with_name("reference-paths")
                .short("p")
                .long("reference-paths")
                .value_name("LIST")
                .takes_value(true)
                .help("Sets the reference paths to be used during bubble detection (comma separated). By default all paths are used."),
        )
        .get_matches();

        let input = matches.value_of("input").unwrap();

        println!("File is: {}", input);
        
        let graph = read_test_gfa(input);

        let path_to_steps_map: HashMap<String, Vec<String>> = paths_to_steps(&graph);

        //println!("Path to steps is: {:#?}", path_to_steps_map);

        let coverage = obtain_coverage(&path_to_steps_map);

        println!("Coverage is: {:#?}",coverage);

	//let node_id_to_path_and_pos_map:: HashMap::String::Vec::String::HashMap::String::Vec::String::get_node_positions_in_paths(&graph);
}
