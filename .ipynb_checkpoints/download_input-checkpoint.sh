#!/bin/bash

set -e

mkdir -p data
mkdir -p serialised_data

cd data
for x in start_nodes.json init_travel_times.json \
	padded_node_values_8am.json travel_time_relationships.json \
    p1_main_nodes_list_8am.json p2_main_nodes_list_8am.json \
	subpurpose_purpose_lookup.json number_of_destination_categories.json;
do
    wget https://storage.googleapis.com/hack-bucket-8204707942/$x
done

