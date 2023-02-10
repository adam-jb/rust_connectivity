use smallvec::SmallVec;
use std::time::Instant;

use fs_err::File;
use std::io::BufWriter;

use crate::shared::{Cost, EdgePT, EdgeWalk, GraphPT, GraphWalk, LeavingTime, NodeID};

pub fn serialise_files() {
    let now = Instant::now();
    serialise_list_immutable_array_i8("subpurpose_purpose_lookup");
    serialise_list("start_nodes");
    serialise_list("init_travel_times");
    serialise_graph_walk_vector(); //serialise_graph_walk();
    serialise_graph_pt_vector(); //serialise_graph_pt();
    serialise_list("padded_node_values_8am");
    serialise_list("travel_time_relationships");
    println!("File serialisation took {:?}", now.elapsed());
}

fn serialise_graph_walk_vector() {
    let contents = fs_err::read_to_string("data/p1_main_nodes_list_8am.json").unwrap();

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

    let file =
        BufWriter::new(File::create("serialised_data/p1_main_nodes_vector_8am.bin").unwrap());
    bincode::serialize_into(file, &graph_walk_vec).unwrap();
}

fn serialise_graph_pt_vector() {
    let contents = fs_err::read_to_string("data/p2_main_nodes_list_8am.json").unwrap();

    // to do: check meaning of the '2' in [usize; 2]
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

    let file =
        BufWriter::new(File::create("serialised_data/p2_main_nodes_vector_8am.bin").unwrap());
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
