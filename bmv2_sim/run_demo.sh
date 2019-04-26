#!/bin/bash

sluice_file=$1
sluice_file_path=net-progs/$1.np

BMV2_SWITCH_EXE=simple_switch_grpc
P4C_ARGS="--p4runtime-file $(basename $@).p4info --p4runtime-format text"

BUILD_DIR=build
PCAP_DIR=pcaps
LOG_DIR=logs

TOPO=topology.json
P4C=p4c-bm2-ss
RUN_SCRIPT=/home/vagrant/tutorials/utils/run_exercise.py

cd ..
cargo run --bin sluice $sluice_file_path
cd bmv2_sim

sudo mn -c
rm -f *.pcap
rm -rf $BUILD_DIR $PCAP_DIR $LOG_DIR

mkdir -p build pcaps logs

function get_p4_files {
python - <<END
import json
print map(str,list(set(json.load(open("topology.json", 'r'))['snippet_loc'].values())))
END
}

p4_files=($(get_p4_files | tr -d '[],'))

for file in ${p4_files[@]};
do
	eval file=$file	
	cp ../out/${file}.p4 .
    p4c-bm2-ss --p4v 14 --p4runtime-file build/${file}.p4info --p4runtime-format text -o build/${file}.json ${file}.p4
done

sudo python $RUN_SCRIPT -t $TOPO -b $BMV2_SWITCH_EXE


# sluice_file=$1
# sluice_file_path=net-progs/$1.np

# cd ..
# cargo run --bin sluice $sluice_file_path
# cp out/${sluice_file}.p4 bmv2_sim/
# cd bmv2_sim

# make clean; make run

