



/// todo: make creation of node_values_1d part of serialisation (so it's only run once)
fn get_node_values_1d() -> Vec<i32> {
    let node_values: Vec<Vec<i32>> = deserialize_bincoded_file("node_values");
    let mut node_values_1d: Vec<i32> = Vec::new();
    for node_vec in &node_values {
        for specific_val in node_vec {
            node_values_1d.push(*specific_val);
        }
    }
    node_values_1d
}

fn deserialize_bincoded_file<T: DeserializeOwned>(filename: &str) -> T {
    let path = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(path).unwrap());
    bincode::deserialize_from(file).unwrap()
}

