//! Information-theoretic primitives.
//!
//! Provides Shannon entropy, discrete mutual information (`dmi`),
//! and normalized mutual information (`nmi`). These are the
//! building blocks for all emergence metrics.
//!
//! # Estimator note
//!
//! The `dmi` function uses the plugin (empirical) estimator. It is
//! biased upward when the observation alphabet is large relative to
//! sample size. For bias-corrected estimates, use the [`mm`] or
//! [`qe`] modules.

use std::collections::HashMap;
use std::hash::Hash;

/// Shannon entropy in bits.
///
/// H(X) = -Σ p(x) log₂ p(x). Returns 0.0 for empty input.
pub(crate) fn entropy<T: Eq + Hash>(values: &[T]) -> f64 {
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
            h -= p * p.log2();
        }
    }
    h
}

/// Plugin estimator returning MI and distinct value counts.
///
/// Returns (mi, k_xy, m_x, m_y) where:
/// - k_xy: number of distinct joint (x,y) pairs observed
/// - m_x: number of distinct x values observed
/// - m_y: number of distinct y values observed
pub(crate) fn dmi_with_counts<T: Eq + Hash + Clone>(
    x_seq: &[T],
    y_seq: &[T],
) -> (f64, usize, usize, usize) {
    // ... same logic as dmi, but return counts alongside mi
    //(mi, joint_counts.len(), x_counter, y_counter)

    let n = x_seq.len();
    if n < 2 || n != y_seq.len() {
        return (0.0, 0, n, y_seq.len());
    }

    let total = n as f64;
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

    //mi.max(0.0)

    (mi, joint_counts.len(), x_counter, y_counter)
}

/// Plugin estimator of discrete mutual information I(X;Y).
///
/// Uses the empirical joint distribution from observed sequences.
/// Observation values must be hashable.
///
/// # Bias
///
/// Biased upward when the observation alphabet size is comparable
/// to sample size. For large-alphabet regimes, use [`mm::dmi_mm`]
/// or [`qe::dmi_qe`] instead.
pub fn dmi<T: Eq + Hash + Clone>(x_seq: &[T], y_seq: &[T]) -> f64 {
    dmi_with_counts(x_seq, y_seq).0.max(0.0)
}

/// Normalized mutual information: I(X;Y) / sqrt(H(X) · H(Y)).
///
/// Bounded in [0, 1]. Normalization makes values comparable across
/// observation operators with different entropy.
pub fn nmi<T: Eq + Hash + Clone>(x_seq: &[T], y_seq: &[T]) -> f64 {
    let mi = dmi(x_seq, y_seq);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_deterministic() {
        assert_eq!(entropy(&[1, 1, 1, 1]), 0.0);
    }

    #[test]
    fn test_entropy_uniform() {
        let h = entropy(&[0, 1, 0, 1]);
        assert!((h - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_dmi_identical() {
        let x = vec![0, 1, 0, 1, 0, 1];
        let mi = dmi(&x, &x);
        assert!(mi > 0.5);
    }

    #[test]
    fn test_nmi_bounded() {
        let x = vec![0, 1, 0, 1];
        let n = nmi(&x, &x);
        assert!((n - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_nmi_clamped() {
        let x = vec![0, 1];
        let n = nmi(&x, &x);
        assert!(n >= 0.0 && n <= 1.0);
    }
}
