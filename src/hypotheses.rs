//! Hypothesis generation, evaluation, and survival criteria.
//!
//! A [`Hypothesis`] expresses a falsifiable claim about an emergent
//! computational property. Its condition identifies the rule sets for which
//! the property is predicted to occur, while its survival criterion defines
//! what constitutes sufficient evidence for the hypothesis to survive.
//!
//! # Survival criteria
//!
//! Survival is deliberately configurable per hypothesis.
//!
//! The default criterion preserves the historical ARCO behaviour:
//!
//! ```text
//! balanced_accuracy >= 0.5 AND score > 0
//! ```
//!
//! Users can replace this criterion with one appropriate to their scientific
//! question. The criterion receives the complete [`ClassificationMetrics`]
//! produced by the hypothesis evaluation and the hypothesis complexity.
//!
//! This keeps three concerns separate:
//!
//! - the hypothesis defines what is being predicted;
//! - the scientific cycle produces classification evidence;
//! - the survival criterion decides whether that evidence is sufficient.
//!
//! # Examples
//!
//! Using the default criterion:
//!
//! ```
//! use arco::hypotheses::Hypothesis;
//!
//! let hypothesis: Hypothesis<u8> = Hypothesis::new(
//!     "H_MIN_SIZE",
//!     |rules: &[u8]| rules.len() >= 2,
//!     "storage",
//!     "Rule set has at least two rules",
//!     1.0,
//! );
//! ```
//!
//! Defining a custom criterion:
//!
//! ```
//! use arco::hypotheses::Hypothesis;
//!
//! let hypothesis: Hypothesis<u8> = Hypothesis::new(
//!     "H_STRONG",
//!     |rules: &[u8]| rules.len() >= 2,
//!     "storage",
//!     "Rule set has at least two rules",
//!     1.0,
//! )
//! .with_survival_criterion(
//!     |metrics, complexity| {
//!         metrics.balanced_accuracy >= 0.80
//!             && metrics.precision >= 0.75
//!             && metrics.recall >= 0.75
//!             && metrics.score > 0.0
//!             && complexity <= 2.0
//!     },
//!     "balanced_accuracy >= 0.80 AND precision >= 0.75 AND \
//!      recall >= 0.75 AND score > 0 AND complexity <= 2.0",
//! );
//! ```
//!
//! The criterion remains accessible through [`Hypothesis::survives`]:
//!
//! ```
//! use arco::hypotheses::Hypothesis;
//!
//! let mut hypothesis: Hypothesis<u8> = Hypothesis::new(
//!     "H_TEST",
//!     |rules: &[u8]| !rules.is_empty(),
//!     "storage",
//!     "Non-empty rule set",
//!     1.0,
//! );
//!
//! hypothesis.classification_metrics.balanced_accuracy = 0.75;
//! hypothesis.classification_metrics.score = 0.20;
//!
//! assert!(hypothesis.survives());
//! ```

use crate::record::ClassificationMetrics;
use crate::types::ConditionPredicate;

/// A user-defined criteria determining whether a hypothesis survives.
///
/// The criterion receives:
///
/// - the complete [`ClassificationMetrics`] produced by evaluating the
///   hypothesis;
/// - the hypothesis's complexity penalty.
///
/// It must return `true` when the available evidence is sufficient for the
/// hypothesis to survive.
///
/// The criterion is intentionally a closure rather than a fixed enum or
/// trait hierarchy. This keeps the hypothesis API simple while allowing
/// users to define arbitrary scientific decision rules.
///
/// `Send + Sync` allows hypotheses to remain usable in parallel research
/// workflows.
pub type SurvivalCriterion = dyn Fn(&ClassificationMetrics, f64) -> bool + Send + Sync;

/// A boxed survival criterion stored inside a [`Hypothesis`].
pub type BoxedSurvivalCriterion = Box<SurvivalCriterion>;

/// A falsifiable hypothesis about an emergent computational property.
///
/// A hypothesis consists of:
///
/// - a structural condition over a rule set;
/// - the emergent property being predicted;
/// - a complexity penalty;
/// - classification metrics produced by the scientific cycle;
/// - a user-defined criterion determining whether the evidence is sufficient
///   for survival.
///
/// The hypothesis does **not** calculate its own classification metrics.
/// Those are produced by the scientific cycle and stored in
/// [`classification_metrics`](Self::classification_metrics).
///
/// # Type parameters
///
/// `R` is the rule type used by the substrate.
///
/// # Default survival criterion
///
/// [`Hypothesis::new`] installs the default criterion:
///
/// ```text
/// balanced_accuracy >= 0.5 AND score > 0
/// ```
///
/// This preserves the original ARCO survival behaviour while making the
/// criterion configurable.
///
/// # Examples
///
/// ```
/// use arco::hypotheses::Hypothesis;
///
/// let hypothesis: Hypothesis<u8> = Hypothesis::new(
///     "H_MIN_SIZE",
///     |rules: &[u8]| rules.len() >= 2,
///     "storage",
///     "Rule set has at least 2 rules",
///     1.0,
/// );
///
/// assert_eq!(hypothesis.name, "H_MIN_SIZE");
/// assert_eq!(hypothesis.property_name, "storage");
/// assert_eq!(hypothesis.complexity, 1.0);
/// assert!(!hypothesis.survives());
/// ```
pub struct Hypothesis<R> {
    /// Unique identifier for the hypothesis.
    pub name: String,

    /// Structural predicate defining when the hypothesis predicts that
    /// the emergent property should occur.
    pub condition_fn: Box<ConditionPredicate<R>>,

    /// Name of the emergent property being predicted.
    ///
    /// Valid ARCO properties are:
    ///
    /// - `"persistence"`
    /// - `"storage"`
    /// - `"memory"`
    pub property_name: String,

    /// Human-readable description of the structural condition.
    pub condition_desc: String,

    /// MDL complexity penalty assigned to the hypothesis.
    pub complexity: f64,

    /// Classification evidence produced by evaluating this hypothesis.
    ///
    /// Before the scientific cycle evaluates the hypothesis, this contains
    /// its default value. The cycle is responsible for replacing it with
    /// the actual measurements.
    pub classification_metrics: ClassificationMetrics,

    /// Human-readable description of the configured survival criterion.
    ///
    /// Rust closures cannot be introspected into source code, so the
    /// description is explicitly supplied by the caller.
    pub survival_description: String,

    /// User-defined survival criterion.
    ///
    /// This is private intentionally: callers configure it through
    /// [`Hypothesis::with_survival_criterion`] and evaluate it through
    /// [`Hypothesis::survives`].
    survival_criterion: BoxedSurvivalCriterion,
}

impl<R> Hypothesis<R> {
    /// Creates a new hypothesis using ARCO's default survival criterion.
    ///
    /// The default criterion is:
    ///
    /// ```text
    /// balanced_accuracy >= 0.5 AND score > 0
    /// ```
    ///
    /// The criterion can be replaced with
    /// [`Hypothesis::with_survival_criterion`].
    ///
    /// # Parameters
    ///
    /// - `name`: Unique hypothesis identifier.
    /// - `condition_fn`: Predicate determining whether a rule set satisfies
    ///   the hypothesis condition.
    /// - `property_name`: Emergent property being predicted.
    /// - `condition_desc`: Human-readable description of the condition.
    /// - `complexity`: Complexity penalty assigned to the hypothesis.
    ///
    /// # Panics
    ///
    /// Panics if `property_name` is not one of:
    ///
    /// - `"persistence"`
    /// - `"storage"`
    /// - `"memory"`
    pub fn new(
        name: impl Into<String>,
        condition_fn: impl Fn(&[R]) -> bool + Send + Sync + 'static,
        property_name: impl Into<String>,
        condition_desc: impl Into<String>,
        complexity: f64,
    ) -> Self {
        let property_name = property_name.into();

        assert!(
            matches!(property_name.as_str(), "persistence" | "storage" | "memory"),
            "property_name must be 'persistence', 'storage', or 'memory'"
        );

        Self {
            name: name.into(),
            condition_fn: Box::new(condition_fn),
            property_name,
            condition_desc: condition_desc.into(),
            complexity,
            classification_metrics: ClassificationMetrics::default(),
            survival_description: "balanced_accuracy >= 0.5 AND score > 0".to_string(),
            survival_criterion: Box::new(|metrics: &ClassificationMetrics, _complexity: f64| {
                metrics.balanced_accuracy >= 0.5 && metrics.score > 0.0
            }),
        }
    }

    /// Replaces the survival criterion for this hypothesis.
    ///
    /// The criterion receives the complete classification metrics and the
    /// hypothesis complexity.
    ///
    /// A human-readable description must also be supplied so that the
    /// criterion can be recorded and understood when inspecting an
    /// experimental result.
    ///
    /// # Arguments
    ///
    /// - `criterion`: Function deciding whether the evidence is sufficient.
    /// - `description`: Human-readable representation of the criterion.
    ///
    /// # Examples
    ///
    /// ```
    /// use arco::hypotheses::Hypothesis;
    ///
    /// let hypothesis: Hypothesis<u8> = Hypothesis::new(
    ///     "H_STRONG",
    ///     |rules: &[u8]| rules.len() >= 2,
    ///     "storage",
    ///     "At least two rules",
    ///     1.0,
    /// )
    /// .with_survival_criterion(
    ///     |metrics, _complexity| {
    ///         metrics.balanced_accuracy >= 0.80
    ///             && metrics.precision >= 0.75
    ///     },
    ///     "balanced_accuracy >= 0.80 AND precision >= 0.75",
    /// );
    ///
    /// assert_eq!(
    ///     hypothesis.survival_description,
    ///     "balanced_accuracy >= 0.80 AND precision >= 0.75"
    /// );
    /// ```
    pub fn with_survival_criterion(
        mut self,
        criterion: impl Fn(&ClassificationMetrics, f64) -> bool + Send + Sync + 'static,
        description: impl Into<String>,
    ) -> Self {
        self.survival_criterion = Box::new(criterion);
        self.survival_description = description.into();
        self
    }

    /// Determines whether the hypothesis survives its configured criterion.
    ///
    /// The method itself does not contain a fixed scientific threshold.
    /// Instead, it delegates the decision to the criterion configured for
    /// this hypothesis.
    ///
    /// This keeps the public API stable:
    ///
    /// ```text
    /// hypothesis.survives()
    /// ```
    ///
    /// while allowing every hypothesis to define what constitutes sufficient
    /// evidence.
    ///
    /// # Examples
    ///
    /// ```
    /// use arco::hypotheses::Hypothesis;
    ///
    /// let mut hypothesis: Hypothesis<u8> = Hypothesis::new(
    ///     "H_TEST",
    ///     |rules: &[u8]| !rules.is_empty(),
    ///     "storage",
    ///     "Non-empty rule set",
    ///     1.0,
    /// );
    ///
    /// hypothesis.classification_metrics.balanced_accuracy = 0.75;
    /// hypothesis.classification_metrics.score = 0.20;
    ///
    /// assert!(hypothesis.survives());
    /// ```
    pub fn survives(&self) -> bool {
        (self.survival_criterion)(&self.classification_metrics, self.complexity)
    }
}

impl<R> std::fmt::Display for Hypothesis<R> {
    /// Formats the hypothesis together with its current evaluation status.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.survives() { "SURVIVES" } else { "FAILS" };

        write!(
            f,
            "{}: {} (balanced_acc={:.3}, score={:.3}, {})",
            self.name,
            self.condition_desc,
            self.classification_metrics.balanced_accuracy,
            self.classification_metrics.score,
            status
        )
    }
}

/// Returns references to all hypotheses that satisfy their configured
/// survival criteria.
///
/// Each hypothesis is evaluated using its own survival criterion. Therefore
/// different hypotheses in the same experiment may legitimately use
/// different definitions of sufficient evidence.
///
/// # Examples
///
/// ```
/// use arco::hypotheses::{surviving_hypotheses, Hypothesis};
///
/// let mut survivor: Hypothesis<u8> = Hypothesis::new(
///     "H_SURVIVES",
///     |_rules: &[u8]| true,
///     "storage",
///     "Always true",
///     1.0,
/// );
/// survivor.classification_metrics.balanced_accuracy = 0.75;
/// survivor.classification_metrics.score = 0.10;
///
/// let failure: Hypothesis<u8> = Hypothesis::new(
///     "H_FAILS",
///     |_rules: &[u8]| true,
///     "storage",
///     "Always true",
///     1.0,
/// );
///
/// let hypotheses = vec![survivor, failure];
/// let survivors = surviving_hypotheses(&hypotheses);
///
/// assert_eq!(survivors.len(), 1);
/// assert_eq!(survivors[0].name, "H_SURVIVES");
/// ```
pub fn surviving_hypotheses<R>(hypotheses: &[Hypothesis<R>]) -> Vec<&Hypothesis<R>> {
    hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.survives())
        .collect()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hypothesis() -> Hypothesis<u8> {
        Hypothesis::new(
            "H_TEST",
            |rules: &[u8]| rules.len() >= 2,
            "storage",
            "At least two rules",
            1.0,
        )
    }

    #[test]
    fn new_uses_expected_defaults() {
        let hypothesis = make_hypothesis();

        assert_eq!(hypothesis.name, "H_TEST");
        assert_eq!(hypothesis.property_name, "storage");
        assert_eq!(hypothesis.condition_desc, "At least two rules");
        assert_eq!(hypothesis.complexity, 1.0);

        assert_eq!(hypothesis.classification_metrics.accuracy, 0.0);
        assert_eq!(hypothesis.classification_metrics.balanced_accuracy, 0.0);
        assert_eq!(hypothesis.classification_metrics.score, 0.0);

        assert_eq!(
            hypothesis.survival_description,
            "balanced_accuracy >= 0.5 AND score > 0"
        );
    }

    #[test]
    fn default_survival_requires_balanced_accuracy_and_positive_score() {
        let mut hypothesis = make_hypothesis();

        hypothesis.classification_metrics.balanced_accuracy = 0.49;
        hypothesis.classification_metrics.score = 1.0;
        assert!(!hypothesis.survives());

        hypothesis.classification_metrics.balanced_accuracy = 0.50;
        hypothesis.classification_metrics.score = 1.0;
        assert!(hypothesis.survives());

        hypothesis.classification_metrics.balanced_accuracy = 0.90;
        hypothesis.classification_metrics.score = 0.0;
        assert!(!hypothesis.survives());

        hypothesis.classification_metrics.balanced_accuracy = 0.90;
        hypothesis.classification_metrics.score = -0.1;
        assert!(!hypothesis.survives());

        hypothesis.classification_metrics.balanced_accuracy = 0.50;
        hypothesis.classification_metrics.score = 0.000001;
        assert!(hypothesis.survives());
    }

    #[test]
    fn custom_criterion_can_replace_default() {
        let mut hypothesis = make_hypothesis().with_survival_criterion(
            |metrics, _complexity| metrics.balanced_accuracy >= 0.80,
            "balanced_accuracy >= 0.80",
        );

        hypothesis.classification_metrics.balanced_accuracy = 0.79;
        hypothesis.classification_metrics.score = -100.0;

        assert!(!hypothesis.survives());

        hypothesis.classification_metrics.balanced_accuracy = 0.80;
        hypothesis.classification_metrics.score = -100.0;

        // The custom criterion deliberately ignores score.
        assert!(hypothesis.survives());

        assert_eq!(hypothesis.survival_description, "balanced_accuracy >= 0.80");
    }

    #[test]
    fn custom_criterion_receives_complexity() {
        let mut hypothesis = make_hypothesis().with_survival_criterion(
            |metrics, complexity| metrics.score >= complexity,
            "score >= complexity",
        );

        hypothesis.classification_metrics.score = 0.99;
        assert!(!hypothesis.survives());

        hypothesis.classification_metrics.score = 1.0;
        assert!(hypothesis.survives());
    }

    #[test]
    fn custom_criterion_can_use_all_classification_metrics() {
        let mut hypothesis = make_hypothesis().with_survival_criterion(
            |metrics, _complexity| {
                metrics.accuracy >= 0.90
                    && metrics.precision >= 0.85
                    && metrics.recall >= 0.80
                    && metrics.specificity >= 0.90
                    && metrics.balanced_accuracy >= 0.85
                    && metrics.coverage >= 0.50
                    && metrics.score > 0.0
                    && metrics.true_positive >= 10
                    && metrics.true_negative >= 10
                    && metrics.false_positive == 0
                    && metrics.false_negative == 0
            },
            "strict classification criterion",
        );

        let metrics = &mut hypothesis.classification_metrics;

        metrics.accuracy = 0.95;
        metrics.precision = 0.90;
        metrics.recall = 0.90;
        metrics.specificity = 0.95;
        metrics.balanced_accuracy = 0.925;
        metrics.coverage = 0.80;
        metrics.score = 0.50;
        metrics.true_positive = 20;
        metrics.true_negative = 20;
        metrics.false_positive = 0;
        metrics.false_negative = 0;

        assert!(hypothesis.survives());
    }

    #[test]
    fn custom_criterion_can_reject_using_confusion_matrix_counts() {
        let mut hypothesis = make_hypothesis().with_survival_criterion(
            |metrics, _complexity| {
                metrics.true_positive >= 10
                    && metrics.true_negative >= 10
                    && metrics.false_positive == 0
                    && metrics.false_negative == 0
            },
            "TP >= 10 AND TN >= 10 AND FP == 0 AND FN == 0",
        );

        hypothesis.classification_metrics.true_positive = 10;
        hypothesis.classification_metrics.true_negative = 10;
        hypothesis.classification_metrics.false_positive = 1;
        hypothesis.classification_metrics.false_negative = 0;

        assert!(!hypothesis.survives());

        hypothesis.classification_metrics.false_positive = 0;

        assert!(hypothesis.survives());
    }

    #[test]
    fn condition_function_is_preserved() {
        let hypothesis = make_hypothesis();

        assert!(!(hypothesis.condition_fn)(&[]));
        assert!(!(hypothesis.condition_fn)(&[1]));
        assert!((hypothesis.condition_fn)(&[1, 2]));
        assert!((hypothesis.condition_fn)(&[1, 2, 3]));
    }

    #[test]
    #[should_panic(expected = "property_name must be")]
    fn invalid_property_name_panics() {
        let _hypothesis: Hypothesis<u8> = Hypothesis::new(
            "H_INVALID",
            |_rules: &[u8]| true,
            "invalid-property",
            "Invalid property",
            1.0,
        );
    }

    #[test]
    fn all_valid_property_names_are_accepted() {
        let persistence: Hypothesis<u8> = Hypothesis::new(
            "H_PERSISTENCE",
            |_rules: &[u8]| true,
            "persistence",
            "Persistence",
            1.0,
        );

        let storage: Hypothesis<u8> =
            Hypothesis::new("H_STORAGE", |_rules: &[u8]| true, "storage", "Storage", 1.0);

        let memory: Hypothesis<u8> =
            Hypothesis::new("H_MEMORY", |_rules: &[u8]| true, "memory", "Memory", 1.0);

        assert_eq!(persistence.property_name, "persistence");
        assert_eq!(storage.property_name, "storage");
        assert_eq!(memory.property_name, "memory");
    }

    #[test]
    fn surviving_hypotheses_uses_each_hypothesis_criterion() {
        let mut default_survivor = make_hypothesis();
        default_survivor.classification_metrics.balanced_accuracy = 0.80;
        default_survivor.classification_metrics.score = 0.50;

        let mut default_failure = Hypothesis::new(
            "H_FAILURE",
            |rules: &[u8]| rules.len() >= 2,
            "storage",
            "At least two rules",
            1.0,
        );
        default_failure.classification_metrics.balanced_accuracy = 0.49;
        default_failure.classification_metrics.score = 0.50;

        let mut custom_survivor = make_hypothesis().with_survival_criterion(
            |metrics, _complexity| metrics.balanced_accuracy >= 0.90,
            "balanced_accuracy >= 0.90",
        );

        custom_survivor.classification_metrics.balanced_accuracy = 0.90;
        custom_survivor.classification_metrics.score = -10.0;

        let hypotheses = vec![default_survivor, default_failure, custom_survivor];

        let survivors = surviving_hypotheses(&hypotheses);

        assert_eq!(survivors.len(), 2);
        assert_eq!(survivors[0].name, "H_TEST");
        assert_eq!(survivors[1].name, "H_TEST");
    }

    #[test]
    fn display_reports_balanced_accuracy_and_score() {
        let mut hypothesis = make_hypothesis();

        hypothesis.classification_metrics.balanced_accuracy = 0.75;
        hypothesis.classification_metrics.score = 0.25;

        let displayed = hypothesis.to_string();

        assert!(displayed.contains("H_TEST"));
        assert!(displayed.contains("At least two rules"));
        assert!(displayed.contains("balanced_acc=0.750"));
        assert!(displayed.contains("score=0.250"));
        assert!(displayed.contains("SURVIVES"));
    }

    #[test]
    fn display_reports_failure_status() {
        let hypothesis = make_hypothesis();

        let displayed = hypothesis.to_string();

        assert!(displayed.contains("H_TEST"));
        assert!(displayed.contains("FAILS"));
    }

    #[test]
    fn criterion_is_evaluated_at_call_time() {
        let mut hypothesis = make_hypothesis().with_survival_criterion(
            |metrics, _complexity| metrics.balanced_accuracy >= 0.75,
            "balanced_accuracy >= 0.75",
        );

        hypothesis.classification_metrics.balanced_accuracy = 0.70;
        assert!(!hypothesis.survives());

        hypothesis.classification_metrics.balanced_accuracy = 0.75;
        assert!(hypothesis.survives());

        hypothesis.classification_metrics.balanced_accuracy = 0.80;
        assert!(hypothesis.survives());
    }

    #[test]
    fn criterion_can_ignore_complexity() {
        let mut hypothesis = make_hypothesis().with_survival_criterion(
            |metrics, _complexity| metrics.balanced_accuracy >= 0.80,
            "balanced_accuracy >= 0.80",
        );

        hypothesis.classification_metrics.balanced_accuracy = 0.80;

        assert!(hypothesis.survives());
    }

    #[test]
    fn criterion_can_use_complexity_as_a_hard_limit() {
        let mut hypothesis = make_hypothesis().with_survival_criterion(
            |metrics, complexity| metrics.balanced_accuracy >= 0.80 && complexity <= 1.0,
            "balanced_accuracy >= 0.80 AND complexity <= 1.0",
        );

        hypothesis.classification_metrics.balanced_accuracy = 0.90;

        assert!(hypothesis.survives());

        hypothesis.complexity = 1.1;

        assert!(!hypothesis.survives());
    }
}
