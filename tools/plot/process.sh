#!/bin/bash

:> time.data
:> memory.data

shopt -s globstar
for i in **/*.roole.runlim; do
    TIME=`grep "time:" "$i" | awk '{print $3}'`
    MEMORY=`grep "space:" "$i" | awk '{print $3}'`
    echo "$TIME" >> time.data
    echo "$MEMORY" >> space.data
done

cat time.data | sort -n | awk '{sum+=$0;printf "%s\t%s\n",NR,sum}' > time.points
cat space.data | sort -n | awk '{printf "%s\t%s\n",$0,NR}' > space.points
