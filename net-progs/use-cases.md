# Use Case Applications

## 1) DPTP (Data Plane Timesynchronization Protocol)
An In-Network time synchronization protocol.
Things to be taken care in sluice :
1) Need a way to parse DPTP header on top of Ethernet Header.
2) Need to access timestamp of switch state.
3) Need to represent switch-to-switch interaction based on topology.

## 2) Record & Replay Snapshots
Capture switch states periodically, and enable mechanism to capture synchronized snapshots
upon a trigger (e.g buffer above threshold)

## 3) QoE Monitoring

## 4) PERC
