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
        z : 1;
        r : 32;
        q : 32;
        m : 32;
        l : 32;
        i : 32;
        reg1 : 32;
        reg2 : 32;
        reg3 : 32;
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
parser start {
    return parse_ethernet;
 }
parser parse_ethernet {
    extract(ethernet);
    return ingress;
}
action action1 () {
    modify_field(mdata.q, 10);
}
action action2 () {
    modify_field(mdata.r, 5);
}
action action3 () {
    register_read(mdata.reg3, reg3, 0);
}
action action4 () {
    modify_field(mdata.l, mdata.reg3);
}
action action5 () {
    add(mdata.i,mdata.q,mdata.l);
}
action action6 () {
    register_write(reg1, 11, 0);
}
action action7 () {
    add(mdata.reg2,mdata.i,5);
    register_write(reg2, 0, mdata.i);
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
control ingress {
    apply(table1);
    apply(table2);
    apply(table3);
    apply(table4);
    apply(table5);
    apply(table6);
    apply(table7);
}
control egress {
}
