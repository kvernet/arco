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
cargo run --release -- graph --estimator nsb
cargo run --release -- graph --estimator mm

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
arco = "0.5"
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

### Estimator Validation

Three mutual information estimators were compared across 10 seeds on both substrates:

| Substrate | Estimator | Storage Rate | Structured Storage | H5_TRANSPORT Acc | Survival |
|-----------|-----------|-------------|-------------------|----------|----------|
| Graph     | plugin    | 37.1–70.7% (48.2) | 90.5–99.6% (93.9) | 50.5–85.2% (65.4) | 10/10 |
| Graph     | mm        | 21.5–26.7% (24.7) | 64.1–76.5% (70.2) | 22.7–35.9% (29.6) | 0/10 |
| Graph     | nsb       | 36.1–66.6% (46.4) | 87.6–98.5% (93.0) | 50.5–85.2% (64.5) | 10/10 |

| Substrate | Estimator | Storage Rate | Structured Storage | H3_LOW_SENSITIVITY Acc | Survival |
|-----------|-----------|-------------|-------------------|----------|----------|
| Ca        | plugin    | 73.7–90.4% (82.7) | 75.5–90.7% (83.7) | 79.7–92.6% (85.4) | 10/10 |
| Ca        | mm        | 72.1–87.1% (81.8) | 74.9–88.2% (83.3) | 77.8–89.3% (84.1) | 10/10 |
| Ca        | nsb       | 80.4–89.3% (85.4) | 80.4–89.6% (85.6) | 84.4–91.8% (87.9) | 10/10 |

**The plugin estimator with shuffle correction agrees with NSB (the gold standard for small-sample MI) within 2 points across both substrates.** Miller-Madow overcorrects for large observation alphabets (graph, 4096 symbols) but works for smaller alphabets (CA, 256 symbols). All results below use the plugin estimator with shuffle correction.

### Graph Substrate — n=1,000

#### Structure-Storage Gradient

| Bracket | Storage Rate Range | Mean |
|---------|-------------------|------|
| Noise (0.00–0.15) | 11.0–41.6% | 21.1% |
| Balanced (0.40–0.60) | 20.8–71.3% | 35.2% |
| Structured (0.85–1.00) | 90.5–99.6% | **93.9%** |

A 4.5× difference. This is ARCO's most robust finding.

#### Hypothesis Survival

| ID | Condition | Survival | Acc. Range | Mean |
|----|-----------|----------|-----------|------|
| H2_MAJORITY_STRUCTURED | Majority of rules are structured | 10/10 | 54.8–86.6% | 68.2% |
| H5_TRANSPORT | Rule set contains an information transport rule | 10/10 | 50.5–85.2% | 65.4% |
| H7_MULTIPLE_LOGIC | Rule set contains at least 2 logic gates | 8/10 | 47.2–79.5% | 63.3% |
| H3_LOGIC_GATE | Rule set contains a logic gate | 7/10 | 38.6–80.5% | 57.8% |
| H1_HAS_STRUCTURED | Rule set contains at least one structured rule | 0/10 | 0.0–2.3% | 1.0% |
| H4_ALL_STRUCTURED | All rules are structured | 0/10 | 0.0–6.2% | 2.3% |
| H6_ALL_DESTRUCTIVE | All rules are destructive (negative control) | 0/10 | 0.0–4.6% | 1.1% |
| H8_MIXED | Mixed structured and destructive rules | 0/10 | 0.0–2.1% | 0.5% |

H2 and H5 (Transport Law) are the most reliable. H1, H4, H6, H8 did not survive (0/10). H6 is a negative control (all-destructive → storage) — its consistent failure validates calibration.

### CA Substrate — n=1,000

Null distribution sampled from a pool of known chaotic rules (30, 45, 86, 106, 135, 149). All hypotheses use measurable properties only. Rule sets are sampled independently for train and test from the full Wolfram pool.

#### Hypothesis Survival

| ID | Condition | Survival | Acc. Range | Mean |
|----|-----------|----------|-----------|------|
| H2_PARITY | Rule conserves parity | 10/10 | 57.1–89.5% | 74.9% |
| H3_LOW_SENSITIVITY | Rule has low sensitivity (< 2.0) | 10/10 | 79.7–92.6% | 85.4% |
| H4_EVEN_RULE | Rule has even Wolfram number | 10/10 | 77.7–89.0% | 83.8% |
| H5_NOT_RULE_0 | Rule is not the zero rule | 10/10 | 76.8–90.3% | 82.5% |
| H6_MID_LAMBDA | Rule has mid-range lambda (edge of chaos) | 10/10 | 77.9–91.1% | 83.2% |
| H1_REVERSIBLE | Rule is reversible | 7/10 | 33.3–81.8% | 55.6% |

H4 (even rule number) corresponds to quiescent rules in Wolfram's classification — ARCO recovered this connection without being told about quiescence.

### Estimators

ARCO supports three mutual information estimators, selectable via `--estimator`:

| Estimator | Flag | Best for |
|-----------|------|----------|
| Plugin + shuffle | `plugin` (default) | General use, validated against NSB |
| NSB | `nsb` | Large alphabets, publication-quality |
| Miller-Madow | `mm` | Small alphabets, fast bias correction |

The plugin estimator with shuffle correction is the default and has been validated against NSB on both substrates (see Estimator Validation above).

### Limitations

- Miller-Madow overcorrects for large observation alphabets. Use NSB or plugin for substrates with >1,000 distinct observations.
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