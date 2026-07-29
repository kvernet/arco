#!/usr/bin/env bash
# Run a 10-seed sweep for both substrates and save JSON records.
# Usage: ./scripts/sweep.sh

SEEDS=(42 99 137 256 512 1024 2048 4096 8192 16384)
DATA_DIR="sweep_data"
mkdir -p "$DATA_DIR"

echo "=== Graph Substrate ==="
for seed in "${SEEDS[@]}"; do
    echo "--- seed=$seed ---"
    cargo run --release --features serialize -- graph --train 1000 --test 300 --seed $seed \
        --output "$DATA_DIR/graph_${seed}.json" 2>&1 | grep -E "(H[0-9]_|Structured|Noise|Storage:)"
done

echo ""
echo "=== CA Substrate ==="
for seed in "${SEEDS[@]}"; do
    echo "--- seed=$seed ---"
    cargo run --release --features serialize -- ca --seed $seed \
        --output "$DATA_DIR/ca_${seed}.json" 2>&1 | grep -E "(H[0-9]_|Storage:)"
done

echo ""
echo "Data saved to $DATA_DIR/"