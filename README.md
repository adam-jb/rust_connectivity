# Notes on running

Run `./download_input.sh` once to download input data. Flip the `if false` part to `serialise_files` in `main.rs`, and then `cargo run --release`

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

Run PT algorithm on 1000 start nodes using 2022 network: 
```
wget --post-file="example_payload_1000_start_nodes.json" \
  --header='Content-Type:application/json' \
  'http://127.0.0.1:7328/floodfill_pt/'
```


Run PT algorithm on 1000 start nodes using 2019 network: 
```
wget --post-file="example_payload_1000_start_nodes_2019.json" \
  --header='Content-Type:application/json' \
  'http://127.0.0.1:7328/floodfill_pt/'
```


# To make and run docker image (2022 only)

Takes about 5 minutes to build

```
docker build --progress=plain -t rust_connectivity:deployment .
docker run -p 127.0.0.1:7328:7328 rust_connectivity:deployment
```


# To deploy with Cloud Run in GCP Cloud Shell (2022 only)
```
gcloud config set run/region europe-west2
gcloud run deploy rust-connectivity --port=7328 --cpu=4 --memory=8Gi --quiet --source .
```

