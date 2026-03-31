#!/bin/bash


if [ $# -ne 1 ]; then
    echo "Usage: plot.sh <directory>"
    exit 1
fi

SCRIPT_DIR=$(dirname "$0")

tee << EOF 
set terminal pdfcairo size 20cm, 15cm
set output "running_time.pdf"
set decimalsign '.'

set xlabel "Running total time [s]"
set ylabel "Number of instances"
set title 'Solving time (running total)'
set key right bottom
plot '$1/time.roole.points' using 1:2 with linespoints title "Roole", '$1/time.roolean.points' using 1:2 with linespoints title "Roolean"
EOF

gnuplot << EOF
set terminal pdfcairo size 20cm, 15cm
set output "running_time.pdf"
set decimalsign '.'

set xlabel "Running total time [s]"
set ylabel "Number of instances"
set title 'Solving time (running total)'
set key right bottom
plot '$1/time.roole.points' using 1:2 with linespoints title "Roole", '$1/time.roolean.points' using 1:2 with linespoints title "Roolean"
EOF


gnuplot  << EOF
set terminal pdfcairo size 20cm, 15cm
set output "running_time_loglin.pdf"
set decimalsign '.'

set xlabel "Running total time [s]"
set ylabel "Number of instances"
set title 'Solving time (running total), log-linear'
set logscale x 10
set key right bottom
set yrange [*<0:1<*]
plot '$1/time.roole.points' with linespoints title "Roole", '$1/time.roolean.points' with linespoints title "Roolean"
EOF

gnuplot << EOF
set terminal pdfcairo size 20cm, 15cm
set output "memory.pdf"
set decimalsign '.'

set xlabel "Used memory [MB]"
set ylabel "Number of instances"
set title 'Memory usage'
set key right bottom
plot '$1/space.roole.points' with linespoints title "Roole", '$1/space.roolean.points' with linespoints title "Roolean"
EOF

gnuplot << EOF
set terminal pdfcairo size 20cm, 15cm
set output "proof_size.pdf"
set decimalsign '.'

set xlabel "Proof size [MB]"
set ylabel "Number of instances"
set title 'Proof sizes'
set key right bottom
plot '$1/proof.points' using ((\$1)/1000000):2 with linespoints title 'Proof size'
EOF

