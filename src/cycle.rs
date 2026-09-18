//! Scientific cycle orchestrator.
//!
//! Per the Mathematical Constitution:
//!     The scientific cycle is: Generate → Calibrate → Observe →
//!     Hypothesize → Predict → Test → Revise. Each cycle produces
//!     a Research Record.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use arco::cycle::{CycleConfig, run_cycle};
//! use arco::substrates::graph::{
//!     BinaryGraphUniverse, generate_standard_hypotheses,
//! };
//! use rand::{rngs::StdRng, SeedableRng};
//!
//! let mut rng = StdRng::seed_from_u64(42);
//! let config = CycleConfig {
//!     n_train: 20,
//!     n_test: 5,
//!     ..CycleConfig::default()
//! };
//! let universe = BinaryGraphUniverse::new(3, "compound", &mut rng, config.n_train + config.n_test);
//! let mut hypotheses = generate_standard_hypotheses();
//! let record = run_cycle(&universe, &config, &mut hypotheses, None);
//! println!("{}", record.summary());
//! ```

use std::time::Instant;

use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;

use crate::calibration::CalibrationConfig;
use crate::calibration::{calibrate, generate_trajectories};
use crate::hypotheses::{Hypothesis, surviving_hypotheses};
use crate::metrics::NsbCardinality;
use crate::metrics::{Estimator, MetricConfig, memory, persistence, storage};
use crate::record::{ClassificationMetrics, HypothesisRecord, ResearchRecord, UniverseResult};
use crate::rules::Rule;
use crate::types::BooleanTester;
use crate::types::TestEnsembles;
use crate::universe::InformationUniverse;

// ===================================================================
// Cycle Configuration
// ===================================================================

/// Configuration for a scientific cycle run.
///
/// All parameters have sensible defaults. The cycle is reproducible
/// given the same config and universe.
#[derive(Debug, Clone)]
pub struct CycleConfig {
    /// Number of training universes.
    pub n_train: usize,
    /// Number of held-out test universes.
    pub n_test: usize,
    /// Ensemble size per universe.
    pub n_ensemble: usize,
    /// Timesteps per trajectory.
    pub steps: usize,
    /// Maximum timescale for storage/memory.
    pub max_delta: usize,
    /// Number of shuffles for bias correction.
    pub n_shuffles: usize,
    /// Number of null universes for calibration.
    pub n_null_universes: usize,
    /// Percentile value.
    pub percentile: f64,
    /// Minimum threshold for persistence.
    pub floor_persistence: f64,
    /// Minimum threshold for storage.
    pub floor_storage: f64,
    /// Minimum threshold for memory.
    pub floor_memory: f64,
    /// Random seed for reproducibility.
    pub seed: u64,
    /// The MI estimator
    pub estimator: Estimator,
    /// NSB estimator cardinality.
    pub cardinality: NsbCardinality,
}

impl Default for CycleConfig {
    fn default() -> Self {
        Self {
            n_train: 300,
            n_test: 100,
            n_ensemble: 10,
            steps: 60,
            max_delta: 15,
            n_shuffles: 10,
            n_null_universes: 30,
            percentile: 95.0,
            floor_persistence: 0.01,
            floor_storage: 0.01,
            floor_memory: 0.01,
            seed: 42,
            estimator: Estimator::Plugin,
            cardinality: NsbCardinality::Observed,
        }
    }
}

// ===================================================================
// Scientific Cycle
// ===================================================================

/// Execute the full ARCO scientific cycle.
///
/// # Type parameters
///
/// - `U: InformationUniverse` — The universe type. The cycle is fully
///   generic over the state, rule, observation, and schedule types.
///
/// # Steps
///
/// 1. **Generate**: Call `universe.generate_rules()` to sample rule sets.
/// 2. **Calibrate**: Compute thresholds from destructive null
///    universes using `universe.null_rules()`.
/// 3. **Observe**: Generate ensembles via `generate_trajectories`
///    and compute storage, memory, and persistence for each
///    training universe.
/// 4. **Hypothesize & Test**: For each hypothesis, evaluate its
///    condition on test rule sets, compute the predicted metric
///    directly, and check against the calibrated threshold.
/// 5. **Revise**: Check failure conditions, optionally verify
///    boolean functions, compile the research record.
///
/// # Parameters
///
/// * `universe` — The Information Universe to study.
/// * `config` — Experimental parameters.
/// * `hypotheses` — Mutable slice of hypotheses to test.
/// * `boolean_tester` — Optional function that verifies boolean
///   functions implemented by rule sets. Pass `None` to skip.
///
/// # Returns
///
/// A [`ResearchRecord`] with all experimental data.
pub fn run_cycle<U: InformationUniverse>(
    universe: &U,
    config: &CycleConfig,
    hypotheses: &mut [Hypothesis<U::Rule>],
    boolean_tester: Option<&BooleanTester<U>>,
) -> ResearchRecord<U> {
    let t0 = Instant::now();
    let mut record = ResearchRecord::new(env!("CARGO_PKG_VERSION"));

    // Store config for reproducibility
    record
        .config
        .insert("n_train".to_string(), config.n_train.to_string());
    record
        .config
        .insert("n_test".to_string(), config.n_test.to_string());
    record
        .config
        .insert("n_ensemble".to_string(), config.n_ensemble.to_string());
    record
        .config
        .insert("steps".to_string(), config.steps.to_string());
    record
        .config
        .insert("max_delta".to_string(), config.max_delta.to_string());
    record
        .config
        .insert("n_shuffles".to_string(), config.n_shuffles.to_string());
    record.config.insert(
        "n_null_universes".to_string(),
        config.n_null_universes.to_string(),
    );
    record
        .config
        .insert("seed".to_string(), config.seed.to_string());
    record
        .config
        .insert("estimator".to_string(), config.estimator.name().to_string());
    if matches!(config.estimator, Estimator::NSB) {
        record.config.insert(
            "cardinality".to_string(),
            config.cardinality.name().to_string(),
        );
    }

    let mut rng = StdRng::seed_from_u64(config.seed);
    let state_space = universe.state_space();
    let schedule = universe.schedule();
    let observer = universe.observation();

    // ================================================================
    // STEP 1: GENERATE
    // ================================================================
    let mut train_subsets: Vec<(Vec<U::Rule>, f64)> = Vec::with_capacity(config.n_train);
    let mut test_subsets: Vec<(Vec<U::Rule>, f64)> = Vec::with_capacity(config.n_test);

    for _ in 0..config.n_train {
        train_subsets.push(universe.generate_rules(&mut rng));
    }
    for _ in 0..config.n_test {
        test_subsets.push(universe.generate_rules(&mut rng));
    }

    // ================================================================
    // STEP 2: CALIBRATE
    // ================================================================
    let met_config = MetricConfig {
        estimator: config.estimator,
        cardinality: config.cardinality.clone(),
        max_delta: config.max_delta,
        n_shuffles: config.n_shuffles,
        seed: config.seed,
    };
    let ca_config = CalibrationConfig {
        metric: met_config,
        percentile: config.percentile,
        floor_persistence: config.floor_persistence,
        floor_storage: config.floor_storage,
        floor_memory: config.floor_memory,
    };
    let calibration = calibrate(
        universe,
        config.n_null_universes,
        config.n_ensemble,
        config.steps,
        &ca_config,
    );

    record
        .thresholds
        .insert("persistence".to_string(), calibration.persistence_threshold);
    record
        .thresholds
        .insert("storage".to_string(), calibration.storage_threshold);
    record
        .thresholds
        .insert("memory".to_string(), calibration.memory_threshold);

    // ================================================================
    // STEP 3: OBSERVE
    // ================================================================
    let mut results: Vec<UniverseResult> = (0..train_subsets.len())
        .map(|i| UniverseResult {
            universe_id: i,
            structured_ratio: 0.0,
            n_rules: 0,
            rule_names: vec![],
            persistence: 0.0,
            storage: 0.0,
            memory: 0.0,
        })
        .collect();

    results
        .par_iter_mut()
        .zip(train_subsets.par_iter())
        .enumerate()
        .for_each(|(i, (result, (rules, ratio)))| {
            let local_seed = config.seed + i as u64 * 137;
            let mut local_rng = StdRng::seed_from_u64(local_seed);

            let n_pool = state_space.len();
            let n_ens = config.n_ensemble.min(n_pool);
            let mut init_indices: Vec<usize> = (0..n_pool).collect();
            for j in 0..n_ens {
                let k = local_rng.random_range(j..n_pool);
                init_indices.swap(j, k);
            }
            let initial_states: Vec<U::State> = init_indices
                .iter()
                .take(n_ens)
                .map(|&idx| state_space[idx].clone())
                .collect();

            let ensemble = generate_trajectories(
                &initial_states,
                rules,
                observer,
                schedule,
                config.steps,
                local_seed,
            );

            *result = UniverseResult {
                universe_id: i,
                structured_ratio: *ratio,
                n_rules: rules.len(),
                rule_names: rules.iter().map(|r| r.name().to_string()).collect(),
                persistence: persistence(&ensemble, 1, &ca_config.metric),
                storage: storage(&ensemble, &ca_config.metric),
                memory: memory(&ensemble, &ca_config.metric),
            };
        });

    record.results = results;

    // ================================================================
    // STEP 4: HYPOTHESIZE & TEST
    // ================================================================
    let mut test_ensembles: TestEnsembles<U> =
        (0..test_subsets.len()).map(|_| Vec::new()).collect();

    test_ensembles
        .par_iter_mut()
        .zip(test_subsets.par_iter())
        .enumerate()
        .for_each(|(i, (ensemble_out, (rules, _ratio)))| {
            let local_seed = config.seed + 10000 + i as u64 * 137;
            let mut local_rng = StdRng::seed_from_u64(local_seed);

            let n_pool = state_space.len();
            let n_ens = config.n_ensemble.min(n_pool);
            let mut init_indices: Vec<usize> = (0..n_pool).collect();
            for j in 0..n_ens {
                let k = local_rng.random_range(j..n_pool);
                init_indices.swap(j, k);
            }
            let initial_states: Vec<U::State> = init_indices
                .iter()
                .take(n_ens)
                .map(|&idx| state_space[idx].clone())
                .collect();

            *ensemble_out = generate_trajectories(
                &initial_states,
                rules,
                observer,
                schedule,
                config.steps,
                local_seed,
            );
        });

    for h in hypotheses.iter_mut() {
        let threshold = record
            .thresholds
            .get(&h.property_name)
            .copied()
            .unwrap_or(0.0);

        // Update the hypothesis classification metrics.
        h.classification_metrics = hypothesis_classification_metrics::<U>(
            h,
            &test_subsets,
            &test_ensembles,
            threshold,
            &ca_config.metric,
        );
    }

    record.hypotheses = hypotheses.iter().map(HypothesisRecord::from).collect();

    // ================================================================
    // STEP 5: BOOLEAN VERIFICATION (OPTIONAL)
    // ================================================================
    if let Some(tester) = boolean_tester {
        for (rules, _ratio) in train_subsets.iter() {
            let verified = tester(rules);
            for (gate, count) in verified {
                *record.boolean_verifications.entry(gate).or_insert(0) += count;
            }
        }
    }

    // ================================================================
    // STEP 6: FAILURE CONDITION CHECK
    // ================================================================
    let nand_count = record
        .boolean_verifications
        .get("NAND")
        .copied()
        .unwrap_or(0);

    if record.n_storage() == 0 {
        record
            .failure_conditions
            .push("F-1 (NULL): No storage universes found.".to_string());
    }

    if nand_count == 0 && boolean_tester.is_some() {
        record
            .failure_conditions
            .push("F-1 (NULL): NAND not verified.".to_string());
    }

    let surviving = surviving_hypotheses(hypotheses);
    if surviving.is_empty() && !hypotheses.is_empty() {
        record
            .failure_conditions
            .push("F-6 (DISCONFIRMATION): No hypotheses survived.".to_string());
    }

    record.elapsed_seconds = t0.elapsed().as_secs_f64();
    record
}

/// Compute standard binary classification metrics for one hypothesis on
/// held-out test data.
///
/// The hypothesis condition determines the predicted class, while the
/// measured property relative to `threshold` determines the actual class:
///
/// - **predicted positive**: the universe satisfies the hypothesis condition
/// - **predicted negative**: the universe does not satisfy the hypothesis
///   condition
/// - **actual positive**: the measured property exceeds `threshold`
/// - **actual negative**: the measured property does not exceed `threshold`
///
/// The confusion matrix is therefore:
///
/// ```text
///                         Actual positive    Actual negative
/// Predicted positive          TP                  FP
/// Predicted negative          FN                  TN
/// ```
///
/// The returned metrics are:
///
/// - `accuracy`: `(TP + TN) / total`
/// - `precision`: `TP / (TP + FP)`
/// - `recall`: `TP / (TP + FN)`
/// - `specificity`: `TN / (TN + FP)`
/// - `balanced_accuracy`: `(recall + specificity) / 2`
/// - `coverage`: `(TP + FP) / total`
///
/// `coverage` measures how frequently the hypothesis condition is satisfied
/// in the held-out test data. It is not a classification-quality metric, but
/// is useful for interpreting the other metrics: a hypothesis with excellent
/// precision but very low coverage applies to only a small fraction of the
/// test universes.
///
/// Only held-out test subsets are considered. Each test subset is paired with
/// its corresponding test ensemble by position.
///
/// If a metric has a zero denominator, its value is returned as `0.0`.
fn hypothesis_classification_metrics<U: InformationUniverse>(
    hypothesis: &Hypothesis<U::Rule>,
    test_subsets: &[(Vec<U::Rule>, f64)],
    test_ensembles: &TestEnsembles<U>,
    threshold: f64,
    metric: &MetricConfig,
) -> ClassificationMetrics {
    let mut true_positive = 0usize;
    let mut false_positive = 0usize;
    let mut false_negative = 0usize;
    let mut true_negative = 0usize;

    for ((rules, _ratio), ensemble) in test_subsets.iter().zip(test_ensembles.iter()) {
        let predicted_positive = (hypothesis.condition_fn)(rules);

        let metric_value = match hypothesis.property_name.as_str() {
            "persistence" => persistence(ensemble, 1, metric),
            "storage" => storage(ensemble, metric),
            "memory" => memory(ensemble, metric),
            _ => 0.0,
        };

        let actual_positive = metric_value > threshold;

        match (predicted_positive, actual_positive) {
            (true, true) => true_positive += 1,
            (true, false) => false_positive += 1,
            (false, true) => false_negative += 1,
            (false, false) => true_negative += 1,
        }
    }

    let total = true_positive + false_positive + false_negative + true_negative;

    let accuracy = if total > 0 {
        (true_positive + true_negative) as f64 / total as f64
    } else {
        0.0
    };

    let precision_denominator = true_positive + false_positive;
    let precision = if precision_denominator > 0 {
        true_positive as f64 / precision_denominator as f64
    } else {
        0.0
    };

    let recall_denominator = true_positive + false_negative;
    let recall = if recall_denominator > 0 {
        true_positive as f64 / recall_denominator as f64
    } else {
        0.0
    };

    let specificity_denominator = true_negative + false_positive;
    let specificity = if specificity_denominator > 0 {
        true_negative as f64 / specificity_denominator as f64
    } else {
        0.0
    };

    let balanced_accuracy = (recall + specificity) / 2.0;

    let coverage_denominator = total;
    let coverage = if coverage_denominator > 0 {
        (true_positive + false_positive) as f64 / coverage_denominator as f64
    } else {
        0.0
    };

    // Compute the score.
    let score = balanced_accuracy - 0.1 * hypothesis.complexity;

    ClassificationMetrics {
        accuracy,
        precision,
        recall,
        specificity,
        balanced_accuracy,
        coverage,
        score,
        true_positive,
        false_positive,
        false_negative,
        true_negative,
    }
}
