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

# Fast test run
cargo run --release -- graph --quick

# Save results to JSON
cargo run --release --features serialize -- graph --output results.json

# See all options
cargo run --release -- graph --help
```

## Installation

### As a library

Add to your `Cargo.toml`:

```toml
[dependencies]
arco = "0.4"
```

### From source

```bash
git clone https://github.com/kvernet/arco.git
cd arco
cargo build --release
```

Requires Rust 1.85+.

## Key Findings

All results from 10 independent seeds. Reproduce with:

```bash
./scripts/sweep.sh
python3 scripts/analyze.py sweep_data
```

### Graph Substrate — n=1,000

#### Structure-Storage Gradient

| Bracket | Storage Rate Range | Mean |
|---------|-------------------|------|
| Noise (0.00–0.15) | 11.0–41.6% | 21.1% |
| Balanced (0.40–0.60) | 20.8–71.3% | 35.2%|
| Structured (0.85–1.00) | 90.5–99.6% | **93.9%** |

A 4.5× difference. This is ARCO's most robust finding.

#### Hypothesis Survival

| ID | Condition | Survival | Acc. Range | Mean |
|----|-----------|----------|-----------|------|
| H2 | Majority structured → memory | 10/10 | 54.8–86.6% | 68.2% |
| H5 | Transport rules → storage | 10/10 | 50.5–85.2% | 65.4% |
| H7 | Multiple logic gates → memory | 8/10 | 47.2–79.5% | 63.3% |
| H3 | Logic gate → memory | 7/10 | 38.6–80.5% | 57.8% |

H5 (Transport Law) and H2 are the most reliable. H1, H4, H6, H8 did not survive (0/10). H6 is a negative control (all-destructive → storage) — its consistent failure validates calibration.

### CA Substrate — n=256

Null distribution sampled from a pool of known chaotic rules. All hypotheses use measurable properties only.

#### Hypothesis Survival

| ID | Condition | Survival | Acc. Range | Mean |
|----|-----------|----------|-----------|------|
| H3 | Low sensitivity → storage | 10/10 | 74.8–91.2% | 84.9% |
| H6 | Mid-lambda → storage | 10/10 | 74.2–88.7% | 83.5% |
| H5 | Not Rule 0 → storage | 10/10 | 73.3–88.0% | 82.4% |
| H2 | Parity conservation → storage | 10/10 | 64.7–100.0% | 78.4% |
| H1 | Reversible → storage | 8/10 | 30.0–100.0% | 56.9% |
| H4 | Even rule number → storage | 0/10 | — | — |

H4 failed because the chaotic null pool contains both even (30, 86, 106) and odd (45, 135, 149) rules with overlapping storage distributions. Evenness does not discriminate.

### Limitations

- The plugin mutual information estimator has known small-sample bias. Shuffle correction mitigates but does not eliminate it.
- The Binary Graph Universe is a **validation substrate** — rules are hand-coded to calibrate the instrument. Discovery substrates are the next milestone.
- All findings are from small state spaces (3-vertex graphs, 8-cell automata).

## Documentation

- [Web Page](https://kvernet.com/arco)
- [Mathematical Constitution](https://github.com/kvernet/arco/blob/main/docs/constitution.md) — the formal specification
- [API documentation](https://docs.rs/arco)
- [Examples](https://github.com/kvernet/arco/tree/main/examples)

## Reproducibility

```bash
./scripts/sweep.sh                    # Run 10-seed sweep, save JSON
python3 scripts/analyze.py sweep_data # Analyze and generate plots
```

Every number in this README is traceable to a specific seed in `sweep_data/` produced by `scripts/sweep.sh`.

## Python Reference

The Python reference implementation that first validated the methodology: [arco-python](https://github.com/kvernet/arco-python).

## License

MIT