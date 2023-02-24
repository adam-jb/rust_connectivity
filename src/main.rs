use rayon::prelude::*;
use std::time::Instant;
use smallvec::SmallVec;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use actix_web::{get, post, web, App, HttpServer};
use std::collections::HashMap;

use crate::shared::{Cost, LeavingTime, EdgePT, EdgeWalk, NodeID};
use floodfill::floodfill;
use read_files::{read_files_serial, read_files_serial_excluding_travel_time_relationships_and_subpurpose_lookup};
use get_time_of_day_index::get_time_of_day_index;

mod floodfill;
mod priority_queue;
mod read_files;
mod shared;
mod get_time_of_day_index;

use serialise_files::serialise_files_all_years;
mod serialise_files;

// This struct represents state
struct AppState {
    node_values_1d: Arc<Mutex<Vec<i32>>>,
    travel_time_relationships_all: Arc<Vec<Arc<Vec<i32>>>>,
    subpurpose_purpose_lookup: [i8; 32],
    graph_walk: Arc<Mutex<Vec<SmallVec<[EdgeWalk; 4]>>>>,
    graph_pt: Arc<Mutex<Vec<SmallVec<[EdgePT; 4]>>>>,
    node_values_padding_row_count: u32,
}

#[derive(Deserialize)]
struct UserInputJSON {
    start_nodes_user_input: Vec<i32>,
    init_travel_times_user_input: Vec<i32>,
    trip_start_seconds: i32,
    graph_walk_additions: Vec<Vec<[usize; 2]>>,
    graph_pt_additions: Vec<Vec<[usize; 2]>>,
    new_nodes_count: usize,
    graph_walk_updates_keys: Vec<usize>,
    graph_walk_updates_additions: Vec<Vec<[usize; 2]>>,
    year: i32,
    new_build_additions: Vec<Vec<i32>>,
}

#[derive(Serialize)]
struct PostOutputJSON {
    all: Vec<(i32, u32, [i64; 32])>,
}

#[get("/")]
async fn index() -> String {
    format!("App is listening")
}

#[get("/get_node_id_count/")]
async fn get_node_id_count(data: web::Data<AppState>) -> String {
    let count_original_nodes = &data.graph_walk.lock().unwrap().len();
    return serde_json::to_string(&count_original_nodes).unwrap();
}

#[post("/floodfill_pt/")]
async fn floodfill_pt(data: web::Data<AppState>, input: web::Json<UserInputJSON>) -> String {
    
    println!("Floodfill request received");
    
    if input.year < 2022 {
        assert!(input.graph_walk_additions.len() == 0);
    }
    
    // need to functionalise this, which adds to graph
    //if input.new_nodes_count > 0 {
    let mut graph_walk_guard = data.graph_walk.lock().unwrap();
    let mut graph_pt_guard = data.graph_pt.lock().unwrap();
    let mut node_values_1d_guard = data.node_values_1d.lock().unwrap();
    let original_node_values_1d_len = node_values_1d_guard.len().clone();

    let len_graph_walk = graph_walk_guard.len();
    let len_graph_pt = graph_pt_guard.len();
    assert!(len_graph_pt == len_graph_walk);

    for input_edges in input.graph_walk_additions.iter() {
        let mut edges: SmallVec<[EdgeWalk; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph_walk_guard.push(edges);
    }

    for input_edges in input.graph_pt_additions.iter() {
        let mut edges: SmallVec<[EdgePT; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgePT {
                leavetime: LeavingTime(array[0] as u32),
                cost: Cost(array[1] as u16),
            });
        }
        graph_pt_guard.push(edges);
    }
    assert!(graph_walk_guard.len() == len_graph_walk + input.new_nodes_count);
    assert!(graph_pt_guard.len() == len_graph_pt + input.new_nodes_count);


    let mut graph_walk_store_for_reset: Vec<SmallVec<[EdgeWalk; 4]>> = vec![];
    for i in 0..input.graph_walk_updates_keys.len() {

        let node = input.graph_walk_updates_keys[i];
        graph_walk_store_for_reset.push(graph_walk_guard[node].clone());

        let mut edges: SmallVec<[EdgeWalk; 4]> = graph_walk_guard[node].clone();
        for array in &input.graph_walk_updates_additions[i] {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph_walk_guard[node] = edges;
    }
    
    for i in 0..input.graph_walk_additions.len() {
        for _ in 0..32 {
            node_values_1d_guard.push(0);
        }
    }
    let expected_len = graph_walk_guard.len() * 32;
    assert!(node_values_1d_guard.len() == expected_len);
    //}
    
    println!("input.new_build_additions.len(): {}", input.new_build_additions.len());
    if input.new_build_additions.len() >= 1 {
        for new_build in &input.new_build_additions {
            let value_to_add = new_build[0];
            let index_of_nearest_node = new_build[1];
            let column_to_change = new_build[2];
            let ix = (index_of_nearest_node * 32) + column_to_change;
            node_values_1d_guard[ix as usize] += value_to_add;
        }
    }
    
    
    let time_of_day_ix:usize = get_time_of_day_index(input.trip_start_seconds);
    let mut model_parameters_each_start = Vec::new();
    
    let arc_node_values_1d: Arc<Mutex<Vec<i32>>>;
    let arc_graph_walk: Arc<Mutex<Vec<SmallVec<[EdgeWalk; 4]>>>>;
    let arc_graph_pt: Arc<Mutex<Vec<SmallVec<[EdgePT; 4]>>>>;
    
    let parallel_res: Vec<(i32, u32, [i64; 32])>;
    
    
    // functionalise or split into two apis
    if input.year < 2022 {
        let (
            node_values_1d,
            graph_walk,
            graph_pt,
            node_values_padding_row_count,
        ) = read_files_serial_excluding_travel_time_relationships_and_subpurpose_lookup(input.year);
    
        arc_node_values_1d = Arc::new(Mutex::new(node_values_1d));
        arc_graph_walk = Arc::new(Mutex::new(graph_walk));
        arc_graph_pt = Arc::new(Mutex::new(graph_pt));
        
        let mut graph_walk_guard = arc_graph_walk.lock().unwrap();
        let mut graph_pt_guard = arc_graph_pt.lock().unwrap();
        let mut node_values_1d_guard = data.node_values_1d.lock().unwrap();
        
        let count_original_nodes: u32 = graph_walk_guard.len() as u32;
        
        let graph_walk_unguarded = &*graph_walk_guard;
        let graph_pt_unguarded = &*graph_pt_guard;
        let node_values_1d_unguarded = &*node_values_1d_guard;
        
        println!("Creating tuples to pass to floodfill for {} data", input.year);
        for i in 0..input.start_nodes_user_input.len() {
            model_parameters_each_start.push((
                graph_walk_unguarded, //&arc_graph_walk,
                NodeID(input.start_nodes_user_input[i] as u32),
                node_values_1d_unguarded, //&arc_node_values_1d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &data.subpurpose_purpose_lookup,
                graph_pt_unguarded, // &arc_graph_pt.lock().unwrap(),
                input.trip_start_seconds,
                Cost(input.init_travel_times_user_input[i] as u16),
                count_original_nodes,
                node_values_padding_row_count,
            ))
        }
        
        println!("Started running floodfill\ttime_of_day_ix: {}\tNodes count: {}", time_of_day_ix, model_parameters_each_start.len());
        let now = Instant::now();
        parallel_res = model_parameters_each_start
            .par_iter()
            .map(|input| floodfill(*input))
            .collect();
        println!(
            "Parallel floodfill took {:?}",
            now.elapsed()
        );
    
    } else {
        
        let count_original_nodes: u32 = graph_walk_guard.len() as u32;
        
        let graph_walk_unguarded = &*graph_walk_guard;
        let graph_pt_unguarded = &*graph_pt_guard;
        let node_values_1d_unguarded = &*node_values_1d_guard;
        
        println!("Creating tuples to pass to floodfill for 2022 data");
        for i in 0..input.start_nodes_user_input.len() {
            model_parameters_each_start.push((
                graph_walk_unguarded,//&data.graph_walk,
                NodeID(input.start_nodes_user_input[i] as u32),
                node_values_1d_unguarded, //&data.node_values_1d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &data.subpurpose_purpose_lookup,
                graph_pt_unguarded, //&data.graph_pt.lock().unwrap(),
                input.trip_start_seconds,
                Cost(input.init_travel_times_user_input[i] as u16),
                count_original_nodes,
                *&data.node_values_padding_row_count,
            ))
        }
        
        println!("Started running floodfill\ttime_of_day_ix: {}\tNodes count: {}", time_of_day_ix, model_parameters_each_start.len());
        let now = Instant::now();
        parallel_res = model_parameters_each_start
            .par_iter()
            .map(|input| floodfill(*input))
            .collect();
        println!(
            "Parallel floodfill took {:?}",
            now.elapsed()
        );
    }
    
    if input.new_nodes_count > 0 {
        graph_walk_guard.truncate(len_graph_walk);
        graph_pt_guard.truncate(len_graph_pt);

        for i in 0..input.graph_walk_updates_keys.len() {
            let node = input.graph_walk_updates_keys[i] as usize;
            graph_walk_guard[node] = graph_walk_store_for_reset[i].clone();
        }
    }
    
    if input.new_build_additions.len() >= 1 {
        for new_build in &input.new_build_additions {
            let value_to_add = new_build[0];
            let index_of_nearest_node = new_build[1];
            let column_to_change = new_build[2];
            let ix = index_of_nearest_node * 32 + column_to_change;
            node_values_1d_guard[ix as usize] -= value_to_add;
        }
    }
    
    node_values_1d_guard.truncate(original_node_values_1d_len);

    return serde_json::to_string(&parallel_res).unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    //serialise_files_all_years();
    
    let year: i32 = 2022;
    let (
        node_values_1d,
        graph_walk,
        graph_pt,
        node_values_padding_row_count,
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
        subpurpose_purpose_lookup,
    ) = read_files_serial(year);
    
    let mut arc_node_values_1d = Arc::new(Mutex::new(node_values_1d));
    let mut arc_graph_walk = Arc::new(Mutex::new(graph_walk));
    let mut arc_graph_pt = Arc::new(Mutex::new(graph_pt));
    let arc_travel_time_relationships_7 = Arc::new(travel_time_relationships_7);
    let arc_travel_time_relationships_10 = Arc::new(travel_time_relationships_10);
    let arc_travel_time_relationships_16 = Arc::new(travel_time_relationships_16);
    let arc_travel_time_relationships_19 = Arc::new(travel_time_relationships_19);
    
    let travel_time_relationships_all: Vec<Arc<Vec<i32>>> = vec![
        arc_travel_time_relationships_7,
        arc_travel_time_relationships_10,
        arc_travel_time_relationships_16,
        arc_travel_time_relationships_19
        ];
    
    let arc_travel_time_relationships_all = Arc::new(travel_time_relationships_all);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                node_values_1d: arc_node_values_1d.clone(),
                travel_time_relationships_all: arc_travel_time_relationships_all.clone(),
                subpurpose_purpose_lookup: subpurpose_purpose_lookup,
                graph_walk: arc_graph_walk.clone(),
                graph_pt: arc_graph_pt.clone(),
                node_values_padding_row_count: node_values_padding_row_count,
            }))
            .data(web::JsonConfig::default().limit(1024 * 1024 * 50))  // allow POST'd JSON payloads up to 50mb
            .service(index)
            .service(get_node_id_count)
            .service(floodfill_pt)
    })
    .bind(("127.0.0.1", 7328))?
    .run()
    .await
}
