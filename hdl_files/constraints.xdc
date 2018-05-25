# W5 is the oscillator pin
set_property PACKAGE_PIN W5 [get_ports clk]
set_property IOSTANDARD LVCMOS33 [get_ports clk]

# set clock frequency of master clock to external crystal's frequency (100 MHz)
# derived clocks are automatically constrained
create_clock -period 10 -name master_clock [get_ports clk]

# Specify physical locations of input and output signals on FPGA board
# Input signal: push button on pin U18
set_property PACKAGE_PIN U18     [get_ports i_reset]
set_property IOSTANDARD LVCMOS33 [get_ports i_reset]

# Output signal: LED on pins L1 and P1
set_property PACKAGE_PIN L1      [get_ports o_count_led]
set_property IOSTANDARD LVCMOS33 [get_ports o_count_led]
set_property PACKAGE_PIN P1      [get_ports o_uart_led]
set_property IOSTANDARD LVCMOS33 [get_ports o_uart_led]

# Output signal: LED on pins Ld7 through 0
set_property PACKAGE_PIN V14      [get_ports {o_debug[0]}]
set_property IOSTANDARD  LVCMOS33 [get_ports {o_debug[0]}]
set_property PACKAGE_PIN U14      [get_ports {o_debug[1]}]
set_property IOSTANDARD  LVCMOS33 [get_ports {o_debug[1]}]
set_property PACKAGE_PIN U15      [get_ports {o_debug[2]}]
set_property IOSTANDARD  LVCMOS33 [get_ports {o_debug[2]}]
set_property PACKAGE_PIN W18      [get_ports {o_debug[3]}]
set_property IOSTANDARD  LVCMOS33 [get_ports {o_debug[3]}]
set_property PACKAGE_PIN V19      [get_ports {o_debug[4]}]
set_property IOSTANDARD  LVCMOS33 [get_ports {o_debug[4]}]
set_property PACKAGE_PIN U19      [get_ports {o_debug[5]}]
set_property IOSTANDARD  LVCMOS33 [get_ports {o_debug[5]}]
set_property PACKAGE_PIN E19      [get_ports {o_debug[6]}]
set_property IOSTANDARD  LVCMOS33 [get_ports {o_debug[6]}]
set_property PACKAGE_PIN U16      [get_ports {o_debug[7]}]
set_property IOSTANDARD  LVCMOS33 [get_ports {o_debug[7]}]

# Seven segment display pins
# Need to set these to high to disable.
set_property PACKAGE_PIN W7 [get_ports {o_segment_enable[0]}]
set_property IOSTANDARD LVCMOS33 [get_ports {o_segment_enable[0]}]
set_property PACKAGE_PIN W6 [get_ports {o_segment_enable[1]}]
set_property IOSTANDARD LVCMOS33 [get_ports {o_segment_enable[1]}]
set_property PACKAGE_PIN U8 [get_ports {o_segment_enable[2]}]
set_property IOSTANDARD LVCMOS33 [get_ports {o_segment_enable[2]}]
set_property PACKAGE_PIN V8 [get_ports {o_segment_enable[3]}]
set_property IOSTANDARD LVCMOS33 [get_ports {o_segment_enable[3]}]
set_property PACKAGE_PIN U5 [get_ports {o_segment_enable[4]}]
set_property IOSTANDARD LVCMOS33 [get_ports {o_segment_enable[4]}]
set_property PACKAGE_PIN V5 [get_ports {o_segment_enable[5]}]
set_property IOSTANDARD LVCMOS33 [get_ports {o_segment_enable[5]}]
set_property PACKAGE_PIN U7 [get_ports {o_segment_enable[6]}]
set_property IOSTANDARD LVCMOS33 [get_ports {o_segment_enable[6]}]

set_property PACKAGE_PIN V7 [get_ports o_dot_enable]
set_property IOSTANDARD LVCMOS33 [get_ports o_dot_enable]

set_property PACKAGE_PIN U2 [get_ports {o_display_enable[0]}]
set_property IOSTANDARD LVCMOS33 [get_ports {o_display_enable[0]}]
set_property PACKAGE_PIN U4 [get_ports {o_display_enable[1]}]
set_property IOSTANDARD LVCMOS33 [get_ports {o_display_enable[1]}]
set_property PACKAGE_PIN V4 [get_ports {o_display_enable[2]}]
set_property IOSTANDARD LVCMOS33 [get_ports {o_display_enable[2]}]
set_property PACKAGE_PIN W4 [get_ports {o_display_enable[3]}]
set_property IOSTANDARD LVCMOS33 [get_ports {o_display_enable[3]}]

# UART TX and RX
set_property PACKAGE_PIN A18 [get_ports o_uart_tx]
set_property IOSTANDARD LVCMOS33 [get_ports o_uart_tx]
set_property PACKAGE_PIN B18 [get_ports i_uart_rx]
set_property IOSTANDARD LVCMOS33 [get_ports i_uart_rx]
