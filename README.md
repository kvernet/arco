# ARCO

**Automated Research into Computational Ontologies**

[![crates.io](https://img.shields.io/crates/v/arco.svg)](https://crates.io/crates/arco)
[![docs.rs](https://img.shields.io/docsrs/arco)](https://docs.rs/arco)
[![CI](https://github.com/kvernet/arco/actions/workflows/ci.yml/badge.svg)](https://github.com/kvernet/arco/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A computational science platform for discovering the conditions under which computation, memory, and learning emerge in arbitrary information systems.

## What ARCO Does

ARCO asks a different question than most computer science: not "what can a given computational model compute?" but "what computational models are possible, and why do they emerge?"

It formalizes this through **Information Universes** — 6-tuples of (state space, transformations, observations, resources, invariants, schedule) — and measures emergent computation via shuffle-corrected normalized mutual information calibrated against destructive null distributions.

## Quick Start

```bash
# Binary Graph Universe
cargo run --release --features serialize -- graph --train 1000 --seed 42

# Cellular Automaton
cargo run --release --features serialize -- ca

# Compare estimators
cargo run --release -- graph --estimator qe

# Fast test run
cargo run --release -- graph --quick

# Save results to JSON
cargo run --release --features serialize -- graph --output results.json
```

## Installation

```toml
[dependencies]
arco = "0.5"
```

Requires Rust 1.85+.

## Documentation

- [Experimental Results](https://github.com/kvernet/arco/tree/main/docs/RESULTS.md) — key findings with data
- [Estimator Consistency Benchmark](https://github.com/kvernet/arco/tree/main/docs/benchmarks/estimator_consistency.md) — exact ground truth validation
- [Mathematical Constitution](https://github.com/kvernet/arco/tree/main/docs/constitution.md) — formal specification
- [API documentation](https://docs.rs/arco) — rustdoc
- [Examples](https://github.com/kvernet/arco/tree/main/examples) — runnable usage examples

## Reproducibility

```bash
./scripts/sweep.sh                    # Run 10-seed sweep, save JSON
python3 scripts/analyze.py sweep_data # Analyze and generate plots
```

## Python Reference

[arco-python](https://github.com/kvernet/arco-python) — the Python implementation that first validated the methodology.

## License

MIT