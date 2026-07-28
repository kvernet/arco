//! Schedule trait — temporal structure in Information Universes.
//!
//! Per the Mathematical Constitution:
//!     K is the update schedule — a rule specifying the order and
//!     selection of transformations at each timestep. Two universes
//!     differing only in schedule are distinct objects of study.
//!
//! # The Schedule trait
//!
//! A schedule determines how rules are applied over time. It answers:
//! which rules fire, in what order, and with what concurrency?
//! The schedule is a first-class component — changing the schedule
//! can change whether computation is detected.
//!
//! # Schedule Semantics
//!
//! Schedules are classified along two axes:
//!
//! **Timing**
//! - *Synchronous*: All updates within a timestep are computed from
//!   the same pre-timestep state. No update within the timestep
//!   sees any other update.
//! - *Asynchronous*: Updates are applied immediately. Later updates
//!   within the same timestep see the results of earlier updates.
//!
//! **Selection**
//! - *Exhaustive*: Every update site is visited exactly once per
//!   timestep.
//! - *Stochastic*: Sites are sampled probabilistically.
//! - *Priority*: Sites are ordered by a fixed criterion.
//!
//! # Quick start
//!
//! Implement `Schedule` to define the temporal order of rule
//! application. The schedule creates rule contexts internally —
//! users never construct contexts manually:
//!
//! ```rust
//! use arco::state::State;
//! use arco::rules::{Rule, NoContext};
//! use arco::schedule::Schedule;
//! use rand::{Rng, rngs::StdRng, SeedableRng};
//!
//! #[derive(Clone, PartialEq, Eq, Hash, Debug)]
//! struct MyState { count: u32 }
//!
//! impl State for MyState {
//!     type Encoding = Vec<u8>;
//!     fn canonical_encoding(&self) -> Self::Encoding { self.count.to_le_bytes().to_vec() }
//!     fn distance(&self, other: &Self) -> u32 {
//!         if self.count == other.count { 0 } else { 1 }
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! struct AddOne;
//! impl Rule<MyState> for AddOne {
//!     type Context = NoContext;
//!     fn name(&self) -> &str { "AddOne" }
//!     fn apply(&self, state: &MyState, _ctx: &NoContext, _rng: &mut dyn Rng) -> MyState {
//!         MyState { count: state.count + 1 }
//!     }
//! }
//!
//! /// Apply all rules in sequence, once per timestep.
//! #[derive(Debug, Clone)]
//! struct AllRulesSchedule;
//!
//! impl Schedule<MyState, AddOne> for AllRulesSchedule {
//!     fn name(&self) -> &str { "all_rules" }
//!     fn timing(&self) -> &str { "asynchronous" }
//!     fn selection(&self) -> &str { "exhaustive" }
//!
//!     fn step(&self, state: &MyState, rules: &[AddOne], rng: &mut dyn Rng) -> MyState {
//!         let mut current = state.clone();
//!         for rule in rules {
//!             current = rule.apply(&current, &NoContext, rng);
//!         }
//!         current
//!     }
//! }
//!
//! let state = MyState { count: 0 };
//! let schedule = AllRulesSchedule;
//! let rules = vec![AddOne, AddOne, AddOne];
//! let mut rng = StdRng::seed_from_u64(42);
//! let result = schedule.step(&state, &rules, &mut rng);
//! assert_eq!(result.count, 3);
//! ```
//!
//! # Generic schedules (substrate-independent)
//!
//! These schedules work with any rule type that uses [`NoContext`].
//! Use them when you don't need substrate-specific scheduling logic.
//!
//! ```rust
//! use arco::state::State;
//! use arco::rules::{Rule, NoContext};
//! use arco::schedule::{Schedule, SequentialSchedule, RandomRuleSchedule};
//! use rand::{Rng, SeedableRng, rngs::StdRng};
//!
//! #[derive(Clone, PartialEq, Eq, Hash, Debug)]
//! struct Counter { value: u32 }
//!
//! impl State for Counter {
//!     type Encoding = Vec<u8>;
//!     fn canonical_encoding(&self) -> Self::Encoding { self.value.to_le_bytes().to_vec() }
//!     fn distance(&self, other: &Self) -> u32 {
//!         if self.value == other.value { 0 } else { 1 }
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! struct Increment;
//! impl Rule<Counter> for Increment {
//!     type Context = NoContext;
//!     fn name(&self) -> &str { "Increment" }
//!     fn apply(&self, state: &Counter, _ctx: &NoContext, _rng: &mut dyn Rng) -> Counter {
//!         Counter { value: state.value + 1 }
//!     }
//! }
//!
//! let state = Counter { value: 0 };
//! let rules = vec![Increment, Increment, Increment];
//! let mut rng = StdRng::seed_from_u64(42);
//!
//! // SequentialSchedule: applies every rule in order
//! let sequential = SequentialSchedule::new();
//! let result = sequential.step(&state, &rules, &mut rng);
//! assert_eq!(result.value, 3);
//!
//! // RandomRuleSchedule: picks one rule at random per timestep
//! let random = RandomRuleSchedule::new();
//! let result = random.step(&state, &rules, &mut rng);
//! assert!(result.value == 1); // one increment applied
//! ```
//!
//! # Built-in schedules
//!
//! Substrate-specific schedules live in their substrate modules.
//! See [`arco::substrates::graph::AllVerticesSchedule`] for the
//! asynchronous exhaustive schedule used in the Binary Graph Universe.
//!
//! Generic schedules (sequential, random) can be defined in
//! [`arco::schedule::generic`] and reused across substrates.

use crate::rules::{NoContext, Rule};
use crate::state::State;
use rand::{Rng, RngExt};
use std::fmt::Debug;

/// The temporal structure of an Information Universe.
///
/// A schedule determines the order and selection of rule applications
/// at each timestep.
///
/// # Type parameters
///
/// - `S: State` — The state type.
/// - `R: Rule<S>` — The rule type.
///
/// # Design contracts
///
/// - **Determinism**: Given the same state, rules, and RNG seed,
///   the schedule must produce the same next state. The RNG is
///   the sole source of stochasticity.
/// - **Immutability**: The schedule returns a new state. The input
///   state is not modified.
/// - **Send + Sync**: Schedules must be shareable across threads.
///
/// /// # Example
///
/// ```rust
/// use arco::state::State;
/// use arco::rules::{NoContext, Rule};
/// use arco::schedule::Schedule;
/// use rand::{Rng, rngs::StdRng, SeedableRng};
///
/// #[derive(Clone, PartialEq, Eq, Hash, Debug)]
/// struct BitState { value: u8 }
///
/// impl State for BitState {
///     type Encoding = Vec<u8>;
///     fn canonical_encoding(&self) -> Self::Encoding { vec![self.value] }
///     fn distance(&self, other: &Self) -> u32 {
///         if self.value == other.value { 0 } else { 1 }
///     }
/// }
///
/// #[derive(Debug, Clone)]
/// struct FlipRule;
///
/// impl Rule<BitState> for FlipRule {
///     type Context = NoContext;
///     fn name(&self) -> &str { "Flip" }
///     fn apply(&self, state: &BitState, _ctx: &NoContext, _rng: &mut dyn Rng) -> BitState {
///         BitState { value: 1 - state.value }
///     }
/// }
///
/// /// A schedule that applies the first rule once per timestep.
/// #[derive(Debug, Clone)]
/// struct FirstRuleSchedule;
///
/// impl Schedule<BitState, FlipRule> for FirstRuleSchedule {
///     fn name(&self) -> &str { "first_rule" }
///     fn timing(&self) -> &str { "asynchronous" }
///     fn selection(&self) -> &str { "priority" }
///
///     fn step(&self, state: &BitState, rules: &[FlipRule], rng: &mut dyn Rng) -> BitState {
///         if let Some(rule) = rules.first() {
///             rule.apply(state, &NoContext, rng)
///         } else {
///             state.clone()
///         }
///     }
/// }
///
/// let state = BitState { value: 0 };
/// let schedule = FirstRuleSchedule;
/// let rules = vec![FlipRule];
/// let mut rng = StdRng::seed_from_u64(42);
/// let new_state = schedule.step(&state, &rules, &mut rng);
/// assert_eq!(new_state.value, 1);
/// ```
pub trait Schedule<S: State, R: Rule<S>>: Debug + Send + Sync {
    /// A human-readable name for this schedule.
    fn name(&self) -> &str;

    /// The timing semantics: `"synchronous"` or `"asynchronous"`.
    ///
    /// - Synchronous: All updates within a timestep are computed
    ///   from the same pre-timestep state.
    /// - Asynchronous: Updates are applied immediately and later
    ///   updates see earlier changes.
    fn timing(&self) -> &str;

    /// The selection semantics: `"exhaustive"`, `"stochastic"`, or
    /// `"priority"`.
    ///
    /// - Exhaustive: Every site is visited exactly once.
    /// - Stochastic: Sites are sampled probabilistically.
    /// - Priority: Sites are ordered by a fixed criterion.
    fn selection(&self) -> &str;

    /// Apply one timestep of evolution.
    ///
    /// # Arguments
    /// * `state` — The state before this timestep. Not modified.
    /// * `rules` — The rule set available for application.
    /// * `rng` — Random number generator for stochastic scheduling.
    ///
    /// # Returns
    /// The state after one timestep.
    fn step(&self, state: &S, rules: &[R], rng: &mut dyn Rng) -> S;
}

// ===================================================================
// Generic schedules (substrate-independent)
// ===================================================================

/// A schedule that applies all rules in sequence.
///
/// Every rule is applied exactly once per timestep, in the order
/// they appear in the rule set. Updates are asynchronous — each
/// rule sees the results of previous rules within the same timestep.
///
/// # Semantics
/// - Timing: asynchronous
/// - Selection: exhaustive (all rules fire, in order)
///
/// This is the simplest possible schedule and works with any rule
/// type.
#[derive(Debug, Clone, Default)]
pub struct SequentialSchedule;

impl SequentialSchedule {
    pub fn new() -> Self {
        Self
    }
}

impl<S: State, R: Rule<S, Context = NoContext>> Schedule<S, R> for SequentialSchedule {
    fn name(&self) -> &str {
        "sequential"
    }

    fn timing(&self) -> &str {
        "asynchronous"
    }

    fn selection(&self) -> &str {
        "exhaustive"
    }

    fn step(&self, state: &S, rules: &[R], rng: &mut dyn Rng) -> S {
        let mut current = state.clone();
        let ctx = NoContext;
        for rule in rules {
            current = rule.apply(&current, &ctx, rng);
        }
        current
    }
}

/// A schedule that selects one rule at random per timestep.
///
/// At each timestep, a single rule is chosen uniformly at random
/// from the rule set and applied once. The same rule may be selected
/// across multiple timesteps — each timestep is an independent
/// random draw.
///
/// This models stochastic asynchronous dynamics where one
/// transformation event occurs per timestep.
///
/// # Semantics
/// - Timing: asynchronous (single update per timestep)
/// - Selection: stochastic (one rule drawn uniformly per timestep)
#[derive(Debug, Clone, Default)]
pub struct RandomRuleSchedule;

impl RandomRuleSchedule {
    pub fn new() -> Self {
        Self
    }
}

impl<S: State, R: Rule<S, Context = NoContext>> Schedule<S, R> for RandomRuleSchedule {
    fn name(&self) -> &str {
        "random_single"
    }

    fn timing(&self) -> &str {
        "asynchronous"
    }

    fn selection(&self) -> &str {
        "stochastic"
    }

    fn step(&self, state: &S, rules: &[R], rng: &mut dyn Rng) -> S {
        if rules.is_empty() {
            return state.clone();
        }
        let idx = rng.random_range(0..rules.len());
        let ctx = NoContext;
        rules[idx].apply(state, &ctx, rng)
    }
}
