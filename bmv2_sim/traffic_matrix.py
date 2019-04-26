import os
import shlex
import json
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


