#!/bin/bash

set -e

mkdir -p data
mkdir -p serialised_data

cd data
for x in travel_time_relationships_10.json \
    travel_time_relationships_16.json travel_time_relationships_19.json \
    travel_time_relationships_7.json \
    subpurpose_purpose_lookup.json number_of_destination_categories.json;
do
    wget https://storage.googleapis.com/hack-bucket-8204707942/$x
done

for YEAR in 2016 2017 2018 2019 2020 2021 2022
do
    for x in p1_main_nodes_list_6am_$YEAR.json \
        p2_main_nodes_list_6am_$YEAR.json \
        padded_node_values_6am_$YEAR.json \
        sparse_node_values_6am_${YEAR}_2d.json \
        node_values_padding_row_count_6am_$YEAR.json;
    do
        wget https://storage.googleapis.com/hack-bucket-8204707942/$x
    done
done
