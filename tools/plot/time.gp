set xlabel "Running total time [s]"
set ylabel "Number of instances"
set title 'Solving time (running total)'
set key right bottom
plot 'time.roole.points' with linespoints title "Roole", 'time.roolean.points' with linespoints title "Roolean"