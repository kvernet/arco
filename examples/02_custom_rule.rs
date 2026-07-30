//! Example 02 — Implement Rule for a custom type.
//!
//! The `Rule` trait requires:
//! - A `Context` type (substrate-specific information for applying the rule)
//! - A `name()` method (human-readable identifier)
//! - An `apply()` method (state + context + rng → new state)
//!
//! We implement three rules for our Counter:
//! - IncrementRule (deterministic, uses NoContext)
//! - SetRule (deterministic, uses custom SetValue context)
//! - MaybeResetRule (stochastic, uses NoContext)
//!
//! # Design principle
//!
//! The rule is the **policy** — it decides *when* and *whether* to
//! transform the state. The state provides reusable **mechanisms**
//! (like `increment()` and `reset()`) that rules call. This keeps
//! rules composable and states reusable across different rule sets.

use arco::rules::{NoContext, Rule, RuleContext};
use arco::state::State;
use rand::{Rng, RngExt, SeedableRng};
use std::fmt;
use std::hash::{Hash, Hasher};

// ===================================================================
// Our state (same as example 01)
// ===================================================================

#[derive(Clone)]
struct Counter {
    value: u8,
    max_value: u8,
}

impl Counter {
    fn new(value: u8, max_value: u8) -> Self {
        assert!(value <= max_value);
        Self { value, max_value }
    }

    /// Reusable mechanism: increment with wrap-around.
    fn increment(&self) -> Self {
        if self.value >= self.max_value {
            Self { value: 0, ..*self }
        } else {
            Self {
                value: self.value + 1,
                ..*self
            }
        }
    }

    /// Reusable mechanism: reset to zero.
    fn reset(&self) -> Self {
        Self { value: 0, ..*self }
    }
}

impl State for Counter {
    type Encoding = Vec<u8>;
    fn canonical_encoding(&self) -> Self::Encoding {
        vec![self.value, self.max_value]
    }
    fn distance(&self, other: &Self) -> u32 {
        let mut diff = 0u32;
        if self.value != other.value {
            diff += 1;
        }
        if self.max_value != other.max_value {
            diff += 1;
        }
        diff
    }
}

impl PartialEq for Counter {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.max_value == other.max_value
    }
}
impl Eq for Counter {}
impl Hash for Counter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.max_value.hash(state);
    }
}
impl fmt::Debug for Counter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Counter({}/{})", self.value, self.max_value)
    }
}
impl fmt::Display for Counter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// ===================================================================
// Rule 1: Increment — always fires, needs no extra context
// ===================================================================

/// A deterministic rule: always increments the counter.
///
/// Uses `NoContext` because it doesn't need any extra information
/// beyond the state itself. The RNG is ignored (deterministic).
#[derive(Debug, Clone)]
struct IncrementRule;

impl Rule<Counter> for IncrementRule {
    /// NoContext means this rule doesn't need extra information.
    type Context = NoContext;

    fn name(&self) -> &str {
        "Increment"
    }

    fn apply(
        &self,
        state: &Counter,
        _context: &NoContext,
        _rng: &mut dyn Rng, // ignored — this rule is deterministic
    ) -> Counter {
        // Policy: always increment. Mechanism: provided by state.
        state.increment()
    }
}

// ===================================================================
// Rule 2: Set — needs a target value passed via context
// ===================================================================

/// Context for SetRule: carries the target value to set.
///
/// Custom context types must implement the `RuleContext` trait.
#[derive(Debug, Clone)]
struct SetValue {
    target: u8,
}

impl RuleContext for SetValue {}

/// A deterministic rule: sets the counter to a specific value.
///
/// The target value comes from the `SetValue` context.
/// The schedule creates the context — you never construct one manually.
#[derive(Debug, Clone)]
struct SetRule;

impl Rule<Counter> for SetRule {
    type Context = SetValue;

    fn name(&self) -> &str {
        "Set"
    }

    fn apply(&self, state: &Counter, context: &SetValue, _rng: &mut dyn Rng) -> Counter {
        // Clamp to max_value, then set
        Counter::new(context.target.min(state.max_value), state.max_value)
    }
}

// ===================================================================
// Rule 3: MaybeReset — stochastic, uses the RNG
// ===================================================================

/// A stochastic rule: increments 70% of the time, resets 30%.
///
/// Uses the RNG to decide. Different runs with the same initial
/// state may produce different outcomes — the ensemble approach
/// in ARCO's metrics averages over this randomness.
#[derive(Debug, Clone)]
struct MaybeResetRule;

impl Rule<Counter> for MaybeResetRule {
    type Context = NoContext;

    fn name(&self) -> &str {
        "MaybeReset"
    }

    fn apply(
        &self,
        state: &Counter,
        _context: &NoContext,
        rng: &mut dyn Rng, // used — this rule is stochastic
    ) -> Counter {
        // Policy: 70% increment, 30% reset. Mechanisms: provided by state.
        if rng.random_bool(0.7) {
            state.increment()
        } else {
            state.reset()
        }
    }
}

// ===================================================================
// Test our rules
// ===================================================================

fn main() {
    println!("=== Example 02: Custom Rules ===\n");

    let state = Counter::new(0, 5);
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    // Deterministic rule with NoContext
    let inc = IncrementRule;
    let ctx = NoContext;
    let s1 = inc.apply(&state, &ctx, &mut rng);
    println!("After Increment: {} (was {})", s1, state);

    // Rule with custom context
    let set = SetRule;
    let ctx = SetValue { target: 3 };
    let s2 = set.apply(&state, &ctx, &mut rng);
    println!("After Set(3):   {} (was {})", s2, state);

    // Stochastic rule — run 10 times to see the distribution
    let maybe = MaybeResetRule;
    print!("\nMaybeReset 10 times: ");
    let mut current = state.clone();
    for _ in 0..10 {
        current = maybe.apply(&current, &NoContext, &mut rng);
        print!("{} ", current);
    }
    println!();

    // Immutability check — original state unchanged
    println!("\nOriginal state unchanged: {}", state);

    println!("\n✓ Three rules implemented:");
    println!("  - IncrementRule: deterministic, NoContext");
    println!("  - SetRule: deterministic, custom SetValue context");
    println!("  - MaybeResetRule: stochastic, NoContext");
}
