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
    extract(ethernet);
    return select(latest.etherType) {
        ETHERTYPE_IPV4 : parse_ipv4;
        default: ingress;
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
    return select(latest.srcPort) {
        1234 : parse_n;
        default: ingress;
    }
}

parser parse_n {
    extract(n);
    return ingress;
}


field_list ipv4_checksum_list {
        ipv4.version;
        ipv4.ihl;
        ipv4.diffserv;
        ipv4.totalLen;
        ipv4.identification;
        ipv4.flags;
        ipv4.fragOffset;
        ipv4.ttl;
        ipv4.protocol;
        ipv4.srcAddr;
        ipv4.dstAddr;
}

field_list_calculation ipv4_checksum {
    input {
        ipv4_checksum_list;
    }
    algorithm : csum16;
    output_width : 16;
}
calculated_field ipv4.hdrChecksum  {
    verify ipv4_checksum;
    update ipv4_checksum;
}


action _drop() {
    drop();
}

action ipv4_forward(dstAddr, port) {
    modify_field(standard_metadata.egress_spec, port);
    modify_field(ethernet.srcAddr, ethernet.dstAddr);
    modify_field(ethernet.dstAddr, dstAddr);
    subtract_from_field(ipv4.ttl, 1);
}

table ipv4_lpm {
    reads {
        ipv4.dstAddr : lpm;
    }
    actions {
        ipv4_forward;
        _drop;
    }
    size: 1024;
}

header_type metadata_t { 
    fields {
        a : 32;
        b : 32;
        z : 1;
        reg1 : 32;
        reg2 : 32;
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
action action1 () {
    modify_field(mdata.a, 10);
}
action action2 () {
    modify_field(mdata.b, 5);
}
action action3 () {
    add(mdata.reg1, mdata.a, mdata.b);
    register_write(reg1, 0, mdata.reg1);
}
action action4 () {
    register_read(mdata.reg1, reg1, 0);
}
action action5 () {
    subtract(mdata.reg2,mdata.reg1,3);
    register_write(reg2, 0, mdata.reg2);
}
action action6 () {
    register_read(mdata.reg1, reg1, 0);
}
action action7 () {
    register_read(mdata.reg2, reg2, 0);
}
action action8 () {
    modify_field(mdata.z, 1); 
}
action action9 () {
    modify_field(mdata.z, 0); 
}
action action10 () {
    modify_field(mdata.if_block_tmp_2, 1); 
}
action action11 () {
    modify_field(mdata.if_block_tmp_2, 0); 
}
action action12 () {
    register_read(mdata.reg2, reg2, 0);
    modify_field(mdata.tmp_0_if_2, mdata.reg2);
}
action action13 () {
    register_read(mdata.reg1, reg1, 0);
    modify_field(mdata.tmp_0_if_2, mdata.reg1);
}
action action14 () {
    register_write(reg1, 0, mdata.tmp_0_if_2);
}
action action15 () {
    register_read(mdata.reg1, reg1, 0);
    register_write(reg1, 0, mdata.reg1);
}
action action16 () {
    register_read(mdata.reg1, reg1, 0);
    modify_field(n.new_one, mdata.reg1);
}
action action17 () {
    modify_field(n.new_one, .n.new_one);
}
action action18 () {
    modify_field(mdata.a, 1);
}
action action19 () {
    modify_field(mdata.a, mdata.a);
}
action action20 () {
    register_read(mdata.reg1, reg1, 0);
}
action action21 () {
    add(mdata.tmp_1_if_3,mdata.reg1,100);
}
action action22 () {
    register_read(mdata.reg1, reg1, 0);
    register_write(reg1, 0, mdata.reg1);
}
action action23 () {
    register_write(reg1, 0, mdata.tmp_1_if_3);
}
action action24 () {
    register_read(mdata.reg1, reg1, 0);
}
action action25 () {
    subtract(mdata.b,mdata.reg1,10);
}
action action26 () {
    modify_field(mdata.z, 1); 
}
action action27 () {
    modify_field(mdata.z, 0); 
}
action action28 () {
    register_write(reg1, 0, mdata.b);
}
action action29 () {
    register_write(reg1, 0, 1234);
}
action action30 () {
    register_read(mdata.reg2, reg2, 0);
}
action action31 () {
    register_read(mdata.reg1, reg1, 0);
}
action action32 () {
    add(mdata.reg1, mdata.reg2, mdata.reg1);
    register_write(reg1, 0, mdata.reg1);
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
        action5;
    }
}
table table6 {
    actions {
        action6;
    }
}
table table7 {
    actions {
        action7;
    }
}
table table8 {
    actions {
        action8;
    }
}
table table9 {
    actions {
        action9;
    }
}
table table10 {
    actions {
        action10;
    }
}
table table11 {
    actions {
        action11;
    }
}
table table12 {
    reads {
        mdata.z : exact;
    }
    actions {
        action12;
        action13;
    }
}
table table13 {
    reads {
        mdata.if_block_tmp_2 : exact;
    }
    actions {
        action14;
        action15;
    }
}
table table14 {
    reads {
        if_block_tmp_2 : exact;
    }
    actions {
        action16;
        action17;
    }
}
table table15 {
    reads {
        mdata.if_block_tmp_2 : exact;
    }
    actions {
        action18;
        action19;
    }
}
table table16 {
    actions {
        action20;
    }
}
table table17 {
    actions {
        action21;
    }
}
table table18 {
    reads {
        mdata.if_block_tmp_2 : exact;
    }
    actions {
        action22;
        action23;
    }
}
table table19 {
    actions {
        action24;
    }
}
table table20 {
    actions {
        action25;
    }
}
table table21 {
    actions {
        action26;
    }
}
table table22 {
    actions {
        action27;
    }
}
table table23 {
    reads {
        mdata.z : exact;
    }
    actions {
        action28;
        action29;
    }
}
table table24 {
    actions {
        action30;
    }
}
table table25 {
    actions {
        action31;
    }
}
table table26 {
    actions {
        action32;
    }
}
control ingress {
    apply(table1);
    apply(table2);
    apply(table3);
    apply(table4);
    apply(table5);
    apply(table6);
    apply(table7);
    if (mdata.reg1 > mdata.reg2) {
        apply(table8);
    } else {
        apply(table9);
    }
    if (mdata.a > mdata.b) {
        apply(table10);
    } else {
        apply(table11);
    }
    apply(table12);
    apply(table13);
    apply(table14);
    apply(table15);
    apply(table16);
    apply(table17);
    apply(table18);
    apply(table19);
    apply(table20);
    if (mdata.a < mdata.b) {
        apply(table21);
    } else {
        apply(table22);
    }
    apply(table23);
    apply(table24);
    apply(table25);
    apply(table26);

    if(valid(ipv4) and ipv4.ttl > 0) {
        apply(ipv4_lpm);
    }
}
control egress {
}
