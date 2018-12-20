#include <tofino/intrinsic_metadata.p4>
#include <tofino/constants.p4>
#include <tofino/primitives.p4>
#include "tofino/stateful_alu_blackbox.p4"
#include "tofino/lpf_blackbox.p4"
#include "tofino/wred_blackbox.p4"
header_type metadata_t { 
    fields {
        z : 2;
        r : 2;
        q : 2;
        m : 2;
    }
}
control ingress {
}
control egress {
}
