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
//! experimental system. It owns or references:
//!
//! - **S** (state space): A collection of possible states, accessed
//!   via `state_space()`.
//! - **T** (transformation set): The rules available for evolution,
//!   accessed via `rules()`.
//! - **O** (observation operators): How states are perceived, accessed
//!   via `observation()`.
//! - **K** (update schedule): The temporal structure, accessed via
//!   `schedule()`.
//!
//! **R** (resource constraints) and **I** (invariant structure) are
//! not yet represented in the trait — they are placeholders for
//! future extensions.
//!
//! # Why a trait?
//!
//! The scientific cycle (`run_cycle`) operates on any type that
//! implements `InformationUniverse`. This means:
//!
//! - The Binary Graph Universe, Cellular Automata, and user-defined
//!   substrates all use the same pipeline.
//! - The cycle doesn't need to know substrate-specific details.
//! - New substrates require only trait implementations, not changes
//!   to ARCO's core.
//!
//! # Implementing InformationUniverse
//!
//! ```rust
//! use arco::observation::Observation;
//! use arco::rules::{NoContext, Rule};
//! use arco::schedule::Schedule;
//! use arco::state::State;
//! use arco::universe::InformationUniverse;
//!
//! use rand::Rng;
//!
//! fn main() {
//!     let universe = MyUniverse {
//!         states: vec![],
//!         rules: vec![],
//!         observer: MyObserver,
//!         schedule: MySchedule,
//!     };
//!
//!     println!("{:?}", universe);
//! }
//!
//! #[derive(Clone, Debug, PartialEq, Eq, Hash)]
//! struct MyState {
//!     data: Vec<u8>,
//! }
//!
//! impl State for MyState {
//!     type Encoding = Vec<u8>;
//!
//!     fn canonical_encoding(&self) -> Self::Encoding {
//!         self.data.clone()
//!     }
//!
//!     fn distance(&self, other: &Self) -> u32 {
//!         let mut d = 0u32;
//!         for (a, b) in self.data.iter().zip(other.data.iter()) {
//!             if a != b {
//!                 d += 1;
//!             }
//!         }
//!
//!         d
//!     }
//! }
//!
//! #[derive(Clone, Debug)]
//! struct MyRule;
//!
//! impl Rule<MyState> for MyRule {
//!     type Context = NoContext;
//!
//!     fn name(&self) -> &str {
//!         "Rule"
//!     }
//!
//!     fn apply(&self, state: &MyState, _context: &Self::Context, _rng: &mut dyn Rng) -> MyState {
//!         state.clone()
//!     }
//! }
//!
//! #[derive(Debug)]
//! struct MyObserver;
//!
//! impl Observation<MyState> for MyObserver {
//!     type Output = Vec<u8>;
//!
//!     fn observe(&self, state: &MyState) -> Self::Output {
//!         state.data.clone()
//!     }
//! }
//!
//! #[derive(Debug)]
//! struct MySchedule;
//!
//! impl Schedule<MyState, MyRule> for MySchedule {
//!     fn name(&self) -> &str {
//!         "Schedule"
//!     }
//!
//!     fn selection(&self) -> &str {
//!         "exhaustive"
//!     }
//!
//!     fn timing(&self) -> &str {
//!         "asynchronous"
//!     }
//!
//!     fn step(&self, state: &MyState, rules: &[MyRule], rng: &mut dyn Rng) -> MyState {
//!         let mut current = state.clone();
//!         let context = NoContext;
//!         for rule in rules {
//!             current = rule.apply(state, &context, rng);
//!         }
//!         current
//!     }
//! }
//!
//! #[derive(Debug)]
//! struct MyUniverse {
//!     states: Vec<MyState>,
//!     rules: Vec<MyRule>,
//!     observer: MyObserver,
//!     schedule: MySchedule,
//! }
//!
//! impl InformationUniverse for MyUniverse {
//!     type State = MyState;
//!     type Rule = MyRule;
//!     type Observation = MyObserver;
//!     type Schedule = MySchedule;
//!
//!     fn state_space(&self) -> &[Self::State] {
//!         &self.states
//!     }
//!     fn rules(&self) -> &[Self::Rule] {
//!         &self.rules
//!     }
//!     fn observation(&self) -> &Self::Observation {
//!         &self.observer
//!     }
//!     fn schedule(&self) -> &Self::Schedule {
//!         &self.schedule
//!     }
//!     fn null_rules(&self, _rng: &mut dyn Rng) -> Vec<Self::Rule> {
//!         self.rules.to_vec()
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
///   `run_cycle::<U: InformationUniverse>(&config)` is cleaner than
///   `run_cycle::<S, R, O, K>(&config)`.
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
    /// May be the full state space (for small universes) or a
    /// representative sample (for large universes).
    fn state_space(&self) -> &[Self::State];

    /// The transformation rules available in this universe.
    fn rules(&self) -> &[Self::Rule];

    /// The observation operator for this universe.
    ///
    /// Defines how states are perceived. The same universe observed
    /// through different operators may show different emergence
    /// properties.
    fn observation(&self) -> &Self::Observation;

    /// The update schedule for this universe.
    ///
    /// Defines the temporal order and selection of rule applications.
    /// Two universes differing only in schedule are distinct objects
    /// of study.
    fn schedule(&self) -> &Self::Schedule;

    /// Generate a destructive rule set for null-distribution calibration.
    ///
    /// Destructive rules should destroy information — they represent
    /// the null hypothesis against which emergence is measured. What
    /// "destructive" means is substrate-specific:
    ///
    /// - In a graph rewriting universe, destructive rules scramble
    ///   vertex labels or randomize edge structure.
    /// - In a cellular automaton, destructive rules are the most
    ///   chaotic rules that rapidly destroy initial conditions.
    /// - In a symbolic universe, destructive rules replace terms
    ///   with random expressions.
    ///
    /// The calibration pipeline calls this method to generate null
    /// universes. Each null universe should contain at least one
    /// strongly destructive rule to prevent degenerate constant
    /// states from inflating the null distribution.
    ///
    /// # Arguments
    /// * `rng` — Random number generator for stochastic rule selection.
    ///
    /// # Returns
    /// A vector of rules that destroy information in this substrate.
    fn null_rules(&self, rng: &mut dyn Rng) -> Vec<Self::Rule>;
}
