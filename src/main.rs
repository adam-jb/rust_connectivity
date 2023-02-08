use std::time::Instant;
use rayon::prelude::*;

use shared::{NodeID, Cost};
use floodfill::floodfill; 
//use serialise_files::serialise_files;
use read_files::read_files_serial;

mod priority_queue;
mod shared;
mod floodfill; 
//mod serialise_files;
mod read_files;

//#[global_allocator]
//static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {

    //serialise_files();
    let (node_values_1d, start_nodes, init_travel_times, graph_walk, graph_pt, travel_time_relationships, subpurpose_purpose_lookup) = read_files_serial();
    

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
