use rayon::prelude::*;
use std::time::Instant;

use crate::shared::{Cost, EdgePT, EdgeWalk, NodeID};
use smallvec::SmallVec;

use actix_web::{get, post, web, App, HttpServer};
use floodfill::floodfill;
use read_files::read_files_serial;
use serde::{Deserialize, Serialize};
use serialise_files::serialise_files;
use std::sync::Arc;

mod floodfill;
mod priority_queue;
mod read_files;
mod serialise_files;
mod shared;

//#[global_allocator]
//static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

// This struct represents state
struct AppState {
    node_values_1d: Arc<Vec<i32>>,
    travel_time_relationships_7: Arc<Vec<i32>>,
    travel_time_relationships_10: Arc<Vec<i32>>,
    travel_time_relationships_16: Arc<Vec<i32>>,
    travel_time_relationships_19: Arc<Vec<i32>>,
    subpurpose_purpose_lookup: [i8; 32],
    graph_walk: Arc<Vec<SmallVec<[EdgeWalk; 4]>>>,
    graph_pt: Arc<Vec<SmallVec<[EdgePT; 4]>>>,
}

#[derive(Deserialize)]
struct UserInputJSON {
    start_nodes_user_input: Vec<i32>,
    init_travel_times_user_input: Vec<i32>,
}

#[derive(Serialize)]
struct PostOutputJSON {
    all: Vec<(i32, u32, [i64; 32])>,
}

#[get("/")]
async fn index() -> String {
    format!("App is listening")
}

#[post("/floodfill_pt/")]
async fn floodfill_pt(data: web::Data<AppState>, input: web::Json<UserInputJSON>) -> String {
    
    
    let trip_start_seconds: i32 = 3600 * 8;
    
    // todo: update graphs in response to new PT routes

    /*
    ##### update_p1_main_nodes
    
    
    
    */
    
    // todo: choose travel_time_relationships. 
    // could make a vector, then push the relevant travel_time_relationships obj 
    // eg: &data.travel_time_relationships_7
    // to the vector
    // then pass first item of the vector to func below

    println!("started api floodfill");
    let mut model_parameters_each_start = Vec::new();
    for i in 0..input.start_nodes_user_input.len() {
        model_parameters_each_start.push((
            &data.graph_walk,
            NodeID(input.start_nodes_user_input[i] as u32),
            &data.node_values_1d,
            &data.travel_time_relationships_7,
            &data.subpurpose_purpose_lookup,
            &data.graph_pt,
            trip_start_seconds,
            Cost(input.init_travel_times_user_input[i] as u16),
        ))
    }
    
    // run for all in parallel
    let now = Instant::now();
    let parallel_res: Vec<(i32, u32, [i64; 32])> = model_parameters_each_start
        .par_iter()
        .map(|input| floodfill(*input))
        .collect();
    println!(
        "Parallel floodfill took {:?}",
        now.elapsed()
    );

    // todo: remove anything added to graphs in response to new routes

    return serde_json::to_string(&parallel_res).unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    //serialise_files();

    let (
        node_values_1d,
        graph_walk,
        graph_pt,
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
        subpurpose_purpose_lookup,
    ) = read_files_serial();
    
    let arc_node_values_1d = Arc::new(node_values_1d);
    let arc_graph_walk = Arc::new(graph_walk);
    let arc_graph_pt = Arc::new(graph_pt);
    let arc_travel_time_relationships_7 = Arc::new(travel_time_relationships_7);
    let arc_travel_time_relationships_10 = Arc::new(travel_time_relationships_10);
    let arc_travel_time_relationships_16 = Arc::new(travel_time_relationships_16);
    let arc_travel_time_relationships_19 = Arc::new(travel_time_relationships_19);
    //let arc_subpurpose_purpose_lookup = Arc::new(subpurpose_purpose_lookup);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                node_values_1d: arc_node_values_1d.clone(),
                travel_time_relationships_7: arc_travel_time_relationships_7.clone(),
                travel_time_relationships_10: arc_travel_time_relationships_10.clone(),
                travel_time_relationships_16: arc_travel_time_relationships_16.clone(),
                travel_time_relationships_19: arc_travel_time_relationships_19.clone(),
                subpurpose_purpose_lookup: subpurpose_purpose_lookup,
                graph_walk: arc_graph_walk.clone(),
                graph_pt: arc_graph_pt.clone(),
            }))
            .service(index)
            .service(floodfill_pt)
    })
    .bind(("127.0.0.1", 7328))?
    .run()
    .await
}
