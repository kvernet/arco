//! Invariant trait — conserved quantities in information universes.
//!
//! Per the Mathematical Constitution:
//!     I is a set of functions I: S → ℝ that are conserved (exactly
//!     or approximately) under all transformations τ ∈ T. All
//!     invariants must be computable in finite time.
//!
//! # The Invariant trait
//!
//! An invariant is a real-valued function of state that a universe
//! claims is conserved by its rules. Because I is a *set* of
//! functions (per Constitution), an
//! [`InformationUniverse`](crate::universe::InformationUniverse)
//! exposes its invariants as a heterogeneous collection —
//! `Vec<Box<dyn Invariant<S>>>` — rather than a single associated
//! type, unlike [`Observation`](crate::observation::Observation) or
//! [`Schedule`](crate::schedule::Schedule), each of which a universe
//! has exactly one of.
//!
//! # Triviality (Constitution, Failure Condition F-2)
//!
//! Not every conserved quantity is scientifically interesting. If a
//! rule set is constructed so that it can *only* leave some quantity
//! unchanged — e.g. a rule set that structurally never touches a
//! particular part of the state — then conservation of that quantity
//! is a trivial consequence of how the rules were built, not a
//! discovery. This module does not attempt to automatically judge
//! triviality (that is a research judgment, not a mechanical check);
//! it only makes conservation checkable. See the substrate-level
//! invariant modules for concrete examples of trivial-by-construction
//! vs. genuinely rule-dependent invariants.
//!
//! # Quick start
//!
//! ```rust
//! use arco::state::State;
//! use arco::invariants::Invariant;
//!
//! #[derive(Clone, PartialEq, Eq, Hash, Debug)]
//! struct Pair { a: i64, b: i64 }
//!
//! impl State for Pair {
//!     type Encoding = Vec<u8>;
//!     fn canonical_encoding(&self) -> Self::Encoding {
//!         [self.a.to_le_bytes(), self.b.to_le_bytes()].concat()
//!     }
//!     fn distance(&self, other: &Self) -> u32 {
//!         ((self.a != other.a) as u32) + ((self.b != other.b) as u32)
//!     }
//! }
//!
//! /// I(s) = a + b.
//! #[derive(Debug, Clone, Copy, Default)]
//! struct SumInvariant;
//!
//! impl Invariant<Pair> for SumInvariant {
//!     fn name(&self) -> &str { "sum" }
//!     fn evaluate(&self, state: &Pair) -> f64 { (state.a + state.b) as f64 }
//! }
//!
//! let before = Pair { a: 3, b: 5 };
//! let after = Pair { a: 4, b: 4 }; // sum unchanged
//! let inv = SumInvariant;
//! assert!(inv.is_conserved(&before, &after));
//!
//! let violated = Pair { a: 4, b: 5 }; // sum changed
//! assert!(!inv.is_conserved(&before, &violated));
//! ```

use std::collections::HashMap;
use std::fmt::Debug;

use crate::rules::{NoContext, Rule};
use crate::schedule::Schedule;
use crate::state::State;
use rand::SeedableRng;
use rand::rngs::StdRng;

// ===================================================================
// Invariant trait
// ===================================================================

/// A single conserved (or claimed-conserved) real-valued function of
/// state.
///
/// # Type parameters
///
/// - `S: State` — The state type this invariant is evaluated over.
///
/// # Design contracts
///
/// - `evaluate` must be computable in finite time (per Constitution)
///   and should be a pure function of the state.
/// - `tolerance` defaults to `0.0` — exact conservation. Override it
///   for invariants that are only approximately conserved (e.g. under
///   floating-point accumulation, or genuinely statistical
///   quantities).
pub trait Invariant<S: State>: Debug + Send + Sync {
    /// A human-readable name for this invariant.
    fn name(&self) -> &str;

    /// `I(s)` — evaluate the invariant on a state.
    fn evaluate(&self, state: &S) -> f64;

    /// The numerical tolerance for "conserved." Default `0.0` (exact
    /// conservation). Override for approximately-conserved invariants.
    fn tolerance(&self) -> f64 {
        0.0
    }

    /// Whether the invariant is conserved between two states, within
    /// `tolerance`.
    fn is_conserved(&self, before: &S, after: &S) -> bool {
        (self.evaluate(before) - self.evaluate(after)).abs() <= self.tolerance()
    }
}

// ===================================================================
// Exhaustive verification (per Constitution: "computable in finite time")
// ===================================================================

/// Exhaustively verify whether `invariant` is conserved by `rule`
/// across every state in `state_space`.
///
/// Only meaningful for finite state spaces small enough to enumerate
/// exhaustively — for larger state spaces, use [`violation_rate`] over
/// sampled trajectories instead.
///
/// Requires `Context = NoContext` because exhaustive checking applies
/// the rule directly to each state without any external match
/// information, mirroring the same constraint used by
/// [`crate::schedule::SequentialSchedule`] and
/// [`crate::schedule::RandomRuleSchedule`].
pub fn exhaustively_conserved<S, R>(
    rule: &R,
    invariant: &dyn Invariant<S>,
    state_space: &[S],
) -> bool
where
    S: State,
    R: Rule<S, Context = NoContext>,
{
    let ctx = NoContext;
    // Deterministic rules ignore the RNG; a fixed seed keeps this
    // reproducible for rules that happen to be stochastic.
    let mut rng = StdRng::seed_from_u64(0);
    state_space.iter().all(|s| {
        let next = rule.apply(s, &ctx, &mut rng);
        invariant.is_conserved(s, &next)
    })
}

// ===================================================================
// Trajectory-level violation rate
// ===================================================================

/// The fraction of consecutive-state transitions along a trajectory
/// at which an invariant was *not* conserved.
///
/// `0.0` means the invariant held at every step; `1.0` means it was
/// violated at every step. Returns `0.0` for trajectories with fewer
/// than two states (nothing to compare).
pub fn violation_rate<S: State>(invariant: &dyn Invariant<S>, trajectory: &[S]) -> f64 {
    if trajectory.len() < 2 {
        return 0.0;
    }
    let mut violations = 0usize;
    let n_transitions = trajectory.len() - 1;
    for pair in trajectory.windows(2) {
        if !invariant.is_conserved(&pair[0], &pair[1]) {
            violations += 1;
        }
    }
    violations as f64 / n_transitions as f64
}

// ===================================================================
// Trajectory walking for invariant accounting
// ===================================================================

/// Walk one trajectory from `initial_state` and report each
/// invariant's [`violation_rate`] across it, keyed by invariant name.
///
/// Mirrors [`crate::calibration::generate_trajectories`]'s stepping
/// logic, but keeps the states themselves (needed to evaluate
/// invariants) rather than observation outputs.
pub fn measure_trajectory_violation_rates<S, R, K>(
    initial_state: &S,
    rules: &[R],
    invariants: &[Box<dyn Invariant<S>>],
    schedule: &K,
    steps: usize,
    seed: u64,
) -> HashMap<String, f64>
where
    S: State,
    R: Rule<S>,
    K: Schedule<S, R>,
{
    let mut rng = StdRng::seed_from_u64(seed);
    let mut trajectory = Vec::with_capacity(steps + 1);
    trajectory.push(initial_state.clone());
    let mut current = initial_state.clone();

    for _ in 0..steps {
        current = schedule.step(&current, rules, &mut rng);
        trajectory.push(current.clone());
    }

    invariants
        .iter()
        .map(|inv| {
            (
                inv.name().to_string(),
                violation_rate(inv.as_ref(), &trajectory),
            )
        })
        .collect()
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    struct Pair {
        a: i64,
        b: i64,
    }

    impl State for Pair {
        type Encoding = Vec<u8>;
        fn canonical_encoding(&self) -> Self::Encoding {
            [self.a.to_le_bytes(), self.b.to_le_bytes()].concat()
        }
        fn distance(&self, other: &Self) -> u32 {
            ((self.a != other.a) as u32) + ((self.b != other.b) as u32)
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct SumInvariant;

    impl Invariant<Pair> for SumInvariant {
        fn name(&self) -> &str {
            "sum"
        }
        fn evaluate(&self, state: &Pair) -> f64 {
            (state.a + state.b) as f64
        }
    }

    #[test]
    fn conserved_when_sum_unchanged() {
        let inv = SumInvariant;
        let before = Pair { a: 3, b: 5 };
        let after = Pair { a: 4, b: 4 };
        assert!(inv.is_conserved(&before, &after));
    }

    #[test]
    fn violated_when_sum_changes() {
        let inv = SumInvariant;
        let before = Pair { a: 3, b: 5 };
        let after = Pair { a: 4, b: 5 };
        assert!(!inv.is_conserved(&before, &after));
    }

    #[test]
    fn violation_rate_over_trajectory() {
        let inv = SumInvariant;
        let trajectory = vec![
            Pair { a: 1, b: 1 }, // sum 2
            Pair { a: 2, b: 0 }, // sum 2 -- conserved
            Pair { a: 3, b: 0 }, // sum 3 -- violated
            Pair { a: 1, b: 2 }, // sum 3 -- conserved
        ];
        // 1 violation out of 3 transitions
        assert!((violation_rate(&inv, &trajectory) - (1.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn violation_rate_trivial_for_short_trajectory() {
        let inv = SumInvariant;
        assert_eq!(violation_rate(&inv, &[Pair { a: 0, b: 0 }]), 0.0);
        assert_eq!(violation_rate::<Pair>(&inv, &[]), 0.0);
    }

    #[derive(Debug, Clone)]
    struct SwapRule;
    impl Rule<Pair> for SwapRule {
        type Context = NoContext;
        fn name(&self) -> &str {
            "Swap"
        }
        fn apply(&self, state: &Pair, _ctx: &NoContext, _rng: &mut dyn rand::Rng) -> Pair {
            Pair {
                a: state.b,
                b: state.a,
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    struct SequentialPairSchedule;
    impl Schedule<Pair, SwapRule> for SequentialPairSchedule {
        fn name(&self) -> &str {
            "sequential_pair"
        }
        fn timing(&self) -> &str {
            "asynchronous"
        }
        fn selection(&self) -> &str {
            "exhaustive"
        }
        fn step(&self, state: &Pair, rules: &[SwapRule], rng: &mut dyn rand::Rng) -> Pair {
            let mut current = state.clone();
            for rule in rules {
                current = rule.apply(&current, &NoContext, rng);
            }
            current
        }
    }

    #[test]
    fn measure_trajectory_violation_rates_reports_zero_for_conserved_sum() {
        let initial = Pair { a: 1, b: 2 };
        let rules = vec![SwapRule];
        let invariants: Vec<Box<dyn Invariant<Pair>>> = vec![Box::new(SumInvariant)];
        let schedule = SequentialPairSchedule;

        let rates =
            measure_trajectory_violation_rates(&initial, &rules, &invariants, &schedule, 5, 0);
        assert_eq!(rates.get("sum").copied(), Some(0.0));
    }

    #[test]
    fn exhaustively_conserved_detects_swap_preserves_sum() {
        let state_space = vec![
            Pair { a: 0, b: 0 },
            Pair { a: 1, b: 0 },
            Pair { a: 0, b: 1 },
            Pair { a: 2, b: 3 },
        ];
        let inv = SumInvariant;
        assert!(exhaustively_conserved(&SwapRule, &inv, &state_space));
    }
}
