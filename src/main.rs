use actix_web::{get, post, web, App, HttpServer};
use rayon::prelude::*;
use smallvec::SmallVec;
use std::time::Instant;

use crate::shared::{Cost, EdgePT, EdgeWalk, LeavingTime, NodeID, UserInputJSON};
use floodfill::{get_travel_times, get_all_scores_and_time_to_target_destinations};
use get_time_of_day_index::get_time_of_day_index;
use read_files::{
    read_files_parallel_excluding_node_values,
    read_small_files_serial,
    deserialize_bincoded_file,
    create_graph_walk_len,
};

mod floodfill;
mod get_time_of_day_index;
mod priority_queue;
mod read_files;
mod serialise_files;
mod shared;

struct AppState {
    travel_time_relationships_all: Vec<Vec<i32>>,
    subpurpose_purpose_lookup: [i8; 32],
}

#[get("/")]
async fn index() -> String {
    format!("App is listening")
}

#[get("/get_node_id_count/")]
async fn get_node_id_count() -> String {
    //let count_original_nodes = data.graph_walk_len;
    let year: i32 = 2022;   //// TODO change this dynamically depending on when user hits this api... OR drop this from Rust api and store in py
    let graph_walk_len: i32 = deserialize_bincoded_file(&format!("graph_walk_len_{year}"));
    return serde_json::to_string(&graph_walk_len).unwrap();
}

#[post("/floodfill_pt/")]
async fn floodfill_pt(data: web::Data<AppState>, input: web::Json<UserInputJSON>) -> String {
    if input.year < 2022 {
        assert!(input.graph_walk_additions.is_empty());
    }
 
    if input.graph_walk_additions.is_empty()
        && input.graph_pt_additions.is_empty()
        && input.graph_walk_updates_keys.is_empty()
        && input.graph_walk_updates_additions.is_empty()
        && input.new_build_additions.is_empty()
    {
        return floodfill_pt_no_changes(data, input);
    }
    
    println!("Floodfill request received, with changes to the graphs");

    // Read in files
    let (mut graph_walk, mut graph_pt, node_values_padding_row_count) =
        read_files_parallel_excluding_node_values(input.year);

    let len_graph_walk = graph_walk.len();
    let len_graph_pt = graph_pt.len();
    assert!(len_graph_pt == len_graph_walk);

    for input_edges in input.graph_walk_additions.iter() {
        let mut edges: SmallVec<[EdgeWalk; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph_walk.push(edges);
    }

    for input_edges in input.graph_pt_additions.iter() {
        let mut edges: SmallVec<[EdgePT; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgePT {
                leavetime: LeavingTime(array[0] as u32),
                cost: Cost(array[1] as u16),
            });
        }
        graph_pt.push(edges);
    }
    assert!(graph_walk.len() == len_graph_walk + input.new_nodes_count);
    assert!(graph_pt.len() == len_graph_pt + input.new_nodes_count);

    for i in 0..input.graph_walk_updates_keys.len() {
        let node = input.graph_walk_updates_keys[i];

        // TODO Just modify in-place
        let mut edges: SmallVec<[EdgeWalk; 4]> = graph_walk[node].clone();
        for array in &input.graph_walk_updates_additions[i] {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph_walk[node] = edges;
    }

    println!(
        "input.new_build_additions.len(): {}",
        input.new_build_additions.len()
    );

    let time_of_day_ix = get_time_of_day_index(input.trip_start_seconds);

    let count_original_nodes = graph_walk.len() as u32;

    println!(
        "Started running floodfill\ttime_of_day_ix: {}\tNodes count: {}",
        time_of_day_ix,
        input.start_nodes_user_input.len()
    );
    
    let now = Instant::now();
    
    let (mut node_values_1d, travel_times) = parallel_node_values_read_and_floodfill(
        &graph_walk,
        &graph_pt,
        &input,
    );
        
    println!("Node values read in and floodfill in parallel {:?}", now.elapsed());
    
    // Altering node_values to reflect changes in graph
    for _i in 0..input.graph_walk_additions.len() {
        for _ in 0..32 {
            node_values_1d.push(0);
        }
    }
    let expected_len = graph_walk.len() * 32;
    assert!(node_values_1d.len() == expected_len);
    
    // TODO Redundant conditional? (Adam in response - the below is edited to fix this; keeping comment in case error shows)
    //if input.new_build_additions.len() >= 1 {
    for new_build in &input.new_build_additions {
        let value_to_add = new_build[0];
        let index_of_nearest_node = new_build[1];
        let column_to_change = new_build[2];
        let ix = (index_of_nearest_node * 32) + column_to_change;
        node_values_1d[ix as usize] += value_to_add;
    }
    //}
    
    let now = Instant::now();
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();
    
    let results: Vec<(i32, u32, [i64; 32], Vec<u32>, Vec<u16>)> = indices
        .par_iter()
        .map(|i| {
            get_all_scores_and_time_to_target_destinations(
                &travel_times[*i],
                &node_values_1d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &data.subpurpose_purpose_lookup,
                count_original_nodes,
                node_values_padding_row_count,
                &input.target_destinations,
            )
        })
        .collect();
    println!("Getting destinations and scores took {:?}", now.elapsed());
    
    serde_json::to_string(&results).unwrap()
}


fn get_travel_times_multicore(
    graph_walk: &Vec<SmallVec<[EdgeWalk; 4]>>,
    graph_pt: &Vec<SmallVec<[EdgePT; 4]>>,
    input: &web::Json<UserInputJSON>
) -> Vec<(u32, Vec<u32>, Vec<u16>)> {
        
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();
    
    return indices
        .par_iter()
        .map(|i| {
            get_travel_times(
                &graph_walk,
                &graph_pt,
                NodeID(*&input.start_nodes_user_input[*i] as u32),
                *&input.trip_start_seconds,
                Cost(*&input.init_travel_times_user_input[*i] as u16),
            )
        })
        .collect(); 
}



fn parallel_node_values_read_and_floodfill(graph_walk: &Vec<SmallVec<[EdgeWalk; 4]>>,
    graph_pt: &Vec<SmallVec<[EdgePT; 4]>>,
    input: &web::Json<UserInputJSON>
) -> (
    Vec<i32>, 
    Vec<(u32, Vec<u32>, Vec<u16>)>
) {
    
    let node_values_filename = format!("padded_node_values_6am_{}", input.year);
    
    let (node_values_1d, travel_times) = rayon::join(
            || {
                deserialize_bincoded_file(&node_values_filename)
            },
            || {
                get_travel_times_multicore(
                    &graph_walk,
                    &graph_pt,
                    &input,
                )
            },
        );
    (node_values_1d, travel_times)
}



// No changes to the graph allowed
fn floodfill_pt_no_changes(data: web::Data<AppState>, input: web::Json<UserInputJSON>) -> String {
    
    println!("Floodfill request received, without changes");
    let time_of_day_ix = get_time_of_day_index(input.trip_start_seconds);
    
    let (graph_walk, graph_pt, node_values_padding_row_count) =
        read_files_parallel_excluding_node_values(input.year);
            
    let count_original_nodes = graph_walk.len() as u32;
    
    println!(
        "Started running floodfill and node values files read\ttime_of_day_ix: {}\tNodes count: {}",
        time_of_day_ix,
        input.start_nodes_user_input.len()
    );
    
    let now = Instant::now();
    
    let (node_values_1d, travel_times) = parallel_node_values_read_and_floodfill(
        &graph_walk,
        &graph_pt,
        &input,
    );
        
    println!("Node values read in and floodfill in parallel {:?}", now.elapsed());
    
    let now = Instant::now();
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();
    
    let results: Vec<(i32, u32, [i64; 32], Vec<u32>, Vec<u16>)> = indices
        .par_iter()
        .map(|i| {
            get_all_scores_and_time_to_target_destinations(
                &travel_times[*i],
                &node_values_1d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &data.subpurpose_purpose_lookup,
                count_original_nodes,
                node_values_padding_row_count,
                &input.target_destinations,
            )
        })
        .collect();
    println!("Getting destinations and scores took {:?}", now.elapsed());
    
    serde_json::to_string(&results).unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let year: i32 = 2022;

    if true {
        serialise_files::serialise_files_all_years();
    }
    if true {
        create_graph_walk_len(year); 
    }
    
    let (
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
        subpurpose_purpose_lookup,
    ) = read_small_files_serial();

    let travel_time_relationships_all = vec![
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
    ];
    let app_state = web::Data::new(AppState {
        travel_time_relationships_all,
        subpurpose_purpose_lookup,
    });
    HttpServer::new(move || {
        App::new()
            // This clone is of an Arc from actix. AppState is immutable, and only one copy exists
            // (except for when we clone some pieces of it to make mutations scoped to a single
            // request.)
            .app_data(app_state.clone())
            .data(web::JsonConfig::default().limit(1024 * 1024 * 50)) // allow POST'd JSON payloads up to 50mb
            .service(index)
            .service(get_node_id_count)
            .service(floodfill_pt)
    })
    .bind(("0.0.0.0", 7328))?
    .run()
    .await
}
