use fs_err::File;
use serde::de::DeserializeOwned;
use smallvec::SmallVec;
use std::io::BufReader;
use std::time::Instant;

use crate::shared::{EdgePT, EdgeWalk};

pub fn read_files_serial() -> (
    Vec<i32>,
    Vec<SmallVec<[EdgeWalk; 4]>>,
    Vec<SmallVec<[EdgePT; 4]>>,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    [i8; 32],
) {
    let now = Instant::now();
    
    let node_values_1d: Vec<i32> = deserialize_bincoded_file("padded_node_values_6am");
    let graph_walk: Vec<SmallVec<[EdgeWalk; 4]>> =
        deserialize_bincoded_file("p1_main_nodes_vector_6am");
    let graph_pt: Vec<SmallVec<[EdgePT; 4]>> =
        deserialize_bincoded_file("p2_main_nodes_vector_6am");
    
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
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
        subpurpose_purpose_lookup,
    )
}

fn deserialize_bincoded_file<T: DeserializeOwned>(filename: &str) -> T {
    let path = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(path).unwrap());
    bincode::deserialize_from(file).unwrap()
}
