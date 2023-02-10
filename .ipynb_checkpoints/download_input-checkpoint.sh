#!/bin/bash

set -e

mkdir -p data
mkdir -p serialised_data

cd data
for x in padded_node_values_6am.json travel_time_relationships_10.json \
    travel_time_relationships_16.json travel_time_relationships_19.json \
    travel_time_relationships_7.json \
    p1_main_nodes_list_6am.json p2_main_nodes_list_6am.json \
	subpurpose_purpose_lookup.json number_of_destination_categories.json;
do
    wget https://storage.googleapis.com/hack-bucket-8204707942/$x
done

