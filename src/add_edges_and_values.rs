use std::sync::{Arc, Mutex, MutexGuard};
use actix_web::{get, post, web, App, HttpServer};
use crate::shared::{UserInputJSON};

//// current doesn't work. Not used due to the below either not having correct lifetimes
//// or the lifetime specified below messing up the lifetimes of node_values_1d_guard in main()

pub fn add_new_node_values(
    node_values_1d_guard: MutexGuard<'a, Vec<i32>>,
    input: &web::Json<UserInputJSON>,
) -> MutexGuard<Vec<i32>> {
    for new_build in &input.new_build_additions {
        let value_to_add = new_build[0];
        let index_of_nearest_node = new_build[1];
        let column_to_change = new_build[2];
        let ix = (index_of_nearest_node * 32) + column_to_change;
        node_values_1d_guard[ix as usize] += value_to_add;
    }
    return node_values_1d_guard
}

