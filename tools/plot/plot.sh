#!/bin/bash

SCRIPT_DIR=$(dirname "$0")

gnuplot -p "$SCRIPT_DIR/time.gp"
gnuplot -p "$SCRIPT_DIR/space.gp"
gnuplot -p "$SCRIPT_DIR/proof.gp"