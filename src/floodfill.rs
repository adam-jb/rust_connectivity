use std::collections::BinaryHeap;
use crate::priority_queue::PriorityQueueItem;
use crate::shared::{Cost, EdgePT, EdgeWalk, NodeID};
use smallvec::SmallVec;

pub fn get_travel_times(
    graph_walk: &Vec<SmallVec<[EdgeWalk; 4]>>,
    graph_pt: &Vec<SmallVec<[EdgePT; 4]>>,
    start: NodeID,
    trip_start_seconds: i32,
    init_travel_time: Cost,
) -> (u32, Vec<u32>, Vec<u16>) {
    
    let time_limit: Cost = Cost(3600);

    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: init_travel_time,
        value: start,
    });
    let mut nodes_visited = vec![false; graph_walk.len()];
    //let mut nodes_visited = HashSet::new();
    let mut destination_ids: Vec<u32> = vec![];
    let mut destination_travel_times: Vec<u16> = vec![];


    // catch where start node is over an hour from centroid
    if init_travel_time >= Cost(3600) {
        return (
            start.0,
            destination_ids,
            destination_travel_times,
        );
    }

    while let Some(current) = queue.pop() {
        
        if nodes_visited[current.value.0 as usize] {
        //if nodes_visited.contains(&current.value) {
            continue;
        }

        destination_ids.push(current.value.0);
        destination_travel_times.push(current.cost.0);

        nodes_visited[current.value.0 as usize] = true;
        //nodes_visited.insert(current.value);

        // Finding adjacent walk nodes
        // skip 1st edge as it has info on whether node also has a PT service
        for edge in &graph_walk[(current.value.0 as usize)][1..] {
            let new_cost = Cost(current.cost.0 + edge.cost.0);
            if new_cost < time_limit {
                queue.push(PriorityQueueItem {
                    cost: new_cost,
                    value: edge.to,
                });
            }
        }

        // if node has a timetable associated with it: the first value in the first 'edge'
        // will be 1 if it does, and 0 if it doesn't
        if graph_walk[(current.value.0 as usize)][0].cost == Cost(1) {
            get_pt_connections(
                &graph_pt,
                current.cost.0,
                &mut queue,
                time_limit,
                trip_start_seconds,
                &current.value,
            );
        }

    }
    return (
        start.0,
        destination_ids,
        destination_travel_times,
    );
}


fn get_pt_connections(
    graph_pt: &Vec<SmallVec<[EdgePT; 4]>>,
    time_so_far: u16,
    queue: &mut BinaryHeap<PriorityQueueItem<Cost, NodeID>>,
    time_limit: Cost,
    trip_start_seconds: i32,
    current_node: &NodeID,
) {
    // find time node is arrived at in seconds past midnight
    let time_of_arrival_current_node = trip_start_seconds as u32 + time_so_far as u32;

    // find time next service leaves
    let mut found_next_service = 0;
    let mut journey_time: u16 = 0;
    let mut next_leaving_time = 0;

    for edge in &graph_pt[(current_node.0 as usize)][1..] {
        if time_of_arrival_current_node <= edge.leavetime.0 as u32 {
            next_leaving_time = edge.leavetime.0;
            journey_time = edge.cost.0;
            found_next_service = 1;
            break;
        }
    }

    // add to queue
    if found_next_service == 1 {
        let wait_time_this_stop = next_leaving_time as u32 - time_of_arrival_current_node;
        let arrival_time_next_stop =
            time_so_far as u32 + wait_time_this_stop as u32 + journey_time as u32;

        if arrival_time_next_stop < time_limit.0 as u32 {
            //// Notice this uses 'leavingTime' from first 'edge' for the ID
            //// of next node: this is legacy from our matrix-based approach in python
            let destination_node = &graph_pt[(current_node.0 as usize)][0].leavetime.0;

            queue.push(PriorityQueueItem {
                cost: Cost(arrival_time_next_stop as u16),
                value: NodeID(*destination_node as u32),
            });
        };
    }
}



pub fn get_all_scores_and_time_to_target_destinations(
    travel_times: &(u32, Vec<u32>, Vec<u16>), // nodeID, destination node IDs, travel times to destinations
    node_values_1d: &[i32], //&Vec<i32>,
    travel_time_relationships: &[i32], //&Vec<i32>,
    subpurpose_purpose_lookup: &[i8; 32],
    count_original_nodes: u32,
    node_values_padding_row_count: u32,
    target_destinations_vector: &[u32], //&Vec<u32>,
) -> (i32, u32, [i64; 32], Vec<u32>, Vec<u16>) {

    let subpurposes_count: usize = 32;
    let count_nodes_no_value = node_values_padding_row_count / 32;
    
    // replacing set below with binary vec for faster lookup than set. Assumes only original nodes can be target destinations
    let mut target_destinations_binary_vec = vec![false; count_original_nodes as usize];
    for id in target_destinations_vector.into_iter() {
        target_destinations_binary_vec[*id as usize] = true;
    }
    //let target_destinations_set: HashSet<u32> = target_destinations_vector.iter().cloned().collect();
    // replacing the 4 lines below with the line above on the advice of gpt4
    /*let mut target_destinations_set: HashSet<u32> = HashSet::new();
    for node_id in target_destinations_vector {
        target_destinations_set.insert(*node_id);
    }*/
    
    let mut scores: [i64; 32] = [0; 32];
    
    let mut target_destination_ids: Vec<u32> = vec![];
    let mut target_destination_travel_times: Vec<u16> = vec![];
    
    let start = travel_times.0;
    
    let destination_ids = &travel_times.1;
    let destination_travel_times = &travel_times.2;
    
    for (&current_node, &current_cost) in destination_ids.iter().zip(destination_travel_times) {
    // replaced 3 lines below with the one above on advice of gpt4
    //for i in 0..destination_ids.len() {
        //let current_node = destination_ids[i];
        //let current_cost = destination_travel_times[i];
        
        // if the node id is not a p2 node (ie, above count_nodes_no_value), then it will have an associated value
        if count_original_nodes >= current_node && current_node >= count_nodes_no_value {

            // this replaces get_scores()
            let start_pos = current_node * 32;
            for i in 0..subpurposes_count {
                let vec_start_pos_this_purpose = (subpurpose_purpose_lookup[i] as i32) * 3601;
                let multiplier = travel_time_relationships[(vec_start_pos_this_purpose + current_cost as i32) as usize];
                scores[i] += (node_values_1d[(start_pos as usize) + i] as i64) * (multiplier as i64);
            }            
        }
            
        if target_destinations_binary_vec[current_node as usize] {
        //if target_destinations_set.contains(&current_node) {
            target_destination_ids.push(current_node);
            target_destination_travel_times.push(current_cost);
        }
    }
    
    return (
        travel_times.1.len() as i32,
        start,
        scores,
        target_destination_ids,
        target_destination_travel_times,
    );

}

