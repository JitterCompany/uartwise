

monitor swdp_scan
attach 1
monitor traceswo
set mem inaccessible-by-default off

# common
# break Reset
break main.rs:87

load

# start the process but immediately halt the processor
stepi
