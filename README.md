# Notes on running

To get started:

1. Run `./download_input.sh` once to download input data

2. Flip the `if false` part of `serialise_files` and `create_graph_walk_len` in `main.rs` to `true` so the files are serialised

3. Run with`cargo run --release`

For subsequent runs, you can flip the `if false` parts of `main.rs` back to `false` to speed up load time.

The current version hosts an API, which accepts start node IDs and initial travel times. It requires about 3gb of RAM and loads in 3s on our GCE instance.


# On querying the API

Check it's listening:
```
curl http://0.0.0.0:7328/
```

Run PT algorithm on 3 start nodes: 
```
wget -O- --post-data='{"start_nodes_user_input": [9380647, 9183046, 2420336], "init_travel_times_user_input": [16, 10, 10], "trip_start_seconds": 28800, "graph_walk_additions": [], "graph_pt_additions": [], "new_nodes_count": 0, "graph_walk_updates_keys": [], "graph_walk_updates_additions": [], "year": 2022, "new_build_additions": [], "target_destinations": []}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/'
```

Run PT algorithm on 1000 start nodes using 2022 network: 
```
wget --post-file="example_payload_1000_start_nodes.json" \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/'
```


Run PT algorithm on 1000 start nodes using 2019 network: 
```
wget --post-file="example_payload_1000_start_nodes_2019.json" \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/'
```

# Deploying with Docker

To make and run docker image (2022 only)
```
# Takes about 5 minutes to build
docker build --progress=plain -t rust_connectivity:latest .
docker run -p 0.0.0.0:7328:7328 rust_connectivity:latest
```

To push build image to dockerhub (2022 only)
```
docker tag connectivity_rust:latest adambricknell/connectivity_rust
docker push adambricknell/connectivity_rust
```

To pull from dockerhub and deploy with Cloud Run (2022 only)
```
docker pull adambricknell/connectivity_rust:latest

docker tag adambricknell/connectivity_rust gcr.io/dft-dst-prt-connectivitymetric/adambricknell/connectivity_rust:latest

docker push gcr.io/dft-dst-prt-connectivitymetric/adambricknell/connectivity_rust:latest

gcloud run deploy connectivity-rust \ 
  --image=gcr.io/dft-dst-prt-connectivitymetric/adambricknell/connectivity_rust:latest \
  --allow-unauthenticated \
  --port 7328 --cpu 8 --memory 8G --quiet
```
