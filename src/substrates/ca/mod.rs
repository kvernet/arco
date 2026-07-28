//! Cellular Automaton substrate.
//!
//! This module implements the Information Universe traits for 1D
//! binary cellular automata with periodic boundary conditions.
//!
//! # Type parameters
//!
//! The CA substrate uses const generics for flexibility:
//! - `N`: Number of cells (state space = 2^N)
//! - `R`: Neighborhood radius (default 1 for elementary CA)
//!
//! # Components
//!
//! - **State**: [`CAState<N, R>`] — N cells with periodic boundaries.
//! - **Rules**: [`CARule<N, R>`] — lookup-table-based rule with
//!   measurable properties for hypothesis generation.
//! - **Observation**: Full state, density, and parity observers.
//! - **Schedule**: [`SynchronousCASchedule`] — all cells update
//!   simultaneously.
//!
//! # Usage
//!
//! ```rust,no_run
//! use arco::substrates::ca::CAUniverse;
//! use arco::cycle::{CycleConfig, run_cycle};
//! use rand::{rngs::StdRng, SeedableRng};
//!
//! let mut rng = StdRng::seed_from_u64(42);
//! let config = CycleConfig::default();
//! let universe = CAUniverse::<8, 1>::new("full_state", &mut rng, config.n_train + config.n_test);
//! let mut hypotheses = vec![];
//! let record = run_cycle(&universe, &config, &mut hypotheses, None);
//! ```

pub mod hypotheses;
pub mod observation;
pub mod rules;
pub mod schedule;
pub mod state;
pub mod universe;

// Re-export commonly used types
pub use hypotheses::generate_ca_hypotheses;
pub use observation::{DensityObserver, FullStateObserver, ParityObserver};
pub use rules::CARule;
pub use schedule::SynchronousCASchedule;
pub use state::CAState;
pub use universe::CAUniverse;
