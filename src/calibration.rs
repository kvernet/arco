//! Threshold calibration for emergence metrics.
//!
//! Per Constitution:
//!     Emergence thresholds must be calibrated per universe class using
//!     the 95th percentile of a null distribution from purely destructive
//!     rule sets. This ensures that "emergence" means "statistically
//!     distinguishable from noise."
//!
//! The calibration procedure:
//!     1. Generate null trajectories using destructive-only rule sets.
//!     2. Compute the emergence metric for each null universe.
//!     3. Set the threshold to the specified percentile of the null
//!        distribution.
//!     4. Apply engineering floors to prevent degenerate thresholds
//!        when the null distribution is pathologically compressed.
//!        These floors are safeguards against statistical artifacts
//!        in small-null-sample regimes, not scientific priors.
//!
//! Design commitments:
//! - Calibration is universe-class-specific.
//! - The null distribution uses destructive rules for information
//!   scrambling, establishing the baseline for emergence detection.
//! - Returns null distribution statistics for effect size estimation.
//! - Observation operators are windowed (window_size ≥ 1).

use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::dynamics::{DEFAULT_SCHEDULE, generate_ensemble};
use crate::metrics::{compute_memory, compute_persistence, compute_storage};
use crate::rules::RewriteRule;
use crate::rules::Rule;
use crate::rules::create_destructive_rules;
use crate::state::BinaryGraphState;

// ===================================================================
// Null distribution generation
// ===================================================================

/// Generate trajectory ensembles for null-distribution calibration.
///
/// Each null universe uses a purely destructive rule set (structured
/// ratio = 0.0). Trajectories are generated from random initial states
/// using the all-vertices schedule.
///
/// # Parameters
/// * `n_universes` — Number of null universes (≥ 30 recommended).
/// * `n_vertices` — Number of vertices per state.
/// * `n_ensemble` — Ensemble size per universe.
/// * `steps` — Timesteps per trajectory.
/// * `window_size` — Window size for the observation operator.
/// * `max_rules_per_subset` — Maximum rules per null universe.
/// * `obs_fn` — Windowed observation operator.
/// * `seed` — Random seed for reproducibility.
///
/// # Returns
/// Vector of null trajectory ensembles.
#[allow(clippy::too_many_arguments)]
pub fn generate_null_trajectories<O: Clone>(
    n_universes: usize,
    n_vertices: usize,
    n_ensemble: usize,
    steps: usize,
    window_size: usize,
    max_rules_per_subset: usize,
    obs_fn: &dyn Fn(&[BinaryGraphState]) -> O,
    seed: u64,
) -> Vec<Vec<Vec<O>>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let destructive_pool = create_destructive_rules();

    // Pre-filter scramblers to guarantee at least one per null universe
    let scramblers: Vec<&RewriteRule> = destructive_pool
        .iter()
        .filter(|r| r.name().starts_with("DESTROY_SCRAMBLE_ALL"))
        .collect();

    let state_pool: Vec<BinaryGraphState> = (0..n_ensemble * 2)
        .map(|_| BinaryGraphState::random(n_vertices, &mut rng))
        .collect();

    let mut null_ensembles = Vec::with_capacity(n_universes);

    for i in 0..n_universes {
        let size = rng.random_range(1..=max_rules_per_subset);
        let mut rules = Vec::with_capacity(size);

        // Always include at least one scrambler
        let scrambler_idx = rng.random_range(0..scramblers.len());
        rules.push(scramblers[scrambler_idx].clone());

        // Fill remaining slots from the full destructive pool
        for _ in 1..size {
            let idx = rng.random_range(0..destructive_pool.len());
            rules.push(destructive_pool[idx].clone());
        }

        // Shuffle
        for j in (1..rules.len()).rev() {
            let k = rng.random_range(0..=j);
            rules.swap(j, k);
        }

        let initial_states: Vec<BinaryGraphState> =
            state_pool.iter().take(n_ensemble).cloned().collect();

        let ensemble = generate_ensemble(
            &initial_states,
            &rules,
            steps,
            n_ensemble,
            window_size,
            &DEFAULT_SCHEDULE,
            obs_fn,
            seed + i as u64 * 1000,
        );

        null_ensembles.push(ensemble);
    }

    null_ensembles
}

// ===================================================================
// Threshold calibration
// ===================================================================

/// Calibration result with thresholds and null distribution statistics.
#[derive(Debug, Clone)]
pub struct CalibrationResult {
    /// Calibrated persistence threshold.
    pub persistence_threshold: f64,
    /// Calibrated storage threshold.
    pub storage_threshold: f64,
    /// Calibrated memory threshold.
    pub memory_threshold: f64,
    /// Null distribution statistics for persistence.
    pub null_persistence: NullStats,
    /// Null distribution statistics for storage.
    pub null_storage: NullStats,
    /// Null distribution statistics for memory.
    pub null_memory: NullStats,
    /// The percentile used for thresholding.
    pub percentile: f64,
}

/// Statistics from a null distribution.
#[derive(Debug, Clone)]
pub struct NullStats {
    /// Mean of the null distribution.
    pub mean: f64,
    /// Standard deviation of the null distribution.
    pub std: f64,
    /// Raw scores from null universes.
    pub scores: Vec<f64>,
}

impl NullStats {
    /// Empirical p-value: fraction of null scores ≥ observed_value.
    pub fn empirical_p(&self, observed_value: f64) -> f64 {
        let count = self.scores.iter().filter(|&&s| s >= observed_value).count();
        if self.scores.is_empty() {
            1.0
        } else {
            count as f64 / self.scores.len() as f64
        }
    }
}

/// Calibrate emergence thresholds from null ensembles.
///
/// Computes each emergence metric on the null ensembles, then sets
/// the threshold to the specified percentile. Engineering floors
/// prevent degenerate thresholds.
///
/// # Parameters
/// * `null_ensembles` — Null trajectory ensembles.
/// * `percentile` — Percentile for threshold (0–100).
/// * `floor_persistence` — Engineering floor for persistence.
/// * `floor_storage` — Engineering floor for storage.
/// * `floor_memory` — Engineering floor for memory.
/// * `max_delta` — Maximum timescale for storage/memory.
/// * `n_shuffles` — Number of shuffles for bias correction.
/// * `seed` — Seed for shuffle RNG.
///
/// # Returns
/// Calibration result with thresholds and null statistics.
#[allow(clippy::too_many_arguments)]
pub fn calibrate_thresholds<O: Eq + std::hash::Hash + Clone>(
    null_ensembles: &[Vec<Vec<O>>],
    percentile: f64,
    floor_persistence: f64,
    floor_storage: f64,
    floor_memory: f64,
    max_delta: usize,
    n_shuffles: usize,
    seed: u64,
) -> CalibrationResult {
    let mut persistence_scores = Vec::with_capacity(null_ensembles.len());
    let mut storage_scores = Vec::with_capacity(null_ensembles.len());
    let mut memory_scores = Vec::with_capacity(null_ensembles.len());

    for ensemble in null_ensembles {
        persistence_scores.push(compute_persistence(ensemble, 1, n_shuffles, seed));
        storage_scores.push(compute_storage(ensemble, max_delta, n_shuffles, seed));
        memory_scores.push(compute_memory(ensemble, max_delta, n_shuffles, seed));
    }

    let persistence_threshold =
        percentile_value(&persistence_scores, percentile).max(floor_persistence);
    let storage_threshold = percentile_value(&storage_scores, percentile).max(floor_storage);
    let memory_threshold = percentile_value(&memory_scores, percentile).max(floor_memory);

    CalibrationResult {
        persistence_threshold,
        storage_threshold,
        memory_threshold,
        null_persistence: NullStats::from_scores(persistence_scores),
        null_storage: NullStats::from_scores(storage_scores),
        null_memory: NullStats::from_scores(memory_scores),
        percentile,
    }
}

/// Compute a percentile value from a slice of scores.
fn percentile_value(scores: &[f64], percentile: f64) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    if scores.len() == 1 {
        return scores[0];
    }
    let mut sorted: Vec<f64> = scores.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let k = (percentile / 100.0) * (sorted.len() - 1) as f64;
    let lo = k.floor() as usize;
    let hi = k.ceil() as usize;

    if lo == hi {
        sorted[lo]
    } else {
        let frac = k - lo as f64;
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    }
}

impl NullStats {
    fn from_scores(scores: Vec<f64>) -> Self {
        let n = scores.len() as f64;
        let mean = if n > 0.0 {
            scores.iter().sum::<f64>() / n
        } else {
            0.0
        };
        let std = if n > 1.0 {
            let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (n - 1.0);
            variance.sqrt()
        } else {
            0.0
        };
        Self { mean, std, scores }
    }
}

// ===================================================================
// Convenience: run full calibration pipeline
// ===================================================================

/// Run the full calibration pipeline and return thresholds.
///
/// Convenience wrapper around `generate_null_trajectories` and
/// `calibrate_thresholds`.
#[allow(clippy::too_many_arguments)]
pub fn calibrate<O: Eq + std::hash::Hash + Clone>(
    n_null_universes: usize,
    n_vertices: usize,
    n_ensemble: usize,
    steps: usize,
    window_size: usize,
    max_rules_per_subset: usize,
    obs_fn: &dyn Fn(&[BinaryGraphState]) -> O,
    percentile: f64,
    max_delta: usize,
    n_shuffles: usize,
    seed: u64,
) -> CalibrationResult {
    let null_ensembles = generate_null_trajectories(
        n_null_universes,
        n_vertices,
        n_ensemble,
        steps,
        window_size,
        max_rules_per_subset,
        obs_fn,
        seed,
    );

    calibrate_thresholds(
        &null_ensembles,
        percentile,
        0.01, // floor_persistence
        0.01, // floor_storage
        0.01, // floor_memory
        max_delta,
        n_shuffles,
        seed,
    )
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::observe_windowed;

    #[test]
    fn test_generate_null_trajectories() {
        let nulls = generate_null_trajectories(10, 3, 5, 20, 1, 3, &observe_windowed, 42);
        assert_eq!(nulls.len(), 10);
        for ensemble in &nulls {
            assert_eq!(ensemble.len(), 5);
            for traj in ensemble {
                assert_eq!(traj.len(), 21);
            }
        }
    }

    #[test]
    fn test_calibrate_thresholds_produces_valid_result() {
        let nulls = generate_null_trajectories(20, 3, 5, 20, 1, 3, &observe_windowed, 42);

        let result = calibrate_thresholds(&nulls, 95.0, 0.01, 0.01, 0.01, 10, 5, 42);

        assert!(result.persistence_threshold >= 0.01);
        assert!(result.storage_threshold >= 0.01);
        assert!(result.memory_threshold >= 0.01);
        assert_eq!(result.percentile, 95.0);
        assert!(!result.null_storage.scores.is_empty());
    }

    #[test]
    fn test_null_stats_empirical_p() {
        let stats = NullStats {
            mean: 0.1,
            std: 0.05,
            scores: vec![0.05, 0.08, 0.10, 0.12, 0.15],
        };
        assert!((stats.empirical_p(0.12) - 0.4).abs() < 0.01);
        assert_eq!(stats.empirical_p(1.0), 0.0);
        assert_eq!(stats.empirical_p(0.0), 1.0);
    }

    #[test]
    fn test_percentile_value() {
        let scores = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
        assert!((percentile_value(&scores, 50.0) - 0.5).abs() < 0.01);
        assert!((percentile_value(&scores, 95.0) - 0.95).abs() < 0.01);
        assert!((percentile_value(&scores, 0.0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_calibrate_convenience() {
        let result = calibrate(10, 3, 5, 20, 1, 3, &observe_windowed, 95.0, 10, 5, 42);
        assert!(result.storage_threshold >= 0.01);
    }
}
