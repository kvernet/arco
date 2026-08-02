//! Shuffle correction for mutual information bias.
//!
//! The plugin MI estimator has small-sample bias. Shuffle correction
//! subtracts the mean NMI of temporally permuted data to estimate
//! and remove this bias.
//!
//! # Limitations
//!
//! Global shuffling assumes no long-range temporal autocorrelation
//! in the null distribution. For periodic or strongly autocorrelated
//! systems, use block shuffling or surrogate null models.

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use crate::metrics::entropy::nmi;

/// Shuffle-corrected normalized mutual information.
///
/// NMI_corrected = NMI_observed - mean(NMI_shuffled), where the
/// shuffle baseline is estimated by randomly permuting Y and
/// recomputing NMI. Subtracts the small-sample bias of the plugin
/// estimator.
///
/// # Arguments
/// * `x_seq`, `y_seq` — Observation sequences.
/// * `n_shuffles` — Number of shuffle iterations (≥ 5 recommended).
/// * `seed` — Seed for reproducible shuffling.
///
/// # Returns
/// Shuffle-corrected NMI in [0, 1]. Zero if observed NMI does not
/// exceed the shuffle baseline.
pub fn shuffle_corrected<T: Eq + std::hash::Hash + Clone>(
    x_seq: &[T],
    y_seq: &[T],
    n_shuffles: usize,
    seed: u64,
) -> f64 {
    if x_seq.len() < 4 || y_seq.len() < 4 {
        return 0.0;
    }

    let nmi_obs = nmi(x_seq, y_seq);
    if nmi_obs == 0.0 {
        return 0.0;
    }

    let mut rng = StdRng::seed_from_u64(seed);
    let mut y_shuffled: Vec<T> = y_seq.to_vec();
    let mut nmi_shuffles = Vec::with_capacity(n_shuffles);

    for _ in 0..n_shuffles {
        // Fisher-Yates shuffle in-place
        for i in (1..y_shuffled.len()).rev() {
            let j = rng.random_range(0..=i);
            y_shuffled.swap(i, j);
        }
        nmi_shuffles.push(nmi(x_seq, &y_shuffled));
    }

    let mean_shuffle: f64 = nmi_shuffles.iter().sum::<f64>() / n_shuffles as f64;
    (nmi_obs - mean_shuffle).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shuffle_correction_reduces_noise() {
        let mut rng = StdRng::seed_from_u64(42);
        let x: Vec<u8> = (0..100).map(|_| rng.random_range(0..=1)).collect();
        let y: Vec<u8> = (0..100).map(|_| rng.random_range(0..=1)).collect();
        let corrected = shuffle_corrected(&x, &y, 5, 42);
        assert!(
            corrected < 0.1,
            "Shuffle-corrected NMI should be near 0 for independent data, got {}",
            corrected
        );
    }

    #[test]
    fn test_identical_sequences_high_nmi() {
        let x = vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
        let corrected = shuffle_corrected(&x, &x, 5, 42);
        assert!(
            corrected > 0.5,
            "Identical sequences should have high NMI, got {}",
            corrected
        );
    }

    #[test]
    fn test_clamped_to_zero_one() {
        let x = vec![0, 1, 0, 1, 0, 1, 0, 1];
        let corrected = shuffle_corrected(&x, &x, 5, 42);
        assert!(corrected >= 0.0 && corrected <= 1.0);
    }
}
