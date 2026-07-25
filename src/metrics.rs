//! Emergence metrics for Information Universes.
//!
//! Per the Mathematical Constitution:
//!     Emergence metrics are objective functions mapping ensemble
//!     trajectories to real-valued scores. All metrics use:
//!     - Ensemble estimation (multiple trajectories from distinct
//!       initial states)
//!     - Shuffle-corrected normalized mutual information
//!     - Calibrated thresholds against null distributions
//!
//! # Metrics
//!
//! - **Storage**: Maximum information preservation across all
//!   timescales, using pooled estimation.
//! - **Memory**: Alias for storage — recoverable information about
//!   the past.
//! - **Persistence**: Per-timestep information preservation at a
//!   given timescale Δ.
//! - **Initial condition separation**: How distinguishable are
//!   futures given different initial states? (Diagnostic only.)
//!
//! # Estimator
//!
//! The plugin mutual information estimator is used with shuffle
//! correction to remove small-sample bias. The estimator is modular:
//! replace `shuffle_corrected_nmi` with a Bayesian (NSB) or k-NN
//! estimator without changing the metric functions.
//!
//! # Limitations
//!
//! - The plugin estimator is biased when the observation alphabet
//!   is large relative to sample size. Shuffle correction mitigates
//!   but does not eliminate this.
//! - Per-timestep persistence requires larger ensembles than storage
//!   to be reliable.
//! - Global shuffling assumes no long-range temporal autocorrelation
//!   in the null distribution.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;

// ===================================================================
// Information-theoretic primitives
// ===================================================================

/// Shannon entropy from a slice of observation values.
fn entropy<T: Eq + Hash>(values: &[T]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let total = values.len() as f64;
    let mut counts: HashMap<&T, usize> = HashMap::new();
    for v in values {
        *counts.entry(v).or_insert(0) += 1;
    }
    let mut h = 0.0;
    for count in counts.values() {
        let p = *count as f64 / total;
        if p > 0.0 {
            h -= p * (p.log2());
        }
    }
    h
}

/// Plugin estimator of mutual information I(X;Y) from discrete
/// observations.
///
/// Uses the empirical joint distribution. Observation values must
/// be hashable. Biased upward when the observation alphabet size
/// is comparable to sample size.
pub fn discrete_mutual_information<T: Eq + Hash + Clone>(x_seq: &[T], y_seq: &[T]) -> f64 {
    if x_seq.len() < 2 || y_seq.len() < 2 || x_seq.len() != y_seq.len() {
        return 0.0;
    }

    let total = x_seq.len() as f64;
    let mut joint_counts: HashMap<(usize, usize), usize> = HashMap::new();
    let mut x_indices: HashMap<&T, usize> = HashMap::new();
    let mut y_indices: HashMap<&T, usize> = HashMap::new();
    let mut x_counter: usize = 0;
    let mut y_counter: usize = 0;

    for (x, y) in x_seq.iter().zip(y_seq.iter()) {
        let xi = *x_indices.entry(x).or_insert_with(|| {
            let idx = x_counter;
            x_counter += 1;
            idx
        });
        let yi = *y_indices.entry(y).or_insert_with(|| {
            let idx = y_counter;
            y_counter += 1;
            idx
        });
        *joint_counts.entry((xi, yi)).or_insert(0) += 1;
    }

    let mut mi = 0.0;
    let mut x_counts: Vec<usize> = vec![0; x_counter];
    let mut y_counts: Vec<usize> = vec![0; y_counter];

    for ((xi, yi), count) in &joint_counts {
        x_counts[*xi] += count;
        y_counts[*yi] += count;
    }

    for ((xi, yi), count) in &joint_counts {
        let p_xy = *count as f64 / total;
        let p_x = x_counts[*xi] as f64 / total;
        let p_y = y_counts[*yi] as f64 / total;
        if p_xy > 0.0 && p_x > 0.0 && p_y > 0.0 {
            mi += p_xy * (p_xy / (p_x * p_y)).log2();
        }
    }

    mi.max(0.0)
}

/// Normalized mutual information: I(X;Y) / sqrt(H(X) * H(Y)).
///
/// Bounded in [0, 1]. Makes values comparable across observation
/// operators with different entropy.
pub fn normalized_mutual_information<T: Eq + Hash + Clone>(x_seq: &[T], y_seq: &[T]) -> f64 {
    let mi = discrete_mutual_information(x_seq, y_seq);
    if mi == 0.0 {
        return 0.0;
    }

    let h_x = entropy(x_seq);
    let h_y = entropy(y_seq);

    if h_x == 0.0 || h_y == 0.0 {
        return 0.0;
    }

    (mi / (h_x * h_y).sqrt()).clamp(0.0, 1.0)
}

/// Bias-corrected normalized mutual information.
///
/// NMI_corrected = NMI_observed - mean(NMI_shuffled), where the
/// shuffle baseline is estimated by randomly permuting Y and
/// recomputing NMI. This subtracts the small-sample bias of the
/// plugin NMI estimator.
///
/// Per the Constitution: all MI-based emergence metrics must use
/// shuffle correction.
///
/// # Limitations
///
/// Global shuffling assumes no long-range temporal autocorrelation
/// in the null distribution. For periodic or strongly autocorrelated
/// systems, use block shuffling or circular phase randomization.
pub fn shuffle_corrected_nmi<T: Eq + Hash + Clone>(
    x_seq: &[T],
    y_seq: &[T],
    n_shuffles: usize,
    seed: u64,
) -> f64 {
    if x_seq.len() < 4 || y_seq.len() < 4 {
        return 0.0;
    }

    let nmi_obs = normalized_mutual_information(x_seq, y_seq);
    if nmi_obs == 0.0 {
        return 0.0;
    }

    let mut rng = StdRng::seed_from_u64(seed);
    let mut y_shuffled: Vec<T> = y_seq.to_vec();
    let mut nmi_shuffles = Vec::with_capacity(n_shuffles);

    for _ in 0..n_shuffles {
        for i in (1..y_shuffled.len()).rev() {
            let j = rng.random_range(0..=i);
            y_shuffled.swap(i, j);
        }
        nmi_shuffles.push(normalized_mutual_information(x_seq, &y_shuffled));
    }

    let mean_shuffle: f64 = nmi_shuffles.iter().sum::<f64>() / n_shuffles as f64;
    (nmi_obs - mean_shuffle).clamp(0.0, 1.0)
}

// ===================================================================
// Emergence metrics
// ===================================================================

/// Persistence: information preservation at timescale Δ.
///
/// Computes average shuffle-corrected NMI between ensemble
/// observations at time t and time t+Δ, averaged over all t.
///
/// # Limitations
///
/// At Δ=1 with small ensembles (n ≤ 10), the per-timestep estimator
/// rarely exceeds the shuffle baseline. Use [`compute_storage`]
/// (pooled estimation) as the primary emergence signal.
pub fn compute_persistence<T: Eq + Hash + Clone>(
    trajectories: &[Vec<T>],
    delta: usize,
    n_shuffles: usize,
    seed: u64,
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
        let score = shuffle_corrected_nmi(&obs_t, &obs_td, n_shuffles, seed + t as u64);
        scores.push(score);
    }

    if scores.is_empty() {
        0.0
    } else {
        scores.iter().sum::<f64>() / scores.len() as f64
    }
}

/// Storage: maximum shuffle-corrected NMI across all timescales,
/// using pooled estimation.
///
/// All observation pairs from all ensemble members and all timesteps
/// are pooled before computing NMI. This gives the estimator
/// sufficient samples to distinguish signal from shuffle baseline.
pub fn compute_storage<T: Eq + Hash + Clone>(
    trajectories: &[Vec<T>],
    max_delta: usize,
    n_shuffles: usize,
    seed: u64,
) -> f64 {
    let n_traj = trajectories.len();
    if n_traj < 2 {
        return 0.0;
    }

    let traj_len = trajectories.iter().map(|t| t.len()).min().unwrap_or(0);
    let max_delta = max_delta.min(traj_len.saturating_sub(1));
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
            let score = shuffle_corrected_nmi(&all_x, &all_y, n_shuffles, seed);
            best = best.max(score);
        }
    }

    best
}

/// Memory: recoverable information about the past.
///
/// Alias for [`compute_storage`]. In ARCO, memory is quantified as
/// delayed mutual information I(O_t; O_{t+Δ}), maximized over Δ.
/// This measures how much information survives over time.
///
/// Note: This is not the same as "active information storage"
/// (Lizier et al.), which conditions on the entire past history.
/// ARCO's definition is intentionally simpler and computable from
/// finite ensembles.
pub fn compute_memory<T: Eq + Hash + Clone>(
    trajectories: &[Vec<T>],
    max_delta: usize,
    n_shuffles: usize,
    seed: u64,
) -> f64 {
    compute_storage(trajectories, max_delta, n_shuffles, seed)
}

/// Initial condition separation: how distinguishable are futures
/// given different initial states?
///
/// For each pair of distinct initial observations (at t=0), computes
/// the total variation distance between their conditional output
/// distributions after Δ steps. High values mean different initial
/// states produce distinguishable futures.
///
/// This measures sensitivity to initial conditions — NOT memory.
/// A chaotic deterministic system can score high even if it
/// preserves no recoverable information about the past.
///
/// # Limitation
///
/// This metric only conditions on the first observation (`traj[0]`),
/// not on every intermediate state. For non-stationary dynamics,
/// consider conditioning on multiple timepoints.
///
/// Preserved for diagnostic use alongside [`compute_memory`].
pub fn compute_initial_condition_separation<T: Eq + Hash + Clone>(
    trajectories: &[Vec<T>],
    max_delta: usize,
) -> f64 {
    let n_traj = trajectories.len();
    if n_traj < 2 {
        return 0.0;
    }

    let traj_len = trajectories.iter().map(|t| t.len()).min().unwrap_or(0);
    let max_delta = max_delta.min(traj_len.saturating_sub(1));
    let mut scores = Vec::new();

    for delta in 1..=max_delta {
        let mut initial_to_later: HashMap<&T, Vec<&T>> = HashMap::new();

        for traj in trajectories {
            initial_to_later
                .entry(&traj[0])
                .or_default()
                .push(&traj[delta]);
        }

        let initial_vals: Vec<&T> = initial_to_later.keys().copied().collect();

        for i in 0..initial_vals.len() {
            for j in (i + 1)..initial_vals.len() {
                let later_i = &initial_to_later[initial_vals[i]];
                let later_j = &initial_to_later[initial_vals[j]];

                let all_keys: HashSet<&&T> = later_i.iter().chain(later_j.iter()).collect();
                let total_i = later_i.len() as f64;
                let total_j = later_j.len() as f64;

                if total_i > 0.0 && total_j > 0.0 {
                    let mut tv = 0.0;
                    for key in &all_keys {
                        let count_i = later_i.iter().filter(|&x| x == *key).count() as f64;
                        let count_j = later_j.iter().filter(|&x| x == *key).count() as f64;
                        tv += (count_i / total_i - count_j / total_j).abs();
                    }
                    scores.push(0.5 * tv);
                }
            }
        }
    }

    if scores.is_empty() {
        0.0
    } else {
        scores.iter().sum::<f64>() / scores.len() as f64
    }
}
