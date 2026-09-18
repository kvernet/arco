//! Example 06 — Run the scientific cycle on a custom universe.
//!
//! This is the final step: take the CounterUniverse from example 05,
//! write structural hypotheses, and run the full ARCO pipeline.
//!
//! After this example, you can build a complete custom substrate
//! and use all of ARCO's metrics, calibration, and hypothesis
//! testing on it.

use arco::cycle::{CycleConfig, run_cycle};
use arco::hypotheses::Hypothesis;
use arco::invariants::Invariant;
use arco::observation::Observation;
use arco::resources::Resources;
use arco::rules::{NoContext, Rule};
use arco::schedule::Schedule;
use arco::state::State;
use arco::universe::InformationUniverse;
use rand::{Rng, RngExt, SeedableRng};
use std::fmt;
use std::hash::{Hash, Hasher};

// ===================================================================
// Our state, rules, observer, schedule, and universe
// (same as example 05, condensed)
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

#[derive(Debug, Clone)]
enum CounterRule {
    Increment,
    Reset,
}

impl Rule<Counter> for CounterRule {
    type Context = NoContext;
    fn name(&self) -> &str {
        match self {
            CounterRule::Increment => "Increment",
            CounterRule::Reset => "Reset",
        }
    }
    fn apply(&self, state: &Counter, _ctx: &NoContext, _rng: &mut dyn Rng) -> Counter {
        match self {
            CounterRule::Increment => state.increment(),
            CounterRule::Reset => state.reset(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct FullObserver;
impl Observation<Counter> for FullObserver {
    type Output = Vec<u8>;
    fn observe(&self, state: &Counter) -> Self::Output {
        state.canonical_encoding()
    }
}

// ===================================================================
// Resources — R_time, R_space, R_local
// ===================================================================

/// Resource accounting for the Counter universe.
///
/// - Space: 2 bytes (`value` + `max_value`, matching the canonical
///   encoding).
/// - Time: 1.0 per rule application.
/// - Locality: 0.0 — both rules use `NoContext` and act on the whole
///   state at once, so there's no notion of a bounded interaction
///   radius here.
#[derive(Debug, Clone, Copy, Default)]
struct CounterResources;

impl Resources<Counter, CounterRule> for CounterResources {
    fn name(&self) -> &str {
        "counter_resources"
    }
    fn space(&self, _state: &Counter) -> f64 {
        2.0
    }
    fn time(&self, _rule: &CounterRule) -> f64 {
        1.0
    }
    fn locality(&self, _rule: &CounterRule) -> f64 {
        0.0
    }
}

// ===================================================================
// Invariants — I: S -> R
// ===================================================================

/// `I(s) = max_value`.
///
/// Genuinely conserved here: `IncrementRule::apply` wraps back to 0
/// at `max_value` but never changes `max_value` itself, and
/// `ResetRule::apply` only zeroes `value`. Neither rule can violate
/// this invariant, which is exactly the property `is_conserved` (via
/// [`Invariant`]'s default tolerance-based check) is for.
#[derive(Debug, Clone, Copy, Default)]
struct MaxValueInvariant;

impl Invariant<Counter> for MaxValueInvariant {
    fn name(&self) -> &str {
        "max_value"
    }
    fn evaluate(&self, state: &Counter) -> f64 {
        state.max_value as f64
    }
}

/// The standard invariant set for the Counter universe.
fn standard_counter_invariants() -> Vec<Box<dyn Invariant<Counter>>> {
    vec![Box::new(MaxValueInvariant)]
}

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

#[derive(Debug)]
struct CounterUniverse {
    states: Vec<Counter>,
    invariants: Vec<Box<dyn Invariant<Counter>>>,
}

impl Clone for CounterUniverse {
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            // Box<dyn Invariant<_>> isn't Clone; the standard set is
            // stateless, so it's cheap to regenerate.
            invariants: standard_counter_invariants(),
        }
    }
}

impl CounterUniverse {
    fn new(rng: &mut impl Rng) -> Self {
        let states: Vec<Counter> = (0..100)
            .map(|_| Counter::new(rng.random_range(0..=5), 5))
            .collect();
        Self {
            states,
            invariants: standard_counter_invariants(),
        }
    }
}

impl InformationUniverse for CounterUniverse {
    type State = Counter;
    type Rule = CounterRule;
    type Observation = FullObserver;
    type Resources = CounterResources;
    type Schedule = AllRulesSchedule;

    fn state_space(&self) -> &[Self::State] {
        &self.states
    }

    fn observation(&self) -> &Self::Observation {
        static OBS: FullObserver = FullObserver;
        &OBS
    }

    fn resources(&self) -> &Self::Resources {
        // Same reasoning as `observation()`: CounterResources has no
        // state, so a static reference avoids a per-universe field.
        static RESOURCES: CounterResources = CounterResources;
        &RESOURCES
    }

    fn invariants(&self) -> &[Box<dyn Invariant<Self::State>>] {
        &self.invariants
    }

    fn schedule(&self) -> &Self::Schedule {
        static SCHED: AllRulesSchedule = AllRulesSchedule;
        &SCHED
    }

    fn generate_rules(&self, rng: &mut dyn Rng) -> (Vec<Self::Rule>, f64) {
        let n = rng.random_range(1..=4);
        let n_inc = rng.random_range(0..=n);
        let mut rules = Vec::with_capacity(n);
        for _ in 0..n_inc {
            rules.push(CounterRule::Increment);
        }
        for _ in 0..(n - n_inc) {
            rules.push(CounterRule::Reset);
        }
        let ratio = if n > 0 { n_inc as f64 / n as f64 } else { 0.0 };
        (rules, ratio)
    }

    fn null_rules(&self, _rng: &mut dyn Rng) -> Vec<Self::Rule> {
        vec![CounterRule::Reset]
    }
}

// ===================================================================
// Main: run the scientific cycle
// ===================================================================

fn main() {
    println!("=== Example 06: Run the Scientific Cycle ===\n");

    // Create the universe
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let universe = CounterUniverse::new(&mut rng);

    // Write hypotheses based on measurable properties of rule sets
    let mut hypotheses: Vec<Hypothesis<CounterRule>> = vec![
        Hypothesis::new(
            "H_INCREMENT",
            |rules: &[CounterRule]| {
                // Condition: rule set contains at least one Increment
                rules.iter().any(|r| matches!(r, CounterRule::Increment))
            },
            "storage",
            "Contains Increment rule",
            1.0,
        ),
        Hypothesis::new(
            "H_ALL_RESET",
            |rules: &[CounterRule]| {
                // Negative control: all-Reset should NOT produce storage
                !rules.is_empty() && rules.iter().all(|r| matches!(r, CounterRule::Reset))
            },
            "storage",
            "All rules are Reset (negative control)",
            0.5,
        ),
    ];

    // Configure the cycle — use small numbers for a fast demo
    let config = CycleConfig {
        n_train: 50,
        n_test: 15,
        n_ensemble: 5,
        steps: 20,
        seed: 42,
        ..CycleConfig::default()
    };

    // Run the cycle
    println!(
        "Running cycle with {} training universes...\n",
        config.n_train
    );
    let record = run_cycle(&universe, &config, &mut hypotheses, None);

    // Print results
    println!("{}", record.summary());

    // Inspect individual hypotheses
    println!("\nHypothesis details:");
    for h in &record.hypotheses {
        println!(
            "  {}: {} (acc={:.3}, score={:.3}, survives={})",
            h.name,
            h.condition_desc,
            h.classification_metrics.balanced_accuracy,
            h.classification_metrics.score,
            h.survives
        );
    }

    println!("\n✓ Full scientific cycle completed on a custom universe.");
    println!("  The cycle calibrated thresholds, tested hypotheses,");
    println!("  and produced a research record — all without knowing");
    println!("  anything about Counters.");
}
