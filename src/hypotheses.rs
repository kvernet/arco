//! Hypothesis generation, testing, and scoring.
//!
//! Per Constitution:
//!     Hypotheses are formal statements of the form "Condition A ∧
//!     Condition B ⇒ Emergent Property C". They are tested on held-out
//!     universes and scored using accuracy minus a complexity penalty.
//!
//! This module provides:
//! - The `Hypothesis` struct: condition, prediction, scoring.
//! - Standard hypothesis generation for the Binary Graph Universe.
//! - Hypothesis testing on held-out rule subsets.
//!
//! Design commitments:
//! - Hypotheses are defined by structural predicates on rule subsets,
//!   not by raw metric values.
//! - The complexity penalty (MDL-inspired) prevents overfitting.
//! - A hypothesis survives if Score > 0 and Accuracy ≥ 0.5.

use std::collections::HashMap;

use crate::rules::{RewriteRule, Rule};

/// Structural condition predicate on rule subsets.
pub type ConditionPredicate = dyn Fn(&[RewriteRule]) -> bool + Send + Sync;

/// Metric function computing emergence scores from trajectories.
pub type MetricFn<O> = dyn Fn(&[Vec<O>]) -> f64;

// ===================================================================
// Hypothesis
// ===================================================================

/// A falsifiable hypothesis about emergent properties.
///
/// A hypothesis states that rule subsets satisfying a structural
/// condition will exhibit a specified emergent property above
/// calibrated threshold.
///
/// # Fields
/// * `name` — Unique identifier (e.g., "H5_TRANSPORT").
/// * `condition_fn` — Structural predicate on rule subsets.
/// * `property_name` — The emergent property predicted: `"persistence"`,
///   `"storage"`, or `"memory"`.
/// * `condition_desc` — Human-readable description.
/// * `complexity` — Description-length penalty weight.
/// * `accuracy` — Set by `test()`. Fraction of held-out universes
///   where the prediction held.
/// * `score` — Set by `test()`. Accuracy minus complexity penalty.
pub struct Hypothesis {
    pub name: String,
    pub condition_fn: Box<ConditionPredicate>,
    pub property_name: String,
    pub condition_desc: String,
    pub complexity: f64,
    pub accuracy: f64,
    pub score: f64,
}

impl Hypothesis {
    /// Create a new Hypothesis.
    ///
    /// # Parameters
    /// * `name` — Unique identifier.
    /// * `condition_fn` — Structural predicate `(rules) -> bool`.
    /// * `property_name` — `"persistence"`, `"storage"`, or `"memory"`.
    /// * `condition_desc` — Human-readable description.
    /// * `complexity` — Penalty weight (higher = more complex condition).
    pub fn new(
        name: impl Into<String>,
        condition_fn: impl Fn(&[RewriteRule]) -> bool + Send + Sync + 'static,
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
    /// For each test universe satisfying the condition, computes the
    /// predicted metric and checks whether it exceeds the calibrated
    /// threshold.
    ///
    /// # Parameters
    /// * `test_data` — Held-out data: `(rules, trajectories)` pairs.
    /// * `metric_fn` — Function computing the predicted metric from
    ///   trajectories.
    /// * `threshold` — Calibrated emergence threshold.
    ///
    /// # Returns
    /// Accuracy: fraction of condition-satisfying universes where the
    /// prediction held.
    ///
    /// Sets `self.accuracy` and `self.score` as side effects.
    /// Score = accuracy - λ × complexity, with λ = 0.1.
    pub fn test<O>(
        &mut self,
        test_data: &[(&[RewriteRule], &[Vec<O>])],
        metric_fn: &MetricFn<O>,
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

impl std::fmt::Display for Hypothesis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.survives() { "SURVIVES" } else { "FAILS" };
        write!(
            f,
            "{}: {} (acc={:.3}, score={:.3}, {})",
            self.name, self.condition_desc, self.accuracy, self.score, status
        )
    }
}

// ===================================================================
// Standard hypothesis generation
// ===================================================================

/// Generate the standard set of hypotheses tested in the Binary Graph.
///
/// These hypotheses test whether structural properties of rule subsets
/// predict emergent properties (persistence, storage, memory).
///
/// # Returns
/// The standard eight hypotheses.
///
/// # Notes
/// H5 (has_transport → storage) is the Transport Law — the first
/// ARCO-discovered law to survive cross-validation across multiple.
pub fn generate_standard_hypotheses() -> Vec<Hypothesis> {
    let mut hypotheses = Vec::new();

    // H1: Has structured rule → persistence
    hypotheses.push(Hypothesis::new(
        "H1_HAS_STRUCTURED",
        |rules: &[RewriteRule]| -> bool { rules.iter().any(|r| r.rule_type() == "structured") },
        "persistence",
        "Rule set contains at least one structured rule",
        1.0,
    ));

    // H2: Majority structured → memory
    hypotheses.push(Hypothesis::new(
        "H2_MAJORITY_STRUCTURED",
        |rules: &[RewriteRule]| -> bool {
            if rules.is_empty() {
                return false;
            }
            let n = rules
                .iter()
                .filter(|r| r.rule_type() == "structured")
                .count();
            n as f64 / rules.len() as f64 >= 0.5
        },
        "memory",
        "Majority of rules are structured",
        1.5,
    ));

    // H3: Has logic gate → memory
    hypotheses.push(Hypothesis::new(
        "H3_LOGIC_GATE",
        |rules: &[RewriteRule]| -> bool {
            let logic = ["NAND", "NOR", "AND", "OR", "XOR", "NOT"];
            rules.iter().any(|r| logic.contains(&r.name()))
        },
        "memory",
        "Rule set contains a logic gate",
        1.5,
    ));

    // H4: All structured → persistence
    hypotheses.push(Hypothesis::new(
        "H4_ALL_STRUCTURED",
        |rules: &[RewriteRule]| -> bool {
            !rules.is_empty() && rules.iter().all(|r| r.rule_type() == "structured")
        },
        "persistence",
        "All rules are structured",
        1.0,
    ));

    // H5: Has transport rule → storage (THE TRANSPORT LAW)
    hypotheses.push(Hypothesis::new(
        "H5_TRANSPORT",
        |rules: &[RewriteRule]| -> bool {
            let transport = ["PROPAGATE", "SWAP", "COPY_TO_OUT", "COPY_FROM_IN"];
            rules.iter().any(|r| transport.contains(&r.name()))
        },
        "storage",
        "Rule set contains an information transport rule",
        1.0,
    ));

    // H6: All destructive → persistence (negative control, expect FAIL)
    hypotheses.push(Hypothesis::new(
        "H6_ALL_DESTRUCTIVE",
        |rules: &[RewriteRule]| -> bool {
            !rules.is_empty() && rules.iter().all(|r| r.rule_type() == "destructive")
        },
        "persistence",
        "All rules are destructive (negative control)",
        0.5,
    ));

    // H7: Multiple logic gates → memory
    hypotheses.push(Hypothesis::new(
        "H7_MULTIPLE_LOGIC",
        |rules: &[RewriteRule]| -> bool {
            let logic = ["NAND", "NOR", "AND", "OR", "XOR", "NOT"];
            rules.iter().filter(|r| logic.contains(&r.name())).count() >= 2
        },
        "memory",
        "Rule set contains at least 2 logic gates",
        2.0,
    ));

    // H8: Mixed rules → persistence
    hypotheses.push(Hypothesis::new(
        "H8_MIXED",
        |rules: &[RewriteRule]| -> bool {
            if rules.is_empty() {
                return false;
            }
            let ratio = rules
                .iter()
                .filter(|r| r.rule_type() == "structured")
                .count() as f64
                / rules.len() as f64;
            ratio > 0.3 && ratio < 0.7
        },
        "persistence",
        "Mixed structured and destructive rules",
        2.0,
    ));

    hypotheses
}

// ===================================================================
// Batch hypothesis testing
// ===================================================================

/// Test a list of hypotheses on held-out data.
///
/// # Parameters
/// * `hypotheses` — Mutable slice of hypotheses to evaluate.
/// * `test_data` — Held-out data.
/// * `metric_map` — Mapping from property name to metric function.
/// * `thresholds` — Calibrated thresholds for each metric.
pub fn test_all_hypotheses<O>(
    hypotheses: &mut [Hypothesis],
    test_data: &[(&[RewriteRule], &[Vec<O>])],
    metric_map: &HashMap<String, Box<MetricFn<O>>>,
    thresholds: &HashMap<String, f64>,
) {
    for h in hypotheses.iter_mut() {
        if let (Some(metric_fn), Some(&threshold)) = (
            metric_map.get(&h.property_name),
            thresholds.get(&h.property_name),
        ) {
            h.test(test_data, metric_fn.as_ref(), threshold);
        }
    }
}

/// Return only hypotheses that survived the complexity penalty.
pub fn surviving_hypotheses(hypotheses: &[Hypothesis]) -> Vec<&Hypothesis> {
    hypotheses.iter().filter(|h| h.survives()).collect()
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_rules(structured: usize, destructive: usize) -> Vec<RewriteRule> {
        let all_structured = crate::rules::create_structured_rules();
        let all_destructive = crate::rules::create_destructive_rules();
        let mut rules = Vec::new();
        for i in 0..structured {
            rules.push(all_structured[i % all_structured.len()].clone());
        }
        for i in 0..destructive {
            rules.push(all_destructive[i % all_destructive.len()].clone());
        }
        rules
    }

    #[test]
    fn test_hypothesis_creation() {
        let h = Hypothesis::new(
            "TEST",
            |rules| !rules.is_empty(),
            "storage",
            "Test hypothesis",
            1.0,
        );
        assert_eq!(h.name, "TEST");
        assert_eq!(h.property_name, "storage");
        assert_eq!(h.complexity, 1.0);
        assert_eq!(h.accuracy, 0.0);
        assert_eq!(h.score, 0.0);
    }

    #[test]
    #[should_panic]
    fn test_invalid_property_name() {
        Hypothesis::new(
            "TEST",
            |rules| !rules.is_empty(),
            "invalid_property",
            "Test",
            1.0,
        );
    }

    #[test]
    fn test_hypothesis_scoring() {
        let mut h = Hypothesis::new("TEST", |rules| !rules.is_empty(), "storage", "Test", 1.0);

        // Create dummy test data where condition matches and metric exceeds threshold
        let rules = make_rules(2, 0);
        let trajectories: Vec<Vec<u8>> = vec![vec![0, 1], vec![1, 0]];
        let test_data = vec![(rules.as_slice(), trajectories.as_slice())];

        // Metric always returns 1.0 (above any threshold < 1.0)
        let metric_fn = |_: &[Vec<u8>]| -> f64 { 1.0 };

        h.test(&test_data, &metric_fn, 0.5);
        assert_eq!(h.accuracy, 1.0);
        assert!((h.score - 0.9).abs() < 0.01); // 1.0 - 0.1*1.0
        assert!(h.survives());
    }

    #[test]
    fn test_hypothesis_fails_when_below_threshold() {
        let mut h = Hypothesis::new("TEST", |rules| !rules.is_empty(), "storage", "Test", 1.0);

        let rules = make_rules(2, 0);
        let trajectories: Vec<Vec<u8>> = vec![vec![0, 1], vec![1, 0]];
        let test_data = vec![(rules.as_slice(), trajectories.as_slice())];

        // Metric returns 0.0 (below threshold)
        let metric_fn = |_: &[Vec<u8>]| -> f64 { 0.0 };

        h.test(&test_data, &metric_fn, 0.5);
        assert_eq!(h.accuracy, 0.0);
        assert!((h.score - (-0.1)).abs() < 0.01);
        assert!(!h.survives());
    }

    #[test]
    fn test_standard_hypotheses_count() {
        let hypotheses = generate_standard_hypotheses();
        assert_eq!(hypotheses.len(), 8);
    }

    #[test]
    fn test_h5_transport_detects_transport_rules() {
        let h = generate_standard_hypotheses()
            .into_iter()
            .find(|h| h.name == "H5_TRANSPORT")
            .unwrap();

        let all_structured = crate::rules::create_structured_rules();

        // Transport rules: explicitly select PROPAGATE
        let transport_rules: Vec<RewriteRule> = all_structured
            .iter()
            .filter(|r| r.name() == "PROPAGATE")
            .cloned()
            .collect();

        // Non-transport: explicitly select TOGGLE
        let non_transport_rules: Vec<RewriteRule> = all_structured
            .iter()
            .filter(|r| r.name() == "TOGGLE")
            .cloned()
            .collect();

        assert!((h.condition_fn)(&transport_rules));
        assert!(!(h.condition_fn)(&non_transport_rules));
    }

    #[test]
    fn test_h6_negative_control() {
        let h = generate_standard_hypotheses()
            .into_iter()
            .find(|h| h.name == "H6_ALL_DESTRUCTIVE")
            .unwrap();

        let destructive = make_rules(0, 3);
        let mixed = make_rules(1, 2);

        assert!((h.condition_fn)(&destructive));
        assert!(!(h.condition_fn)(&mixed));
    }

    #[test]
    fn test_surviving_hypotheses_filters_correctly() {
        let mut hypotheses = vec![
            Hypothesis::new("PASS", |_| true, "storage", "Passes", 0.5),
            Hypothesis::new("FAIL", |_| true, "storage", "Fails", 10.0), // high complexity
        ];

        let rules = make_rules(2, 0);
        let trajectories: Vec<Vec<u8>> = vec![vec![0, 1], vec![1, 0]];
        let test_data = vec![(rules.as_slice(), trajectories.as_slice())];
        let metric_fn = |_: &[Vec<u8>]| -> f64 { 1.0 };

        for h in hypotheses.iter_mut() {
            h.test(&test_data, &metric_fn, 0.5);
        }

        let surviving = surviving_hypotheses(&hypotheses);
        assert_eq!(surviving.len(), 1);
        assert_eq!(surviving[0].name, "PASS");
    }

    #[test]
    fn test_test_all_hypotheses() {
        let mut hypotheses = vec![Hypothesis::new("H_TEST", |_| true, "storage", "Test", 1.0)];

        let rules = make_rules(2, 0);
        let trajectories: Vec<Vec<u8>> = vec![vec![0, 1], vec![1, 0]];
        let test_data = vec![(rules.as_slice(), trajectories.as_slice())];

        let mut metric_map: HashMap<String, Box<dyn Fn(&[Vec<u8>]) -> f64>> = HashMap::new();
        metric_map.insert(
            "storage".to_string(),
            Box::new(|_: &[Vec<u8>]| -> f64 { 1.0 }),
        );

        let mut thresholds = HashMap::new();
        thresholds.insert("storage".to_string(), 0.5);

        test_all_hypotheses(&mut hypotheses, &test_data, &metric_map, &thresholds);

        assert!(hypotheses[0].survives());
    }
}
