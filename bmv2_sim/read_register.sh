sluice_file=$1

source env.sh

CLI_PATH=$BMV2_PATH/targets/simple_switch/sswitch_CLI

echo "register_read reg1 0" | $CLI_PATH ${sluice_file}.json
