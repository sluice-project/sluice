import json

with open("topo.txt", 'r') as f:
	topo = json.load(f)

with open("topology.json", 'r') as f:
	sn_loc = json.load(f)

data = {}
for i in topo['switches']:
	data[i] = { "runtime_json" : "%s-runtime.json" % i, 
		"cli_input" : "commands/%s.txt" % sn_loc['snippet_loc'][i] }
 
topo['switches'] = data
topo['snippet_loc'] = sn_loc['snippet_loc']

with open("topology.json", 'w') as f:
    json.dump(topo, f)
