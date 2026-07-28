# ARCO

**Automated Research into Computational Ontologies**

[![crates.io](https://img.shields.io/crates/v/arco.svg)](https://crates.io/crates/arco)
[![docs.rs](https://img.shields.io/docsrs/arco)](https://docs.rs/arco)
[![CI](https://github.com/kvernet/arco/actions/workflows/ci.yml/badge.svg)](https://github.com/kvernet/arco/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A computational science platform for discovering the conditions under which computation, memory, and learning emerge in arbitrary information systems.

## What ARCO Does

ARCO asks a different question than most computer science: not "what can a given computational model compute ?" but "what computational models are possible, and why do they emerge ?"

It formalizes this through **Information Universes** — 6-tuples of (state space, transformations, observations, resources, invariants, schedule) — and measures emergent computation via shuffle-corrected normalized mutual information calibrated against destructive null distributions.

## Quick Start

```bash
# Binary Graph Universe
cargo run --release -- graph --train 1000 --seed 42

# Cellular Automaton
cargo run --release -- ca

# Fast test run
cargo run --release -- graph --quick

# Custom observation
cargo run --release -- graph --obs label_sum

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

All results below are from the refactored pipeline across 10 independent seeds.

### Structure-Storage Gradient (Binary Graph)

| Structured Ratio | Storage Rate (Range) | Mean |
|------------------|---------------------|------|
| 0.00–0.15 (Noise) | 11.0–41.6% | 21.1% |
| 0.85–1.00 (Structured) | 90.5–99.6% | **93.9%** |

A 4.5× difference, stable across all seeds and sample sizes.

### Transport Law (H5)

Rule sets containing transport operations (PROPAGATE, SWAP, COPY) exhibit storage above threshold. Accuracy: 50.5–85.2% (mean 65.4%), survives at 10/10 seeds.

### Paradigm-Neutrality (Cellular Automata)

The same storage metric, applied to all 256 Wolfram rules without modification, produces structural hypotheses from measurable properties only:

| Hypothesis | Accuracy Range | Mean | Survival |
|-----------|---------------|------|----------|
| H3: Low sensitivity → Storage | 88.6–97.6% | **93.2%** | 10/10 |
| H6: Mid-lambda → Storage | 84.4–96.0% | **90.0%** | 10/10 |

ARCO recovers known CA taxonomy without being told about Wolfram classes.

### Limitations

- The plugin MI estimator has known small-sample bias. Shuffle correction mitigates but does not eliminate it.
- The Binary Graph Universe is a **validation substrate** — rules are hand-coded to calibrate the instrument. Genuine discovery substrates are the next milestone.
- All findings are from small state spaces (3-vertex graphs, 8-cell automata).

## Documentation

- [Web Page](https://kvernet.com/arco)
- [Mathematical Constitution](https://github.com/kvernet/arco/blob/main/docs/constitution.md) — the formal specification
- [API documentation](https://docs.rs/arco) — rustdoc for the latest release
- [Examples](https://github.com/kvernet/arco/tree/main/examples) — runnable usage examples

## Python Reference

The Python reference implementation that first validated the methodology is available at [arco-python](https://github.com/kvernet/arco-python).

## License

MIT