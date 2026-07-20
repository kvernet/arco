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

## Installation

### As a library

Add to your `Cargo.toml`:

```toml
[dependencies]
arco = "0.2"
```

### From source

```bash
git clone https://github.com/kvernet/arco.git
cd arco
cargo build --release
```

Requires Rust 1.85+.

## Quick Start

```rust
use arco::cycle::{CycleConfig, run_cycle};

fn main() {
    let config = CycleConfig::default();
    let record = run_cycle(&config);
    println!("{}", record.summary());
}
```

Or via the CLI:

```bash
cargo run --release
cargo run --release -- --train 1000 --test 300 --seed 42
cargo run --release -- --quick
```

See [`examples/`](https://github.com/kvernet/arco/tree/main/examples) for more usage patterns.

## Experimental Findings

All results below are at n=10,000 training universes with 2,000 held-out test universes, across 10 independent seeds. The current Binary Graph Universe is a **validation substrate** — it uses hand-coded computational primitives to calibrate ARCO's measurement apparatus. See the [Mathematical Constitution](https://github.com/kvernet/arco/blob/main/docs/constitution.md) for the distinction between validation and discovery substrates.

### Structure-Storage Gradient

Storage probability increases monotonically with the fraction of structured rules. This is ARCO's most robust finding.

| Structured Ratio | Storage Rate (Range) |
|------------------|----------------------|
| 0.00 – 0.15 (Noise) | 14.2 – 34.3% |
| 0.85 – 1.00 (Structured) | 93.8 – 99.0% |

Full spectrum across all 10 seeds:

| Seed | Noise Storage % | Structured Storage % |
|------|----------------|---------------------|
| 42 | 17.7 | 94.0 |
| 99 | 24.1 | 96.5 |
| 137 | 26.3 | 97.5 |
| 256 | 14.2 | 93.8 |
| 512 | 19.9 | 94.9 |
| 1024 | 25.0 | 97.2 |
| 2048 | 21.3 | 95.7 |
| 4096 | 34.3 | 98.8 |
| 8192 | 32.5 | 99.0 |
| 16384 | 28.7 | 97.9 |

### Structural Hypotheses

These hypotheses test whether specific structural properties of rule sets predict storage. All consistently outperform chance (50%) at large sample sizes.

**H5 — Transport Law:** Rule sets containing information transport operations (PROPAGATE, SWAP, COPY_TO_OUT, COPY_FROM_IN) exhibit storage.

| Seed | 42 | 99 | 137 | 256 | 512 | 1024 | 2048 | 4096 | 8192 | 16384 |
|------|-----|-----|-----|-----|-----|------|------|------|------|-------|
| Acc | 60.5 | 62.3 | 70.0 | 55.8 | 62.1 | 70.3 | 71.6 | 81.6 | 82.3 | 78.2 |

Range: 55.8 – 82.3%. Mean: 69.5%.

**H2 — Majority Structure:** Majority-structured rule sets exhibit memory.

| Seed | 42 | 99 | 137 | 256 | 512 | 1024 | 2048 | 4096 | 8192 | 16384 |
|------|-----|-----|-----|-----|-----|------|------|------|------|-------|
| Acc | 63.5 | 68.8 | 73.7 | 59.9 | 67.0 | 70.4 | 73.7 | 81.4 | 82.0 | 79.2 |

Range: 59.9 – 82.0%. Mean: 72.0%.

**H7 — Multiple Logic:** Rule sets with at least two logic gates exhibit memory.

| Seed | 42 | 99 | 137 | 256 | 512 | 1024 | 2048 | 4096 | 8192 | 16384 |
|------|-----|-----|-----|-----|-----|------|------|------|------|-------|
| Acc | 61.6 | 63.3 | 73.3 | 55.5 | 62.9 | 70.1 | 72.6 | 81.5 | 79.5 | 73.1 |

Range: 55.5 – 81.5%. Mean: 69.3%.

**H3 — Logic Gate:** Rule sets containing at least one logic gate exhibit memory.

| Seed | 42 | 99 | 137 | 256 | 512 | 1024 | 2048 | 4096 | 8192 | 16384 |
|------|-----|-----|-----|-----|-----|------|------|------|------|-------|
| Acc | 53.6 | 56.8 | 65.4 | — | 53.6 | 60.4 | 61.9 | 73.6 | 72.5 | 70.4 |

Range: 53.6 – 73.6%. Mean: 63.0%. H3 failed to survive at seed 256 (below 50% threshold). Survives at 9 of 10 seeds.

### Negative Control

**H6 — All Destructive:** Rule sets composed entirely of destructive rules should not exhibit emergence. This hypothesis consistently fails across all seeds, validating the null-distribution calibration.

### Boolean Function Verification

NAND, AND, OR, NOR, and XOR gates are verified to function under stochastic, asynchronous, first-match scheduling — confirming that logical computation can be robust to nondeterministic rule ordering. These gates are hand-coded in the current Binary Graph Universe as part of the validation substrate.

### Statistical Notes

- The plugin mutual information estimator has known small-sample bias. Shuffle correction (10 permutations) subtracts the mean baseline. For larger state spaces, a Bayesian (NSB) or Miller-Madow estimator is recommended.
- Hypothesis accuracies at n=300 can vary by ±20 percentage points across seeds. The n=10,000 estimates above are stable to within ~5 points.
- Earlier reports of 91.2% H5 accuracy from the Python reference implementation (n=300, seed=42) were a high-variance draw at small sample size. The large-n Rust estimates are more reliable.

## Performance

Benchmarked on a 20-core machine (release build):

| Universes | Time |
|-----------|------|
| 1,000 | 3.4s |
| 5,000 | 50.6s |
| 10,000 | 92.1s |
| 50,000 | 6m17s |

## Package Structure

| Module | Purpose |
|--------|---------|
| `state` | State trait and BinaryGraphState |
| `rules` | RewriteRule, MatchInfo, compose, generators |
| `dynamics` | Schedule, trajectory and ensemble generation |
| `observation` | Single-state and windowed observers |
| `metrics` | Shuffle-corrected NMI, storage, memory |
| `calibration` | Null distribution threshold calibration |
| `hypotheses` | Hypothesis generation, testing, MDL scoring |
| `universe` | InformationUniverse container and factories |
| `cycle` | Scientific cycle orchestrator |

## Documentation

- [Mathematical Constitution](https://github.com/kvernet/arco/blob/main/docs/constitution.md) — the formal specification
- [API documentation](https://docs.rs/arco) — rustdoc for the latest release
- [Examples](https://github.com/kvernet/arco/tree/main/examples) — runnable usage examples

## Python Reference

The Python reference implementation that first validated the methodology is available at [arco-python](https://github.com/kvernet/arco-python).

## License

MIT