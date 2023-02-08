
use serde::de::DeserializeOwned;
use std::time::Instant;

use fs_err::File;
use std::io::{BufReader};
//use rayon::prelude::*;

use crate::shared::{GraphWalk, GraphPT};


pub fn read_files_serial() -> (
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    GraphWalk,
    GraphPT,
    Vec<i32>,
    [i8; 32],
)  {
    let now = Instant::now();
    //let node_values_1d = get_node_values_1d();
    let node_values_1d: Vec<i32> = deserialize_bincoded_file("node_values");
    let start_nodes: Vec<i32> = deserialize_bincoded_file("start_nodes");
    let init_travel_times: Vec<i32> = deserialize_bincoded_file("init_travel_times");
    let graph_walk: GraphWalk = deserialize_bincoded_file("p1_main_nodes");
    let graph_pt: GraphPT = deserialize_bincoded_file("p2_main_nodes");
    //let travel_time_relationships: Vec<Vec<i32>> = deserialize_bincoded_file("travel_time_relationships");
    let travel_time_relationships: Vec<i32> = deserialize_bincoded_file("travel_time_relationships");
    let subpurpose_purpose_lookup: [i8; 32] = deserialize_bincoded_file("subpurpose_purpose_lookup");
    println!("Serial loading took {:?}", now.elapsed());
    (
        node_values_1d,
        start_nodes,
        init_travel_times,
        graph_walk,
        graph_pt,
        travel_time_relationships,
        subpurpose_purpose_lookup,
    )
}

fn deserialize_bincoded_file<T: DeserializeOwned>(filename: &str) -> T {
    let path = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(path).unwrap());
    bincode::deserialize_from(file).unwrap()
}



/*
pub fn read_files_parallel() {
// This section attempts to read as per the above with multiproc.
    // Exclude subpurpose_purpose_lookup as it's tiny
    // ResultType allows one func to return different types of objects: right now
    // am stuck with a hashmap of ResultType objects, each of which contains an object I want to be
    // accessible normally (ie, by calling the variable name, with no hashmap involved). I expect spawning
    // processes to be inefficient (bc I assume it involves copying objects between memory at some point, 
    // unless all processes can write to a shared section of memory)
    enum ResultType {
        ListOfLists(Vec<Vec<i32>>),
        GraphWalk(GraphWalk),
        GraphPT(GraphPT),
        List(Vec<i32>),
    }

    let mut files_to_read_vec = Vec::new();
    files_to_read_vec.push(("read_serialised_vect32", "start_nodes"));
    files_to_read_vec.push(("read_serialised_vect32", "init_travel_times"));
    files_to_read_vec.push(("read_graph_walk", ""));
    files_to_read_vec.push(("read_graph_pt", ""));
    files_to_read_vec.push(("read_list_of_lists_vect32", "node_values"));
    files_to_read_vec.push(("read_list_of_lists_vect32", "travel_time_relationships"));

    fn execute_read_func_from_tuple(tin: (&str, &str)) -> ResultType {
        return match tin.0 {
            "read_list_of_lists_vect32" => {
                ResultType::ListOfLists(read_list_of_lists_vect32(tin.1))
            }
            "read_graph_walk" => ResultType::GraphWalk(read_graph_walk()),
            "read_graph_pt" => ResultType::GraphPT(read_graph_pt()),
            "read_serialised_vect32" => ResultType::List(read_serialised_vect32(tin.1)),
            _ => panic!("Unknown function"),
        };
    }

    // parallel load test
    let now = Instant::now();
    let inputs_map: HashMap<String, ResultType> = files_to_read_vec
        .par_iter()
        .map(|input| {
            (
                input.0.to_string() + "-" + &input.1,
                execute_read_func_from_tuple(*input),
            )
        })
        .collect();
    println!("Parallel file reading took {:?}", now.elapsed());

    for key in inputs_map.keys() {
        println!("{}", key);
    }
    print_type_of(&inputs_map["read_serialised_vect32-init_travel_times"]);
    // end of attempt to multiprocess inputs
    // todo: functionalise the above section
}
*/