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

## Estimators

Every information metric is estimated with a selectable estimator,
chosen with `--estimator <plugin|mm|qe|nsb>` (default `plugin`) or
`MetricConfig::estimator` in the library. Shuffle correction is applied
to all of them, and calibration and scoring always use the same one.

| CLI value | `Estimator` | Method | Since |
|---|---|---|---|
| `plugin` | `Plugin` | Empirical-frequency (plug-in) estimate. Default. | all |
| `mm` | `MM` | Miller–Madow first-order bias correction | 0.5.0 |
| `qe` | `QE` | Quadratic extrapolation (Strong et al., 1998) | 0.5.0 |
| `nsb` | `NSB` | Nemenman–Shafee–Bialek (2002), Bayesian mixture of Dirichlet priors; alphabet cardinalities set by `MetricConfig::cardinality` | 0.6.0 |

QE and NSB are different methods; cite them accordingly. In yanked
releases before 0.5.0, `--estimator nsb` actually ran QE (see [CHANGELOG](https://github.com/kvernet/arco/blob/main/CHANGELOG.md)).
Estimator choice changes calibrated thresholds, so compare results only within one
estimator, and report the estimator (and the NSB cardinality policy) with any result.

## Installation

```toml
[dependencies]
arco = "0.6"
```

Requires Rust 1.85+.

## Documentation

- [Experimental Results](https://github.com/kvernet/arco/tree/main/docs/RESULTS.md) — key findings with data
- [Estimator Consistency Benchmark](https://github.com/kvernet/arco/tree/main/docs/benchmarks/estimator_consistency.md) — exact ground truth validation
- [Mathematical Constitution](https://github.com/kvernet/arco/tree/main/docs/constitution.md) — formal specification
- [API documentation](https://docs.rs/arco) — rustdoc
- [Examples](https://github.com/kvernet/arco/tree/main/examples) — runnable usage examples


## Python Reference

[arco-python](https://github.com/kvernet/arco-python) — the Python implementation that first validated the methodology.

## License

MIT