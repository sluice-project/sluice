#include <tofino/intrinsic_metadata.p4>
#include <tofino/constants.p4>
#include <tofino/primitives.p4>
#include "tofino/stateful_alu_blackbox.p4"
#include "tofino/lpf_blackbox.p4"
#include "tofino/wred_blackbox.p4"
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
control ingress {
}
control egress {
}
