header_type ethernet_t {
    fields {
        dstAddr : 48;
        srcAddr : 48;
        etherType : 16;
    }
}
header ethernet_t ethernet;
header_type metadata_t { 
    fields {
        p : 2;
        m : 2;
        z : 2;
        h : 2;
        q : 2;
    }
}
metadata metadata_t mdata;
register p {
     width : 2; 
     instance_count : 1;
}
register m {
     width : 2; 
     instance_count : 3;
}
parser start {
    return parse_ethernet;
 }
parser parse_ethernet {
    extract(ethernet);
    return ingress;
}
action action1 () {
    modify_field(mdata.q, 5);
}
action action2 () {
    modify_field(mdata.z, 6);
}
action action3 () {
    register_write(m, 5, 0);
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
control ingress {
    apply(table1);
    apply(table2);
    apply(table3);
}
control egress {
}
