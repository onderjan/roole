#!/bin/bash

if [ $# -ne 1 ]; then
    echo "Usage: proof_sizes.sh <directory>"
    exit 1
fi

declare -a names=("roole" "roolean")

:>"proof.data"
find "$1" -name "*.proof" | while read i; do
    #echo "$i"
    SIZE=`wc -c "$i"`
    #echo "Size: $SIZE"
    echo "$SIZE" >> "proof.data"
done

sort -n "proof.data" | cut -d' ' -f1 | awk '{printf "%s\t%s\n",$0,NR}' > "proof.points"
