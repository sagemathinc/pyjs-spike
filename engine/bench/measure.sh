#!/bin/bash
# measure.sh LABEL CMD...  -> "LABEL wall_s maxrss_MB | program output (hash line)"
label=$1; shift
out=$( { /usr/bin/time -f "TIME %e %M" "$@" > /tmp/measure.out; } 2>&1 )
t=$(echo "$out" | grep TIME | awk '{print $2}')
m=$(echo "$out" | grep TIME | awk '{printf "%.0f", $3/1024}')
h=$(grep -oE "charpoly_hash=[0-9]+" /tmp/measure.out | head -1)
printf "%-22s %8.2f s %7s MB  %s\n" "$label" "$t" "$m" "$h"
