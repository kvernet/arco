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

See [`examples/`](examples/) for more usage patterns.

## Discovered Laws

Running the scientific cycle reproduces five structural laws:

- **Transport Law** (H5): Rule sets containing information transport operations (PROPAGATE, SWAP, COPY_TO_OUT, COPY_FROM_IN) exhibit storage at significantly higher rates (~53% accuracy at n=50,000, 91% in Python reference).
- **Structure-Storage Gradient**: Storage probability increases monotonically with the fraction of structured rules, from ~18% (pure noise) to ~95% (pure structure).
- **Majority Structure Law** (H2): Majority-structured rule sets exhibit memory (~62% accuracy).
- **Logic Gate Law** (H3): Rule sets containing logic gates exhibit memory (~53% accuracy).
- **Multiple Logic Law** (H7): Rule sets with multiple logic gates exhibit memory (~58% accuracy).

Additionally, NAND, AND, OR, NOR, and XOR gates are rediscovered without explicit encoding.

## Performance

Benchmarked on a 20-core machine (n=50,000 universes, release build):

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

## Building from Source

```bash
git clone https://github.com/kvernet/arco.git
cd arco
make all        # format, clippy, tests
cargo build --release
./target/release/arco --help
```

## Documentation

- [Mathematical Constitution](https://kvernet.com/arco/docs/constitution/) — the formal specification for ARCO
- [API documentation](https://docs.rs/arco) — rustdoc for the latest release

## Python Reference

The Python reference implementation that validated the methodology is available at [ARCO Python](https://github.com/kvernet/arco-python).

## License

MIT