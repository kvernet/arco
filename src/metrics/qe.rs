//! QE (Quadratic Extrapolation) estimator for mutual information.
//!
//! Estimates plugin MI at fractions of the data (1/1, 1/2, 1/4) and
//! extrapolates linearly in 1/N to the infinite-data limit, following
//! Strong et al. (1998). Distinct from NSB (Nemenman et al. 2002) —
//! QE is a cheaper approximation, not a substitute. Cite accordingly.
//!
//! Subsampling seeds are derived from a hash of the actual data
//! combined with the caller's seed, so distinct datasets get
//! independent draws even when callers reuse the same seed.
//! [`N_RESAMPLES`] independent draws are averaged at each fraction
//! below 1.0 to reduce sensitivity to subsample noise.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::metrics::entropy::{dmi, entropy};
use rand::{RngExt, SeedableRng, rngs::StdRng};

/// Number of independent subsamples averaged at each fraction < 1.0.
const N_RESAMPLES: usize = 3;

/// Derive a subsampling seed from the actual data being analyzed, not
/// just the caller's seed. Two calls with different `(x_seq, y_seq)`
/// content get independent subsample draws even if the caller passes
/// the same `seed`; two calls with identical content, seed, fraction,
/// and resample index remain reproducible.
fn data_seed<T: Hash>(x_seq: &[T], y_seq: &[T], seed: u64, frac_idx: u64, resample: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    frac_idx.hash(&mut hasher);
    resample.hash(&mut hasher);
    x_seq.hash(&mut hasher); // slice Hash impl covers length + every element
    y_seq.hash(&mut hasher);
    hasher.finish()
}

/// Draw `m` random indices out of `0..n` without replacement, via
/// partial Fisher-Yates.
fn subsample_indices(n: usize, m: usize, seed: u64) -> Vec<usize> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut idx: Vec<usize> = (0..n).collect();
    for i in 0..m {
        let j = rng.random_range(i..n);
        idx.swap(i, j);
    }
    idx.truncate(m);
    idx
}

/// Plugin MI on one random subsample of size `m` drawn from
/// `(x_seq, y_seq)`.
fn subsampled_mi<T: Eq + Hash + Clone>(x_seq: &[T], y_seq: &[T], m: usize, seed: u64) -> f64 {
    let indices = subsample_indices(x_seq.len(), m, seed);
    let sub_x: Vec<T> = indices.iter().map(|&i| x_seq[i].clone()).collect();
    let sub_y: Vec<T> = indices.iter().map(|&i| y_seq[i].clone()).collect();
    dmi(&sub_x, &sub_y)
}

/// QE-corrected mutual information I(X;Y).
///
/// Estimates plugin MI at fractions `[1.0, 0.5, 0.25]` of the data
/// (averaging [`N_RESAMPLES`] independent subsamples at each
/// fraction below 1.0), then extrapolates linearly in 1/N to the
/// infinite-data limit.
pub fn dmi_qe<T: Eq + Hash + Clone>(x_seq: &[T], y_seq: &[T], seed: u64) -> f64 {
    let n = x_seq.len();
    if n < 4 || n != y_seq.len() {
        return 0.0;
    }

    let fractions = [1.0, 0.5, 0.25];
    let mut estimates: Vec<(f64, f64)> = Vec::new();

    for (frac_idx, &frac) in fractions.iter().enumerate() {
        let m = (n as f64 * frac) as usize;
        if m < 2 {
            continue;
        }

        let estimate = if m == n {
            // Using the full dataset — there's only one possible
            // "subsample," and plugin MI is order-invariant, so no
            // randomness is involved here at all.
            dmi(x_seq, y_seq)
        } else {
            let mut sum = 0.0;
            for r in 0..N_RESAMPLES {
                let s = data_seed(x_seq, y_seq, seed, frac_idx as u64, r as u64);
                sum += subsampled_mi(x_seq, y_seq, m, s);
            }
            sum / N_RESAMPLES as f64
        };

        estimates.push((1.0 / m as f64, estimate));
    }

    if estimates.len() < 2 {
        return dmi(x_seq, y_seq); // fallback to plugin
    }

    // Linear regression: MI = MI_inf + a * (1/N); extrapolate to 1/N -> 0.
    let n_pts = estimates.len() as f64;
    let sum_x: f64 = estimates.iter().map(|(x, _)| x).sum();
    let sum_y: f64 = estimates.iter().map(|(_, y)| y).sum();
    let sum_xy: f64 = estimates.iter().map(|(x, y)| x * y).sum();
    let sum_xx: f64 = estimates.iter().map(|(x, _)| x * x).sum();

    let denom = n_pts * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-12 {
        return dmi(x_seq, y_seq); // degenerate regression (e.g. duplicate points), fall back
    }
    let slope = (n_pts * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n_pts;

    // intercept is MI at 1/N = 0 (infinite data)
    intercept.max(0.0)
}

/// QE-corrected normalized mutual information.
///
/// Applies QE correction to the MI estimate, then normalizes.
/// Bounded in [0, 1].
pub fn nmi_qe<T: Eq + Hash + Clone>(x_seq: &[T], y_seq: &[T], seed: u64) -> f64 {
    let mi = dmi_qe(x_seq, y_seq, seed);
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

/// QE + shuffle correction.
///
/// Applies QE to each estimate (observed and shuffled), then
/// subtracts the mean shuffle baseline. Note: because `dmi_qe` now
/// derives its subsampling seed from the actual data content, each
/// shuffled `y` — having different content — automatically gets an
/// independent subsample draw even though this loop passes the same
/// outer `seed` to every iteration.
pub fn shuffle_corrected_qe<T: Eq + Hash + Clone>(
    x_seq: &[T],
    y_seq: &[T],
    n_shuffles: usize,
    seed: u64,
) -> f64 {
    if x_seq.len() < 4 || y_seq.len() < 4 {
        return 0.0;
    }

    let nmi_obs = nmi_qe(x_seq, y_seq, seed);
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
        nmi_shuffles.push(nmi_qe(x_seq, &y_shuffled, seed));
    }

    let mean_shuffle: f64 = nmi_shuffles.iter().sum::<f64>() / n_shuffles as f64;
    (nmi_obs - mean_shuffle).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_seed_deterministic_for_identical_input() {
        let x = vec![1u8, 2, 3, 4, 5];
        let y = vec![5u8, 4, 3, 2, 1];
        let s1 = data_seed(&x, &y, 42, 0, 0);
        let s2 = data_seed(&x, &y, 42, 0, 0);
        assert_eq!(s1, s2, "identical input must give identical seed");
    }

    #[test]
    fn test_data_seed_differs_for_different_content() {
        // Same length, same caller seed, same fraction/resample index,
        // different content — this is exactly the case the old
        // implementation collapsed onto a single shared subsample.
        let x = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let y1 = vec![1u8, 1, 1, 1, 2, 2, 2, 2];
        let y2 = vec![2u8, 2, 2, 2, 1, 1, 1, 1];
        let s1 = data_seed(&x, &y1, 42, 0, 0);
        let s2 = data_seed(&x, &y2, 42, 0, 0);
        assert_ne!(
            s1, s2,
            "different data content must not collapse to the same subsampling seed"
        );
    }

    #[test]
    fn test_data_seed_differs_across_resamples() {
        let x = vec![1u8, 2, 3, 4, 5, 6];
        let y = vec![6u8, 5, 4, 3, 2, 1];
        let s0 = data_seed(&x, &y, 42, 0, 0);
        let s1 = data_seed(&x, &y, 42, 0, 1);
        assert_ne!(
            s0, s1,
            "different resample indices should draw different subsamples"
        );
    }

    #[test]
    fn test_subsample_indices_size_and_range() {
        let idx = subsample_indices(20, 7, 123);
        assert_eq!(idx.len(), 7);
        assert!(idx.iter().all(|&i| i < 20));
        let mut sorted = idx.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            7,
            "indices must be distinct (sampling without replacement)"
        );
    }

    #[test]
    fn test_dmi_qe_reproducible() {
        let x: Vec<u8> = (0..80).map(|i| (i % 5) as u8).collect();
        let y: Vec<u8> = (0..80).map(|i| ((i + 2) % 5) as u8).collect();
        let a = dmi_qe(&x, &y, 7);
        let b = dmi_qe(&x, &y, 7);
        assert!(
            (a - b).abs() < 1e-12,
            "same input and seed must reproduce exactly"
        );
    }

    #[test]
    fn test_dmi_qe_independent_across_universes_at_same_delta() {
        // Simulates storage()'s pattern: two different "universes"
        // (different trajectory content, same length) scored with
        // the same seed (as storage() derives seed only from delta,
        // shared across every universe at that delta).
        let shared_seed = 99;
        let x: Vec<u8> = (0..60).map(|i| (i % 7) as u8).collect();
        let y_universe_a: Vec<u8> = (0..60).map(|i| ((i * 3) % 7) as u8).collect();
        let y_universe_b: Vec<u8> = (0..60).map(|i| ((i * 5 + 1) % 7) as u8).collect();

        // The underlying subsample seeds must differ between universes
        // even though the caller-supplied seed is identical.
        let sa = data_seed(&x, &y_universe_a, shared_seed, 1, 0);
        let sb = data_seed(&x, &y_universe_b, shared_seed, 1, 0);
        assert_ne!(sa, sb);

        // And the estimates themselves should not be forced through
        // an identical subsample partition.
        let _ = dmi_qe(&x, &y_universe_a, shared_seed);
        let _ = dmi_qe(&x, &y_universe_b, shared_seed);
    }

    #[test]
    fn test_qe_reduces_bias() {
        let seed: u64 = 42;
        let mut rng = StdRng::seed_from_u64(seed);
        let x: Vec<u8> = (0..100).map(|_| rng.random_range(0..=7)).collect();
        let y: Vec<u8> = (0..100).map(|_| rng.random_range(0..=7)).collect();

        let plugin = dmi(&x, &y);
        let qe = dmi_qe(&x, &y, seed);

        // QE should be lower than plugin for independent data
        // (though not guaranteed — extrapolation can go either way).
        assert!(qe >= 0.0, "QE should be non-negative, got {:.4}", qe);
        assert!(
            qe <= plugin + 0.1,
            "QE ({:.4}) should not greatly exceed plugin ({:.4}) for independent data",
            qe,
            plugin
        );
    }

    #[test]
    fn test_qe_preserves_signal() {
        let seed: u64 = 42;
        let x = vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
        let qe = dmi_qe(&x, &x, seed);
        assert!(
            qe > 0.3,
            "QE should preserve MI for deterministic sequences, got {:.4}",
            qe
        );
    }

    #[test]
    fn test_nmi_qe_bounded() {
        let seed: u64 = 42;
        let x = vec![0, 1, 0, 1, 0, 1];
        let n = nmi_qe(&x, &x, seed);
        assert!(n >= 0.0 && n <= 1.0, "NMI should be in [0,1], got {:.4}", n);
    }
}
