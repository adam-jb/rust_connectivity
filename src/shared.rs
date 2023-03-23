use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct NodeID(pub u32);

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Cost(pub u16);

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct LeavingTime(pub u32);

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct EdgeWalk {
    pub to: NodeID,
    pub cost: Cost,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct EdgePT {
    pub leavetime: LeavingTime,
    pub cost: Cost,
}

#[derive(Deserialize)]
pub struct UserInputJSON {
    pub start_nodes_user_input: Vec<i32>,
    pub init_travel_times_user_input: Vec<i32>,
    pub trip_start_seconds: i32,
    pub graph_walk_additions: Vec<Vec<[usize; 2]>>,
    pub graph_pt_additions: Vec<Vec<[usize; 2]>>,
    pub new_nodes_count: usize,
    pub graph_walk_updates_keys: Vec<usize>,
    pub graph_walk_updates_additions: Vec<Vec<[usize; 2]>>,
    pub year: i32,
    pub new_build_additions: Vec<Vec<i32>>,
    pub target_destinations: Vec<u32>,
    pub max_travel_time: u16,
    pub return_all_destinations: bool,
}
