//! Emergence metrics for Information Universes.
//!
//! # Estimators
//!
//! - **Plugin** (default): empirical MI with shuffle correction.
//!   Fast but biased with large alphabets.
//! - **Miller-Madow**: first-order bias correction. Better for
//!   moderate alphabet sizes.
//! - **QE**: Quadratic Extrapolation estimator for small-sample, large-alphabet
//!   regimes.

pub mod entropy;
pub mod mm;
pub mod nsb;
pub mod persistence;
pub mod qe;
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
    /// Default Plugin estimator
    Plugin,
    /// Miller-Madow estimator
    MM,
    /// Quadratic Extrapolation estimator
    QE,
    /// Nemenman-Shafee-Bialek
    NSB,
}

#[derive(Debug, Clone)]
pub struct MetricConfig {
    pub estimator: Estimator,
    pub max_delta: usize,
    pub n_shuffles: usize,
    /// Nominal alphabet size for NSB estimator.
    pub k_x: usize,
    pub k_y: usize,
    pub k_xy: usize,
    pub seed: u64,
}

impl Default for MetricConfig {
    fn default() -> Self {
        Self {
            estimator: Estimator::Plugin,
            max_delta: 15,
            n_shuffles: 10,
            k_x: 256,
            k_y: 256,
            k_xy: 256 * 256,
            seed: 42,
        }
    }
}
