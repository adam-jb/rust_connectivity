use std::collections::{BinaryHeap, HashSet};
use std::time::Instant;

use crate::priority_queue::PriorityQueueItem;
use crate::shared::{Cost, EdgePT, EdgeWalk, GraphPT, GraphWalk, NodeID};
use smallvec::SmallVec;

pub fn floodfill(
    (
        graph_walk,
        start,
        node_values_1d,
        travel_time_relationships,
        subpurpose_purpose_lookup,
        graph_pt,
        trip_start_seconds,
        init_travel_time,
    ): (
        &Vec<SmallVec<[EdgeWalk; 4]>>, //&GraphWalk,
        NodeID,
        &Vec<i32>,
        &Vec<i32>,
        &[i8; 32],
        &Vec<SmallVec<[EdgePT; 4]>>, //&GraphPT,
        i32,
        Cost,
    ),
) -> (i32, u32, [i64; 32]) {
    let time_limit: Cost = Cost(3600);
    let subpurposes_count: usize = 32 as usize;
    let now = Instant::now();

    // 74444736 is calculated and stored in GCS: will be diff for each time of day as the contiguous
    // network will have a different number of nodes with active PT routes for each time of day
    // https://storage.googleapis.com/hack-bucket-8204707942/node_values_padding_row_count_8am.json
    let offset_nodes_no_value = 74444736 as u32;
    let count_nodes_no_value = offset_nodes_no_value / 32;

    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: init_travel_time,
        value: start,
    });
    let mut nodes_visited = HashSet::new();
    let mut total_iters = 0;
    let mut pt_iters = 0;

    let mut scores: [i64; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];

    while let Some(current) = queue.pop() {
        if nodes_visited.contains(&current.value) {
            continue;
        }

        nodes_visited.insert(current.value);

        // if the node id is not a p2 node (ie, over that reference ID), then it will have an associated value
        if current.value.0 >= count_nodes_no_value {
            get_scores(
                current.value.0,
                &node_values_1d,
                current.cost.0,
                travel_time_relationships,
                subpurpose_purpose_lookup,
                subpurposes_count,
                &mut scores,
            );
        }

        // Finding adjacent walk nodes
        // skip 1st edge as it has info on whether node also has a PT service
        //for edge in &graph_walk.edges_per_node[(current.value.0 as usize)][1..] {
        for edge in &graph_walk[(current.value.0 as usize)][1..] {
            let new_cost = Cost(current.cost.0 + edge.cost.0);
            if new_cost < time_limit {
                queue.push(PriorityQueueItem {
                    cost: new_cost, //Cost(current.cost.0 + edge.cost.0),
                    value: edge.to,
                });
            }
        }

        // if node has a timetable associated with it: the first value in the first 'edge'
        // will be 1 if it does, and 0 if it doesn't
        //if graph_walk.edges_per_node[(current.value.0 as usize)][0].cost == Cost(1) {
        if graph_walk[(current.value.0 as usize)][0].cost == Cost(1) {
            get_pt_connections(
                &graph_pt,
                current.cost.0,
                &mut queue,
                time_limit,
                trip_start_seconds,
                &current.value,
                &mut pt_iters, 
            );
        }

        total_iters += 1;
    }
    println!("total_iters: {}\tpt_iters: {}\tstart {}\t{:?}", total_iters, pt_iters, start.0, now.elapsed());

    return (total_iters, start.0, scores);
}

fn get_scores(
    node_id: u32,
    node_values_1d: &Vec<i32>,
    time_so_far: u16,
    travel_time_relationships: &Vec<i32>,
    subpurpose_purpose_lookup: &[i8; 32],
    subpurposes_count: usize,
    scores: &mut [i64; 32],
) {
    // to subset node_values_1d
    let start_pos = node_id * 32;

    // 32 subpurposes
    for i in 0..subpurposes_count {
        let vec_start_pos_this_purpose = (subpurpose_purpose_lookup[(i as usize)] as i32) * 4105;
        let multiplier =
            travel_time_relationships[(vec_start_pos_this_purpose + time_so_far as i32) as usize];

        // this line could be faster, eg if node_values_1d was an array
        scores[i] += (node_values_1d[(start_pos as usize) + i] * multiplier) as i64;
    }
}

fn get_pt_connections(
    graph_pt: &Vec<SmallVec<[EdgePT; 4]>>, //&GraphPT,
    time_so_far: u16,
    queue: &mut BinaryHeap<PriorityQueueItem<Cost, NodeID>>,
    time_limit: Cost,
    trip_start_seconds: i32,
    current_node: &NodeID,
    pt_iters: &mut i32,
) {
    // find time node is arrived at in seconds past midnight
    let time_of_arrival_current_node = trip_start_seconds as u32 + time_so_far as u32;

    // find time next service leaves
    let mut found_next_service = 0;
    let mut journey_time: u32 = 0;
    let mut next_leaving_time = 0;
    //for edge in &graph_pt.edges_per_node[(current_node.0 as usize)][1..] {
    for edge in &graph_pt[(current_node.0 as usize)][1..] {
        if time_of_arrival_current_node <= edge.cost.0 as u32 {
            next_leaving_time = edge.cost.0;
            journey_time = edge.leavetime.0 as u32;
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
            //// Notice this uses 'leavingTime' as first 'edge' for each node stores ID
            //// of next node: this is legacy from our matrix-based approach in python
            //let destination_node = &graph_pt.edges_per_node[(current_node.0 as usize)][0]
            let destination_node = &graph_pt[(current_node.0 as usize)][0].leavetime.0;

            queue.push(PriorityQueueItem {
                cost: Cost(arrival_time_next_stop as u16),
                value: NodeID(*destination_node as u32),
            });
            
            *pt_iters += 1;
        };
    }
}
