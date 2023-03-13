

while [ "$(curl http://0.0.0.0:7328/)" != "App is listening" ]; do sleep 1; done


for YEAR in 2016 2017 2018 2019 2020 2021 2022
do
    wget -O- --post-data='{"start_nodes_user_input": [4000000, 4000001, 4000002], "init_travel_times_user_input": [16, 10, 10], "trip_start_seconds": 28800, "graph_walk_additions": [], "graph_pt_additions": [], "new_nodes_count": 0, "graph_walk_updates_keys": [], "graph_walk_updates_additions": [], "year": '$YEAR', "new_build_additions": [], "target_destinations": []}' \
      --header='Content-Type:application/json' \
      'http://0.0.0.0:7328/floodfill_pt/'
done


