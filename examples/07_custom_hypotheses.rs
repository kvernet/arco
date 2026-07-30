//! Example 07 — Advanced hypothesis patterns.
//!
//! This example shows how to write more sophisticated hypotheses:
//! - Multi-clause conditions (AND, OR combinations)
//! - Conditions based on measurable rule properties
//! - Complexity penalties for different condition types
//! - Interpreting why hypotheses survive or fail
//!
//! We use the same CounterUniverse from examples 05–06.

use arco::cycle::{CycleConfig, run_cycle};
use arco::hypotheses::Hypothesis;
use arco::observation::Observation;
use arco::rules::{NoContext, Rule};
use arco::schedule::Schedule;
use arco::state::State;
use arco::universe::InformationUniverse;
use rand::{Rng, RngExt, SeedableRng};
use std::fmt;
use std::hash::{Hash, Hasher};

// ===================================================================
// Same Counter universe as examples 05–06 (condensed)
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
        for rule in rules {
            current = rule.apply(&current, &NoContext, rng);
        }
        current
    }
}

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
        static O: FullObserver = FullObserver;
        &O
    }
    fn schedule(&self) -> &Self::Schedule {
        static S: AllRulesSchedule = AllRulesSchedule;
        &S
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
// Advanced hypotheses
// ===================================================================

fn main() {
    println!("=== Example 07: Advanced Hypothesis Patterns ===\n");

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let universe = CounterUniverse::new(&mut rng);

    // ── Simple condition (complexity 0.5–1.0) ──────────────────
    // One clause, easy to interpret.
    let h_simple = Hypothesis::new(
        "H_SIMPLE",
        |rules: &[CounterRule]| rules.len() >= 2,
        "storage",
        "Rule set has at least 2 rules",
        0.5,
    );

    // ── Compound AND (complexity 1.5–2.0) ──────────────────────
    // Multiple clauses that must ALL be true. Higher complexity
    // means it needs higher accuracy to survive the penalty.
    let h_compound = Hypothesis::new(
        "H_COMPOUND",
        |rules: &[CounterRule]| {
            let has_inc = rules.iter().any(|r| matches!(r, CounterRule::Increment));
            let has_reset = rules.iter().any(|r| matches!(r, CounterRule::Reset));
            has_inc && has_reset && rules.len() >= 3
        },
        "storage",
        "Has both Increment AND Reset, AND at least 3 rules",
        2.0,
    );

    // ── Measurable property (complexity 1.0) ───────────────────
    // Condition based on counting, not on specific names.
    let h_majority = Hypothesis::new(
        "H_MAJORITY",
        |rules: &[CounterRule]| {
            if rules.is_empty() {
                return false;
            }
            let inc_count = rules
                .iter()
                .filter(|r| matches!(r, CounterRule::Increment))
                .count();
            inc_count as f64 / rules.len() as f64 >= 0.5
        },
        "storage",
        "Majority of rules are Increment",
        1.0,
    );

    // ── Negative control (complexity 0.5) ──────────────────────
    // Should always fail. If it survives, calibration is broken.
    let h_control = Hypothesis::new(
        "H_NEGATIVE_CONTROL",
        |rules: &[CounterRule]| {
            !rules.is_empty() && rules.iter().all(|r| matches!(r, CounterRule::Reset))
        },
        "storage",
        "All rules are Reset (expect FAIL)",
        0.5,
    );

    // ── Overly specific (complexity 3.0) ────────────────────────
    // Very high complexity. Would need ~80%+ accuracy to survive.
    // This hypothesis is probably overfitting — it memorizes a
    // specific combination rather than capturing a general principle.
    let h_overfit = Hypothesis::new(
        "H_OVERFIT",
        |rules: &[CounterRule]| {
            rules.len() == 3
                && matches!(rules[0], CounterRule::Increment)
                && matches!(rules[1], CounterRule::Reset)
                && matches!(rules[2], CounterRule::Increment)
        },
        "storage",
        "Exact sequence: Increment, Reset, Increment (overfit)",
        3.0,
    );

    let mut hypotheses = vec![h_simple, h_compound, h_majority, h_control, h_overfit];

    let config = CycleConfig {
        n_train: 200,
        n_test: 50,
        n_ensemble: 10,
        steps: 60,
        seed: 42,
        ..CycleConfig::default()
    };

    println!("Running cycle...\n");
    let record = run_cycle(&universe, &config, &mut hypotheses, None);

    // ── Analyze results ────────────────────────────────────────
    println!("Hypothesis Analysis:");
    println!(
        "{:<22} {:<12} {:<8} {:<10} {}",
        "Name", "Accuracy", "Score", "Survives?", "Why?"
    );
    println!("{}", "-".repeat(75));

    for h in &record.hypotheses {
        let diagnosis = if !h.survives && h.complexity >= 3.0 {
            "Overfit — complexity penalty too high"
        } else if !h.survives && h.accuracy < 0.5 {
            "Condition doesn't predict storage"
        } else if !h.survives && h.score <= 0.0 {
            "Accuracy too low for this complexity"
        } else if h.survives && h.complexity <= 1.0 {
            "Simple condition, good accuracy"
        } else if h.survives {
            "Survives despite complexity"
        } else {
            ""
        };

        println!(
            "{:<22} {:5.1} %      {:<8.3} {:<10} {}",
            h.name,
            h.accuracy * 100.0,
            h.score,
            if h.survives { "✓ YES" } else { "✗ NO" },
            diagnosis,
        );
    }

    // ── Key takeaways ──────────────────────────────────────────
    println!("\n─── Key Takeaways ───");
    println!("• Simple conditions (≤1.0 complexity) are easier to survive.");
    println!("• Compound conditions need higher accuracy to offset the penalty.");
    println!("• Negative controls SHOULD fail — their failure validates calibration.");
    println!("• Overly specific conditions fail due to the complexity penalty.");
    println!("• Base conditions on measurable properties, not exact sequences.");
    println!("• A hypothesis that survives is a structural regularity,");
    println!("  not necessarily a causal law.");

    println!("\n✓ Five hypotheses tested with different complexity levels.");
    println!("  This is the last example in the series.");
    println!("  You can now build custom substrates and write your own hypotheses.");
}
