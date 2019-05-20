import os
import shlex
import json
import time
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.mlab as mlab
from subprocess import *
from scipy.optimize import curve_fit
from scipy.misc import factorial


BMV2_PATH = "/home/vagrant/tutorials/vm/behavioral-model"


def poisson(k, lamb):
    return (lamb**k / factorial(k)) * np.exp(-lamb)

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

def matrix_gen_time():
	s = time.time()
	a = gen_traffic_matrix('traffic_example', 'cnt')
	return time.time() - s


if __name__ == '__main__':

	ingress_counts = []
	# collect ingress port counts once every second, run simulation for a couple min
	start = time.time()
	while time.time() < start + 180:
		s = time.time()
		ingress_counts.append(gen_traffic_matrix('traffic_example','cnt'))
		time.sleep(1 - (time.time() -  s))

	switch_links = [[i[0],i[1]] for i in json.load(open("topo.txt", 'r'))['links'] if 's' in i[0] and 's' in i[1]]
	sw_port_mapping = json.load(open("port_map.txt", 'r'))
	link_counts = {(i[0], i[1]) : [0 for j in xrange(len(ingress_counts))] for i in switch_links if 's' in i[0] and 's' in i[1] }
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
	link_util = {(i[0], i[1]) : [] for i in switch_links }
	for i in link_counts:
		link_util[i] = (np.array(link_counts[i])[1:] - np.array(link_counts[i])[:-1])

	for i in link_util:
		plt.plot(range(len(link_util[i])), link_util[i], label = "link %s" % str(i), marker='o' , )

	plt.title("Scatter Plot - Utilization over Time")
	plt.xlabel("Time (sec)")
	plt.ylabel("Link Utilization")
	plt.tight_layout()
	plt.legend()
	plt.show()
	plt.clf()

	# the bins should be of integer width, because poisson is an integer distribution
	# entries, bin_edges, patches = plt.hist(link_util[('s1','s2')], bins=11, range=[-0.5, 10.5], normed=True)
	
	# plt.figure(figsize=(3.2,4))
	# plot cumulative histogram of packet rate on link (s1','s2)
	entries, bin_edges, patches = plt.hist(link_util[('s1','s3')], bins=11, 
								range=[-0.5, max(link_util[('s1','s3')])], 
								normed=True, cumulative=True, label = "Observed",
								color = 'skyblue', ec="black")
	dx = 0.01
	X  = np.arange(0, max(link_util[('s1','s3')]), dx)
	Y  = poisson(X, np.mean(link_util[('s1','s3')]))
	# Normalize the data to a proper PDF
	Y /= (dx * Y).sum()
	# Compute the CDF
	CY = np.cumsum(Y * dx)
	# plot expected CDF of packet rate
	plt.plot(X, CY, 'k--', label = "Expected")
	# plt.title("Expected vs Observed CDF of Packet Rate on ('s1','s2')")
	plt.xlabel("Packets/sec", weight = 'bold')
	plt.ylabel("CDF", weight = 'bold')
	plt.legend(prop = dict(weight='bold'))
	plt.show()
	plt.clf()


	# calculate binmiddles
	bin_middles = 0.5*(bin_edges[1:] + bin_edges[:-1])
	# poisson function, parameter lamb is the fit parameter
	# fit with curve_fit
	parameters, cov_matrix = curve_fit(poisson, bin_middles, entries) 
	# plot poisson-deviation with fitted parameter
	x_plot = np.linspace(0, 20, 1000)
	plt.plot(x_plot, poisson(x_plot, *parameters), 'r-', lw=2)
	plt.show()
	plt.clf()


	m, s = stats.poisson.fit(link_util[('s1','s3')]) # get mean and standard deviation  
	pdf_g = stats.poisson.pdf(range(len(link_util[('s1','s3')])), m, s) # now get theoretical values in our interval  
	plt.plot(lnspc, pdf_g, label="Norm")
	plt.title("Packets/sec on Link ('s3','s4')")
	plt.xlabel("payout")
	plt.ylabel("normed frequency")
	plt.tight_layout()
	plt.grid([True])






