use std::hash::Hash;

use crate::metrics::{
    Estimator, MetricConfig,
    mm::{MMShuffleConfig, shuffle_corrected_mm},
    nsb::{NsbShuffleConfig, shuffle_corrected_nsb},
    qe::{QEShuffleConfig, shuffle_corrected_qe},
    shuffle::{ShuffleConfig, shuffle_corrected},
};

pub fn corrected_nmi<T: Eq + Hash + Clone>(x: &[T], y: &[T], config: &MetricConfig) -> f64 {
    match config.estimator {
        Estimator::Plugin => {
            let plugin_config = ShuffleConfig::new(config.n_shuffles, config.seed);
            shuffle_corrected(x, y, &plugin_config)
        }
        Estimator::MM => {
            let mm_config = MMShuffleConfig::new(config.n_shuffles, config.seed);
            shuffle_corrected_mm(x, y, &mm_config)
        }
        Estimator::QE => {
            let qe_config = QEShuffleConfig::new(config.n_shuffles, config.seed);
            shuffle_corrected_qe(x, y, &qe_config)
        }
        Estimator::NSB => {
            let nsb_config = NsbShuffleConfig {
                cardinality: config.cardinality.clone(),
                n_shuffles: config.n_shuffles,
                seed: config.seed,
                ..NsbShuffleConfig::default()
            };
            shuffle_corrected_nsb(x, y, &nsb_config)
        }
    }
}

/// Storage: maximum shuffle-corrected NMI across all timescales,
/// using pooled estimation.
///
/// All observation pairs from all ensemble members and all timesteps
/// are pooled before computing NMI. This gives the estimator
/// sufficient samples to distinguish signal from shuffle baseline.
pub fn storage<T: Eq + Hash + Clone>(trajectories: &[Vec<T>], config: &MetricConfig) -> f64 {
    storage_profile(trajectories, config)
        .into_iter()
        .fold(0.0, f64::max)
}

/// Pooled shuffle-corrected NMI at every timescale Δ ∈ [1, Δmax] that
/// had enough pooled pairs to estimate (more than 10, matching the
/// threshold `storage` has always used).
///
/// Shared by [`storage`] (max over Δ) and [`memory`] (mean over Δ) so
/// the two metrics are computed from the exact same per-Δ estimates,
/// not independent runs that could disagree due to shuffle-seed
/// differences.
fn storage_profile<T: Eq + Hash + Clone>(
    trajectories: &[Vec<T>],
    config: &MetricConfig,
) -> Vec<f64> {
    let n_traj = trajectories.len();
    if n_traj < 2 {
        return Vec::new();
    }

    let traj_len = trajectories.iter().map(|t| t.len()).min().unwrap_or(0);
    let max_delta = config.max_delta.min(traj_len.saturating_sub(1));
    let mut scores = Vec::with_capacity(max_delta);

    for delta in 1..=max_delta {
        let mut all_x = Vec::new();
        let mut all_y = Vec::new();

        for traj in trajectories {
            for t in 0..(traj.len().saturating_sub(delta)) {
                all_x.push(&traj[t]);
                all_y.push(&traj[t + delta]);
            }
        }

        if all_x.len() > 10 {
            let tconfig = MetricConfig {
                seed: config.seed + delta as u64,
                ..config.clone()
            };
            scores.push(corrected_nmi(&all_x, &all_y, &tconfig));
        }
    }

    scores
}

/// Memory: average recoverable information about the past, across
/// timescales.
///
/// This is **not** an alias for [`storage`]. Both are computed from
/// the same per-Δ pooled, shuffle-corrected NMI profile
/// ([`storage_profile`]) — storage takes the maximum over Δ, memory
/// takes the mean. Each term is itself clamped to `[0, 1]`
/// (per Constitution), so both metrics live on the same `[0, 1]`
/// scale and are directly comparable:
///
/// - **High storage, low memory**: information survives at one
///   specific timescale and nowhere else — a sharp resonance, not
///   durable retention.
/// - **Storage ≈ memory**: information persists broadly and roughly
///   equally across timescales.
/// - **Memory ≤ storage always**, since storage is the max of the
///   same terms memory averages.
///
/// Returns `0.0` if no Δ had enough pooled samples to estimate (the
/// same condition under which [`storage`] also returns `0.0`).
///
/// # Relation to Active Information Storage
///
/// This is not "active information storage" (Lizier et al.),
/// which conditions on the entire past history rather than a single
/// lagged pair. That estimator needs ensembles much larger than
/// ARCO's calibration currently uses to avoid being dominated by
/// small-sample bias — this metric is the version that is actually
/// trustworthy at the ensemble sizes ARCO runs today. It differs from
/// `storage` in aggregation (mean vs. max across Δ), not in what kind
/// of information-theoretic quantity it estimates.
pub fn memory<T: Eq + Hash + Clone>(trajectories: &[Vec<T>], config: &MetricConfig) -> f64 {
    let scores = storage_profile(trajectories, config);
    if scores.is_empty() {
        0.0
    } else {
        scores.iter().sum::<f64>() / scores.len() as f64
    }
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> MetricConfig {
        MetricConfig {
            max_delta: 4,
            n_shuffles: 5,
            ..MetricConfig::default()
        }
    }

    #[test]
    fn memory_and_storage_agree_on_degenerate_input() {
        let cfg = config();
        // Fewer than 2 trajectories: both metrics are defined as 0.0.
        let single: Vec<Vec<u8>> = vec![vec![0, 1, 0, 1]];
        assert_eq!(storage(&single, &cfg), 0.0);
        assert_eq!(memory(&single, &cfg), 0.0);

        let empty: Vec<Vec<u8>> = vec![];
        assert_eq!(storage(&empty, &cfg), 0.0);
        assert_eq!(memory(&empty, &cfg), 0.0);
    }

    #[test]
    fn memory_never_exceeds_storage() {
        // memory is the mean of the same per-delta profile storage
        // takes the max of -- mean(x) <= max(x) for any non-empty x,
        // so this must hold regardless of the underlying trajectories.
        let cfg = config();
        let trajectories: Vec<Vec<u8>> = vec![
            vec![0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0],
            vec![1, 0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 1],
            vec![0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 1],
            vec![1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0],
        ];

        let s = storage(&trajectories, &cfg);
        let m = memory(&trajectories, &cfg);
        assert!(
            m <= s + 1e-9,
            "memory ({m}) exceeded storage ({s}); the mean of a profile can never exceed its max"
        );
    }

    #[test]
    fn memory_and_storage_are_different_reductions() {
        // Regression guard against silently reverting to `memory =
        // storage`: recompute the shared profile independently and
        // check `storage` is wired to its max and `memory` to its
        // mean. Deliberately does not assert a specific numeric gap
        // -- the exact NMI values depend on the shuffle-correction
        // RNG -- only that the two public functions are genuinely
        // different reductions over the same profile.
        let cfg = config();
        let trajectories: Vec<Vec<u8>> = vec![
            vec![0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0],
            vec![1, 0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 1],
            vec![0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 1],
            vec![1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0],
        ];

        let profile = storage_profile(&trajectories, &cfg);
        assert!(
            profile.len() > 1,
            "need more than one timescale in the profile for this test to be meaningful"
        );

        let expected_max = profile.iter().cloned().fold(0.0, f64::max);
        let expected_mean = profile.iter().sum::<f64>() / profile.len() as f64;

        assert!((storage(&trajectories, &cfg) - expected_max).abs() < 1e-9);
        assert!((memory(&trajectories, &cfg) - expected_mean).abs() < 1e-9);
    }
}
