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

    if len(sys.argv)<3:
        print 'pass 2 arguments: <destination IP> <srcPort>'
        exit(1)

    addr = socket.gethostbyname(sys.argv[1])
    srcPort = int(sys.argv[2])    
    iface = get_if()
    print "sending on interface %s to %s" % (iface, str(addr))



    nhops = [0]

    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.bind(('', srcPort))
    count = 0
    for t in xrange(1): 
        try:
            data = struct.pack('>I', nhops[t])
            s.sendto(data, (addr, DPORT))
            count = count + 1
            time.sleep(0.01)
            print(nhops[t])

        except Exception as error:
            print error


if __name__ == '__main__':
    main()

