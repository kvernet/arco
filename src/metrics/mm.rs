//! Miller-Madow bias correction for mutual information.
//!
//! The Miller-Madow correction subtracts a first-order bias term
//! from the plugin MI estimate: MI_mm = MI_plugin - (K_xy - m_x - m_y + 1) / (2N ln 2)
//! where m_x and m_y are the number of distinct values observed.
//!
//! # Limitations
//!
//! Miller-Madow is a first-order correction. It works well for
//! moderate alphabet sizes but underestimates bias in severely
//! undersampled regimes. For large-alphabet, small-sample cases,
//! use the QE estimator instead.

use std::f64::consts;
use std::hash::Hash;

use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::metrics::entropy::{dmi_with_counts, entropy};

/// Miller-Madow corrected mutual information I(X;Y).
///
/// Subtracts (K_xy - m_x - m_y + 1) / (2N ln 2) from the plugin estimate,
/// where m_x and m_y count distinct values and N is the sample size.
pub fn dmi_mm<T: Eq + Hash + Clone>(x_seq: &[T], y_seq: &[T]) -> f64 {
    let n = x_seq.len();
    if n < 2 || n != y_seq.len() {
        return 0.0;
    }

    let total = n as f64;
    let (plugin, k_xy, m_x, m_y) = dmi_with_counts(x_seq, y_seq);

    // Standard Miller-Madow: (K_xy - m_x - m_y + 1) / (2N ln 2)
    let k_xy = k_xy as f64;
    let m_x = m_x as f64;
    let m_y = m_y as f64;
    let correction = (k_xy - m_x - m_y + 1.0) / (2.0 * total * consts::LN_2);

    (plugin - correction).max(0.0)
}

/// Miller-Madow corrected normalized mutual information.
///
/// Applies Miller-Madow to the MI estimate, then normalizes.
/// Bounded in [0, 1].
pub fn nmi_mm<T: Eq + Hash + Clone>(x_seq: &[T], y_seq: &[T]) -> f64 {
    let mi = dmi_mm(x_seq, y_seq);
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

/// Miller-Madow + shuffle correction.
///
/// Applies Miller-Madow to each estimate (observed and shuffled),
/// then subtracts the mean shuffle baseline. This combines the
/// first-order bias correction with temporal null calibration.
pub fn shuffle_corrected_mm<T: Eq + Hash + Clone>(
    x_seq: &[T],
    y_seq: &[T],
    n_shuffles: usize,
    seed: u64,
) -> f64 {
    if x_seq.len() < 4 || y_seq.len() < 4 {
        return 0.0;
    }

    let nmi_obs = nmi_mm(x_seq, y_seq);
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
        nmi_shuffles.push(nmi_mm(x_seq, &y_shuffled));
    }

    let mean_shuffle: f64 = nmi_shuffles.iter().sum::<f64>() / n_shuffles as f64;
    (nmi_obs - mean_shuffle).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::dmi;

    #[test]
    fn test_mm_reduces_plugin_bias() {
        // Independent random sequences — plugin overestimates, MM should be lower
        let mut rng = StdRng::seed_from_u64(42);
        let x: Vec<u8> = (0..50).map(|_| rng.random_range(0..=3)).collect();
        let y: Vec<u8> = (0..50).map(|_| rng.random_range(0..=3)).collect();

        let plugin = dmi(&x, &y);
        let mm = dmi_mm(&x, &y);

        // MM should be ≤ plugin (correction is always positive or zero)
        assert!(
            mm <= plugin + 0.001,
            "MM {:.4} should be ≤ plugin {:.4}",
            mm,
            plugin
        );
    }

    #[test]
    fn test_mm_preserves_signal() {
        // Identical sequences — MM should still show high MI
        let x = vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
        let mm = dmi_mm(&x, &x);
        assert!(
            mm > 0.5,
            "MM should preserve high MI for identical sequences, got {:.4}",
            mm
        );
    }

    #[test]
    fn test_nmi_mm_bounded() {
        let x = vec![0, 1, 0, 1, 0, 1];
        let n = nmi_mm(&x, &x);
        assert!(n >= 0.0 && n <= 1.0, "NMI should be in [0,1], got {:.4}", n);
    }
}
