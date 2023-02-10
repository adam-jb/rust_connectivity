use rayon::prelude::*;
use std::time::Instant;

use crate::shared::{Cost, EdgePT, EdgeWalk, GraphPT, GraphWalk, NodeID};
use smallvec::SmallVec;

use actix_web::{get, post, web, App, HttpServer};
use floodfill::floodfill;
use read_files::read_files_serial;
use serde::{Deserialize, Serialize};
use serialise_files::serialise_files;

mod floodfill;
mod priority_queue;
mod read_files;
mod serialise_files;
mod shared;

//#[global_allocator]
//static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

// This struct represents state
struct AppState {
    node_values_1d: Vec<i32>,
    trip_start_seconds: i32,
    travel_time_relationships: Vec<i32>,
    subpurpose_purpose_lookup: [i8; 32],
    graph_walk: Vec<SmallVec<[EdgeWalk; 4]>>,
    graph_pt: Vec<SmallVec<[EdgePT; 4]>>,
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
async fn index(data: web::Data<AppState>) -> String {
    format!(
        "App is listening! Len of node_values_1d {}",
        &data.node_values_1d.len()
    )
}

#[post("/floodfill_pt/")]
async fn floodfill_pt(data: web::Data<AppState>, input: web::Json<UserInputJSON>) -> String {
    // todo: update graphs in response to new PT routes

    println!("started api floodfill");
    let mut model_parameters_each_start = Vec::new();
    for i in 0..input.start_nodes_user_input.len() {
        model_parameters_each_start.push((
            &data.graph_walk,
            NodeID(input.start_nodes_user_input[i] as u32),
            &data.node_values_1d,
            &data.travel_time_relationships,
            &data.subpurpose_purpose_lookup,
            &data.graph_pt,
            data.trip_start_seconds,
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
        "Parallel floodfill took {:?}\tFirst node scores: {:?}",
        now.elapsed(),
        parallel_res[0]
    );

    // todo: remove anything added to graphs in response to new routes

    return serde_json::to_string(&parallel_res).unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let trip_start_seconds = 3600 * 8;

    //serialise_files();

    let (
        node_values_1d,
        start_nodes,
        init_travel_times,
        graph_walk,
        graph_pt,
        travel_time_relationships,
        subpurpose_purpose_lookup,
    ) = read_files_serial();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                node_values_1d: node_values_1d.to_vec(),
                trip_start_seconds: trip_start_seconds,
                travel_time_relationships: travel_time_relationships.to_vec(),
                subpurpose_purpose_lookup: subpurpose_purpose_lookup,
                graph_walk: graph_walk.to_vec(),
                graph_pt: graph_pt.to_vec(),
                // start_nodes and init_travel_times are POSTed by user
                //start_nodes: start_nodes.to_vec(),
                //init_travel_times: init_travel_times.to_vec(),
            }))
            .service(index)
            .service(floodfill_pt)
    })
    .bind(("127.0.0.1", 7328))?
    .run()
    .await
}
