#!/bin/bash

if [ $# -ne 1 ]; then
    echo "Usage: process.sh <directory>"
    exit 1
fi

declare -a names=("roole" "roolean")

for name in "${names[@]}"; do
    :>"time.$name.data"
    :>"space.$name.data"
    find "$1" -name "*.$name.runlim" | while read i; do
        #echo "$i"
        TIME=`grep "time:" "$i" | awk '{print $3}'`
        MEMORY=`grep "space:" "$i" | awk '{print $3}'`
        #echo "Time: $TIME, memory: $MEMORY"
        echo "$TIME" >> "time.$name.data"
        echo "$MEMORY" >> "space.$name.data"
    done

    sort -n "time.$name.data" | awk '{sum+=$0;printf "%s\t%s\n",sum,NR}' > "time.$name.points"
    sort -n "space.$name.data" | awk '{printf "%s\t%s\n",$0,NR}' > "space.$name.points"
done
