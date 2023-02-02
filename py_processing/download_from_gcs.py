import requests

def get_and_save_file(filename):
    
    fullpath = 'https://storage.googleapis.com/hack-bucket-8204707942/' + filename
    txt = requests.get(fullpath).text
    with open(f'../data/{filename}', 'w') as w:
        w.write(txt)
    print(f'saved {filename}')



files_to_get = [
    'start_nodes.json',
    'init_travel_times.json',
    'p1_main_nodes.json',
    'p2_main_nodes.json',
    'node_values.json',
    'travel_time_relationships.json',
    'subpurpose_purpose_lookup.json',
    'number_of_destination_categories.json',
]

for filename in files_to_get:
    get_and_save_file(filename)



