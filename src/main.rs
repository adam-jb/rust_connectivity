use serde::de::DeserializeOwned;
use smallvec::SmallVec;
use std::time::Instant;

use fs_err::File;
use std::collections::{HashMap};
use std::io::{BufReader, BufWriter};
use rayon::prelude::*;

use shared::{NodeID, Cost, LeavingTime, EdgeWalk, GraphWalk, EdgePT, GraphPT};
use floodfill::floodfill; 
use serialise_files::serialise_files;

mod priority_queue;
mod shared;
mod floodfill; 
mod serialise_files;

//#[global_allocator]
//static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {

    // serialise_files();

    let now = Instant::now();
    let node_values_1d = get_node_values_1d();
    let start_nodes: Vec<i32> = deserialize_bincoded_file("start_nodes");
    let init_travel_times: Vec<i32> = deserialize_bincoded_file("init_travel_times");
    let graph_walk: GraphWalk = deserialize_bincoded_file("p1_main_nodes");
    let graph_pt: GraphPT = deserialize_bincoded_file("p2_main_nodes");
    let travel_time_relationships: Vec<Vec<i32>> = deserialize_bincoded_file("travel_time_relationships");
    let subpurpose_purpose_lookup: [i8; 32] = deserialize_bincoded_file("subpurpose_purpose_lookup");
    println!("generic loading took {:?}", now.elapsed());

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

    let trip_start_seconds = 3600 * 8;

    let mut model_parameters_each_start = Vec::new();
    for i in 0..100 {
        model_parameters_each_start.push((
            &graph_walk,
            NodeID(start_nodes[i] as u32),
            &node_values_1d,
            &travel_time_relationships,
            &subpurpose_purpose_lookup,
            &graph_pt,
            trip_start_seconds,
            Cost(init_travel_times[i] as u16),
        ))
    }

    let now = Instant::now();
    let mut score_store = Vec::new();
    let mut total_iters_counter = 0;
    for input in &model_parameters_each_start {
        let (total_iters, scores) = floodfill(*input);

        total_iters_counter += total_iters;
        score_store.push(scores);
    }
    println!(
        "Calculating routes took {:?}\nReached {} nodes in total",
        now.elapsed(),
        total_iters_counter
    );
    println!("Score from last start node {:?}", score_store.pop());

    // parallel speed test
    let now = Instant::now();
    let parallel_res: Vec<(i32, [i64; 32])> = model_parameters_each_start
        .par_iter()
        .map(|input| floodfill(*input))
        .collect();
    println!(
        "Parallel floodfill took {:?}\tFirst node scores: {:?}",
        now.elapsed(),
        parallel_res[0]
    );
}
