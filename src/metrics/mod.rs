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
//! - **NSB**: Nemenman-Shafee-Bialek for finite alphabet.

pub mod entropy;
pub mod mm;
pub mod nsb;
pub mod persistence;
pub mod qe;
pub mod separation;
pub mod shuffle;
pub mod storage;

pub use entropy::{dmi, nmi};
pub use nsb::NsbCardinality;
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
impl Estimator {
    pub fn name(&self) -> &str {
        match self {
            Self::Plugin => "Plugin",
            Self::MM => "MM",
            Self::QE => "QE",
            Self::NSB => "NSB",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricConfig {
    pub estimator: Estimator,
    pub cardinality: NsbCardinality,
    pub max_delta: usize,
    pub n_shuffles: usize,
    pub seed: u64,
}

impl Default for MetricConfig {
    fn default() -> Self {
        Self {
            estimator: Estimator::Plugin,
            cardinality: NsbCardinality::Observed,
            max_delta: 15,
            n_shuffles: 10,
            seed: 42,
        }
    }
}
