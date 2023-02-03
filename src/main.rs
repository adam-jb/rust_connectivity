use arrayvec::ArrayVec;
use rand::prelude::*;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::time::Instant;

use nanorand::{Rng, WyRand};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::thread;
use std::time::Duration;

use rayon::prelude::*;

use google_cloud_storage::client::Client;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::UploadObjectRequest;
use google_cloud_storage::http::Error;
use google_cloud_storage::sign::SignedURLMethod;
use google_cloud_storage::sign::SignedURLOptions;

use self::priority_queue::PriorityQueueItem;

mod priority_queue;

//#[global_allocator]
//static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
struct NodeID(u32);

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Cost(u16);

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct LeavingTime(u32);

#[derive(Serialize, Deserialize, Clone, Copy)]
struct EdgeWalk {
    to: NodeID,
    cost: Cost,
}

#[derive(Serialize, Deserialize, Clone)]
struct GraphWalk {
    edges_per_node: HashMap<usize, SmallVec<[EdgeWalk; 4]>>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct EdgePT {
    leavingTime: LeavingTime,
    cost: Cost,
}

#[derive(Serialize, Deserialize, Clone)]
struct GraphPT {
    edges_per_node: HashMap<usize, SmallVec<[EdgePT; 4]>>,
}

fn main() {
    /// these are for dev only: understanding time to run different
    //assess_cost_of_casting();
    //test_vec_subset_speed();
    //demonstrate_mutable_q();

    //serialise_list_i8("subpurpose_purpose_lookup");
    //serialise_list("start_nodes");
    //serialise_list("init_travel_times");
    //serialise_GraphWalk();
    //serialise_GraphPT();
    //serialise_list_of_lists("node_values");
    //serialise_list_of_lists("travel_time_relationships");
    let now = Instant::now();
    let start_nodes = read_serialised_vect32("start_nodes");
    let init_travel_times = read_serialised_vect32("init_travel_times");
    let graph_walk = read_GraphWalk();
    let graph_pt = read_GraphPT();
    let node_values_1d = get_node_values_1d();
    let travel_time_relationships = read_list_of_lists_vect32("travel_time_relationships");
    let subpurpose_purpose_lookup = read_serialised_vect8("subpurpose_purpose_lookup");
    println!("Loading took {:?}", now.elapsed());


    // to delete
    let node_values = read_list_of_lists_vect32("node_values");



    // Read as per the above with multiproc. 
    // Exclude subpurpose_purpose_lookup as it's tiny
     // ResultType allows one func to return different types of objects
     enum ResultType {
        list_of_lists(Vec<Vec<i32>>),
        GraphWalk(GraphWalk),
        GraphPT(GraphPT),
        list(Vec<i32>),
    }

    let mut files_to_read_vec = Vec::new();
    files_to_read_vec.push(("read_serialised_vect32","start_nodes"));
    files_to_read_vec.push(("read_serialised_vect32","init_travel_times"));
    files_to_read_vec.push(("read_GraphWalk", ""));
    files_to_read_vec.push(("read_GraphPT", ""));
    files_to_read_vec.push(("read_list_of_lists_vect32", "node_values"));
    files_to_read_vec.push(("read_list_of_lists_vect32", "travel_time_relationships"));
   
    fn execute_read_func_from_tuple(tin: (&str,&str)) -> ResultType {
        return match tin.0 {
            "read_list_of_lists_vect32" => ResultType::list_of_lists(read_list_of_lists_vect32(tin.1)),
            "read_GraphWalk" => ResultType::GraphWalk(read_GraphWalk()),
            "read_GraphPT" => ResultType::GraphPT(read_GraphPT()),
            "read_serialised_vect32" => ResultType::list(read_serialised_vect32(tin.1)),
            _ => panic!("Unknown function"),
        };
    }

    // parallel load test
    let now = Instant::now();
    let inputs_map: HashMap<String, ResultType> = files_to_read_vec
        .par_iter()
        .map(|input| (input.0.to_string() + "-" + &input.1, execute_read_func_from_tuple(*input)))
        .collect();
    println!("Parallel file reading took {:?}", now.elapsed());

    for key in inputs_map.keys() {
        println!("{}", key);
    }
    // todo: functionalise the above section




    let trip_start_seconds = 3600 * 8;

    let now = Instant::now();
    let mut score_store = Vec::new();
    let mut total_iters_counter = 0;

    let mut model_parameters_each_start = Vec::new();
    for i in 0..100 {
        model_parameters_each_start.push((
            &graph_walk,
            NodeID((start_nodes[i] as u32)),
            &node_values_1d,
            &travel_time_relationships,
            &subpurpose_purpose_lookup,
            &graph_pt,
            trip_start_seconds,
        ))
    }
    for input in &model_parameters_each_start {
        let (total_iters, scores) = floodfill(
            *input,
        );

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
    let parallel_res: Vec<(i32, Vec<i64>)> = model_parameters_each_start
        .par_iter()
        .map(|input| floodfill(*input))
        .collect();
    println!(
        "Parallel floodfill took {:?}\tFirst node scores: {:?}",
        now.elapsed(),
        parallel_res[0]
    );
}



/// todo: make creation of node_values_1d part of serialisation (one-time-only run) 
fn get_node_values_1d() -> Vec<i32> {
    let node_values = read_list_of_lists_vect32("node_values");
    let mut node_values_1d: Vec<i32> = Vec::new();
    for node_vec in &node_values {
        for specific_val in node_vec {
            node_values_1d.push(*specific_val);
        }
    }
    node_values_1d
}



fn floodfill(
    (
        graph_walk,
        start,
        node_values_1d,
        travel_time_relationships,
        subpurpose_purpose_lookup,
        graph_pt,
        trip_start_seconds,
    ): (
        &GraphWalk,
        NodeID,
        &Vec<i32>,  //&Vec<Vec<i32>>,
        &Vec<Vec<i32>>,
        &Vec<i8>,
        &GraphPT,
        i32,
    ), 
) -> (i32, Vec<i64>) {

    const time_limit: Cost = Cost(3600);
    let subpurposes_count: usize = 32 as usize;
    let now = Instant::now();

    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: Cost(0),
        value: start,
    });
    let mut nodes_visited = HashSet::new();
    let mut total_iters = 0;
    let mut pt_iters = 0;

    // hard coding with 32 subpurposes to fill scores for
    //let mut scores = ArrayVec::<_, 32>::new();
    let mut scores: Vec<i64> = Vec::new();
    for i in 0..32 {
        //scores[i] = 0;
        scores.insert(i, 0);
    }

    while let Some(current) = queue.pop() {
        if nodes_visited.contains(&current.value) {
            continue;
        }
        if current.cost > time_limit {
            continue;
        }

        nodes_visited.insert(current.value);

        // if the node id is under 40m, then it will have an associated value
        if current.value.0 < 40_000_000 {
            get_scores(
                current.value.0,
                &node_values_1d,
                current.cost.0,
                travel_time_relationships,
                subpurpose_purpose_lookup,
                subpurposes_count,
                &mut scores,
            );
        }

        // Finding adjacent walk nodes
        // skip 1st edge as it has info on whether node also has a PT service
        for edge in &graph_walk.edges_per_node[&(current.value.0 as usize)][1..] {
            queue.push(PriorityQueueItem {
                cost: Cost(current.cost.0 + edge.cost.0),
                value: edge.to,
            });
        }

        // if node has a timetable associated with it: the first value in the first 'edge'
        // will be 1 if it does, and 0 if it doesn't
        if graph_walk.edges_per_node[&(current.value.0 as usize)][0].cost == Cost(1) {
            //let pt_connection = get_pt_connections(
            get_pt_connections(
                &graph_walk,
                &graph_pt, // alter this to be a vector of vectors
                current.cost.0,
                &mut queue,
                time_limit,
                trip_start_seconds,
                &current.value,
            );
        }

        total_iters += 1;
    }
    println!("total_iters: {}\t{:?}", total_iters, now.elapsed());

    return (total_iters, scores);
}

fn get_scores(
    node_id: u32,
    node_values_1d: &Vec<i32>,
    time_so_far: u16,
    travel_time_relationships: &Vec<Vec<i32>>,
    subpurpose_purpose_lookup: &Vec<i8>,
    subpurposes_count: usize,
    scores: &mut Vec<i64>,
    //scores: &mut ArrayVec<i64, 32>,
) {

    // to subset node_values_1d
    let start_pos = node_id * 32;

    // 32 subpurposes
    for i in 0..subpurposes_count {
        let ix_purpose = subpurpose_purpose_lookup[(i as usize)];
        let multiplier = travel_time_relationships[ix_purpose as usize][time_so_far as usize];
        
        // this line is slowing the whole thing down !!!!
        scores[i] += (node_values_1d[(start_pos as usize) + i] * multiplier) as i64;

        // the thing that takes ages is: scores[i] += values_this_node[i]
        // scores[i] += 1; is mega-fast
        // doesnt make much difference if 'multiplier' is included or not
    }
}

fn get_pt_connections(
    graph_walk: &GraphWalk,
    graph_pt: &GraphPT,
    time_so_far: u16,
    queue: &mut BinaryHeap<PriorityQueueItem<Cost, NodeID>>,
    time_limit: Cost,
    trip_start_seconds: i32,
    current_node: &NodeID,
) {
    // find time node is arrived at in seconds past midnight
    let time_of_arrival_current_node = trip_start_seconds as u32 + time_so_far as u32;

    // find time next service leaves
    let mut found_next_service = 0;
    let mut journey_time: u32 = 0;
    let mut next_leaving_time = 0;
    for edge in &graph_pt.edges_per_node[&(current_node.0 as usize)][1..] {
        if time_of_arrival_current_node <= edge.cost.0 as u32 {
            next_leaving_time = edge.cost.0;
            journey_time = edge.leavingTime.0 as u32;
            found_next_service = 1;
            break;
        }
    }

    // add to queue
    if found_next_service == 1 {
        let wait_time_this_stop = next_leaving_time as u32 - time_of_arrival_current_node;
        let arrival_time_next_stop =
            time_so_far as u32 + wait_time_this_stop as u32 + journey_time as u32;

        if arrival_time_next_stop < time_limit.0 as u32 {
            //// Notice this uses 'leavingTime' as first 'edge' for each node stores ID
            //// of next node: this is legacy from our matrix-based approach in python
            let destination_node = &graph_pt.edges_per_node[&(current_node.0 as usize)][0]
                .leavingTime
                .0;

            queue.push(PriorityQueueItem {
                cost: Cost(arrival_time_next_stop as u16),
                value: NodeID(*destination_node as u32),
            });
        };
    }
}

/// No longer used subpurpose lookup is a vector not a hashmap now
/*
fn serialise_hashmap_i8(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: HashMap<i8, i8> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn read_hashmap_i8(filename: &str) -> HashMap<i8, i8> {
    let inpath = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(inpath).unwrap());
    let output: HashMap<i8, i8> = bincode::deserialize_from(file).unwrap();
    output
}
*/

fn read_list_of_lists_vect32(filename: &str) -> Vec<Vec<i32>> {
    let inpath = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(inpath).unwrap());
    let output: Vec<Vec<i32>> = bincode::deserialize_from(file).unwrap();
    output
}

fn serialise_list_of_lists(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: Vec<Vec<i32>> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn serialise_GraphPT() {
    let contents = std::fs::read_to_string("data/p2_main_nodes.json").unwrap();

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
                leavingTime: LeavingTime(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph.edges_per_node.insert(from, edges);
    }

    let file = BufWriter::new(File::create("serialised_data/p2_main_nodes.bin").unwrap());
    bincode::serialize_into(file, &graph).unwrap();
}

fn read_GraphWalk() -> GraphWalk {
    let file = BufReader::new(File::open("serialised_data/p1_main_nodes.bin").unwrap());
    let output: GraphWalk = bincode::deserialize_from(file).unwrap();
    output
}

fn read_GraphPT() -> GraphPT {
    let file = BufReader::new(File::open("serialised_data/p2_main_nodes.bin").unwrap());
    let output: GraphPT = bincode::deserialize_from(file).unwrap();
    output
}

fn serialise_GraphWalk() {
    let contents = std::fs::read_to_string("data/p1_main_nodes.json").unwrap();

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
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: Vec<i32> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}


fn serialise_list_i8(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: Vec<i8> = serde_json::from_str(&contents).unwrap();
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

fn read_serialised_vect8(filename: &str) -> Vec<i8> {
    let inpath = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(inpath).unwrap());
    let output: Vec<i8> = bincode::deserialize_from(file).unwrap();
    output
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

/// this and push_to_q() are for reference only
fn demonstrate_mutable_q() {
    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: Cost(0),
        value: NodeID(1),
    });
    push_to_q(&mut queue);
    push_to_q(&mut queue);
    while let Some(current) = queue.pop() {
        println!("{}, {}", current.value.0, current.cost.0);
    }
}

fn push_to_q(queue: &mut BinaryHeap<PriorityQueueItem<Cost, NodeID>>) {
    queue.push(PriorityQueueItem {
        cost: Cost(1),
        value: NodeID(2),
    });
}

fn test_vec_subset_speed() {
    let mut VoV = Vec::new();

    //let mut VoV: <Vec<Vec<i32>>;
    for _ in 1..1000 {
        let mut scores: Vec<i64> = Vec::new();
        for i in 1..2000 {
            scores.push(0);
        }
        VoV.push(scores);
    }
    println!("VoV len: {:?}", VoV.len());
    println!("VoV inner len: {:?}", VoV[0].len());

    let now = Instant::now();
    let mut topps: i32 = 0;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            VoV[i][k];
            //iters += 1;
        }
    }
    println!("VoV took {:?}", now.elapsed());

    let now = Instant::now();
    let mut topps: i64 = 0;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            topps += VoV[i][k];
            //iters += 1;
        }
    }
    println!("VoV took {:?}\ttopps: {}", now.elapsed(), topps);

    let now = Instant::now();
    let mut topps: i64 = 0;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            topps += VoV[i][k];
            iters += 1;
        }
    }
    println!("VoV took {:?}\t with iters {}", now.elapsed(), iters);
    // all the above shows assigning to 'iters' is much more time intensive than subsetting:
    // dont bother with any other data structure
}

fn assess_cost_of_casting() {
    let mut VoV = Vec::new();

    //let mut VoV: <Vec<Vec<i32>>;
    for _ in 1..1000 {
        let mut scores: Vec<i64> = Vec::new();
        for i in 1..2000 {
            scores.push(0);
        }
        VoV.push(scores);
    }
    let now = Instant::now();
    let mut topps: i64 = 1;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            VoV[i][k] += topps;
            //iters += 1;
        }
    }
    println!("VoV without casting took {:?}", now.elapsed());

    let now = Instant::now();
    let mut topps: i64 = 1;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            VoV[i][k] += topps;
            //iters += 1;
        }
    }
    println!("VoV WITH casting took {:?}, {}", now.elapsed(), VoV[5][5]);

    let now = Instant::now();
    let mut topps: i32 = 1;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            //VoV[i][k] += topps as i32;
            iters += topps as i32;
        }
    }
    println!("Topps WITH casting took {:?}, {}", now.elapsed(), iters);
}
