//! Scientific cycle orchestrator.
//!
//! Per the Mathematical Constitution:
//!     The scientific cycle is: Generate → Calibrate → Observe →
//!     Hypothesize → Predict → Test → Revise. Each cycle produces
//!     a Research Record.
//!
//! # Design
//!
//! The cycle is fully generic over any [`InformationUniverse`] type.
//! It does not know about graph rewriting, cellular automata, or
//! any specific substrate.
//!
//! # Usage
//!
//! ```rust
//! use arco::{
//!     cycle::{CycleConfig, run_cycle},
//!     hypotheses::Hypothesis,
//!     rules::Rule,
//!     schedule::Schedule,
//!     substrates::graph::{BinaryGraphState, MatchInfo, RewriteRule, observation::CompoundObserver},
//!     universe::InformationUniverse,
//! };
//!
//! fn main() {
//!     let universe = MyUniverse {
//!         states: vec![],
//!         rules: vec![],
//!         observation: CompoundObserver,
//!         schedule: SeqSchedule,
//!     };
//!     let config = CycleConfig::default();
//!     let mut hypotheses = vec![
//!         Hypothesis::new("H1", |_rules| true, "storage", "H1 desc", 1.0),
//!         Hypothesis::new("H2", |_| true, "storage", "H2 desc", 0.5),
//!     ];
//!     let record = run_cycle(
//!         &universe,
//!         &config,
//!         &mut hypotheses,
//!         &mut |_| (vec![], 0.0),
//!         None,
//!     );
//!
//!     println!("{}", record.summary());
//! }
//!
//! #[derive(Debug)]
//! struct SeqSchedule;
//!
//! impl Schedule<BinaryGraphState, RewriteRule> for SeqSchedule {
//!     fn name(&self) -> &str {
//!         "Schedule"
//!     }
//!
//!     fn selection(&self) -> &str {
//!         "exhaustive"
//!     }
//!
//!     fn timing(&self) -> &str {
//!         "asynchronous"
//!     }
//!
//!     fn step(
//!         &self,
//!         state: &BinaryGraphState,
//!         rules: &[RewriteRule],
//!         rng: &mut dyn rand::prelude::Rng,
//!     ) -> BinaryGraphState {
//!         let mut current = state.clone();
//!         let context = MatchInfo::Unconditional { vertex: 0 };
//!         for rule in rules {
//!             current = rule.apply(state, &context, rng);
//!         }
//!         current
//!     }
//! }
//!
//! struct MyUniverse {
//!     states: Vec<BinaryGraphState>,
//!     rules: Vec<RewriteRule>,
//!     observation: CompoundObserver,
//!     schedule: SeqSchedule,
//! }
//!
//! impl InformationUniverse for MyUniverse {
//!     type State = BinaryGraphState;
//!     type Rule = RewriteRule;
//!     type Observation = CompoundObserver;
//!     type Schedule = SeqSchedule;
//!
//!     fn state_space(&self) -> &[Self::State] {
//!         &self.states
//!     }
//!
//!     fn rules(&self) -> &[Self::Rule] {
//!         &self.rules
//!     }
//!
//!     fn observation(&self) -> &Self::Observation {
//!         &self.observation
//!     }
//!
//!     fn schedule(&self) -> &Self::Schedule {
//!         &self.schedule
//!     }
//!
//!     fn null_rules(&self, _rng: &mut dyn rand::prelude::Rng) -> Vec<Self::Rule> {
//!         vec![]
//!     }
//! }
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
use crate::metrics::{compute_memory, compute_persistence, compute_storage};
use crate::observation::Observation;
use crate::record::{HypothesisRecord, ResearchRecord, UniverseResult};
use crate::rules::Rule;
use crate::types::BooleanTester;
use crate::types::RuleGenerator;
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
    /// Ensemble size per universe (trajectories from distinct initial states).
    pub n_ensemble: usize,
    /// Timesteps per trajectory.
    pub steps: usize,
    /// Maximum timescale for storage/memory.
    pub max_delta: usize,
    /// Number of shuffles for bias correction.
    pub n_shuffles: usize,
    /// Number of null universes for calibration.
    pub n_null_universes: usize,
    /// Random seed for reproducibility.
    pub seed: u64,
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
            seed: 42,
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
///   No restrictions on `Observation::Output` — any hashable,
///   comparable type works.
///
/// # Steps
///
/// 1. **Generate**: Call `rule_generator` to sample rule sets.
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
/// * `hypotheses` — Mutable slice of hypotheses to test. Their
///   `accuracy` and `score` fields will be updated in place.
/// * `rule_generator` — Function that generates a rule set and its
///   structured ratio given an RNG. This is substrate-specific.
/// * `boolean_tester` — Optional function that tests whether a
///   rule set implements boolean functions. Pass `None` to skip
///   boolean verification.
///
/// # Returns
///
/// A [`ResearchRecord`] with all experimental data.
pub fn run_cycle<U: InformationUniverse>(
    universe: &U,
    config: &CycleConfig,
    hypotheses: &mut [Hypothesis<U::Rule>],
    rule_generator: &mut RuleGenerator<U>,
    boolean_tester: Option<&BooleanTester<U>>,
) -> ResearchRecord<U>
where
    <U::Observation as Observation<U::State>>::Output: Eq + std::hash::Hash + Clone + Send + Sync,
    U::Rule: Send + Sync,
    U::State: Send + Sync,
{
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
        train_subsets.push(rule_generator(&mut rng));
    }
    for _ in 0..config.n_test {
        test_subsets.push(rule_generator(&mut rng));
    }

    // ================================================================
    // STEP 2: CALIBRATE
    // ================================================================
    let ca_config = CalibrationConfig {
        percentile: 95.0,
        floor_persistence: 0.01,
        floor_storage: 0.01,
        floor_memory: 0.01,
        max_delta: config.max_delta,
        n_shuffles: config.n_shuffles,
        seed: config.seed,
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
    // Pre-allocate results vector for parallelization
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

    // Parallel observation
    results
        .par_iter_mut()
        .zip(train_subsets.par_iter())
        .enumerate()
        .for_each(|(i, (result, (rules, ratio)))| {
            let mut local_rng = StdRng::seed_from_u64(config.seed + i as u64 * 137);

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
                config.steps,
                schedule,
                observer,
                config.seed + i as u64 * 137,
            );

            *result = UniverseResult {
                universe_id: i,
                structured_ratio: *ratio,
                n_rules: rules.len(),
                rule_names: rules.iter().map(|r| r.name().to_string()).collect(),
                persistence: compute_persistence(&ensemble, 1, config.n_shuffles, config.seed),
                storage: compute_storage(
                    &ensemble,
                    config.max_delta,
                    config.n_shuffles,
                    config.seed,
                ),
                memory: compute_memory(&ensemble, config.max_delta, config.n_shuffles, config.seed),
            };
        });

    record.results = results;

    // ================================================================
    // STEP 4: HYPOTHESIZE & TEST
    // ================================================================
    // Generate test ensembles

    // Pre-allocate test ensembles
    let mut test_ensembles: TestEnsembles<U> =
        (0..test_subsets.len()).map(|_| Vec::new()).collect();

    // Generate test ensembles in parallel
    test_ensembles
        .par_iter_mut()
        .zip(test_subsets.par_iter())
        .enumerate()
        .for_each(|(i, (ensemble_out, (rules, _ratio)))| {
            let mut local_rng = StdRng::seed_from_u64(config.seed + 10000 + i as u64 * 137);

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
                config.steps,
                schedule,
                observer,
                config.seed + 10000 + i as u64 * 137,
            );
        });

    // Test each hypothesis directly.
    // The compiler monomorphizes the correct metric function for the
    // concrete Observation::Output type.
    for h in hypotheses.iter_mut() {
        let threshold = record
            .thresholds
            .get(&h.property_name)
            .copied()
            .unwrap_or(0.0);

        let mut positive = 0usize;
        let mut correct = 0usize;

        for ((rules, _ratio), ensemble) in test_subsets.iter().zip(test_ensembles.iter()) {
            if (h.condition_fn)(rules) {
                positive += 1;
                let metric_value = match h.property_name.as_str() {
                    "persistence" => {
                        compute_persistence(ensemble, 1, config.n_shuffles, config.seed)
                    }
                    "storage" => {
                        compute_storage(ensemble, config.max_delta, config.n_shuffles, config.seed)
                    }
                    "memory" => {
                        compute_memory(ensemble, config.max_delta, config.n_shuffles, config.seed)
                    }
                    _ => 0.0,
                };
                if metric_value > threshold {
                    correct += 1;
                }
            }
        }

        h.accuracy = if positive > 0 {
            correct as f64 / positive as f64
        } else {
            0.0
        };
        h.score = h.accuracy - 0.1 * h.complexity;
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
