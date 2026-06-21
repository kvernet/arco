# ARCO

**Automated Research into Computational Ontologies**

A computational science platform for discovering the conditions under which computation, memory, and learning emerge in arbitrary information systems.

## Status

v0.1.0 — Validated against the [reference implementation](https://github.com/kvernet/arco-python). Core library complete.

## What ARCO Does

ARCO asks a different question than most computer science: not "what can a given computational model compute?" but "what computational models are possible, and why do they emerge?"

It formalizes this through **Information Universes** — 6-tuples of (state space, transformations, observations, resources, invariants, schedule) — and measures emergent computation via shuffle-corrected normalized mutual information calibrated against destructive null distributions.

## Discovered Laws

Running the scientific cycle with default parameters reproduces:

- **Transport Law** (H5): Rule sets containing information transport operations (PROPAGATE, SWAP, COPY_TO_OUT, COPY_FROM_IN) exhibit storage at significantly higher rates (91.2% accuracy in Python reference, 60% in Rust v0.1.0).
- **Structure-Storage Gradient**: Storage probability increases monotonically with the fraction of structured rules, from ~14% (pure noise) to ~91% (pure structure).
- **Boolean Rediscovery**: NAND, AND, OR, NOR, and XOR gates are rediscovered without explicit encoding.

## Quick Start

```rust
use arco::cycle::{CycleConfig, run_cycle};

fn main() {
    let config = CycleConfig::default();
    let record = run_cycle(&config);
    println!("{}", record.summary());
}
```

## Build

```bash
make all        # fmt, clippy, test
cargo run       # run the scientific cycle
```

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

## Python Reference

The Python reference implementation that validated the methodology is available at [ARCO Python](https://github.com/kvernet/arco-python).

## License

MIT