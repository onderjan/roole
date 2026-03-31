set xlabel "Proof size [MB]"
set ylabel "Number of instances"
set title 'Proof sizes'
set key right bottom
plot 'proof.points' using (($1)/1000000):2 with linespoints title 'Proof size'
