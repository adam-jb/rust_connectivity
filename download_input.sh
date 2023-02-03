#!/bin/bash

set -e

mkdir -p data
mkdir -p serialised_data

cd data
for x in start_nodes.json init_travel_times.json p1_main_nodes.json \
	p2_main_nodes.json node_values.json travel_time_relationships.json \
	subpurpose_purpose_lookup.json number_of_destination_categories.json;
do
    wget https://storage.googleapis.com/hack-bucket-8204707942/$x
done
