sluce_file=$1
sluice_file_path=net_progs/$1.np

cd ..
cargo run --bin sluice $sluice_file_path
cp out/${sluce_file}.p4 bmv2_sim/
cd bmv2_sim
source env.sh

P4C_BM_SCRIPT=$P4C_BM_PATH/p4c_bm/__main__.py

SWITCH_PATH=$BMV2_PATH/targets/simple_switch/simple_switch

CLI_PATH=$BMV2_PATH/targets/simple_switch/sswitch_CLI

set -m
$P4C_BM_SCRIPT ${sluce_file}.p4 --json ${sluce_file}.json

# sudo $SWITCH_PATH >/dev/null 2>&1
# sudo $SWITCH_PATH test1.json \
#     -i 0@veth0 -i 1@veth2 -i 2@veth4 -i 3@veth6 -i 4@veth8 \
#     --nanolog ipc:///tmp/bm-0-log.ipc \
#     --pcap

sudo python $BMV2_PATH/mininet/1sw_demo.py \
    --behavioral-exe $BMV2_PATH/targets/simple_switch/simple_switch \
    --json ${sluce_file}.json

# sleep 2
# $CLI_PATH test1.json < commands1.txt
# echo "READY!!!"
# fg
