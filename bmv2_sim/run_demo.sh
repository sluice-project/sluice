sluice_file=$1
sluice_file_path=net-progs/$1.np

cd ..
cargo run --bin sluice $sluice_file_path
cp out/${sluice_file}.p4 bmv2_sim/
cd bmv2_sim

make clean; make run
