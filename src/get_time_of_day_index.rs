pub fn get_time_of_day_index(trip_start_seconds: i32) -> usize {
    let mut time_of_day_ix = 0;
    if trip_start_seconds > 3600 * 10 {
        time_of_day_ix = 1;
    }
    if trip_start_seconds > 3600 * 16 {
        time_of_day_ix = 2;
    }
    if trip_start_seconds > 3600 * 19 {
        time_of_day_ix = 3;
    }
    time_of_day_ix as usize
}
