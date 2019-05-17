import os
import shlex
import json
import time
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.mlab as mlab
from subprocess import *

BMV2_PATH = "/home/vagrant/tutorials/vm/behavioral-model"

def gen_traffic_matrix(snippet_name, reg_name):
	with open('thrift_port.json', 'r') as f:
		thrift_ports = json.load(f)
	CLI_PATH = "%s/targets/simple_switch/sswitch_CLI" % BMV2_PATH
	traffic_mat = {}
	for sw_name in thrift_ports.keys():
		p1 = Popen(['echo', 'register_read %s' % (reg_name)], stdout=PIPE)
		p2 = Popen(shlex.split("%s %s.json %d" % (CLI_PATH, snippet_name, 
			thrift_ports[sw_name])), stdin=p1.stdout, stdout=PIPE)
		p1.stdout.close()
		output = p2.communicate()[0]
		traffic_mat[sw_name] = map(int, output.split('=')[1][1:-14].split(', '))
	return traffic_mat


if __name__ == '__main__':
	# collect ingress port counts once a second
	ingress_counts = []
	while True:
		ingress_counts.append(gen_traffic_matrix('traffic_example','cnt'))
		time.sleep(1)

	link_counts = {(i['node1'], i['node2']) : [0 for j in xrange(len(ingress_counts))] for i in switch_links }
	c = 0
	for ing in ingress_counts:
		for l in link_counts:
			for i in sw_port_mapping[l[0]]:
				if i[1] == l[1]:
					link_counts[l][c] += ing[l[0]][i[0]]
			for i in sw_port_mapping[l[1]]:
				if i[1] == l[0]:
					link_counts[l][c] += ing[l[1]][i[0]]
		c += 1

	# get the number of packets travelling on each link, for each time step, and plot
	plt.title("Link Utilization over Time")
	plt.xlabel("Time (sec")
	plt.ylabel("Link Utilization")
	plt.tight_layout()
	link_util = {(i['node1'], i['node2']) : [] for i in switch_links }

	for i in link_counts:
		link_util[i] = (np.array(link_counts[i])[1:] - np.array(link_counts[i])[:-1])

	for i in link_util:
		plt.plot(range(len(link_util[i])), link_util[i], label = "link %s" % str(i), marker='o' )

	plt.legend()
	plt.show()








