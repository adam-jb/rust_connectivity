# Notes on running

Run `./download_input.sh` once to download input data. Uncomment serialise_files() in main.rs to run

Run `./download_input_mac.sh` if on a mac

The current version hosts an API, which accepts start node IDs and initial travel times. It requires about 3gb of RAM and loads in 3s on our GCE instance.


# On querying the API

Check it's listening:
```
curl http://127.0.0.1:7328/
```

Run PT algorithm on 3 start nodes: 
```
wget -O- --post-data='{"start_nodes_user_input": [9380647, 9183046, 2420336], "init_travel_times_user_input": [16, 10, 10], "trip_start_seconds": 28800, "graph_walk_additions": [], "graph_pt_additions": [], "new_nodes_count": 0, "graph_walk_updates_keys": [], "graph_walk_updates_additions": [], "year": 2022, "new_build_additions": [], "target_destinations": []}' \
  --header='Content-Type:application/json' \
  'http://127.0.0.1:7328/floodfill_pt/'
```



