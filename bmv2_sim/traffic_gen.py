#!/usr/bin/env python

import argparse
import sys
import socket
import random
import struct
import re
import readline
import numpy as np
import time
import string

from scapy.all import sendp, send, srp1, get_if_list, get_if_hwaddr
from scapy.all import Packet, hexdump
from scapy.all import Ether, IP, UDP, ShortField, StrFixedLenField, XByteField, IntField
from scapy.all import bind_layers
from scapy.config import conf

DPORT = 0x04d2 # MY_PORT 0x04d2 

def get_if():
    ifs=get_if_list()
    iface=None # "h1-eth0"
    for i in get_if_list():
        if "eth0" in i:
            iface=i
            break;
    if not iface:
        print "Cannot find eth0 interface"
        exit(1)
    return iface

def main():

    if len(sys.argv)<2:
        print 'pass 1 arguments: <srcPort>'
        exit(1)

    # TODO : write ip addrs to a tmp file and read from that file here (put
    # in same file as thrift_port)
    this_host = ''.join(re.findall(r'h\d+-', get_if()))[:-1]
    host_num = ''.join(re.findall(r'\d+', this_host))
    my_ip = "10.0.%s.%s" % (host_num, host_num)
    addrs = [socket.gethostbyname(i) for i in ['10.0.1.1', '10.0.2.2', '10.0.3.3'] if i != my_ip]
    srcPort = int(sys.argv[1])    
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(('', srcPort))

    while True:
        try:
            # sending packts at a rate of 2 packets per sec, i.e. a mean inter-arrival time
            # of 0.5 sec. 
            data = struct.pack('>I', 0)
            start = time.time()
            s.sendto(data, (addrs[0], DPORT))
            time.sleep(np.random.exponential(0.5))
            s.sendto(data, (addrs[1], DPORT))
            time.sleep(np.random.exponential(0.5))

            # time.sleep(np.random.exponential(1) - (time.time() - start))

        except Exception as error:
            print error

if __name__ == '__main__':
    main()

