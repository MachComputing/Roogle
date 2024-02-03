#!/bin/sh

# Run the benchmark
for i in $(seq 1 20)
do
  cargo run --release --bin testing "$i" "$i" "$1" 2>&-
  echo "========================================================================="
done