use serde::de::DeserializeOwned;
use smallvec::SmallVec;
use std::time::Instant;

use fs_err::File;
use std::collections::{HashMap};
use std::io::{BufReader, BufWriter};
use rayon::prelude::*;

use shared::{NodeID, Cost, LeavingTime, EdgeWalk, GraphWalk, EdgePT, GraphPT};
use floodfill::floodfill; 

mod priority_queue;
mod shared;
mod floodfill; 

//#[global_allocator]
//static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    // these are for dev only: understanding time to run different
    //assess_cost_of_casting();
    //test_vec_subset_speed();
    //demonstrate_mutable_q();

    // If you need to regenerate the serialised files, change to if (true)
    if false {
        serialise_list_immutable_array_i8("subpurpose_purpose_lookup");
        serialise_list("start_nodes");
        serialise_list("init_travel_times");
        serialise_graph_walk();
        serialise_graph_pt();
        serialise_list_of_lists("node_values");
        serialise_list_of_lists("travel_time_relationships");
    }

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

/// todo: make creation of node_values_1d part of serialisation (so it's only run once)
fn get_node_values_1d() -> Vec<i32> {
    let node_values: Vec<Vec<i32>> = deserialize_bincoded_file("node_values");
    let mut node_values_1d: Vec<i32> = Vec::new();
    for node_vec in &node_values {
        for specific_val in node_vec {
            node_values_1d.push(*specific_val);
        }
    }
    node_values_1d
}

fn deserialize_bincoded_file<T: DeserializeOwned>(filename: &str) -> T {
    let path = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(path).unwrap());
    bincode::deserialize_from(file).unwrap()
}

fn read_list_of_lists_vect32(filename: &str) -> Vec<Vec<i32>> {
    let inpath = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(inpath).unwrap());
    let output: Vec<Vec<i32>> = bincode::deserialize_from(file).unwrap();
    output
}

fn serialise_list_of_lists(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output: Vec<Vec<i32>> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn serialise_graph_pt() {
    let contents = fs_err::read_to_string("data/p2_main_nodes.json").unwrap();

    // to do: check meaning of the '2' in [usize; 2]
    let input: HashMap<usize, Vec<[usize; 2]>> = serde_json::from_str(&contents).unwrap();

    // make empty dict
    let mut graph = GraphPT {
        edges_per_node: HashMap::new(),
    };

    // populate dict
    for (from, input_edges) in input {
        let mut edges = SmallVec::new();
        for array in input_edges {
            edges.push(EdgePT {
                leavetime: LeavingTime(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph.edges_per_node.insert(from, edges);
    }

    let file = BufWriter::new(File::create("serialised_data/p2_main_nodes.bin").unwrap());
    bincode::serialize_into(file, &graph).unwrap();
}

fn read_graph_walk() -> GraphWalk {
    let file = BufReader::new(File::open("serialised_data/p1_main_nodes.bin").unwrap());
    let output: GraphWalk = bincode::deserialize_from(file).unwrap();
    output
}

fn read_graph_pt() -> GraphPT {
    let file = BufReader::new(File::open("serialised_data/p2_main_nodes.bin").unwrap());
    let output: GraphPT = bincode::deserialize_from(file).unwrap();
    output
}

fn serialise_graph_walk() {
    let contents = fs_err::read_to_string("data/p1_main_nodes.json").unwrap();

    // to do: check meaning of the '2' in [usize; 2]
    let input: HashMap<usize, Vec<[usize; 2]>> = serde_json::from_str(&contents).unwrap();

    // make empty dict
    let mut graph = GraphWalk {
        edges_per_node: HashMap::new(),
    };

    // populate dict
    for (from, input_edges) in input {
        let mut edges = SmallVec::new();
        for array in input_edges {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph.edges_per_node.insert(from, edges);
    }

    let file = BufWriter::new(File::create("serialised_data/p1_main_nodes.bin").unwrap());
    bincode::serialize_into(file, &graph).unwrap();
}

fn serialise_list(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output: Vec<i32> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn serialise_list_immutable_array_i8(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: [i8; 32] = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn read_serialised_vect32(filename: &str) -> Vec<i32> {
    let inpath = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(inpath).unwrap());
    let output: Vec<i32> = bincode::deserialize_from(file).unwrap();
    output
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
