//! Resources trait — resource constraints on Information Universes.
//!
//! Per the Mathematical Constitution:
//!     R specifies required resources: Time (transformation steps),
//!     Space (state representation size), and Locality (maximum
//!     interaction radius). Resources are subadditive under
//!     composition: R_i(τ1∘τ2) ≤ R_i(τ1) + R_i(τ2).
//!
//! # The Resources trait
//!
//! A resource model assigns a cost to states and rules along three
//! required dimensions:
//!
//! - **Space** (`R_space`): the size of a state's representation.
//! - **Time** (`R_time`): the cost of one application of a rule.
//! - **Locality** (`R_local`): the maximum interaction radius a rule
//!   can reach.
//!
//! Every [`InformationUniverse`](crate::universe::InformationUniverse)
//! provides exactly one resource model, mirroring how it provides
//! exactly one observation operator and one schedule.
//!
//! # Two distinct kinds of accounting
//!
//! This module deliberately keeps two things separate:
//!
//! 1. **The Resource Algebra** ([`satisfies_resource_algebra`]) — a
//!    static law about how the cost of a *composed transformation*
//!    relates to the costs of its parts. This is about the
//!    transformation semigroup itself.
//! 2. **Trajectory resource usage** ([`ResourceUsage`],
//!    [`measure_resource_usage`]) — practical bookkeeping of how much
//!    of each resource a specific run actually used. This is about a
//!    particular sequence of states and rule applications.
//!
//! Conflating the two would be a mistake: composing two rules into
//! one is not the same operation as running many timesteps of a
//! trajectory, even though both are governed by the same algebra.
//!
//! # A note on granularity
//!
//! [`Schedule::step`](crate::schedule::Schedule::step) applies a rule
//! set to a state and returns only the resulting state — it does not
//! expose which individual rule fired at which site. This means
//! trajectory-level resource accounting can report the maximum
//! locality *available* to the schedule at each step, but not
//! necessarily the locality that *actually fired*. This is documented
//! precisely on [`ResourceUsage`] rather than glossed over.
//!
//! # Quick start
//!
//! ```rust
//! use arco::state::State;
//! use arco::rules::{Rule, NoContext};
//! use arco::resources::Resources;
//!
//! #[derive(Clone, PartialEq, Eq, Hash, Debug)]
//! struct MyState { data: Vec<u8> }
//!
//! impl State for MyState {
//!     type Encoding = Vec<u8>;
//!     fn canonical_encoding(&self) -> Self::Encoding { self.data.clone() }
//!     fn distance(&self, other: &Self) -> u32 {
//!         self.data.iter().zip(other.data.iter())
//!             .map(|(a, b)| if a != b { 1 } else { 0 }).sum()
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! struct FlipRule;
//! impl Rule<MyState> for FlipRule {
//!     type Context = NoContext;
//!     fn name(&self) -> &str { "Flip" }
//!     fn apply(&self, state: &MyState, _ctx: &NoContext, _rng: &mut dyn rand::Rng) -> MyState {
//!         MyState { data: state.data.iter().map(|b| 1 - b).collect() }
//!     }
//! }
//!
//! /// Space = number of bytes; time = 1 per application; pointwise (locality 0).
//! #[derive(Debug, Clone, Copy, Default)]
//! struct MyResources;
//!
//! impl Resources<MyState, FlipRule> for MyResources {
//!     fn name(&self) -> &str { "my_resources" }
//!     fn space(&self, state: &MyState) -> f64 { state.data.len() as f64 }
//!     fn time(&self, _rule: &FlipRule) -> f64 { 1.0 }
//!     fn locality(&self, _rule: &FlipRule) -> f64 { 0.0 }
//! }
//!
//! let state = MyState { data: vec![0, 1, 0] };
//! let model = MyResources;
//! assert_eq!(model.space(&state), 3.0);
//! assert_eq!(model.time(&FlipRule), 1.0);
//! ```

use std::fmt::Debug;

use crate::rules::Rule;
use crate::schedule::Schedule;
use crate::state::State;
use rand::SeedableRng;
use rand::rngs::StdRng;

// ===================================================================
// Resources trait
// ===================================================================

/// A resource-accounting model for an Information Universe.
///
/// Assigns the three required resource costs from the Constitution
/// to states and rules. Implementors define what "space," "time," and
/// "locality" mean for their substrate.
///
/// # Type parameters
///
/// - `S: State` — The state type.
/// - `R: Rule<S>` — The rule type.
///
/// # Design contracts
///
/// - All three methods must return finite, non-negative values.
/// - `space` and `locality` should be cheap to compute — they may be
///   called once per state/rule along a trajectory.
/// - `time` describes the cost of a single rule *application*, not a
///   whole timestep — a schedule may apply a rule zero, one, or many
///   times within one call to [`Schedule::step`].
pub trait Resources<S: State, R: Rule<S>>: Debug + Send + Sync {
    /// A human-readable name for this resource model.
    fn name(&self) -> &str;

    /// `R_space` — the cost of representing this state.
    fn space(&self, state: &S) -> f64;

    /// `R_time` — the cost of one application of this rule.
    fn time(&self, rule: &R) -> f64;

    /// `R_local` — the maximum interaction radius this rule can reach.
    fn locality(&self, rule: &R) -> f64;
}

// ===================================================================
// The Resource Algebra
// ===================================================================

/// Verify the Resource Algebra for a composed transformation.
///
/// Per Constitution, resources are subadditive under
/// composition: `R_i(τ1∘τ2) ≤ R_i(τ1) + R_i(τ2)`. Given the
/// individual `(time, locality)` costs of two transformations and the
/// cost actually assigned to their composition, this checks that the
/// law holds — that combining two transformations never manufactures
/// resource for free, only ever consolidates what was already
/// required.
///
/// `space` is intentionally not part of this check: it is a property
/// of a *state*, not of a transformation, so composition of
/// transformations does not have a well-defined "space cost" to sum.
///
/// # Example
///
/// See `substrates::graph::resources` for a concrete use with
/// [`crate::substrates::graph::compose`], which already combines
/// locality via `max` — a tighter bound that still satisfies this law
/// since `max(a, b) ≤ a + b` for non-negative costs.
pub fn satisfies_resource_algebra(
    cost1: (f64, f64),
    cost2: (f64, f64),
    composed: (f64, f64),
) -> bool {
    composed.0 <= cost1.0 + cost2.0 && composed.1 <= cost1.1 + cost2.1
}

// ===================================================================
// Trajectory resource usage
// ===================================================================

/// Accumulated resource usage over a trajectory.
///
/// Distinct from the Resource Algebra above: this is practical
/// bookkeeping over a specific run, not a law about the
/// transformation semigroup.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResourceUsage {
    /// Number of transformation steps taken (Constitution:
    /// "transformation steps"). One unit per call to
    /// [`Schedule::step`], matching the granularity at which the
    /// schedule itself operates.
    pub total_time: f64,

    /// Sum of `Resources::time` over the rule set available at each
    /// step. A declared cost of the rule set itself, independent of
    /// how many steps were run.
    pub rule_set_time_cost: f64,

    /// The maximum `Resources::space` observed at any point along the
    /// trajectory (peak state-representation size).
    pub peak_space: f64,

    /// The maximum `Resources::locality` over the rule set available
    /// to the schedule at every step.
    ///
    /// This is the largest interaction radius the universe *could*
    /// exercise, not necessarily what fired at every step —
    /// [`Schedule::step`] does not expose which individual rule fired
    /// at which site, so per-firing locality cannot be measured from
    /// outside the schedule.
    pub max_locality: f64,
}

impl ResourceUsage {
    /// The zero usage — no steps taken, no space or locality observed.
    pub const ZERO: Self = Self {
        total_time: 0.0,
        rule_set_time_cost: 0.0,
        peak_space: 0.0,
        max_locality: 0.0,
    };
}

/// Measure resource usage along one trajectory per initial state.
///
/// Mirrors [`crate::calibration::generate_trajectories`]'s structure
/// and determinism (same seeding scheme), but tracks resource costs
/// instead of observation outputs.
///
/// # Parameters
/// * `initial_states` — Starting states, one trajectory per state.
/// * `rules` — The rule set available to the schedule at every step.
/// * `resources` — The resource model to evaluate costs with.
/// * `schedule` — The update schedule.
/// * `steps` — Number of timesteps per trajectory.
pub fn measure_resource_usage<S, R, K>(
    initial_states: &[S],
    rules: &[R],
    resources: &dyn Resources<S, R>,
    schedule: &K,
    steps: usize,
) -> Vec<ResourceUsage>
where
    S: State,
    R: Rule<S>,
    K: Schedule<S, R>,
{
    let rule_set_time_cost: f64 = rules.iter().map(|r| resources.time(r)).sum();
    let max_locality: f64 = rules
        .iter()
        .map(|r| resources.locality(r))
        .fold(0.0, f64::max);

    initial_states
        .iter()
        .enumerate()
        .map(|(i, initial)| {
            let seed = i as u64 * 137;
            let mut rng = StdRng::seed_from_u64(seed);
            let mut peak_space = resources.space(initial);
            let mut current = initial.clone();

            for _ in 0..steps {
                current = schedule.step(&current, rules, &mut rng);
                peak_space = peak_space.max(resources.space(&current));
            }

            ResourceUsage {
                total_time: steps as f64,
                rule_set_time_cost,
                peak_space,
                max_locality,
            }
        })
        .collect()
}

// ===================================================================
// Generic resource model (substrate-independent)
// ===================================================================

/// A trivial resource model: unit time per application, zero space,
/// pointwise locality.
///
/// Use this as a placeholder when a substrate has no meaningful
/// resource structure of its own, or as a starting point when
/// building a new substrate. Works with any state and rule type.
///
/// # Example
///
/// ```rust
/// use arco::state::State;
/// use arco::rules::{Rule, NoContext};
/// use arco::resources::{Resources, UnitResources};
///
/// #[derive(Clone, PartialEq, Eq, Hash, Debug)]
/// struct Bit { value: u8 }
/// impl State for Bit {
///     type Encoding = Vec<u8>;
///     fn canonical_encoding(&self) -> Self::Encoding { vec![self.value] }
///     fn distance(&self, other: &Self) -> u32 {
///         if self.value == other.value { 0 } else { 1 }
///     }
/// }
///
/// #[derive(Debug, Clone)]
/// struct FlipRule;
/// impl Rule<Bit> for FlipRule {
///     type Context = NoContext;
///     fn name(&self) -> &str { "Flip" }
///     fn apply(&self, state: &Bit, _ctx: &NoContext, _rng: &mut dyn rand::Rng) -> Bit {
///         Bit { value: 1 - state.value }
///     }
/// }
///
/// let model = UnitResources::default();
/// // `time`/`locality` don't mention the state type, and `UnitResources`
/// // implements `Resources<S, R>` for every `S`, so calling them directly
/// // needs the state type spelled out explicitly.
/// assert_eq!(<UnitResources as Resources<Bit, FlipRule>>::time(&model, &FlipRule), 1.0);
/// assert_eq!(<UnitResources as Resources<Bit, FlipRule>>::locality(&model, &FlipRule), 0.0);
/// assert_eq!(<UnitResources as Resources<Bit, FlipRule>>::space(&model, &Bit { value: 0 }), 0.0);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct UnitResources;

impl<S: State, R: Rule<S>> Resources<S, R> for UnitResources {
    fn name(&self) -> &str {
        "unit_resources"
    }

    fn space(&self, _state: &S) -> f64 {
        0.0
    }

    fn time(&self, _rule: &R) -> f64 {
        1.0
    }

    fn locality(&self, _rule: &R) -> f64 {
        0.0
    }
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_algebra_holds_for_equal_costs() {
        // Composing two transformations of cost (1,1) into one of
        // cost (1,1) (e.g. overwrite semantics) never exceeds the sum.
        assert!(satisfies_resource_algebra(
            (1.0, 1.0),
            (1.0, 1.0),
            (1.0, 1.0)
        ));
    }

    #[test]
    fn resource_algebra_holds_for_max_composition() {
        // max(a, b) <= a + b for non-negative a, b.
        assert!(satisfies_resource_algebra(
            (1.0, 0.0),
            (0.5, 1.0),
            (1.0, 1.0)
        ));
    }

    #[test]
    fn resource_algebra_rejects_manufactured_cost() {
        // A composed cost exceeding the sum of its parts violates
        // subadditivity.
        assert!(!satisfies_resource_algebra(
            (1.0, 0.0),
            (1.0, 0.0),
            (3.0, 0.0)
        ));
    }

    #[test]
    fn resource_usage_zero_is_zero() {
        let z = ResourceUsage::ZERO;
        assert_eq!(z.total_time, 0.0);
        assert_eq!(z.peak_space, 0.0);
        assert_eq!(z.max_locality, 0.0);
    }
}
