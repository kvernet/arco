//! Emergence metrics for Information Universes.
//!
//! # Estimators
//!
//! - **Plugin** (default): empirical MI with shuffle correction.
//!   Fast but biased with large alphabets.
//! - **Miller-Madow**: first-order bias correction. Better for
//!   moderate alphabet sizes.
//! - **NSB**: Bayesian estimator for small-sample, large-alphabet
//!   regimes. Gold standard, slower.

pub mod entropy;
pub mod mm;
pub mod nsb;
pub mod persistence;
pub mod separation;
pub mod shuffle;
pub mod storage;

pub use entropy::{dmi, nmi};
pub use persistence::persistence;
pub use separation::init_separation;
pub use shuffle::shuffle_corrected;
pub use storage::{memory, storage};

// In mod.rs
#[derive(Debug, Clone, Copy)]
pub enum Estimator {
    Plugin,
    MillerMadow,
    Nsb,
}

#[derive(Debug, Clone)]
pub struct MetricConfig {
    pub estimator: Estimator,
    pub max_delta: usize,
    pub n_shuffles: usize,
    pub seed: u64,
}

impl Default for MetricConfig {
    fn default() -> Self {
        Self {
            estimator: Estimator::Plugin,
            max_delta: 15,
            n_shuffles: 10,
            seed: 42,
        }
    }
}
