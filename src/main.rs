use rayon::prelude::*;
use std::time::Instant;

use crate::shared::{Cost, EdgePT, EdgeWalk, NodeID};
use smallvec::SmallVec;

use actix_web::{get, post, web, App, HttpServer};
use floodfill::floodfill;
use read_files::read_files_serial;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod floodfill;
mod priority_queue;
mod read_files;
mod shared;

use serialise_files::serialise_files;
mod serialise_files;


// This struct represents state
struct AppState {
    node_values_1d: Arc<Vec<i32>>,
    travel_time_relationships_all: Arc<Vec<Arc<Vec<i32>>>>,
    subpurpose_purpose_lookup: [i8; 32],
    graph_walk: Arc<Vec<SmallVec<[EdgeWalk; 4]>>>,
    graph_pt: Arc<Vec<SmallVec<[EdgePT; 4]>>>,
    node_values_padding_row_count: u32,
}

#[derive(Deserialize)]
struct UserInputJSON {
    start_nodes_user_input: Vec<i32>,
    init_travel_times_user_input: Vec<i32>,
    trip_start_seconds: i32,
    //pt_additions: Vec<Vec<Vec<i32>>>,
    year: i32,
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
    let count_original_nodes = &data.graph_walk.len();
    return serde_json::to_string(&count_original_nodes).unwrap();
}

#[post("/floodfill_pt/")]
async fn floodfill_pt(data: web::Data<AppState>, input: web::Json<UserInputJSON>) -> String {
    
    println!("Floodfill request received");
    
    // todo: update graphs in response to new PT routes
    
    /*
    ##### update_p1_main_nodes
    
    
    
    */
    
    
    
    // find which travel time relationships to use
    let mut time_of_day_ix = 0;
    if input.trip_start_seconds > 3600 * 10 {
        time_of_day_ix = 1;
    }
    if input.trip_start_seconds > 3600 * 16 {
        time_of_day_ix = 2;
    }
    if input.trip_start_seconds > 3600 * 19 {
        time_of_day_ix = 3;
    }
    
    let count_original_nodes: u32 = data.graph_walk.len() as u32;
    let mut model_parameters_each_start = Vec::new();
    
    let arc_node_values_1d: Arc<Vec<i32>>;
    let arc_graph_walk: Arc<Vec<SmallVec<[EdgeWalk; 4]>>>;
    let arc_graph_pt: Arc<Vec<SmallVec<[EdgePT; 4]>>>;
    
    if input.year < 2022 {
        // read graphs and node values from previous year to use
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
        ) = read_files_serial(input.year);
    
        arc_node_values_1d = Arc::new(node_values_1d);
        arc_graph_walk = Arc::new(graph_walk);
        arc_graph_pt = Arc::new(graph_pt);
        
        println!("Creating tuples to pass to floodfill for {} data", input.year);
        for i in 0..input.start_nodes_user_input.len() {
            model_parameters_each_start.push((
                &arc_graph_walk,
                NodeID(input.start_nodes_user_input[i] as u32),
                &arc_node_values_1d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &data.subpurpose_purpose_lookup,
                &arc_graph_pt,
                input.trip_start_seconds,
                Cost(input.init_travel_times_user_input[i] as u16),
                count_original_nodes,
                node_values_padding_row_count,
            ))
        }
        
    } else {
        println!("Creating tuples to pass to floodfill for 2022 data");
        for i in 0..input.start_nodes_user_input.len() {
            model_parameters_each_start.push((
                &data.graph_walk,
                NodeID(input.start_nodes_user_input[i] as u32),
                &data.node_values_1d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &data.subpurpose_purpose_lookup,
                &data.graph_pt,
                input.trip_start_seconds,
                Cost(input.init_travel_times_user_input[i] as u16),
                count_original_nodes,
                *&data.node_values_padding_row_count,
            ))
        }
    }
    
    // run for all in parallel
    println!("Started running floodfill\ttime_of_day_ix: {}", time_of_day_ix);
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

    //for year in 2016..2023 {
    //    serialise_files(year);
    //}
    
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
    
    let arc_node_values_1d = Arc::new(node_values_1d);
    let arc_graph_walk = Arc::new(graph_walk);
    let arc_graph_pt = Arc::new(graph_pt);
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
            .service(index)
            .service(get_node_id_count)
            .service(floodfill_pt)
    })
    .bind(("127.0.0.1", 7328))?
    .run()
    .await
}
