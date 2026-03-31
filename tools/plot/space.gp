set xlabel "Used memory [MB]"
set ylabel "Number of instances"
set title 'Memory usage'
set key right bottom
plot 'space.roole.points' with linespoints title "Roole", 'space.roolean.points' with linespoints title "Roolean"