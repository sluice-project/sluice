#define ETHERTYPE_IPV4 0x0800
#define IP_PROTOCOLS_TCP 6
#define IP_PROTOCOLS_UDP 17
#define IP_PROTOCOLS_TCP 6
header_type ethernet_t {
    fields {
        dstAddr : 48;
        srcAddr : 48;
        etherType : 16;
    }
}
header_type ipv4_t {
    fields {
        version : 4;
        ihl : 4;
        diffserv : 8;
        totalLen : 16;
        identification : 16;
        flags : 3;
        fragOffset : 13;
        ttl : 8;
        protocol : 8;
        hdrChecksum : 16;
        srcAddr : 32;
        dstAddr: 32;
    }
}
header_type tcp_t {
    fields {
        srcPort : 16;
        dstPort : 16;
        seqNo : 32;
        ackNo : 32;
        dataOffset : 4;
        res : 4;
        flags : 8;
        window : 16;
        checksum : 16;
        urgentPtr : 16;
    }
}
header_type udp_t {
    fields {
        srcPort : 16;
        dstPort : 16;
        len : 16;
        checksum : 16;
    }
}
header_type n_t {
    fields {
        new_one : 32;
    }
}
header n_t n;
header ethernet_t ethernet;
header ipv4_t ipv4;
header tcp_t tcp;
header udp_t udp;
parser start {
    return parse_ethernet;
}
parser parse_ethernet {
        return select(latest.etherType) {
            ETHERTYPE_IPV4 : parse_ipv4;
        Value { value: 1234 } : parse_n;
}
}
parser parse_ipv4 {
    extract(ipv4);
    return select(latest.protocol) {
        IP_PROTOCOLS_TCP : parse_tcp;
        IP_PROTOCOLS_UDP : parse_udp;
        default: ingress;
    }
}
parser parse_tcp {
    extract(tcp);
    return ingress;
}
parser parse_udp {
    extract(udp);
    return ingress;
}
header_type metadata_t { 
    fields {
        z : 1;
        r : 32;
        q : 32;
        m : 32;
        l : 32;
        i : 32;
        reg1 : 32;
        reg2 : 32;
        reg3 : 32;
        if_block_tmp_2 : 1;
        tmp_0_if_2 : 32;
        tmp_1_if_3 : 32;
    }
}
metadata metadata_t mdata;
register reg1 {
     width : 32; 
     instance_count : 1;
}
register reg2 {
     width : 32; 
     instance_count : 1;
}
register reg3 {
     width : 32; 
     instance_count : 1;
}
action action1 () {
    modify_field(mdata.q, 10);
}
action action2 () {
    modify_field(mdata.r, 5);
}
action action3 () {
    modify_field(mdata.if_block_tmp_2, 1); 
}
action action4 () {
    modify_field(mdata.if_block_tmp_2, 0); 
}
action action5 () {
action action7 () {
    register_read(mdata.reg3, reg3, 0);
}
    modify_field(mdata.l, mdata.reg3);
}
action action6 () {
    modify_field(mdata.l, mdata.l);
}
action action8 () {
    add(mdata.tmp_0_if_2,mdata.q,mdata.l);
}
action action9 () {
    modify_field(mdata.i, mdata.tmp_0_if_2);
}
action action10 () {
    modify_field(mdata.i, mdata.i);
}
action action11 () {
    modify_field(mdata.l, mdata.l);
}
action action12 () {
action action13 () {
    register_read(mdata.reg1, reg1, 0);
}
    modify_field(mdata.l, mdata.reg1);
}
action action14 () {
    subtract(mdata.tmp_1_if_3,mdata.q,mdata.l);
}
action action15 () {
    modify_field(mdata.i, mdata.i);
}
action action16 () {
    modify_field(mdata.i, mdata.tmp_1_if_3);
}
action action17 () {
    register_write(reg1, 11, 0);
}
action action18 () {
    modify_field(mdata.z, 1); 
}
action action19 () {
    modify_field(mdata.z, 0); 
}
action action20 () {
    modify_field(mdata.m, mdata.q);
}
action action21 () {
    modify_field(mdata.m, mdata.r);
}
action action22 () {
    add(mdata.reg2,mdata.i,5);
    register_write(reg2, 0, mdata.reg2);
}
table table1 {
    actions {
        action1;
    }
}
table table2 {
    actions {
        action2;
    }
}
table table3 {
    actions {
        action3;
    }
}
table table4 {
    actions {
        action4;
    }
}
table table5 {
    actions {
        action7;
    }
}
table table6 {
    reads {
        mdata.if_block_tmp_2 : exact;
    }
    actions {
        action5;
        action6;
    }
}
table table7 {
    actions {
        action8;
    }
}
table table8 {
    reads {
        mdata.if_block_tmp_2 : exact;
    }
    actions {
        action9;
        action10;
    }
}
table table9 {
    actions {
        action13;
    }
}
table table10 {
    reads {
        mdata.if_block_tmp_2 : exact;
    }
    actions {
        action11;
        action12;
    }
}
table table11 {
    actions {
        action14;
    }
}
table table12 {
    reads {
        mdata.if_block_tmp_2 : exact;
    }
    actions {
        action15;
        action16;
    }
}
table table13 {
    actions {
        action17;
    }
}
table table14 {
    actions {
        action18;
    }
}
table table15 {
    actions {
        action19;
    }
}
table table16 {
    reads {
        mdata.z : exact;
    }
    actions {
        action20;
        action21;
    }
}
table table17 {
    actions {
        action22;
    }
}
control ingress {
    apply(table1);
    apply(table2);
    if (mdata.q > mdata.r) {
        apply(table3);
    } else {
        apply(table4);
    }
    apply(table5);
    apply(table5);
    apply(table7);
    apply(table8);
    apply(table9);
    apply(table9);
    apply(table11);
    apply(table12);
    apply(table13);
    if (mdata.q >= 10) {
        apply(table14);
    } else {
        apply(table15);
    }
    apply(table16);
    apply(table17);
}
control egress {
}
