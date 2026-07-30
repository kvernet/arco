//! Example 05 — Bundle everything into an InformationUniverse.
//!
//! The `InformationUniverse` trait binds state space, rules,
//! observation, and schedule into a single type. The scientific
//! cycle (`run_cycle`) operates on any type implementing this trait.
//!
//! We build a complete Counter universe with:
//! - Two rule types (Increment, Reset) via an enum
//! - Sequential schedule
//! - Full state observer
//! - Rule generation and null rules for calibration
//!
//! # Design principle
//!
//! The universe is the **complete experimental system**. It provides
//! everything the cycle needs: state space for sampling, rules for
//! evolution, an observer for perception, and a schedule for time.
//! The cycle doesn't know about Counters — it only knows the trait.

use arco::observation::Observation;
use arco::rules::{NoContext, Rule};
use arco::schedule::Schedule;
use arco::state::State;
use arco::universe::InformationUniverse;
use rand::{Rng, RngExt, SeedableRng};
use std::fmt;
use std::hash::{Hash, Hasher};

// ===================================================================
// Our state
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
// Rule enum — mixes multiple rule types in one Vec
// ===================================================================

/// Enum that wraps all rule types so they can live in the same Vec.
/// This is the standard pattern for substrates with multiple rule types.
#[derive(Debug, Clone)]
enum CounterRule {
    Increment(IncrementRule),
    Reset(ResetRule),
}

impl Rule<Counter> for CounterRule {
    type Context = NoContext;

    fn name(&self) -> &str {
        match self {
            CounterRule::Increment(r) => r.name(),
            CounterRule::Reset(r) => r.name(),
        }
    }

    fn apply(&self, state: &Counter, context: &NoContext, rng: &mut dyn Rng) -> Counter {
        match self {
            CounterRule::Increment(r) => r.apply(state, context, rng),
            CounterRule::Reset(r) => r.apply(state, context, rng),
        }
    }
}

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
// Observer
// ===================================================================

#[derive(Debug, Clone, Default)]
struct FullObserver;

impl Observation<Counter> for FullObserver {
    type Output = Vec<u8>;
    fn observe(&self, state: &Counter) -> Self::Output {
        state.canonical_encoding()
    }
}

// ===================================================================
// Schedule — applies all rules in sequence
// ===================================================================

#[derive(Debug, Clone, Default)]
struct AllRulesSchedule;

impl Schedule<Counter, CounterRule> for AllRulesSchedule {
    fn name(&self) -> &str {
        "all_rules"
    }
    fn timing(&self) -> &str {
        "asynchronous"
    }
    fn selection(&self) -> &str {
        "exhaustive"
    }

    fn step(&self, state: &Counter, rules: &[CounterRule], rng: &mut dyn Rng) -> Counter {
        let mut current = state.clone();
        let ctx = NoContext;
        for rule in rules {
            current = rule.apply(&current, &ctx, rng);
        }
        current
    }
}

// ===================================================================
// The Counter Universe
// ===================================================================

/// A complete Information Universe for our Counter system.
///
/// Provides state space, rule generation, observation, and schedule.
/// This is what you pass to `run_cycle`.
#[derive(Debug, Clone)]
struct CounterUniverse {
    states: Vec<Counter>,
}

impl CounterUniverse {
    fn new(rng: &mut impl Rng) -> Self {
        let states: Vec<Counter> = (0..100)
            .map(|_| Counter::new(rng.random_range(0..=5), 5))
            .collect();
        Self { states }
    }
}

impl InformationUniverse for CounterUniverse {
    type State = Counter;
    type Rule = CounterRule;
    type Observation = FullObserver;
    type Schedule = AllRulesSchedule;

    fn state_space(&self) -> &[Self::State] {
        &self.states
    }

    fn observation(&self) -> &Self::Observation {
        // Return a static reference since FullObserver has no state.
        // In a real application, store the observer in the universe struct.
        static OBSERVER: FullObserver = FullObserver;
        &OBSERVER
    }

    fn schedule(&self) -> &Self::Schedule {
        static SCHEDULE: AllRulesSchedule = AllRulesSchedule;
        &SCHEDULE
    }

    fn generate_rules(&self, rng: &mut dyn Rng) -> (Vec<Self::Rule>, f64) {
        // Generate a random mix of Increment and Reset rules.
        // structured_ratio: fraction of Increment rules (non-destructive).
        let n = rng.random_range(1..=4);
        let n_inc = rng.random_range(0..=n);
        let mut rules: Vec<CounterRule> = Vec::with_capacity(n);

        for _ in 0..n_inc {
            rules.push(CounterRule::Increment(IncrementRule));
        }
        for _ in 0..(n - n_inc) {
            rules.push(CounterRule::Reset(ResetRule));
        }

        let ratio = if n > 0 { n_inc as f64 / n as f64 } else { 0.0 };
        (rules, ratio)
    }

    fn null_rules(&self, _rng: &mut dyn Rng) -> Vec<Self::Rule> {
        // Destructive null: only Reset rules (destroy information by
        // resetting to zero).
        vec![CounterRule::Reset(ResetRule)]
    }
}

// ===================================================================
// Test the universe
// ===================================================================

fn main() {
    println!("=== Example 05: Custom InformationUniverse ===\n");

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let universe = CounterUniverse::new(&mut rng);

    println!("State space: {} states", universe.state_space().len());
    println!("Observer:    FullObserver (sees value + max_value)");
    println!(
        "Schedule:    {} ({}, {})",
        universe.schedule().name(),
        universe.schedule().timing(),
        universe.schedule().selection(),
    );

    // Generate some rule sets
    println!("\nGenerated rule sets:");
    for i in 0..5 {
        let (rules, ratio) = universe.generate_rules(&mut rng);
        let names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
        println!("  {}: {:?} (structured ratio: {:.2})", i + 1, names, ratio);
    }

    // Null rules
    let null = universe.null_rules(&mut rng);
    let null_names: Vec<&str> = null.iter().map(|r| r.name()).collect();
    println!("\nNull rules: {:?}", null_names);

    println!("\n✓ CounterUniverse implements InformationUniverse.");
    println!("  It provides everything the scientific cycle needs.");
}
