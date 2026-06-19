//! Emergence metrics for Information Universes.
//!
//! Per Constitution:
//!     Emergence metrics are objective functions mapping ensemble
//!     trajectories to real-valued scores. All metrics use:
//!     - Ensemble estimation (multiple trajectories from distinct
//!       initial states)
//!     - Shuffle-corrected normalized mutual information (bias correction)
//!     - Calibrated thresholds against null distributions
//!
//! Metrics defined:
//! - Storage: maximum information preservation across all timescales
//! - Memory: alias for storage (information about the past remains
//!   recoverable)
//! - Persistence: per-timestep information preservation (requires
//!   large ensembles to be reliable)
//! - Trajectory separation: distinguishability of futures given
//!   different initial conditions (diagnostic only)
//!
//! Design commitments:
//! - All MI-based metrics use bias correction via temporal shuffling.
//! - Metrics operate on observation sequences, not raw states.
//! - Storage uses pooled estimation (all timesteps + ensemble members).
//!
//! # Estimator limitations
//!
//! The plugin MI estimator used here is not asymptotically unbiased.
//! For publication-quality results on larger state spaces, replace
//! with a Bayesian (NSB), Miller-Madow, or kNN-based estimator.
//! The estimator boundary is the `shuffle_corrected_nmi` function.

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

/// Plugin estimator of mutual information I(X;Y) from discrete observations.
///
/// Uses the empirical joint distribution. Observation values must be hashable.
/// Biased upward when the observation alphabet size is comparable to sample size.
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
/// NMI_corrected = NMI_observed - mean(NMI_shuffled), where the shuffle
/// baseline is estimated by randomly permuting Y and recomputing NMI.
/// This subtracts the small-sample bias of the plugin NMI estimator.
///
/// Per Constitution: all MI-based emergence metrics
/// must use shuffle correction.
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
        // Fisher-Yates shuffle on y_shuffled
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
/// Computes average shuffle-corrected NMI between ensemble observations
/// at time t and time t+Δ, averaged over all t.
///
/// Per Constitution.
///
/// # Limitations
///
/// At Δ=1 with small ensembles (n ≤ 10), the per-timestep estimator
/// rarely exceeds the shuffle baseline. Use storage (pooled estimation)
/// as the primary emergence signal.
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

/// Persistence at multiple timescales.
///
/// Returns a mapping from Δ to persistence score. Used to diagnose
/// timescale separation — universes with zero Δ=1 persistence but
/// nonzero Δ≫1 persistence exhibit the Persistence-Storage Decoupling.
pub fn compute_persistence_multiscale<T: Eq + Hash + Clone>(
    trajectories: &[Vec<T>],
    deltas: &[usize],
    n_shuffles: usize,
    seed: u64,
) -> HashMap<usize, f64> {
    deltas
        .iter()
        .map(|&delta| {
            (
                delta,
                compute_persistence(trajectories, delta, n_shuffles, seed),
            )
        })
        .collect()
}

/// Storage: maximum persistence across all timescales.
///
/// Uses pooled estimation: all observation pairs from all ensemble
/// members and all timesteps are pooled before computing NMI. This
/// gives the estimator sufficient samples to distinguish signal
/// from shuffle baseline.
///
/// Per Constitution.
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

/// Memory: information about past observations remains recoverable.
///
/// Alias for `compute_storage`. Memory is the capacity of a system
/// to preserve information about its past such that it can be
/// recovered later — which is exactly what storage measures via
/// I(O_t; O_{t+Δ}).
///
/// Per Constitution.
pub fn compute_memory<T: Eq + Hash + Clone>(
    trajectories: &[Vec<T>],
    max_delta: usize,
    n_shuffles: usize,
    seed: u64,
) -> f64 {
    compute_storage(trajectories, max_delta, n_shuffles, seed)
}

/// Trajectory separation: how distinguishable are futures given
/// different initial conditions?
///
/// Measures total variation distance between conditional output
/// distributions. High values indicate sensitivity to initial
/// conditions — NOT memory.
///
/// # Warning
///
/// This metric may be confused as "memory". It is NOT memory.
/// A chaotic system with no information preservation can
/// score high. Use `compute_memory` for actual memory measurement.
/// This metric is preserved for diagnostic purposes.
pub fn compute_trajectory_separation<T: Eq + Hash + Clone>(
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
        // Build conditional distributions: initial observation -> later observations
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

                // Build count maps for total variation distance
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

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_deterministic() {
        let values = vec![1, 1, 1, 1];
        assert_eq!(entropy(&values), 0.0);
    }

    #[test]
    fn test_entropy_uniform() {
        let values = vec![0, 1, 0, 1];
        let h = entropy(&values);
        assert!((h - 1.0).abs() < 0.01); // log2(2) = 1.0
    }

    #[test]
    fn test_mi_identical_sequences() {
        let x = vec![0, 1, 0, 1, 0, 1, 0, 1];
        let y = x.clone();
        let mi = discrete_mutual_information(&x, &y);
        assert!(mi > 0.5); // should be close to 1.0 in NMI terms
    }

    #[test]
    fn test_mi_independent() {
        let x = vec![0, 0, 1, 1, 0, 0, 1, 1];
        let y = vec![0, 1, 0, 1, 0, 1, 0, 1];
        let mi = discrete_mutual_information(&x, &y);
        // Should be low since x and y are independent
        assert!(mi < 0.3);
    }

    #[test]
    fn test_nmi_bounded() {
        let x = vec![0, 1, 0, 1, 0, 1, 0, 1];
        let nmi = normalized_mutual_information(&x, &x);
        assert!((nmi - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_shuffle_correction_reduces_noise() {
        // Random data should have NMI near 0 after shuffle correction
        let mut rng = StdRng::seed_from_u64(42);
        let x: Vec<u8> = (0..100).map(|_| rng.random_range(0..=1)).collect();
        let y: Vec<u8> = (0..100).map(|_| rng.random_range(0..=1)).collect();
        let corrected = shuffle_corrected_nmi(&x, &y, 5, 42);
        // Should be near 0 for independent sequences
        assert!(
            corrected < 0.1,
            "Shuffle-corrected NMI should be near 0 for independent data, got {}",
            corrected
        );
    }

    #[test]
    fn test_storage_finds_signal() {
        // Create trajectories with clear temporal dependence
        let traj1 = vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
        let traj2 = vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
        let trajectories = vec![traj1, traj2];
        let storage = compute_storage(&trajectories, 5, 5, 42);
        // Should detect the deterministic alternation
        assert!(
            storage > 0.3,
            "Storage should detect temporal pattern, got {}",
            storage
        );
    }

    #[test]
    fn test_memory_is_storage() {
        let traj1 = vec![0, 0, 1, 1, 0, 0, 1, 1];
        let traj2 = vec![0, 0, 1, 1, 0, 0, 1, 1];
        let trajectories = vec![traj1, traj2];
        let storage = compute_storage(&trajectories, 5, 5, 42);
        let memory = compute_memory(&trajectories, 5, 5, 42);
        assert_eq!(storage, memory);
    }

    #[test]
    fn test_trajectory_separation_low_for_identical_trajectories() {
        let traj1 = vec![0, 1, 0, 1, 0];
        let traj2 = vec![0, 1, 0, 1, 0];
        let trajectories = vec![traj1, traj2];
        let sep = compute_trajectory_separation(&trajectories, 3);
        // Same trajectories should have 0 separation
        assert!(
            sep < 0.01,
            "Identical trajectories should have near-zero separation, got {}",
            sep
        );
    }
}
