import json

with open("topo.txt", 'r') as f:
	topo = json.load(f)

with open("topology.json", 'r') as f:
	sn_loc = json.load(f)

data1 = {}
data2 = {}

if type(sn_loc['snippet_loc']) == unicode:
	for i in topo['switches']:
		data1[i] = { "runtime_json" : "%s-runtime.json" % i, 
			"cli_input" : "commands/%s.txt" % sn_loc['snippet_loc'] }
		data2[i] = sn_loc['snippet_loc']
	topo['snippet_loc'] = data2

else:
	for i in topo['switches']:
		data1[i] = { "runtime_json" : "%s-runtime.json" % i, 
			"cli_input" : "commands/%s.txt" % sn_loc['snippet_loc'][i] }
 		topo['snippet_loc'] = sn_loc['snippet_loc']

topo['switches'] = data1

with open("topology.json", 'w') as f:
    json.dump(topo, f)
