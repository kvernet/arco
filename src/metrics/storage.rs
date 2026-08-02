use std::hash::Hash;

use crate::metrics::{
    Estimator, MetricConfig, mm::shuffle_corrected_mm, nsb::shuffle_corrected_nsb,
    shuffle::shuffle_corrected,
};

pub fn corrected_nmi<T: Eq + Hash + Clone>(x: &[T], y: &[T], config: &MetricConfig) -> f64 {
    match config.estimator {
        Estimator::Plugin => shuffle_corrected(x, y, config.n_shuffles, config.seed),
        Estimator::MillerMadow => shuffle_corrected_mm(x, y, config.n_shuffles, config.seed),
        Estimator::Nsb => shuffle_corrected_nsb(x, y, config.n_shuffles, config.seed),
    }
}

/// Storage: maximum shuffle-corrected NMI across all timescales,
/// using pooled estimation.
///
/// All observation pairs from all ensemble members and all timesteps
/// are pooled before computing NMI. This gives the estimator
/// sufficient samples to distinguish signal from shuffle baseline.
pub fn storage<T: Eq + Hash + Clone>(trajectories: &[Vec<T>], config: &MetricConfig) -> f64 {
    let n_traj = trajectories.len();
    if n_traj < 2 {
        return 0.0;
    }

    let traj_len = trajectories.iter().map(|t| t.len()).min().unwrap_or(0);
    let max_delta = config.max_delta.min(traj_len.saturating_sub(1));
    let mut best: f64 = 0.0;

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
            let score = corrected_nmi(&all_x, &all_y, &tconfig);
            best = best.max(score);
        }
    }

    best
}

/// Memory: recoverable information about the past.
///
/// Alias for [`storage`]. In ARCO, memory is quantified as
/// delayed mutual information I(O_t; O_{t+Δ}), maximized over Δ.
/// This measures how much information survives over time.
///
/// Note: This is not the same as "active information storage"
/// (Lizier et al.), which conditions on the entire past history.
/// ARCO's definition is intentionally simpler and computable from
/// finite ensembles.
pub fn memory<T: Eq + Hash + Clone>(trajectories: &[Vec<T>], config: &MetricConfig) -> f64 {
    storage(trajectories, config)
}
