sluice_file=$1
thrift_port=$2

source env.sh

CLI_PATH=$BMV2_PATH/targets/simple_switch/sswitch_CLI

# reads the index 7 in reg1
# echo "register_read reg1 7" | $CLI_PATH ${sluice_file}.json $thrift_port

# reads the whole array reg1
echo "register_read reg1" | $CLI_PATH ${sluice_file}.json $thrift_port