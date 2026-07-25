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
//! receives a slice of rules and returns a boolean. This keeps
//! hypothesis testing substrate-independent — a hypothesis written
//! for graph rewriting rules can be applied to cellular automaton
//! rules without modification, as long as the rule type implements
//! the necessary trait.
//!
//! # Standard hypotheses
//!
//! Substrate-specific hypothesis sets (like the Binary Graph
//! Transport Law) live in their substrate modules. The core
//! hypothesis infrastructure is substrate-independent.

use crate::types::{ConditionPredicate, MetricFn};

// ===================================================================
// Hypothesis
// ===================================================================

/// A falsifiable hypothesis about emergent properties.
///
/// A hypothesis states that rule sets satisfying a structural
/// condition will exhibit a specified emergent property above
/// a calibrated threshold.
///
/// # Type parameters
///
/// - `R`: The rule type. The condition predicate receives `&[R]`.
///
/// # Fields
///
/// - `name`: Unique identifier (e.g., "H5_TRANSPORT").
/// - `condition_fn`: Structural predicate on rule sets.
/// - `property_name`: `"persistence"`, `"storage"`, or `"memory"`.
/// - `condition_desc`: Human-readable description.
/// - `complexity`: MDL penalty weight.
/// - `accuracy`: Set by `test()`. Fraction of held-out universes
///   where the prediction held.
/// - `score`: Set by `test()`. Accuracy minus complexity penalty.
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
    /// * `complexity` — Penalty weight (higher = more complex condition).
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

    /// Evaluate this hypothesis on held-out data.
    ///
    /// For each test universe satisfying the condition, computes
    /// the predicted metric and checks whether it exceeds the
    /// calibrated threshold.
    ///
    /// # Parameters
    /// * `test_data` — Held-out data: `(rules, trajectories)` pairs.
    /// * `metric_fn` — Function computing the predicted metric.
    /// * `threshold` — Calibrated emergence threshold.
    ///
    /// # Returns
    /// Accuracy: fraction of qualifying universes where the
    /// prediction held.
    ///
    /// Sets `self.accuracy` and `self.score` as side effects.
    /// Score = accuracy - λ × complexity, with λ = 0.1.
    pub fn test<T>(
        &mut self,
        test_data: &[(&[R], &[Vec<T>])],
        metric_fn: &MetricFn<T>,
        threshold: f64,
    ) -> f64 {
        let mut positive_condition = 0usize;
        let mut correct_predictions = 0usize;

        for (rules, trajectories) in test_data {
            if (self.condition_fn)(rules) {
                positive_condition += 1;
                let metric_value = metric_fn(trajectories);
                if metric_value > threshold {
                    correct_predictions += 1;
                }
            }
        }

        self.accuracy = if positive_condition > 0 {
            correct_predictions as f64 / positive_condition as f64
        } else {
            0.0
        };

        let lambda = 0.1;
        self.score = self.accuracy - lambda * self.complexity;

        self.accuracy
    }

    /// Whether this hypothesis survives the complexity penalty.
    ///
    /// A hypothesis survives if Score > 0 and Accuracy ≥ 0.5.
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
