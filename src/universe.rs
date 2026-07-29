//! InformationUniverse trait — the top-level abstraction.
//!
//! Per the Mathematical Constitution:
//!     An Information Universe is a 6-tuple U = (S, T, O, R, I, K).
//!     This module defines the trait that binds these components
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
//! - **K** (update schedule): The temporal structure, via
//!   [`schedule()`].
//!
//! **R** (resource constraints) and **I** (invariant structure) are
//! not yet represented in the trait — they are placeholders for
//! future extensions.
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
//! struct MyUniverse {
//!     states: Vec<BitState>,
//!     schedule: SequentialSchedule,
//! }
//!
//! impl InformationUniverse for MyUniverse {
//!     type State = BitState;
//!     type Rule = FlipRule;
//!     type Observation = BitObserver;
//!     type Schedule = SequentialSchedule;
//!
//!     fn state_space(&self) -> &[Self::State] { &self.states }
//!     fn observation(&self) -> &Self::Observation { &BitObserver }
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
//! ```

use rand::Rng;

use crate::observation::Observation;
use crate::rules::Rule;
use crate::schedule::Schedule;
use crate::state::State;

/// The top-level abstraction for an Information Universe.
///
/// Bundles the four core components — state space, transformation
/// rules, observation operators, and update schedule — into a single
/// type. The scientific cycle operates on any implementor of this
/// trait.
///
/// # Type parameters
///
/// - `State`: The state type (must implement [`State`]).
/// - `Rule`: The rule type (must implement [`Rule<State>`]).
/// - `Observation`: The observer type (must implement
///   [`Observation<State>`]).
/// - `Schedule`: The schedule type (must implement
///   [`Schedule<State, Rule>`]).
///
/// # Design notes
///
/// - **R** (resource constraints) and **I** (invariant structure)
///   are not yet represented. They are placeholders for future
///   extensions.
/// - The trait uses associated types rather than generic parameters
///   so that a single type can represent a complete universe.
pub trait InformationUniverse {
    /// The state type for this universe.
    type State: State;

    /// The rule type for this universe.
    type Rule: Rule<Self::State>;

    /// The observation operator type for this universe.
    type Observation: Observation<Self::State> + Sync;

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
