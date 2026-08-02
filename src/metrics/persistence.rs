use std::{collections::HashMap, hash::Hash};

use crate::metrics::{MetricConfig, storage::corrected_nmi};

/// Persistence: information preservation at timescale Δ.
///
/// Computes average corrected NMI between ensemble observations
/// at time t and time t+Δ, averaged over all t.
///
/// # Limitations
///
/// At Δ=1 with small ensembles (n ≤ 10), the per-timestep estimator
/// rarely exceeds the shuffle baseline. Use [`storage`]
/// (pooled estimation) as the primary emergence signal.
pub fn persistence<T: Eq + Hash + Clone>(
    trajectories: &[Vec<T>],
    delta: usize,
    config: &MetricConfig,
) -> f64 {
    let n_traj = trajectories.len();
    if n_traj < 2 {
        return 0.0;
    }

    let traj_len = trajectories.iter().map(|t| t.len()).min().unwrap_or(0);
    if delta >= traj_len {
        return 0.0;
    }

    let mut scores = Vec::new();

    for t in 0..(traj_len - delta) {
        let obs_t: Vec<&T> = trajectories.iter().map(|traj| &traj[t]).collect();
        let obs_td: Vec<&T> = trajectories.iter().map(|traj| &traj[t + delta]).collect();
        let tconfig = MetricConfig {
            seed: config.seed + t as u64 * 137,
            ..config.clone()
        };
        let score = corrected_nmi(&obs_t, &obs_td, &tconfig);
        scores.push(score);
    }

    if scores.is_empty() {
        0.0
    } else {
        scores.iter().sum::<f64>() / scores.len() as f64
    }
}

/// Persistence at multiple timescales.
///
/// Returns a mapping from Δ to persistence score. Used to diagnose
/// timescale separation — universes with zero Δ=1 persistence but
/// nonzero Δ≫1 persistence exhibit the Persistence-Storage Decoupling.
pub fn persistence_ms<T: Eq + Hash + Clone>(
    trajectories: &[Vec<T>],
    deltas: &[usize],
    config: &MetricConfig,
) -> HashMap<usize, f64> {
    deltas
        .iter()
        .map(|&d| {
            let dconfig = MetricConfig {
                seed: config.seed + d as u64 * 1000,
                ..config.clone()
            };
            (d, persistence(trajectories, d, &dconfig))
        })
        .collect()
}
