//! ARCO — Automated Research into Computational Ontologies
//!
//! A computational science platform for discovering and characterizing
//! the minimal mathematical conditions under which computation, memory,
//! and learning emerge within arbitrary information systems.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use arco::cycle::{CycleConfig, run_cycle};
//! use arco::substrates::graph::{
//!     BinaryGraphUniverse, generate_standard_hypotheses,
//! };
//! use rand::{SeedableRng, rngs::StdRng};
//!
//! let mut rng = StdRng::seed_from_u64(42);
//! let config = CycleConfig::default();
//! let universe = BinaryGraphUniverse::new(3, "compound", &mut rng, config.n_train + config.n_test);
//! let mut hypotheses = generate_standard_hypotheses();
//! let record = run_cycle(&universe, &config, &mut hypotheses, None);
//! println!("{}", record.summary());
//! ```
//!
//! # Documentation
//!
//! - [Mathematical Constitution](https://github.com/kvernet/arco/blob/main/docs/constitution.md)
//! - [API documentation](https://docs.rs/arco)
//! - [Examples](https://github.com/kvernet/arco/tree/main/examples)

pub mod calibration;
pub mod cycle;
pub mod hypotheses;
pub mod metrics;
pub mod observation;
pub mod record;
pub mod rules;
pub mod schedule;
pub mod state;
pub mod substrates;
pub mod types;
pub mod universe;
