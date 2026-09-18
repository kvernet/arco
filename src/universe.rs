//! InformationUniverse trait — the top-level abstraction.
//!
//! Per the Mathematical Constitution:
//!     An Information Universe is a 6-tuple U = (S, T, O, R, I, K).
//!     This module defines the trait that binds all six components
//!     together into a single type that the scientific cycle can
//!     operate on.
//!
//! # The InformationUniverse trait
//!
//! A type implementing `InformationUniverse` represents a complete
//! experimental system. It provides:
//!
//! - **S** (state space): A collection of possible states for
//!   sampling initial conditions, via [`state_space()`].
//! - **T** (transformation set): Rules generated on demand via
//!   [`generate_rules()`] and [`null_rules()`].
//! - **O** (observation operators): How states are perceived, via
//!   [`observation()`].
//! - **R** (resource constraints): Time, space, and locality costs,
//!   via [`resources()`].
//! - **I** (invariant structure): Conserved quantities, via
//!   [`invariants()`].
//! - **K** (update schedule): The temporal structure, via
//!   [`schedule()`].
//!
//! All six components are required — the tuple is not considered
//! complete without a resource model and an (possibly empty) set of
//! invariants. A substrate with no meaningful resource structure of
//! its own can use [`crate::resources::UnitResources`] as a
//! placeholder; a substrate with no invariants to claim can return an
//! empty slice from `invariants()`. Both must still be stated
//! explicitly, rather than silently defaulted, so that every universe
//! is honest about what it does and does not claim.
//!
//! # Why a trait?
//!
//! The scientific cycle ([`run_cycle`]) operates on any type that
//! implements `InformationUniverse`. This means:
//!
//! - The Binary Graph Universe, Cellular Automata, and user-defined
//!   substrates all use the same pipeline.
//! - The cycle doesn't need to know substrate-specific details.
//! - New substrates require only trait implementations, not changes
//!   to ARCO's core.
//!
//! # Quick start
//!
//! ```rust
//! use arco::state::State;
//! use arco::rules::{Rule, NoContext};
//! use arco::observation::Observation;
//! use arco::resources::{Resources, UnitResources};
//! use arco::invariants::Invariant;
//! use arco::schedule::SequentialSchedule;
//! use arco::universe::InformationUniverse;
//! use rand::{Rng, RngExt};
//!
//! #[derive(Clone, PartialEq, Eq, Hash, Debug)]
//! struct BitState { value: u8 }
//!
//! impl State for BitState {
//!     type Encoding = Vec<u8>;
//!     fn canonical_encoding(&self) -> Self::Encoding { vec![self.value] }
//!     fn distance(&self, other: &Self) -> u32 {
//!         if self.value == other.value { 0 } else { 1 }
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! struct FlipRule;
//! impl Rule<BitState> for FlipRule {
//!     type Context = NoContext;
//!     fn name(&self) -> &str { "Flip" }
//!     fn apply(&self, state: &BitState, _ctx: &NoContext, _rng: &mut dyn Rng) -> BitState {
//!         BitState { value: 1 - state.value }
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! struct BitObserver;
//! impl Observation<BitState> for BitObserver {
//!     type Output = u8;
//!     fn observe(&self, state: &BitState) -> Self::Output { state.value }
//! }
//!
//! /// I(s) = value. Not conserved by FlipRule -- included to show a
//! /// candidate invariant that fails, not just ones that hold.
//! #[derive(Debug, Clone, Copy, Default)]
//! struct ValueInvariant;
//! impl Invariant<BitState> for ValueInvariant {
//!     fn name(&self) -> &str { "value" }
//!     fn evaluate(&self, state: &BitState) -> f64 { state.value as f64 }
//! }
//!
//! struct MyUniverse {
//!     states: Vec<BitState>,
//!     resources: UnitResources,
//!     invariants: Vec<Box<dyn Invariant<BitState>>>,
//!     schedule: SequentialSchedule,
//! }
//!
//! impl InformationUniverse for MyUniverse {
//!     type State = BitState;
//!     type Rule = FlipRule;
//!     type Observation = BitObserver;
//!     type Resources = UnitResources;
//!     type Schedule = SequentialSchedule;
//!
//!     fn state_space(&self) -> &[Self::State] { &self.states }
//!     fn observation(&self) -> &Self::Observation { &BitObserver }
//!     fn resources(&self) -> &Self::Resources { &self.resources }
//!     fn invariants(&self) -> &[Box<dyn Invariant<Self::State>>] { &self.invariants }
//!     fn schedule(&self) -> &Self::Schedule { &self.schedule }
//!
//!     fn generate_rules(&self, rng: &mut dyn Rng) -> (Vec<Self::Rule>, f64) {
//!         let n = rng.random_range(1..=3);
//!         let rules: Vec<FlipRule> = (0..n).map(|_| FlipRule).collect();
//!         (rules, 1.0)
//!     }
//!
//!     fn null_rules(&self, _rng: &mut dyn Rng) -> Vec<Self::Rule> {
//!         vec![FlipRule] // flipping is maximally destructive in this universe
//!     }
//! }
//!
//! let universe = MyUniverse {
//!     states: vec![BitState { value: 0 }, BitState { value: 1 }],
//!     schedule: SequentialSchedule::new(),
//!     resources: UnitResources,
//!     invariants: vec![Box::new(ValueInvariant)],
//! };
//! assert_eq!(universe.invariants().len(), 1);
//! let resources = universe.resources();
//! assert_eq!(<UnitResources as Resources<BitState, FlipRule>>::space(resources, &universe.states[0]), 0.0);
//! ```

use rand::Rng;

use crate::invariants::Invariant;
use crate::observation::Observation;
use crate::resources::Resources;
use crate::rules::Rule;
use crate::schedule::Schedule;
use crate::state::State;

/// The top-level abstraction for an Information Universe.
///
/// Bundles all six components of the Constitution's 6-tuple
/// `U = (S, T, O, R, I, K)` — state space, transformation rules,
/// observation operators, resource constraints, invariant structure,
/// and update schedule — into a single type. The scientific cycle
/// operates on any implementor of this trait.
///
/// # Type parameters
///
/// - `State`: The state type (must implement [`State`]).
/// - `Rule`: The rule type (must implement [`Rule<State>`]).
/// - `Observation`: The observer type (must implement
///   [`Observation<State>`]).
/// - `Resources`: The resource-accounting type (must implement
///   [`Resources<State, Rule>`]).
/// - `Schedule`: The schedule type (must implement
///   [`Schedule<State, Rule>`]).
///
/// # Design notes
///
/// - `Resources` is an associated type, like `Observation` and
///   `Schedule`: a universe has exactly one resource-accounting
///   scheme.
/// - `invariants()` returns a `Vec<Box<dyn Invariant<Self::State>>>`
///   rather than an associated type, because the Constitution defines
///   `I` as a *set* of functions — a universe may claim zero, one, or
///   many invariants, and they need not share a concrete type.
/// - The trait uses associated types rather than generic parameters
///   so that a single type can represent a complete universe.
pub trait InformationUniverse {
    /// The state type for this universe.
    type State: State;

    /// The rule type for this universe.
    type Rule: Rule<Self::State>;

    /// The observation operator type for this universe.
    type Observation: Observation<Self::State> + Sync;

    /// The resource-accounting type for this universe.
    type Resources: Resources<Self::State, Self::Rule>;

    /// The schedule type for this universe.
    type Schedule: Schedule<Self::State, Self::Rule>;

    /// The state space — a collection of possible states.
    ///
    /// Used for sampling initial conditions for ensemble generation.
    fn state_space(&self) -> &[Self::State];

    /// The observation operator for this universe.
    ///
    /// Defines how states are perceived. The same universe observed
    /// through different operators may show different emergence
    /// properties.
    fn observation(&self) -> &Self::Observation;

    /// The resource-accounting model for this universe (`R`).
    ///
    /// Assigns time, space, and locality costs to states and rules.
    /// Substrates with no meaningful resource structure of their own
    /// may use [`crate::resources::UnitResources`].
    fn resources(&self) -> &Self::Resources;

    /// The set of invariants this universe claims (`I`).
    ///
    /// May be empty — not every universe claims conserved quantities.
    /// Each invariant is independently checkable via
    /// [`Invariant::is_conserved`], [`crate::invariants::violation_rate`],
    /// or [`crate::invariants::exhaustively_conserved`].
    fn invariants(&self) -> &[Box<dyn Invariant<Self::State>>];

    /// The update schedule for this universe.
    ///
    /// Defines the temporal order and selection of rule applications.
    fn schedule(&self) -> &Self::Schedule;

    /// Generate a rule set for this universe.
    ///
    /// Returns a tuple of (rules, structured_ratio) where
    /// `structured_ratio` is a substrate-specific measure of how
    /// "structured" the rule set is (0.0 = purely destructive,
    /// 1.0 = purely structured). This is used for spectrum analysis
    /// and hypothesis testing.
    ///
    /// # Arguments
    /// * `rng` — Random number generator for stochastic rule selection.
    ///
    /// The `rng` parameter is provided for substrates that need stochastic
    /// rule generation. Implementations that use deterministic cycling
    /// (e.g., pre-generated rule sets) may ignore it.
    fn generate_rules(&self, rng: &mut dyn Rng) -> (Vec<Self::Rule>, f64);

    /// Generate a destructive rule set for null-distribution calibration.
    ///
    /// Destructive rules should destroy information — they represent
    /// the null hypothesis against which emergence is measured.
    /// Each null universe should contain at least one strongly
    /// destructive rule to prevent degenerate constant states from
    /// inflating the null distribution.
    ///
    /// # Arguments
    /// * `rng` — Random number generator for stochastic rule selection.
    fn null_rules(&self, rng: &mut dyn Rng) -> Vec<Self::Rule>;
}
