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

- [Web Page](https://kvernet.com/arco) — the web page
- [Mathematical Constitution](https://github.com/kvernet/arco/blob/main/docs/constitution.md) — the formal specification
- [API documentation](https://docs.rs/arco) — rustdoc for the latest release
- [Examples](https://github.com/kvernet/arco/tree/main/examples) — runnable usage examples

## Python Reference

The Python reference implementation that first validated the methodology is available at [arco-python](https://github.com/kvernet/arco-python).

## License

MIT