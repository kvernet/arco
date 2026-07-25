//! Rule trait — transformations in Information Universes.
//!
//! Per the Mathematical Constitution:
//!     T is a set of maps from S to S. Rules define how states
//!     evolve over time.
//!
//! # Rule context
//!
//! Different substrates need different information to apply a rule:
//!
//! - Cellular automata rules need nothing beyond the state itself.
//! - Graph rewrite rules need a vertex index and match information.
//! - Symbolic rewrite rules need a pattern location.
//!
//! The [`RuleContext`] trait and the associated `Context` type on
//! [`Rule`] allow each substrate to define what context its rules
//! require. The schedule creates the appropriate context internally;
//! users never need to construct contexts manually.
//!
//! # Implementing Rule
//!
//! ```rust
//! use arco::state::State;
//! use arco::rules::{Rule, RuleContext, NoContext};
//! use rand::Rng;
//!
//! // A CA rule needs no context
//! #[derive(Debug)]
//! struct CARule;
//!
//! impl<CAState: State> Rule<CAState> for CARule {
//!     type Context = NoContext;
//!     fn name(&self) -> &str { "Rule 110" }
//!     fn apply(&self, state: &CAState, _ctx: &NoContext, _rng: &mut dyn Rng) -> CAState {
//!         state.clone()
//!     }
//! }
//!
//! // A graph rule needs match info
//! #[derive(Debug)]
//! struct RewriteRule;
//!
//! impl<BinaryGraphState: State> Rule<BinaryGraphState> for RewriteRule {
//!     type Context = MatchInfo;
//!     fn name(&self) -> &str { "NAND" }
//!     fn apply(&self, state: &BinaryGraphState, _ctx: &MatchInfo, _rng: &mut dyn Rng) -> BinaryGraphState {
//!         state.clone()
//!     }
//! }
//!
//! #[derive(Clone, Debug)]
//! struct MatchInfo;
//!
//! impl RuleContext for MatchInfo {}
//!
//! fn main() {
//!     let ca = CARule;
//!     let re = RewriteRule;
//!
//!     println!("rules: {:?}, {:?}", ca, re);
//! }
//! ```

use crate::state::State;
use rand::Rng;
use std::fmt::Debug;

/// Marker trait for rule context types.
///
/// A rule context carries substrate-specific information needed to
/// apply a rule. Each substrate defines its own context type.
///
/// # Examples
///
/// - [`NoContext`] — for rules that need nothing beyond the state.
/// - Graph substrates use match information (vertex, neighbors).
/// - Symbolic substrates use pattern locations (start, end).
pub trait RuleContext: Clone + Send + Sync + Debug {}

/// A context for rules that need no additional information.
///
/// Use this when a rule can be applied to a state without knowing
/// which part of the state to modify. Cellular automata rules and
/// global transformation rules use `NoContext`.
#[derive(Debug, Clone, Default)]
pub struct NoContext;

impl RuleContext for NoContext {}

/// A transformation rule for an Information Universe.
///
/// Rules define how states evolve. Each rule specifies a [`Context`]
/// type that carries the information needed to apply it.
///
/// # Type parameters
///
/// - `S: State` — The state type this rule operates on.
/// - `Context: RuleContext` — The context type needed for application.
///
/// # Design contracts
///
/// - **Immutability**: Rules return new states. They never modify
///   the input state in place.
/// - **Purity**: Rule application should be a pure function of the
///   state, context, and RNG.
/// - **Send + Sync**: Rules must be shareable across threads.
///
/// # Examples
///
/// ```
/// use arco::{
///     rules::{NoContext, Rule},
///     state::State,
/// };
/// use rand::{Rng, SeedableRng, rngs::StdRng};
///
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// struct CounterState {
///     value: u8,
/// }
///
/// impl CounterState {
///     fn new(value: u8) -> Self {
///         Self { value }
///     }
/// }
///
/// impl State for CounterState {
///     type Encoding = Vec<u8>;
///
///     fn canonical_encoding(&self) -> Self::Encoding {
///         vec![self.value]
///     }
///
///     fn distance(&self, other: &Self) -> u32 {
///         if self.value == other.value { 1 } else { 0 }
///     }
/// }
///
/// #[derive(Debug)]
/// struct IncrementRule;
///
/// impl Rule<CounterState> for IncrementRule {
///     type Context = NoContext;
///     fn name(&self) -> &str {
///         "Increment"
///     }
///
///     fn apply(
///         &self,
///         state: &CounterState,
///         _context: &Self::Context,
///         _rng: &mut dyn Rng,
///     ) -> CounterState {
///         CounterState {
///             value: state.value.wrapping_add(1),
///         }
///     }
/// }
///
/// fn main() {
///     let mut rng = StdRng::seed_from_u64(42);
///
///     let mut state = CounterState::new(1);
///     println!("{:?}", state);
///
///     let rule = IncrementRule;
///     let context = NoContext;
///
///     for _ in 1..=254 {
///         state = rule.apply(&state, &context, &mut rng);
///     }
///     
///     println!("{:?}", state);
/// }
/// ```
pub trait Rule<S: State>: Debug + Send + Sync {
    /// The context type this rule needs when applying.
    ///
    /// This is substrate-specific. The schedule creates the
    /// appropriate context internally.
    type Context: RuleContext;

    /// A human-readable name for this rule.
    fn name(&self) -> &str;

    /// Apply this rule to a state, returning the new state.
    ///
    /// # Arguments
    /// * `state` — The state before transformation. Not modified.
    /// * `context` — Substrate-specific context for application.
    /// * `rng` — Random number generator for stochastic rules.
    fn apply(&self, state: &S, context: &Self::Context, rng: &mut dyn Rng) -> S;
}
