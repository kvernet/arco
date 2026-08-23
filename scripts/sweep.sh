#!/usr/bin/env bash
# Run a seed sweep across all estimators for both substrates.
# Usage: ./scripts/sweep.sh

# Includes edge cases, small values, patterned values, and large 32-bit values.
SEEDS=(0 1 7 42 69 123 256 512 999 1024 1337 2024 4096 8192 12345 54321 65535 65536 100000 123456 271828 314159 424242 999999 1000000 8675309 2147483647 2147483648 3735928559 4294967295)
ESTIMATORS=("plugin" "mm" "qe" "nsb")
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