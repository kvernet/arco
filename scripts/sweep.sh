#!/usr/bin/env bash
# Run a 10-seed sweep across all estimators for both substrates.
# Usage: ./scripts/sweep.sh

SEEDS=(42 99 137 256 512 1024 2048 4096 8192 16384)
ESTIMATORS=("plugin" "mm" "qe")
DATA_DIR="sweep_data"
mkdir -p "$DATA_DIR"

for estimator in "${ESTIMATORS[@]}"; do
    echo "=== Graph Substrate (estimator=$estimator) ==="
    for seed in "${SEEDS[@]}"; do
        echo "--- seed=$seed ---"
        cargo run --release --features serialize -- graph \
            --train 1000 --test 300 --seed $seed --estimator $estimator \
            --output "$DATA_DIR/graph_${estimator}_${seed}.json" 2>&1 \
            | grep -E "(H[0-9]_|Structured|Noise|Storage:)"
    done
    echo ""
done

for estimator in "${ESTIMATORS[@]}"; do
    echo "=== CA Substrate (estimator=$estimator) ==="
    for seed in "${SEEDS[@]}"; do
        echo "--- seed=$seed ---"
        cargo run --release --features serialize -- ca \
            --train 1000 --test 300 --seed $seed --estimator $estimator \
            --output "$DATA_DIR/ca_${estimator}_${seed}.json" 2>&1 \
            | grep -E "(H[0-9]_|Storage:)"
    done
    echo ""
done

echo "Data saved to $DATA_DIR/"