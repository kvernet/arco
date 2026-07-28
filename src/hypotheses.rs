//! Hypothesis generation, testing, and scoring.
//!
//! Per the Mathematical Constitution:
//!     Hypotheses are formal statements of the form "Condition A ∧
//!     Condition B ⇒ Emergent Property C". They are tested on
//!     held-out universes and scored using accuracy minus a
//!     complexity penalty.
//!
//! # Design
//!
//! Hypotheses are generic over any rule type. The condition function
//! receives a slice of rules and returns a boolean. Testing is
//! performed by the scientific cycle ([`run_cycle`]), which evaluates
//! each hypothesis against held-out test universes and computes
//! accuracy and MDL-penalized scores.
//!
//! # Standard hypotheses
//!
//! Substrate-specific hypothesis sets live in their substrate modules.
//! The core hypothesis infrastructure is substrate-independent.
//!
//! # Quick start
//!
//! ```rust
//! use arco::state::State;
//! use arco::rules::{Rule, NoContext};
//! use arco::hypotheses::Hypothesis;
//! use rand::Rng;
//!
//! #[derive(Clone, PartialEq, Eq, Hash, Debug)]
//! struct MyState { value: u8 }
//!
//! impl State for MyState {
//!     type Encoding = Vec<u8>;
//!     fn canonical_encoding(&self) -> Self::Encoding { vec![self.value] }
//!     fn distance(&self, other: &Self) -> u32 {
//!         if self.value == other.value { 0 } else { 1 }
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! struct MyRule { name: String }
//!
//! impl Rule<MyState> for MyRule {
//!     type Context = NoContext;
//!     fn name(&self) -> &str { &self.name }
//!     fn apply(&self, state: &MyState, _ctx: &NoContext, _rng: &mut dyn Rng) -> MyState {
//!         MyState { value: 1 - state.value }
//!     }
//! }
//!
//! // Create a hypothesis: rule sets with ≥2 rules → storage
//! let h: Hypothesis<MyRule> = Hypothesis::new(
//!     "H_MIN_SIZE",
//!     |rules: &[MyRule]| rules.len() >= 2,
//!     "storage",
//!     "Rule set has at least 2 rules",
//!     1.0,
//! );
//!
//! // Hypotheses are tested by the scientific cycle.
//! // Use `run_cycle()` to evaluate them against held-out data.
//! // The cycle sets `accuracy` and `score` on each hypothesis.
//! assert_eq!(h.accuracy, 0.0); // not yet tested
//! assert_eq!(h.property_name, "storage");
//! ```

use crate::types::ConditionPredicate;

// ===================================================================
// Hypothesis
// ===================================================================

/// A falsifiable hypothesis about emergent properties.
///
/// A hypothesis states that rule sets satisfying a structural
/// condition will exhibit a specified emergent property above
/// a calibrated threshold. Hypotheses are tested by the scientific
/// cycle, which sets `accuracy` and `score` based on held-out data.
///
/// # Type parameters
///
/// - `R`: The rule type.
pub struct Hypothesis<R> {
    pub name: String,
    pub condition_fn: Box<ConditionPredicate<R>>,
    pub property_name: String,
    pub condition_desc: String,
    pub complexity: f64,
    pub accuracy: f64,
    pub score: f64,
}

impl<R> Hypothesis<R> {
    /// Create a new Hypothesis.
    ///
    /// # Parameters
    /// * `name` — Unique identifier.
    /// * `condition_fn` — Structural predicate `(&[R]) -> bool`.
    /// * `property_name` — `"persistence"`, `"storage"`, or `"memory"`.
    /// * `condition_desc` — Human-readable description.
    /// * `complexity` — MDL penalty weight.
    ///
    /// # Panics
    /// Panics if `property_name` is not one of the valid options.
    pub fn new(
        name: impl Into<String>,
        condition_fn: impl Fn(&[R]) -> bool + Send + Sync + 'static,
        property_name: impl Into<String>,
        condition_desc: impl Into<String>,
        complexity: f64,
    ) -> Self {
        let property_name = property_name.into();
        assert!(
            property_name == "persistence"
                || property_name == "storage"
                || property_name == "memory",
            "property_name must be 'persistence', 'storage', or 'memory'"
        );

        Self {
            name: name.into(),
            condition_fn: Box::new(condition_fn),
            property_name,
            condition_desc: condition_desc.into(),
            complexity,
            accuracy: 0.0,
            score: 0.0,
        }
    }

    /// Whether this hypothesis survives the complexity penalty.
    ///
    /// A hypothesis survives if Score > 0 and Accuracy ≥ 0.5.
    /// `accuracy` and `score` must be set by the scientific cycle
    /// before calling this method.
    pub fn survives(&self) -> bool {
        self.score > 0.0 && self.accuracy >= 0.5
    }
}

impl<R> std::fmt::Display for Hypothesis<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.survives() { "SURVIVES" } else { "FAILS" };
        write!(
            f,
            "{}: {} (acc={:.3}, score={:.3}, {})",
            self.name, self.condition_desc, self.accuracy, self.score, status
        )
    }
}

/// Return only hypotheses that survived the complexity penalty.
pub fn surviving_hypotheses<R>(hypotheses: &[Hypothesis<R>]) -> Vec<&Hypothesis<R>> {
    hypotheses.iter().filter(|h| h.survives()).collect()
}
