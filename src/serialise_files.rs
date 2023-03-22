use smallvec::SmallVec;
use std::time::Instant;

use fs_err::File;
use std::io::BufWriter;

use crate::shared::{Cost, EdgePT, EdgeWalk, LeavingTime, NodeID};

pub fn serialise_files_all_years() {
    for year in 2016..2023 {
        serialise_files(year);
    }
}

pub fn serialise_sparse_node_values_2d_all_years() {
    for year in 2016..2023 {
        serialise_sparse_node_values_2d(year);
    }
}

fn serialise_sparse_node_values_2d(year: i32) {
    
    let inpath = format!("data/sparse_node_values_6am_{}_2d.json", year);
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output: Vec<Vec<[i32;2]>> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/sparse_node_values_6am_{}_2d.bin", year);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

pub fn serialise_files(year: i32) {
    let now = Instant::now();

    let padded_nodes_filename = format!("padded_node_values_6am_{}", year);
    serialise_list(&padded_nodes_filename);
    let len_graph_walk = serialise_graph_walk_vector(year);
    serialise_graph_pt_vector(year, len_graph_walk);
    serialise_node_values_padding_count(year);

    serialise_list_immutable_array_i8("subpurpose_purpose_lookup");
    serialise_list("travel_time_relationships_7");
    serialise_list("travel_time_relationships_10");
    serialise_list("travel_time_relationships_16");
    serialise_list("travel_time_relationships_19");
    println!("File serialisation year {}/tTook {:?}", year, now.elapsed());
}

fn serialise_node_values_padding_count(year: i32) {
    let contents_filename = format!("data/node_values_padding_row_count_6am_{}.json", year);
    let contents = fs_err::read_to_string(contents_filename).unwrap();
    let input_value: u32 = serde_json::from_str(&contents).unwrap();
    let filename = format!(
        "serialised_data/node_values_padding_row_count_6am_{}.bin",
        year
    );
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &input_value).unwrap();
}

fn serialise_graph_walk_vector(year: i32) -> usize {
    let contents_filename = format!("data/p1_main_nodes_list_6am_{}.json", year);
    let contents = fs_err::read_to_string(contents_filename).unwrap();

    let input: Vec<Vec<[usize; 2]>> = serde_json::from_str(&contents).unwrap();

    let mut graph_walk_vec = Vec::new();
    for input_edges in input.iter() {
        let mut edges: SmallVec<[EdgeWalk; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph_walk_vec.push(edges);
    }

    let filename = format!("serialised_data/p1_main_nodes_vector_6am_{}.bin", year);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_walk_vec).unwrap();
    return graph_walk_vec.len();
}

fn serialise_graph_pt_vector(year: i32, len_graph_walk: usize) {
    let contents_filename = format!("data/p2_main_nodes_list_6am_{}.json", year);
    let contents = fs_err::read_to_string(contents_filename).unwrap();

    let input: Vec<Vec<[usize; 2]>> = serde_json::from_str(&contents).unwrap();

    let mut graph_pt_vec = Vec::new();
    for input_edges in input.iter() {
        let mut edges: SmallVec<[EdgePT; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgePT {
                leavetime: LeavingTime(array[0] as u32),
                cost: Cost(array[1] as u16),
            });
        }
        graph_pt_vec.push(edges);
    }

    for _ in graph_pt_vec.len()..len_graph_walk {
        let edges: SmallVec<[EdgePT; 4]> = SmallVec::new();
        graph_pt_vec.push(edges);
    }
    assert!(graph_pt_vec.len() == len_graph_walk);

    let filename = format!("serialised_data/p2_main_nodes_vector_6am_{}.bin", year);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_pt_vec).unwrap();
}

fn serialise_list(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output: Vec<i32> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn serialise_list_immutable_array_i8(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: [i8; 32] = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}
