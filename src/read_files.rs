use fs_err::File;
use serde::de::DeserializeOwned;
use smallvec::SmallVec;
use std::io::BufReader;
use std::time::Instant;

use crate::shared::{EdgePT, EdgeWalk};

pub fn read_files_serial(
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

pub fn read_files_serial_excluding_travel_time_relationships_and_subpurpose_lookup(
    year: i32,
) -> (
    Vec<i32>,
    Vec<SmallVec<[EdgeWalk; 4]>>,
    Vec<SmallVec<[EdgePT; 4]>>,
    u32,
) {
    let now = Instant::now();

    let padded_node_values_filename = format!("padded_node_values_6am_{}", year);
    let p1_filename = format!("p1_main_nodes_vector_6am_{}", year);
    let p2_filename = format!("p2_main_nodes_vector_6am_{}", year);
    let node_values_padding_row_count_filename = format!("node_values_padding_row_count_6am_{}", year);

    let node_values_1d: Vec<i32> = deserialize_bincoded_file(&padded_node_values_filename);
    let graph_walk: Vec<SmallVec<[EdgeWalk; 4]>> = deserialize_bincoded_file(&p1_filename);
    let graph_pt: Vec<SmallVec<[EdgePT; 4]>> = deserialize_bincoded_file(&p2_filename);
    let node_values_padding_row_count: u32 = deserialize_bincoded_file(&node_values_padding_row_count_filename);

    println!("Serial loading for files excluding travel time relationships took {:?}", now.elapsed());
    (
        node_values_1d,
        graph_walk,
        graph_pt,
        node_values_padding_row_count,
    )
}

fn deserialize_bincoded_file<T: DeserializeOwned>(filename: &str) -> T {
    let path = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(path).unwrap());
    bincode::deserialize_from(file).unwrap()
}
