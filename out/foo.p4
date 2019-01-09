#include <core.p4>
#include <v1model.p4>
header_type metadata_t { 
    fields {
        z : 2;
        h : 2;
        q : 2;
    }
}
register p {
     width : 2; 
     instance_count : 1;
}
register m {
     width : 2; 
     instance_count : 3;
}
action action1 () {
    modify_field(mdata.q, 5);
}
table table1 () {
    actions {
        action1;
    }
}
control ingress {
    table1();
}
control egress {
}
