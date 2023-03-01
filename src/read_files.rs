use fs_err::File;
//use rayon::prelude::*;
use serde::de::DeserializeOwned;
use smallvec::SmallVec;
use std::io::{BufReader, BufWriter};
use std::time::Instant;

use crate::shared::{EdgePT, EdgeWalk};

pub fn read_files_parallel_excluding_travel_time_relationships_and_subpurpose_lookup(
    year: i32,
) -> (
    Vec<i32>,
    Vec<SmallVec<[EdgeWalk; 4]>>,
    Vec<SmallVec<[EdgePT; 4]>>,
    u32,
) {
    let now = Instant::now();
    
    let (node_values_1d, (graph_walk, graph_pt)) = rayon::join(
        || deserialize_bincoded_file::<Vec<i32>>(&format!("padded_node_values_6am_{year}")),
        || {
            rayon::join(
                || {
                    deserialize_bincoded_file::<Vec<SmallVec<[EdgeWalk; 4]>>>(&format!(
                        "p1_main_nodes_vector_6am_{year}"
                    ))
                },
                || {
                    deserialize_bincoded_file::<Vec<SmallVec<[EdgePT; 4]>>>(&format!(
                        "p2_main_nodes_vector_6am_{year}"
                    ))
                },
            )
        },
    );

    let node_values_padding_row_count: u32 =
        deserialize_bincoded_file(&format!("node_values_padding_row_count_6am_{year}"));

    println!(
        "Parallel loading for files excluding travel time relationships took {:?}",
        now.elapsed()
    );
    (
        node_values_1d,
        graph_walk,
        graph_pt,
        node_values_padding_row_count,
    )
}

pub fn read_small_files_serial() -> (
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    [i8; 32],
) {
    let now = Instant::now();
    
    let travel_time_relationships_7: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_7");
    let travel_time_relationships_10: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_10");
    let travel_time_relationships_16: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_16");
    let travel_time_relationships_19: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_19");
    let subpurpose_purpose_lookup: [i8; 32] =
        deserialize_bincoded_file("subpurpose_purpose_lookup");

    println!("Serial loading took {:?}", now.elapsed());
    (
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
        subpurpose_purpose_lookup,
    )
}

pub fn deserialize_bincoded_file<T: DeserializeOwned>(filename: &str) -> T {
    let path = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(path).unwrap());
    bincode::deserialize_from(file).unwrap()
}

pub fn create_graph_walk_len(year: i32) {
    let graph_walk = deserialize_bincoded_file::<Vec<SmallVec<[EdgeWalk; 4]>>>(&format!(
                        "p1_main_nodes_vector_6am_{year}"
                    ));
    
    let graph_walk_len = graph_walk.len();
    
    let outpath = format!("serialised_data/graph_walk_len_{}.bin", year);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &graph_walk_len).unwrap();
    println!("Created graph_walk_len at {}", outpath);
}


/*
pub fn read_files_parallel(
    year: i32,
) -> (
    Vec<i32>,
    Vec<SmallVec<[EdgeWalk; 4]>>,
    Vec<SmallVec<[EdgePT; 4]>>,
    u32,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    [i8; 32],
) {
    let now = Instant::now();

    // These're absolutely tiny
    let node_values_padding_row_count: u32 =
        deserialize_bincoded_file(&format!("node_values_padding_row_count_6am_{year}"));
    let subpurpose_purpose_lookup: [i8; 32] =
        deserialize_bincoded_file("subpurpose_purpose_lookup");

    // These're < 100KB. Loading in parallel is honestly overkill.
    let mut travel_time_relationships: Vec<Vec<i32>> = vec![7, 10, 16, 19]
        .par_iter()
        .map(|i| deserialize_bincoded_file(&format!("travel_time_relationships_{i}")))
        .collect();

    // There are 3 big files worth loading in parallel. They have different types, so par_iter
    // doesn't work. https://github.com/rayon-rs/rayon/issues/865 would make this nicer to write.
    let (node_values_1d, (graph_walk, graph_pt)) = rayon::join(
        || deserialize_bincoded_file::<Vec<i32>>(&format!("padded_node_values_6am_{year}")),
        || {
            rayon::join(
                || {
                    deserialize_bincoded_file::<Vec<SmallVec<[EdgeWalk; 4]>>>(&format!(
                        "p1_main_nodes_vector_6am_{year}"
                    ))
                },
                || {
                    deserialize_bincoded_file::<Vec<SmallVec<[EdgePT; 4]>>>(&format!(
                        "p2_main_nodes_vector_6am_{year}"
                    ))
                },
            )
        },
    );

    println!("Parallel loading took {:?}", now.elapsed());
    (
        node_values_1d,
        graph_walk,
        graph_pt,
        node_values_padding_row_count,
        travel_time_relationships.remove(0),
        travel_time_relationships.remove(0),
        travel_time_relationships.remove(0),
        travel_time_relationships.remove(0),
        subpurpose_purpose_lookup,
    )
}

pub fn _read_files_serial(
    year: i32,
) -> (
    Vec<i32>,
    Vec<SmallVec<[EdgeWalk; 4]>>,
    Vec<SmallVec<[EdgePT; 4]>>,
    u32,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    [i8; 32],
) {
    let now = Instant::now();

    let padded_node_values_filename = format!("padded_node_values_6am_{}", year);
    let p1_filename = format!("p1_main_nodes_vector_6am_{}", year);
    let p2_filename = format!("p2_main_nodes_vector_6am_{}", year);
    let node_values_padding_row_count_filename =
        format!("node_values_padding_row_count_6am_{}", year);

    let node_values_1d: Vec<i32> = deserialize_bincoded_file(&padded_node_values_filename);
    let graph_walk: Vec<SmallVec<[EdgeWalk; 4]>> = deserialize_bincoded_file(&p1_filename);
    let graph_pt: Vec<SmallVec<[EdgePT; 4]>> = deserialize_bincoded_file(&p2_filename);
    let node_values_padding_row_count: u32 =
        deserialize_bincoded_file(&node_values_padding_row_count_filename);

    let travel_time_relationships_7: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_7");
    let travel_time_relationships_10: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_10");
    let travel_time_relationships_16: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_16");
    let travel_time_relationships_19: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_19");
    let subpurpose_purpose_lookup: [i8; 32] =
        deserialize_bincoded_file("subpurpose_purpose_lookup");

    println!("Serial loading took {:?}", now.elapsed());
    (
        node_values_1d,
        graph_walk,
        graph_pt,
        node_values_padding_row_count,
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
        subpurpose_purpose_lookup,
    )
}
*/