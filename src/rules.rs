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
//! # Quick start
//!
//! Implement `Rule` for your transformation type. Choose a `Context`
//! that carries the information your rule needs:
//!
//! ```rust
//! use arco::state::State;
//! use arco::rules::{Rule, RuleContext, NoContext};
//! use rand::{Rng, rngs::StdRng, SeedableRng};
//!
//! #[derive(Clone, PartialEq, Eq, Hash, Debug)]
//! struct MyState { data: Vec<u8> }
//!
//! impl State for MyState {
//!     type Encoding = Vec<u8>;
//!     fn canonical_encoding(&self) -> Self::Encoding { self.data.clone() }
//!     fn distance(&self, other: &Self) -> u32 {
//!         self.data.iter().zip(other.data.iter())
//!             .map(|(a,b)| if a != b { 1 } else { 0 }).sum()
//!     }
//! }
//!
//! // Use NoContext for rules that need nothing beyond the state.
//! #[derive(Debug, Clone)]
//! struct IncrementRule;
//!
//! impl Rule<MyState> for IncrementRule {
//!     type Context = NoContext;
//!     fn name(&self) -> &str { "Increment" }
//!     fn apply(&self, state: &MyState, _ctx: &NoContext, _rng: &mut dyn Rng) -> MyState {
//!         let new_data: Vec<u8> = state.data.iter().map(|b| b.wrapping_add(1)).collect();
//!         MyState { data: new_data }
//!     }
//! }
//!
//! // For rules that need match information, define a custom context.
//! #[derive(Debug, Clone)]
//! struct MatchInfo { position: usize }
//! impl RuleContext for MatchInfo {}
//!
//! #[derive(Debug, Clone)]
//! struct FlipAtPosition;
//!
//! impl Rule<MyState> for FlipAtPosition {
//!     type Context = MatchInfo;
//!     fn name(&self) -> &str { "FlipAtPosition" }
//!     fn apply(&self, state: &MyState, ctx: &MatchInfo, _rng: &mut dyn Rng) -> MyState {
//!         let mut new_data = state.data.clone();
//!         if ctx.position < new_data.len() {
//!             new_data[ctx.position] = 1 - new_data[ctx.position];
//!         }
//!         MyState { data: new_data }
//!     }
//! }
//!
//! let state = MyState { data: vec![0, 1, 0] };
//! let increment = IncrementRule;
//! let ctx = NoContext;
//! let result = increment.apply(&state, &ctx, &mut StdRng::seed_from_u64(42));
//! assert_eq!(result.data, vec![1, 2, 1]);
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
/// # Example
///
/// ```rust
/// use arco::rules::{NoContext, Rule};
/// use arco::state::State;
/// use rand::rngs::StdRng;
/// use rand::{Rng, SeedableRng};
///
/// #[derive(Clone, PartialEq, Eq, Hash, Debug)]
/// struct BitState {
///     value: u8,
/// }
///
/// impl State for BitState {
///     type Encoding = Vec<u8>;
///     fn canonical_encoding(&self) -> Self::Encoding {
///         vec![self.value]
///     }
///     fn distance(&self, other: &Self) -> u32 {
///         if self.value == other.value { 0 } else { 1 }
///     }
/// }
///
/// /// A rule that flips the bit.
/// #[derive(Debug, Clone)]
/// struct FlipRule;
///
/// impl Rule<BitState> for FlipRule {
///     type Context = NoContext;
///     fn name(&self) -> &str {
///         "Flip"
///     }
///     fn apply(&self, state: &BitState, _context: &NoContext, _rng: &mut dyn Rng) -> BitState {
///         BitState {
///             value: 1 - state.value,
///         }
///     }
/// }
///
/// let state = BitState { value: 0 };
/// let rule = FlipRule;
/// let ctx = NoContext;
/// let new_state = rule.apply(&state, &ctx, &mut StdRng::seed_from_u64(42));
/// assert_eq!(new_state.value, 1);
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
