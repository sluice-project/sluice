sluice_file=$1

source env.sh

CLI_PATH=$BMV2_PATH/targets/simple_switch/sswitch_CLI

$CLI_PATH ${sluice_file}.json < commands/${sluice_file}.txt
