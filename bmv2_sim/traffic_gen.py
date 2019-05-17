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

    addrs = [socket.gethostbyname(i) for i in ['10.0.1.1', '10.0.2.2', '10.0.3.3', '10.0.4.4', '10.0.1.5']]
    srcPort = int(sys.argv[1])    
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.bind(('', srcPort))
    start = time.time()
    # run simulation for 2 min
    while time.time() < start + 120:
        try:
            # sending packts at a rate of 2 packets per sec, i.e. a mean inter-arrival time
            # of 0.5 sec. 
            data = struct.pack('>I', 0)
            time.sleep(np.random.exponential(0.5))
            for i in addrs:
                s.sendto(data, (i, DPORT))

        except Exception as error:
            print error

if __name__ == '__main__':
    main()

