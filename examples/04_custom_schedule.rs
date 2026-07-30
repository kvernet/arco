//! Example 04 — Implement a custom Schedule.
//!
//! The `Schedule` trait requires:
//! - `name()`, `timing()`, `selection()` — metadata
//! - `step()` — apply rules to a state for one timestep
//!
//! We implement two schedules for our Counter:
//! - SequentialSchedule: applies every rule in order
//! - RandomRuleSchedule: picks one rule at random per timestep
//!
//! These are the same generic schedules shipped with ARCO.
//! We reimplement them here to show how the trait works.
//!
//! # Design principle
//!
//! The schedule is the **temporal structure** — it determines the
//! order and concurrency of rule application. Two universes
//! differing only in schedule are distinct objects of study.
//! The schedule creates rule contexts internally; users never
//! construct contexts manually.

use arco::rules::{NoContext, Rule};
use arco::schedule::Schedule;
use arco::state::State;
use rand::{Rng, RngExt, SeedableRng};
use std::fmt;
use std::hash::{Hash, Hasher};

// ===================================================================
// Our state (same as examples 01–03)
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
// Our rules (same as example 02, simplified)
// ===================================================================

#[derive(Debug, Clone)]
struct IncrementRule;

impl Rule<Counter> for IncrementRule {
    type Context = NoContext;
    fn name(&self) -> &str {
        "Increment"
    }
    fn apply(&self, state: &Counter, _ctx: &NoContext, _rng: &mut dyn Rng) -> Counter {
        state.increment()
    }
}

// Note: ResetRule is defined but not used by the schedules in this example.
// Real substrates use an enum to mix rule types (see example 05).
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ResetRule;

impl Rule<Counter> for ResetRule {
    type Context = NoContext;
    fn name(&self) -> &str {
        "Reset"
    }
    fn apply(&self, state: &Counter, _ctx: &NoContext, _rng: &mut dyn Rng) -> Counter {
        state.reset()
    }
}

// ===================================================================
// Schedule 1: Sequential — apply every rule in order
// ===================================================================

/// Applies all rules in the order they appear in the rule set.
///
/// Each rule sees the result of the previous rule within the
/// same timestep (asynchronous). This is the simplest schedule
/// and works with any rule type.
///
/// Semantics:
/// - Timing: asynchronous
/// - Selection: exhaustive
#[derive(Debug, Clone, Default)]
struct SequentialSchedule;

impl Schedule<Counter, IncrementRule> for SequentialSchedule {
    fn name(&self) -> &str {
        "sequential"
    }
    fn timing(&self) -> &str {
        "asynchronous"
    }
    fn selection(&self) -> &str {
        "exhaustive"
    }

    fn step(&self, state: &Counter, rules: &[IncrementRule], rng: &mut dyn Rng) -> Counter {
        let mut current = state.clone();
        let ctx = NoContext;
        for rule in rules {
            current = rule.apply(&current, &ctx, rng);
        }
        current
    }
}

// ===================================================================
// Schedule 2: Random — pick one rule per timestep
// ===================================================================

/// Selects one rule uniformly at random and applies it.
///
/// Only one rule fires per timestep. The same rule may be
/// selected across multiple timesteps — each timestep is
/// an independent random draw.
///
/// Semantics:
/// - Timing: asynchronous (single update)
/// - Selection: stochastic
#[derive(Debug, Clone, Default)]
struct RandomRuleSchedule;

impl Schedule<Counter, IncrementRule> for RandomRuleSchedule {
    fn name(&self) -> &str {
        "random_rule"
    }
    fn timing(&self) -> &str {
        "asynchronous"
    }
    fn selection(&self) -> &str {
        "stochastic"
    }

    fn step(&self, state: &Counter, rules: &[IncrementRule], rng: &mut dyn Rng) -> Counter {
        if rules.is_empty() {
            return state.clone();
        }
        let idx = rng.random_range(0..rules.len());
        let ctx = NoContext;
        rules[idx].apply(state, &ctx, rng)
    }
}

// ===================================================================
// Test our schedules
// ===================================================================

fn main() {
    println!("=== Example 04: Custom Schedules ===\n");

    let state = Counter::new(0, 5);
    let rules = vec![IncrementRule, IncrementRule, IncrementRule];
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    // Sequential schedule: applies all three increment rules
    let seq = SequentialSchedule;
    let s1 = seq.step(&state, &rules, &mut rng);
    println!("Sequential schedule (3 Increment rules):");
    println!("  Start: {}", state);
    println!("  After 1 step: {} (incremented 3 times)", s1);

    // Random schedule: picks one rule per timestep
    let rand_sched = RandomRuleSchedule;
    print!("\nRandom schedule (10 timesteps): ");
    let mut current = state.clone();
    for _ in 0..10 {
        current = rand_sched.step(&current, &rules, &mut rng);
        print!("{} ", current);
    }
    println!();
    println!("  (each step applies one random increment)");

    // Schedule metadata
    println!("\nSchedule metadata:");
    println!(
        "  Sequential: {} ({}, {})",
        seq.name(),
        seq.timing(),
        seq.selection()
    );
    println!(
        "  Random:     {} ({}, {})",
        rand_sched.name(),
        rand_sched.timing(),
        rand_sched.selection()
    );

    println!("\n✓ Two schedules implemented:");
    println!("  - SequentialSchedule: applies every rule, in order");
    println!("  - RandomRuleSchedule: picks one rule at random");
}
