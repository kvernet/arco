//! Binary Graph Universe — ARCO's validation substrate.
//!
//! This module implements the Information Universe traits for
//! directed graphs with binary vertex labels and binary edge
//! labels. This is the first substrate built for ARCO and
//! served as the validation case for the measurement apparatus.
//!
//! # Components
//!
//! - **State**: [`BinaryGraphState`] — a directed graph with n
//!   vertices, each labeled {0, 1}, each edge labeled {0, 1}.
//! - **Rules**: [`RewriteRule`] — local graph rewrite rules with
//!   pattern matching via [`MatchInfo`]. Includes 16 structured
//!   rules (logic gates, transport, identity) and 8 destructive
//!   rules (scramblers, constants).
//! - **Observation**: Single-state and windowed observers at
//!   varying granularities (full state, labels only, edges only,
//!   scalar aggregates).
//! - **Schedule**: [`AllVerticesSchedule`] — asynchronous
//!   exhaustive update: every vertex visited once per timestep
//!   in random order, first matching rule fires.
//! - **Universe**: [`BinaryGraphUniverse`] — bundles all components
//!   and implements [`InformationUniverse`].
//! - **Hypotheses**: Standard hypothesis set including the
//!   Transport Law (H5) and the Structure-Storage Gradient.
//! - **Validation**: Boolean function verification for NAND, AND,
//!   OR, NOR, and XOR under stochastic scheduling.
//!
//! # Usage
//!
//! ```rust,no_run
//! use arco::cycle::{CycleConfig, run_cycle};
//! use arco::substrates::graph::{BinaryGraphUniverse, generate_standard_hypotheses};
//! use rand::SeedableRng;
//! use rand::rngs::StdRng;
//!
//!
//! let mut rng = StdRng::seed_from_u64(42);
//! let config = CycleConfig::default();
//! let universe = BinaryGraphUniverse::new(3, "compound", &mut rng, config.n_train + config.n_test);
//!
//! let mut hypotheses = generate_standard_hypotheses();
//! let record = run_cycle(&universe, &config, &mut hypotheses, None);
//!
//! println!("{}", record.summary());
//! ```

pub mod hypotheses;
pub mod observation;
pub mod rules;
pub mod schedule;
pub mod state;
pub mod universe;
pub mod validation;

// Re-export commonly used types
pub use hypotheses::generate_standard_hypotheses;
pub use observation::observe_compound;
pub use rules::{MatchInfo, RewriteRule, create_destructive_rules, create_structured_rules};
pub use schedule::AllVerticesSchedule;
pub use state::BinaryGraphState;
pub use universe::BinaryGraphUniverse;
pub use validation::{test_boolean_function, verify_boolean_functions};
