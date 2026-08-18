//! Nemenman–Shafee–Bialek entropy and mutual-information estimation.
//!
//! The NSB estimator is a Bayesian mixture of symmetric Dirichlet priors.
//!
//! `K` is represented as `u128` throughout the statistical/model layer.
//! Observed sample counts remain `usize`, since they cannot exceed the
//! number of samples actually present in memory.
//!
//! Numerical evaluation converts cardinalities to `f64` only at the
//! floating-point boundary.
//!
//! References:
//!
//! - Nemenman, Shafee & Bialek (2002),
//!   "Entropy and inference, revisited."
//! - Nemenman, Bialek & de Ruyter van Steveninck (2004),
//!   "Entropy and information in neural spike trains."
//! - Wolpert & Wolf (1995),
//!   posterior moments for Dirichlet distributions.
//!
//! https://arxiv.org/abs/physics/0108025

use std::collections::HashMap;
use std::hash::Hash;

use rand::{RngExt, SeedableRng, rngs::StdRng};

// ============================================================================
// Cardinality policy
// ============================================================================

/// Policy used to determine the finite alphabet cardinalities supplied to
/// the NSB estimator.
///
/// The important distinction is between:
///
/// - `Observed`: infer the cardinality from the actual sequence passed to
///   the estimator. This is resolved every time an NSB estimate is made.
/// - `Explicit`: use a known/modelled alphabet cardinality independently of
///   the observed support.
///
/// In particular, when `Observed` is used during shuffle correction, the
/// cardinality is re-resolved for every shuffled `(x, y)` pair. This means
/// that a shuffle that changes the observed support gets the corresponding
/// NSB cardinalities.
///
/// This is intentional: cardinality is a property of the dataset being
/// estimated, not a value that should necessarily be frozen before the
/// shuffle procedure.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NsbCardinality {
    /// Use the observed support as the finite alphabet.
    ///
    /// For a pair `(x, y)`:
    ///
    /// - `k_x`  = number of distinct values observed in `x`
    /// - `k_y`  = number of distinct values observed in `y`
    /// - `k_xy` = number of distinct `(x, y)` pairs observed
    ///
    /// These values are recomputed every time `resolve()` is called.
    #[default]
    Observed,

    /// Use known/modelled alphabet cardinalities.
    ///
    /// This is appropriate when the observer's codomain is known
    /// independently of the sampled trajectories.
    Explicit { k_x: u128, k_y: u128, k_xy: u128 },
}

impl NsbCardinality {
    /// Resolve the cardinalities for the supplied observations.
    ///
    /// This function is deliberately called from `nmi_nsb()` rather than
    /// once by the caller before shuffle correction.
    ///
    /// Consequently:
    ///
    /// ```text
    /// nmi_nsb(x, y)
    /// nmi_nsb(x, shuffled_y_1)
    /// nmi_nsb(x, shuffled_y_2)
    /// ...
    /// ```
    ///
    /// each independently resolves `Observed` cardinalities.
    pub fn resolve<T: Eq + Hash + Clone>(
        &self,
        x_seq: &[T],
        y_seq: &[T],
    ) -> Option<(u128, u128, u128)> {
        if x_seq.len() != y_seq.len() || x_seq.is_empty() {
            return None;
        }

        match self {
            Self::Observed => {
                let x_hist = histogram(x_seq);
                let y_hist = histogram(y_seq);

                let mut xy_hist: HashMap<(T, T), usize> = HashMap::new();

                for (x, y) in x_seq.iter().zip(y_seq.iter()) {
                    *xy_hist.entry((x.clone(), y.clone())).or_insert(0) += 1;
                }

                let k_x = x_hist.len() as u128;
                let k_y = y_hist.len() as u128;
                let k_xy = xy_hist.len() as u128;

                if k_x == 0 || k_y == 0 || k_xy == 0 {
                    return None;
                }

                Some((k_x, k_y, k_xy))
            }

            Self::Explicit { k_x, k_y, k_xy } => {
                if *k_x == 0 || *k_y == 0 || *k_xy == 0 {
                    None
                } else {
                    Some((*k_x, *k_y, *k_xy))
                }
            }
        }
    }
}

// ============================================================================
// Configuration
// ============================================================================

/// Numerical configuration for one NSB entropy estimate.
#[derive(Clone, Debug)]
pub struct NsbConfig {
    /// Finite alphabet cardinality.
    ///
    /// This is a model parameter, not the number of observed categories.
    ///
    /// `u128` is intentional because observer cardinalities can be much
    /// larger than `usize::MAX`.
    pub k: u128,

    /// Relative numerical tolerance for the adaptive quadrature.
    pub integration_tolerance: f64,

    /// Maximum adaptive subdivision depth.
    pub max_depth: usize,

    /// Maximum number of integrand evaluations.
    pub max_evaluations: usize,
}

impl Default for NsbConfig {
    fn default() -> Self {
        Self {
            k: 2,
            integration_tolerance: 1e-8,
            max_depth: 24,
            max_evaluations: 10_000,
        }
    }
}

/// Configuration for shuffle-corrected NSB normalized mutual information.
///
/// Cardinalities are represented by a policy rather than by already-resolved
/// `k_x`, `k_y`, and `k_xy` values.
///
/// This is important for `NsbCardinality::Observed`: the policy is resolved
/// independently for the observed data and for every shuffled dataset.
#[derive(Clone, Debug)]
pub struct NsbShuffleConfig {
    /// Policy used to determine `(k_x, k_y, k_xy)`.
    pub cardinality: NsbCardinality,

    /// Numerical configuration shared by all entropy calculations.
    pub entropy: NsbConfig,

    /// Number of random Y permutations used for shuffle correction.
    pub n_shuffles: usize,

    /// Seed for deterministic shuffling.
    pub seed: u64,
}

impl Default for NsbShuffleConfig {
    fn default() -> Self {
        Self {
            cardinality: NsbCardinality::Observed,
            entropy: NsbConfig::default(),
            n_shuffles: 10,
            seed: 0,
        }
    }
}

// ============================================================================
// Diagnostics
// ============================================================================

/// Numerical diagnostics for an NSB entropy estimate.
#[derive(Clone, Debug)]
pub struct NsbDiagnostics {
    /// Estimated entropy in bits.
    pub entropy_bits: f64,

    /// Posterior mean of beta.
    ///
    /// For the standard NSB hyperposterior this moment is not generally
    /// finite because the large-beta tail is heavy. Therefore this field is
    /// reported as +∞ rather than as a misleading finite cutoff-dependent
    /// number.
    pub posterior_mean_beta: f64,

    /// Conservative numerical integration error estimate in bits.
    pub integration_error: f64,

    /// Number of integrand evaluations.
    pub evaluations: usize,

    /// True when the requested quadrature tolerance was reached.
    pub converged: bool,

    /// Number of observed categories.
    pub k_observed: u128,

    /// Number of repeated observations, N - K_observed.
    pub coincidences: u128,

    /// True if K is smaller than the observed support.
    pub k_mismatch: bool,
}

// ============================================================================
// Special functions
// ============================================================================

#[allow(clippy::excessive_precision)]
pub fn lgamma(x: f64) -> f64 {
    const G: f64 = 7.0;

    const COEF: [f64; 9] = [
        0.999_999_999_999_809_93,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_13,
        -176.615_029_162_140_59,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];

    if x.is_nan() || x <= 0.0 {
        return f64::NAN;
    }

    if x < 0.5 {
        let pi = std::f64::consts::PI;
        return (pi / (pi * x).sin()).ln() - lgamma(1.0 - x);
    }

    let z = x - 1.0;

    let mut sum = COEF[0];

    for (i, coefficient) in COEF.iter().enumerate().skip(1) {
        sum += coefficient / (z + i as f64);
    }

    let t = z + G + 0.5;

    0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + sum.ln()
}

pub fn digamma(mut x: f64) -> f64 {
    if x.is_nan() || x <= 0.0 {
        return f64::NAN;
    }

    let mut result = 0.0;

    while x < 12.0 {
        result -= 1.0 / x;
        x += 1.0;
    }

    let inv = 1.0 / x;
    let inv2 = inv * inv;

    result += x.ln() - 0.5 * inv;

    result -= inv2
        * (1.0 / 12.0
            - inv2
                * (1.0 / 120.0
                    - inv2
                        * (1.0 / 252.0
                            - inv2
                                * (1.0 / 240.0
                                    - inv2 * (1.0 / 132.0 - inv2 * (691.0 / 32760.0))))));

    result
}

pub fn trigamma(mut x: f64) -> f64 {
    if x.is_nan() || x <= 0.0 {
        return f64::NAN;
    }

    let mut result = 0.0;

    while x < 12.0 {
        result += 1.0 / (x * x);
        x += 1.0;
    }

    let inv = 1.0 / x;
    let inv2 = inv * inv;

    result += inv + 0.5 * inv2;

    result += inv2
        * inv
        * (1.0 / 6.0
            - inv2
                * (1.0 / 30.0
                    - inv2
                        * (1.0 / 42.0
                            - inv2
                                * (1.0 / 30.0 - inv2 * (5.0 / 66.0 - inv2 * (691.0 / 2730.0))))));

    result
}

// ============================================================================
// Stable scalar helpers
// ============================================================================

fn log_gamma_ratio(z: f64, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }

    if !z.is_finite() || z <= 0.0 {
        return f64::NAN;
    }

    let nf = n as f64;

    if z > 100.0 * nf.max(1.0) {
        let n1 = nf - 1.0;

        let s1 = nf * n1 / 2.0;
        let s2 = nf * n1 * (2.0 * nf - 1.0) / 6.0;
        let s3 = nf * nf * n1 * n1 / 4.0;

        return nf * z.ln() + s1 / z - s2 / (2.0 * z * z) + s3 / (3.0 * z * z * z);
    }

    lgamma(z + nf) - lgamma(z)
}

fn dxi_dbeta(beta: f64, k: f64) -> f64 {
    if beta <= 0.0 || !beta.is_finite() || k <= 1.0 {
        return 0.0;
    }

    if beta < 1.0e6 {
        let value = k * trigamma(k * beta + 1.0) - trigamma(beta + 1.0);

        return if value.is_finite() && value > 0.0 {
            value
        } else {
            0.0
        };
    }

    let inv = 1.0 / beta;
    let inv2 = inv * inv;
    let inv3 = inv2 * inv;
    let inv5 = inv3 * inv2;
    let inv7 = inv5 * inv2;

    let k_inv = 1.0 / k;

    let value = (k - 1.0) * k_inv * 0.5 * inv2
        + (k_inv * k_inv - 1.0) / 6.0 * inv3
        + (1.0 - k_inv.powi(4)) / 30.0 * inv5
        + (1.0 - k_inv.powi(6)) / 42.0 * inv7;

    value.max(0.0)
}

// ============================================================================
// Count grouping
// ============================================================================

fn group_counts(counts: &[usize]) -> (Vec<(usize, usize)>, usize, usize) {
    let mut frequencies: HashMap<usize, usize> = HashMap::new();

    let mut n = 0usize;

    for &count in counts {
        if count == 0 {
            continue;
        }

        *frequencies.entry(count).or_insert(0) += 1;
        n = n.saturating_add(count);
    }

    let k_observed = frequencies.values().sum();

    let mut grouped: Vec<(usize, usize)> = frequencies.into_iter().collect();

    grouped.sort_unstable_by_key(|&(count, _)| count);

    (grouped, k_observed, n)
}

// ============================================================================
// NSB equations
// ============================================================================

fn log_evidence(grouped: &[(usize, usize)], beta: f64, k: f64, n: usize) -> f64 {
    let k_beta = k * beta;

    let mut result = -log_gamma_ratio(k_beta, n);

    for &(count, multiplicity) in grouped {
        result += multiplicity as f64 * log_gamma_ratio(beta, count);
    }

    result
}

/// Posterior mean entropy in nats at fixed beta.
fn posterior_mean_entropy_nats(
    grouped: &[(usize, usize)],
    k_observed: u128,
    beta: f64,
    k: f64,
    n: usize,
) -> f64 {
    let total = n as f64 + k * beta;

    if !total.is_finite() || total <= 0.0 {
        return f64::NAN;
    }

    let psi_total = digamma(total + 1.0);

    let mut weighted = 0.0;

    for &(count, multiplicity) in grouped {
        let alpha = count as f64 + beta;

        weighted += multiplicity as f64 * alpha * (psi_total - digamma(alpha + 1.0));
    }

    let k_observed_f64 = k_observed as f64;

    if k_observed_f64 > k {
        return f64::NAN;
    }

    let empty = k - k_observed_f64;

    if empty > 0.0 {
        weighted += empty * beta * (psi_total - digamma(beta + 1.0));
    }

    weighted / total
}

// ============================================================================
// Complete beta-domain transformation
// ============================================================================

fn beta_from_w(w: f64) -> f64 {
    let log_beta = 2.0 * (w.ln() - (-w).ln_1p());

    if log_beta >= 709.0 {
        f64::MAX
    } else {
        log_beta.exp()
    }
}

fn log_beta_jacobian(w: f64) -> f64 {
    2.0_f64.ln() + w.ln() - 3.0 * (-w).ln_1p()
}

// ============================================================================
// Log-scaled NSB integrand
// ============================================================================

fn log_weight(grouped: &[(usize, usize)], k: f64, n: usize, w: f64) -> Option<(f64, f64)> {
    if !(w > 0.0 && w < 1.0) {
        return None;
    }

    let beta = beta_from_w(w);

    if !beta.is_finite() || beta <= 0.0 {
        return None;
    }

    let dxi = dxi_dbeta(beta, k);

    if !dxi.is_finite() || dxi <= 0.0 {
        return None;
    }

    let evidence = log_evidence(grouped, beta, k, n);

    if !evidence.is_finite() {
        return None;
    }

    let log_weight = evidence + dxi.ln() + log_beta_jacobian(w);

    if !log_weight.is_finite() {
        return None;
    }

    Some((log_weight, beta))
}

fn integration_scale(grouped: &[(usize, usize)], k: f64, n: usize) -> f64 {
    let mut maximum = f64::NEG_INFINITY;

    for i in 1..256 {
        let w = i as f64 / 256.0;

        if let Some((log_weight, _)) = log_weight(grouped, k, n, w) {
            maximum = maximum.max(log_weight);
        }
    }

    maximum
}

fn integrand(
    grouped: &[(usize, usize)],
    k_observed: u128,
    k: f64,
    n: usize,
    log_scale: f64,
    w: f64,
) -> [f64; 2] {
    let Some((log_weight, beta)) = log_weight(grouped, k, n, w) else {
        return [0.0, 0.0];
    };

    let scaled_weight = (log_weight - log_scale).exp();

    if !scaled_weight.is_finite() {
        return [0.0, 0.0];
    }

    let entropy = posterior_mean_entropy_nats(grouped, k_observed, beta, k, n);

    if !entropy.is_finite() {
        return [0.0, 0.0];
    }

    [scaled_weight, scaled_weight * entropy]
}

// ============================================================================
// Gauss–Legendre quadrature
// ============================================================================

#[derive(Clone, Copy, Debug, Default)]
struct VectorIntegral {
    denominator: f64,
    numerator: f64,
}

impl VectorIntegral {
    fn add(self, other: Self) -> Self {
        Self {
            denominator: self.denominator + other.denominator,
            numerator: self.numerator + other.numerator,
        }
    }

    fn sub(self, other: Self) -> Self {
        Self {
            denominator: self.denominator - other.denominator,
            numerator: self.numerator - other.numerator,
        }
    }

    fn abs_max(self) -> f64 {
        self.denominator.abs().max(self.numerator.abs())
    }
}

#[derive(Clone, Copy, Debug)]
struct Interval {
    a: f64,
    b: f64,
    estimate: VectorIntegral,
    tolerance: f64,
    depth: usize,
}

fn gauss8<F>(f: &mut F, a: f64, b: f64) -> VectorIntegral
where
    F: FnMut(f64) -> [f64; 2],
{
    const NODES: [f64; 4] = [
        0.183_434_642_495_649_8,
        0.525_532_409_916_329,
        0.796_666_477_413_626_7,
        0.960_289_856_497_536_3,
    ];

    const WEIGHTS: [f64; 4] = [
        0.362_683_783_378_362,
        0.313_706_645_877_887_3,
        0.222_381_034_453_374_5,
        0.101_228_536_290_376_3,
    ];

    let midpoint = 0.5 * (a + b);
    let half_width = 0.5 * (b - a);

    let mut denominator = 0.0;
    let mut numerator = 0.0;

    for i in 0..4 {
        let delta = half_width * NODES[i];

        let left = f(midpoint - delta);
        let right = f(midpoint + delta);

        let weight = WEIGHTS[i];

        denominator += weight * (left[0] + right[0]);
        numerator += weight * (left[1] + right[1]);
    }

    VectorIntegral {
        denominator: denominator * half_width,
        numerator: numerator * half_width,
    }
}

fn gauss16<F>(f: &mut F, a: f64, b: f64) -> VectorIntegral
where
    F: FnMut(f64) -> [f64; 2],
{
    const NODES: [f64; 8] = [
        0.095_012_509_837_637_44,
        0.281_603_550_779_258_9,
        0.458_016_777_657_227_4,
        0.617_876_244_402_643_8,
        0.755_404_408_355_003,
        0.865_631_202_387_831_8,
        0.944_575_023_073_232_6,
        0.989_400_934_991_649_9,
    ];

    const WEIGHTS: [f64; 8] = [
        0.189_450_610_455_068_5,
        0.182_603_415_044_923_6,
        0.169_156_519_395_002_5,
        0.149_595_988_816_576_7,
        0.124_628_971_255_533_9,
        0.095_158_511_682_492_8,
        0.062_253_523_938_647_9,
        0.027_152_459_411_754_1,
    ];

    let midpoint = 0.5 * (a + b);
    let half_width = 0.5 * (b - a);

    let mut denominator = 0.0;
    let mut numerator = 0.0;

    for i in 0..8 {
        let delta = half_width * NODES[i];

        let left = f(midpoint - delta);
        let right = f(midpoint + delta);

        let weight = WEIGHTS[i];

        denominator += weight * (left[0] + right[0]);
        numerator += weight * (left[1] + right[1]);
    }

    VectorIntegral {
        denominator: denominator * half_width,
        numerator: numerator * half_width,
    }
}

fn adaptive_gauss<F>(
    f: &mut F,
    tolerance: f64,
    max_depth: usize,
    max_evaluations: usize,
) -> (VectorIntegral, f64, usize, bool)
where
    F: FnMut(f64) -> [f64; 2],
{
    let mut evaluations = 24usize;

    let whole8 = gauss8(f, 0.0, 1.0);
    let whole16 = gauss16(f, 0.0, 1.0);

    let initial_error = whole16.sub(whole8).abs_max();

    let mut stack = vec![Interval {
        a: 0.0,
        b: 1.0,
        estimate: whole16,
        tolerance,
        depth: 0,
    }];

    let mut result = VectorIntegral::default();
    let mut error = 0.0;
    let mut converged = true;

    while let Some(interval) = stack.pop() {
        if evaluations + 48 > max_evaluations {
            result = result.add(interval.estimate);
            converged = false;
            continue;
        }

        let midpoint = 0.5 * (interval.a + interval.b);

        let left8 = gauss8(f, interval.a, midpoint);
        let left16 = gauss16(f, interval.a, midpoint);

        let right8 = gauss8(f, midpoint, interval.b);
        let right16 = gauss16(f, midpoint, interval.b);

        evaluations += 48;

        let refined = left16.add(right16);

        let local_error = refined.sub(left8.add(right8)).abs_max();

        let scale = refined.abs_max().max(1.0);

        if local_error <= interval.tolerance * scale {
            result = result.add(refined);
            error += local_error;
            continue;
        }

        if interval.depth >= max_depth {
            result = result.add(refined);
            error += local_error;
            converged = false;
            continue;
        }

        let child_tolerance = interval.tolerance * 0.5;

        stack.push(Interval {
            a: midpoint,
            b: interval.b,
            estimate: right16,
            tolerance: child_tolerance,
            depth: interval.depth + 1,
        });

        stack.push(Interval {
            a: interval.a,
            b: midpoint,
            estimate: left16,
            tolerance: child_tolerance,
            depth: interval.depth + 1,
        });
    }

    error = error.max(initial_error);

    (result, error, evaluations, converged)
}

// ============================================================================
// Entropy estimation
// ============================================================================

pub fn nsb_entropy(counts: &[usize], config: &NsbConfig) -> f64 {
    nsb_entropy_diagnostics(counts, config).entropy_bits
}

pub fn nsb_entropy_diagnostics(counts: &[usize], config: &NsbConfig) -> NsbDiagnostics {
    let invalid = NsbDiagnostics {
        entropy_bits: f64::NAN,
        posterior_mean_beta: f64::INFINITY,
        integration_error: f64::NAN,
        evaluations: 0,
        converged: false,
        k_observed: 0,
        coincidences: 0,
        k_mismatch: false,
    };

    if counts.is_empty()
        || config.k == 0
        || config.integration_tolerance <= 0.0
        || !config.integration_tolerance.is_finite()
        || config.max_depth == 0
        || config.max_evaluations < 24
    {
        return invalid;
    }

    if config.k == 1 {
        let k_observed = counts.iter().filter(|&&c| c > 0).count();

        let n = counts.iter().sum::<usize>();

        return NsbDiagnostics {
            entropy_bits: 0.0,
            posterior_mean_beta: f64::INFINITY,
            integration_error: 0.0,
            evaluations: 0,
            converged: true,
            k_observed: k_observed as u128,
            coincidences: n.saturating_sub(k_observed) as u128,
            k_mismatch: k_observed > 1,
        };
    }

    let (grouped, k_observed_usize, n) = group_counts(counts);

    if k_observed_usize == 0 || n == 0 {
        return invalid;
    }

    let coincidences_usize = n.saturating_sub(k_observed_usize);

    let k_observed = k_observed_usize as u128;
    let coincidences = coincidences_usize as u128;

    if k_observed > config.k {
        return NsbDiagnostics {
            k_observed,
            coincidences,
            k_mismatch: true,
            ..invalid
        };
    }

    let k = config.k as f64;

    if !k.is_finite() || k <= 0.0 {
        return NsbDiagnostics {
            k_observed,
            coincidences,
            ..invalid
        };
    }

    let log_scale = integration_scale(&grouped, k, n);

    if !log_scale.is_finite() {
        return NsbDiagnostics {
            k_observed,
            coincidences,
            ..invalid
        };
    }

    let mut function = |w: f64| integrand(&grouped, k_observed, k, n, log_scale, w);

    let (integral, integration_error_scaled, evaluations, converged) = adaptive_gauss(
        &mut function,
        config.integration_tolerance,
        config.max_depth,
        config.max_evaluations,
    );

    let denominator = integral.denominator;

    if !denominator.is_finite() || denominator <= 0.0 {
        return NsbDiagnostics {
            k_observed,
            coincidences,
            evaluations,
            converged,
            integration_error: f64::NAN,
            ..invalid
        };
    }

    let entropy_nats = integral.numerator / denominator;

    if !entropy_nats.is_finite() {
        return NsbDiagnostics {
            k_observed,
            coincidences,
            evaluations,
            converged,
            integration_error: f64::NAN,
            ..invalid
        };
    }

    let log_k = k.ln();

    let entropy_nats = entropy_nats.clamp(0.0, log_k);

    let denominator_error = integration_error_scaled * denominator.abs();

    let numerator_error = integration_error_scaled * integral.numerator.abs();

    let ratio_error = if denominator > 0.0 {
        (numerator_error + entropy_nats.abs() * denominator_error) / denominator
    } else {
        f64::INFINITY
    };

    let entropy_bits = entropy_nats / std::f64::consts::LN_2;

    let integration_error = ratio_error / std::f64::consts::LN_2;

    NsbDiagnostics {
        entropy_bits,
        posterior_mean_beta: f64::INFINITY,
        integration_error,
        evaluations,
        converged,
        k_observed,
        coincidences,
        k_mismatch: false,
    }
}

// ============================================================================
// Histograms
// ============================================================================

fn histogram<T: Eq + Hash + Clone>(values: &[T]) -> HashMap<T, usize> {
    let mut result = HashMap::new();

    for value in values {
        *result.entry(value.clone()).or_insert(0) += 1;
    }

    result
}

// ============================================================================
// Mutual information
// ============================================================================

/// Compute the three NSB entropies needed for mutual information.
///
/// Cardinality resolution is deliberately performed here, immediately before
/// the entropy calculations.
///
/// This is the key architectural point:
///
/// ```text
/// dmi_nsb(x, y)
///     └── resolve cardinality for (x, y)
///
/// shuffle_corrected_nsb(x, y)
///     ├── nmi_nsb(x, y)
///     │      └── resolve cardinality for observed (x, y)
///     │
///     ├── nmi_nsb(x, shuffle_1(y))
///     │      └── resolve cardinality for shuffle 1
///     │
///     ├── nmi_nsb(x, shuffle_2(y))
///     │      └── resolve cardinality for shuffle 2
///     │
///     └── ...
/// ```
///
/// Therefore `NsbCardinality::Observed` is never accidentally frozen to
/// the cardinality of the original dataset.
fn dmi_nsb_full<T: Eq + Hash + Clone>(
    x_seq: &[T],
    y_seq: &[T],
    config: &NsbShuffleConfig,
) -> (f64, f64, f64) {
    if x_seq.len() != y_seq.len() || x_seq.len() < 2 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }

    // ---------------------------------------------------------------
    // Resolve cardinality HERE.
    //
    // For NsbCardinality::Observed this is a fresh resolution for the
    // exact `(x_seq, y_seq)` pair passed to this invocation.
    // ---------------------------------------------------------------

    let Some((k_x, k_y, k_xy)) = config.cardinality.resolve(x_seq, y_seq) else {
        return (f64::NAN, f64::NAN, f64::NAN);
    };

    let x_hist = histogram(x_seq);
    let y_hist = histogram(y_seq);

    let mut xy_hist: HashMap<(T, T), usize> = HashMap::new();

    for (x, y) in x_seq.iter().zip(y_seq.iter()) {
        *xy_hist.entry((x.clone(), y.clone())).or_insert(0) += 1;
    }

    let x_counts: Vec<usize> = x_hist.into_values().collect();

    let y_counts: Vec<usize> = y_hist.into_values().collect();

    let xy_counts: Vec<usize> = xy_hist.into_values().collect();

    let x_config = NsbConfig {
        k: k_x,
        ..config.entropy.clone()
    };

    let y_config = NsbConfig {
        k: k_y,
        ..config.entropy.clone()
    };

    let xy_config = NsbConfig {
        k: k_xy,
        ..config.entropy.clone()
    };

    let h_x = nsb_entropy(&x_counts, &x_config);

    let h_y = nsb_entropy(&y_counts, &y_config);

    let h_xy = nsb_entropy(&xy_counts, &xy_config);

    (h_x, h_y, h_xy)
}

pub fn dmi_nsb<T: Eq + Hash + Clone>(x_seq: &[T], y_seq: &[T], config: &NsbShuffleConfig) -> f64 {
    let (h_x, h_y, h_xy) = dmi_nsb_full(x_seq, y_seq, config);

    if !h_x.is_finite() || !h_y.is_finite() || !h_xy.is_finite() {
        return f64::NAN;
    }

    (h_x + h_y - h_xy).max(0.0)
}

pub fn nmi_nsb<T: Eq + Hash + Clone>(x_seq: &[T], y_seq: &[T], config: &NsbShuffleConfig) -> f64 {
    let (h_x, h_y, h_xy) = dmi_nsb_full(x_seq, y_seq, config);

    if !h_x.is_finite() || !h_y.is_finite() || !h_xy.is_finite() {
        return f64::NAN;
    }

    if h_x <= 0.0 || h_y <= 0.0 {
        return 0.0;
    }

    let mi = (h_x + h_y - h_xy).max(0.0);

    (mi / (h_x * h_y).sqrt()).clamp(0.0, 1.0)
}

// ============================================================================
// Shuffle correction
// ============================================================================

/// Shuffle-corrected normalized mutual information using NSB.
///
/// The procedure is intentionally structurally identical to the other
/// estimators:
///
/// 1. Compute NMI on the observed `(X, Y)`.
/// 2. Shuffle `Y`.
/// 3. Compute NMI on `(X, shuffled_Y)`.
/// 4. Repeat `n_shuffles` times.
/// 5. Subtract the mean shuffle baseline.
/// 6. Clamp to `[0, 1]`.
///
/// Crucially, cardinality is NOT resolved here.
///
/// Instead every call to `nmi_nsb()` resolves cardinality through
/// `config.cardinality`.
///
/// Thus with:
///
/// ```text
/// NsbCardinality::Observed
/// ```
///
/// the sequence of operations is:
///
/// ```text
/// observed:
///     k_x  = support(X)
///     k_y  = support(Y)
///     k_xy = support(X,Y)
///
/// shuffle 1:
///     k_x  = support(X)
///     k_y  = support(shuffle_1(Y))
///     k_xy = support(X, shuffle_1(Y))
///
/// shuffle 2:
///     k_x  = support(X)
///     k_y  = support(shuffle_2(Y))
///     k_xy = support(X, shuffle_2(Y))
///
/// ...
/// ```
///
/// With:
///
/// ```text
/// NsbCardinality::Explicit { ... }
/// ```
///
/// all estimates use the same explicitly supplied cardinalities.
pub fn shuffle_corrected_nsb<T: Eq + Hash + Clone>(
    x_seq: &[T],
    y_seq: &[T],
    config: &NsbShuffleConfig,
) -> f64 {
    if x_seq.len() != y_seq.len() || x_seq.len() < 4 {
        return 0.0;
    }

    // ---------------------------------------------------------------
    // Observed NMI.
    //
    // nmi_nsb() resolves cardinality for THIS exact dataset.
    // ---------------------------------------------------------------

    let nmi_obs = nmi_nsb(x_seq, y_seq, config);

    if !nmi_obs.is_finite() {
        return 0.0;
    }

    // ---------------------------------------------------------------
    // No shuffle correction requested.
    // ---------------------------------------------------------------

    if config.n_shuffles == 0 {
        return nmi_obs;
    }

    // ---------------------------------------------------------------
    // Shuffle Y and recompute the complete NMI estimator.
    // ---------------------------------------------------------------

    let mut rng = StdRng::seed_from_u64(config.seed);

    let mut y_shuffled = y_seq.to_vec();

    let mut nmi_shuffles = Vec::with_capacity(config.n_shuffles);

    for _ in 0..config.n_shuffles {
        // Fisher-Yates shuffle.
        for i in (1..y_shuffled.len()).rev() {
            let j = rng.random_range(0..=i);

            y_shuffled.swap(i, j);
        }

        // IMPORTANT:
        //
        // Cardinality isn't resolved outside this call.
        //
        // nmi_nsb() -> dmi_nsb_full() -> resolve()
        //
        // Consequently `Observed` cardinality is recomputed for this
        // shuffled dataset.
        let nmi_shuffle = nmi_nsb(x_seq, &y_shuffled, config);

        if nmi_shuffle.is_finite() {
            nmi_shuffles.push(nmi_shuffle);
        }
    }

    if nmi_shuffles.is_empty() {
        return 0.0;
    }

    let mean_shuffle = nmi_shuffles.iter().sum::<f64>() / nmi_shuffles.len() as f64;

    (nmi_obs - mean_shuffle).clamp(0.0, 1.0)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn entropy_config(k: u128) -> NsbConfig {
        NsbConfig {
            k,
            integration_tolerance: 1e-7,
            max_depth: 20,
            max_evaluations: 5_000,
        }
    }

    fn mi_config(k_x: u128, k_y: u128, k_xy: u128) -> NsbShuffleConfig {
        NsbShuffleConfig {
            cardinality: NsbCardinality::Explicit { k_x, k_y, k_xy },
            entropy: NsbConfig {
                k: 2,
                integration_tolerance: 1e-7,
                max_depth: 20,
                max_evaluations: 5_000,
            },
            n_shuffles: 10,
            seed: 42,
        }
    }

    fn observed_mi_config() -> NsbShuffleConfig {
        NsbShuffleConfig {
            cardinality: NsbCardinality::Observed,
            entropy: NsbConfig {
                k: 2,
                integration_tolerance: 1e-7,
                max_depth: 20,
                max_evaluations: 5_000,
            },
            n_shuffles: 10,
            seed: 42,
        }
    }

    #[test]
    fn digamma_known_values() {
        let gamma = 0.577_215_664_901_532_9;

        assert!((digamma(1.0) + gamma).abs() < 1e-10);

        assert!((digamma(2.0) - (1.0 - gamma)).abs() < 1e-10);
    }

    #[test]
    fn lgamma_known_values() {
        assert!((lgamma(5.0) - 24.0_f64.ln()).abs() < 1e-10);

        assert!((lgamma(0.5) - std::f64::consts::PI.sqrt().ln()).abs() < 1e-10);
    }

    #[test]
    fn uniform_distribution() {
        let diagnostics = nsb_entropy_diagnostics(&[100, 100, 100, 100], &entropy_config(4));

        assert!(diagnostics.converged, "{diagnostics:?}");

        assert!(diagnostics.entropy_bits.is_finite());

        assert!((diagnostics.entropy_bits - 2.0).abs() < 0.02);
    }

    #[test]
    fn deterministic_distribution_has_small_entropy() {
        let diagnostics = nsb_entropy_diagnostics(&[1000], &entropy_config(4));

        assert!(diagnostics.converged, "{diagnostics:?}");

        assert!(diagnostics.entropy_bits.is_finite());

        assert!(diagnostics.entropy_bits < 0.05);

        assert!(diagnostics.entropy_bits >= 0.0);
    }

    #[test]
    fn deterministic_entropy_decreases_with_more_data() {
        let h_100 = nsb_entropy(&[100], &entropy_config(4));

        let h_1000 = nsb_entropy(&[1000], &entropy_config(4));

        assert!(h_100.is_finite());
        assert!(h_1000.is_finite());

        assert!(h_1000 < h_100);
    }

    #[test]
    fn severe_undersampling_requires_coincidences() {
        let counts = vec![1usize; 40]
            .into_iter()
            .chain([10usize, 10, 10, 10, 10])
            .collect::<Vec<_>>();

        let diagnostics = nsb_entropy_diagnostics(&counts, &entropy_config(4096));

        assert!(diagnostics.entropy_bits.is_finite(), "{diagnostics:?}");

        assert!(diagnostics.coincidences > 0);

        let n = 90.0;

        let plugin = counts
            .iter()
            .map(|&c| c as f64)
            .filter(|&c| c > 0.0)
            .fold(0.0, |h, c| {
                let p = c / n;
                h - p * p.log2()
            });

        assert!(diagnostics.entropy_bits > plugin);
    }

    #[test]
    fn no_coincidences_are_valid() {
        let counts = vec![1usize; 50];

        let diagnostics = nsb_entropy_diagnostics(&counts, &entropy_config(4096));

        assert!(diagnostics.entropy_bits.is_finite());

        assert_eq!(diagnostics.coincidences, 0);
    }

    #[test]
    fn k_mismatch_is_detected() {
        let diagnostics = nsb_entropy_diagnostics(&[1, 1, 1, 1], &entropy_config(3));

        assert!(diagnostics.k_mismatch);

        assert!(!diagnostics.entropy_bits.is_finite());
    }

    #[test]
    fn integration_converges_without_grid_parameter() {
        let counts = [25usize, 25, 25, 25];

        let loose = nsb_entropy(
            &counts,
            &NsbConfig {
                k: 4,
                integration_tolerance: 1e-5,
                max_depth: 18,
                max_evaluations: 5_000,
            },
        );

        let tight = nsb_entropy(
            &counts,
            &NsbConfig {
                k: 4,
                integration_tolerance: 1e-9,
                max_depth: 24,
                max_evaluations: 10_000,
            },
        );

        assert!(loose.is_finite());
        assert!(tight.is_finite());

        assert!((loose - tight).abs() < 1e-5);
    }

    #[test]
    fn independent_mi_is_small() {
        use rand::{RngExt, SeedableRng, rngs::StdRng};

        let mut rng_x = StdRng::seed_from_u64(1);

        let mut rng_y = StdRng::seed_from_u64(2);

        let x: Vec<u8> = (0..600).map(|_| rng_x.random_range(0..4)).collect();

        let y: Vec<u8> = (0..600).map(|_| rng_y.random_range(0..4)).collect();

        let mi = dmi_nsb(&x, &y, &mi_config(4, 4, 16));

        assert!(mi.is_finite());
        assert!(mi < 0.3);
    }

    #[test]
    fn deterministic_mi_is_large() {
        let x: Vec<u8> = (0..400).map(|i| (i % 4) as u8).collect();

        let mi = dmi_nsb(&x, &x, &mi_config(4, 4, 4));

        assert!(mi.is_finite());
        assert!(mi > 1.5);
    }

    #[test]
    fn nmi_is_bounded() {
        let x: Vec<u8> = (0..400).map(|i| (i % 4) as u8).collect();

        let nmi = nmi_nsb(&x, &x, &mi_config(4, 4, 4));

        assert!(nmi.is_finite());
        assert!((0.0..=1.0).contains(&nmi));
        assert!(nmi > 0.9);
    }

    #[test]
    fn shuffle_correction_preserves_dependency() {
        use rand::{RngExt, SeedableRng, rngs::StdRng};

        let mut rng = StdRng::seed_from_u64(123);

        let x: Vec<u8> = (0..400).map(|_| rng.random_range(0..8)).collect();

        let dependent = x.clone();

        let independent: Vec<u8> = (0..400).map(|_| rng.random_range(0..8)).collect();

        let config = mi_config(8, 8, 64);

        let dependent_score = shuffle_corrected_nsb(&x, &dependent, &config);

        let independent_score = shuffle_corrected_nsb(&x, &independent, &config);

        assert!(dependent_score.is_finite());

        assert!(independent_score.is_finite());

        assert!(dependent_score > independent_score);
    }

    // ------------------------------------------------------------------------
    // Observed cardinality tests
    // ------------------------------------------------------------------------

    #[test]
    fn observed_cardinality_uses_support() {
        let x = [0u8, 1, 0, 1, 0, 1];
        let y = [1u8, 1, 0, 1, 0, 0];

        let cardinality = NsbCardinality::Observed;

        let resolved = cardinality.resolve(&x, &y).unwrap();

        assert_eq!(resolved, (2, 2, 4));
    }

    #[test]
    fn observed_cardinality_is_resolved_again_after_shuffle() {
        let x = [0u8, 0, 0, 0, 1, 1, 1, 1];

        let y = [0u8, 0, 0, 0, 1, 1, 1, 1];

        let cardinality = NsbCardinality::Observed;

        let original = cardinality.resolve(&x, &y).unwrap();

        let shuffled = [0u8, 1, 0, 1, 0, 1, 0, 1];

        let shuffled_cardinality = cardinality.resolve(&x, &shuffled).unwrap();

        assert_eq!(original.0, shuffled_cardinality.0);

        assert_eq!(original.1, 2);

        assert_eq!(shuffled_cardinality.1, 2);

        // The important part is that this is resolved from the actual
        // `(x, shuffled_y)` data rather than copied from the original pair.
        assert_eq!(shuffled_cardinality.2, 4);
    }

    #[test]
    fn explicit_cardinality_is_stable() {
        let cardinality = NsbCardinality::Explicit {
            k_x: 4096,
            k_y: 4096,
            k_xy: 16_777_216,
        };

        let x = [0u8, 1, 0, 1];
        let y = [0u8, 0, 1, 1];

        let shuffled_y = [1u8, 0, 1, 0];

        assert_eq!(
            cardinality.resolve(&x, &y).unwrap(),
            (4096, 4096, 16_777_216)
        );

        assert_eq!(
            cardinality.resolve(&x, &shuffled_y).unwrap(),
            (4096, 4096, 16_777_216)
        );
    }

    #[test]
    fn observed_nsb_mi_works_without_known_cardinality() {
        let x = [0u8, 1, 1, 0, 0, 0];

        let y = [1u8, 1, 1, 1, 0, 1];

        let config = observed_mi_config();

        let mi = dmi_nsb(&x, &y, &config);

        assert!(mi.is_finite(), "MI = {mi}");
    }

    #[test]
    fn observed_nmi_works_without_known_cardinality() {
        let x = [0u8, 1, 1, 0, 0, 0];

        let y = [1u8, 1, 1, 1, 0, 1];

        let config = observed_mi_config();

        let nmi = nmi_nsb(&x, &y, &config);

        assert!(nmi.is_finite(), "NMI = {nmi}");

        assert!((0.0..=1.0).contains(&nmi));
    }

    #[test]
    fn observed_shuffle_correction_is_finite() {
        let x: Vec<u8> = (0..100).map(|i| (i % 2) as u8).collect();

        let y = x.clone();

        let config = observed_mi_config();

        let score = shuffle_corrected_nsb(&x, &y, &config);

        assert!(score.is_finite(), "score = {score}");

        assert!((0.0..=1.0).contains(&score));
    }

    // ------------------------------------------------------------------------
    // u128 cardinality regression tests
    // ------------------------------------------------------------------------

    #[test]
    fn large_u128_cardinality_is_accepted() {
        let k = 1u128 << 100;

        let config = entropy_config(k);

        assert_eq!(config.k, k);

        let diagnostics = nsb_entropy_diagnostics(&[1usize], &config);

        assert!(diagnostics.entropy_bits.is_finite(), "{diagnostics:?}");

        assert_eq!(diagnostics.k_observed, 1);

        assert_eq!(diagnostics.coincidences, 0);

        assert!(!diagnostics.k_mismatch);
    }

    #[test]
    fn u128_cardinality_does_not_truncate_to_usize() {
        let k = 1u128 << 100;

        let diagnostics = nsb_entropy_diagnostics(&[1usize], &entropy_config(k));

        assert_eq!(diagnostics.k_observed, 1);

        assert!(!diagnostics.k_mismatch);
    }

    #[test]
    fn large_cardinality_mismatch_still_works() {
        let k = 1u128 << 100;

        let diagnostics = nsb_entropy_diagnostics(&[1usize, 1usize], &entropy_config(1));

        assert!(diagnostics.k_mismatch);

        assert_eq!(diagnostics.k_observed, 2);

        let diagnostics_large = nsb_entropy_diagnostics(&[1usize], &entropy_config(k));

        assert!(!diagnostics_large.k_mismatch);
    }

    // ------------------------------------------------------------------------
    // Shuffle cardinality regression
    // ------------------------------------------------------------------------

    #[test]
    fn observed_cardinality_is_not_frozen_before_shuffle() {
        let x = vec![0u8, 0, 0, 0, 1, 1, 1, 1];

        let y = vec![0u8, 0, 0, 0, 1, 1, 1, 1];

        let config = observed_mi_config();

        // Resolve the original pair.
        let original = config.cardinality.resolve(&x, &y).unwrap();

        // Construct a pair whose support differs from the original.
        let y_reduced = vec![0u8, 0, 0, 0, 0, 0, 0, 0];

        let reduced = config.cardinality.resolve(&x, &y_reduced).unwrap();

        assert_ne!(original, reduced);

        assert_eq!(original.1, 2);

        assert_eq!(reduced.1, 1);

        // This is exactly what shuffle_corrected_nsb() now does:
        // every nmi_nsb() call resolves cardinality against its actual
        // `(x, y)` arguments.
    }
}
