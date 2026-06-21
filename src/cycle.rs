//! Scientific cycle orchestrator and research record.
//!
//! Per Constitution:
//!     The scientific cycle is: Generate → Calibrate → Observe →
//!     Hypothesize → Predict → Test → Revise. Each cycle produces a
//!     Research Record.
//!
//! This module provides:
//! - `run_cycle()`: end-to-end execution of the scientific cycle.
//! - `ResearchRecord`: structured output with full reproducibility data.

use std::collections::HashMap;
use std::time::Instant;

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use crate::calibration::calibrate;
use crate::dynamics::{DEFAULT_SCHEDULE, generate_ensemble, test_boolean_function};
use crate::hypotheses::{
    Hypothesis, generate_standard_hypotheses, surviving_hypotheses, test_all_hypotheses,
};
use crate::metrics::{compute_memory, compute_storage};
use crate::observation::observe_windowed;
use crate::rules::{
    RewriteRule, Rule, create_destructive_rules, create_structured_rules,
    generate_mixed_rule_subsets,
};
use crate::state::BinaryGraphState;

/// Test data: (rules, ensemble trajectories)
type TestDataEntry<'a> = (&'a [RewriteRule], &'a [Vec<Vec<u8>>]);

/// Metric function type used in hypothesis testing
type MetricFn = dyn Fn(&[Vec<Vec<u8>>]) -> f64;

/// Truth table for a 2-input Boolean function
type TruthTable<'a> = Vec<((u8, u8), u8)>;

// ===================================================================
// Research Record
// ===================================================================

/// The output of one complete scientific cycle.
///
/// Contains all data needed to reproduce, audit, or extend the
/// experimental results.
#[derive(Debug, Clone)]
pub struct ResearchRecord {
    /// ARCO version string.
    pub version: String,
    /// Wall-clock duration in seconds.
    pub elapsed_seconds: f64,
    /// Configuration parameters.
    pub config: HashMap<String, String>,
    /// Calibrated emergence thresholds.
    pub thresholds: HashMap<String, f64>,
    /// Per-universe results.
    pub results: Vec<UniverseResult>,
    /// All hypotheses tested.
    pub hypotheses: Vec<HypothesisRecord>,
    /// Boolean function discoveries.
    pub boolean_discoveries: HashMap<String, usize>,
    /// Triggered failure conditions.
    pub failure_conditions: Vec<String>,
}

/// Per-universe experimental result.
#[derive(Debug, Clone)]
pub struct UniverseResult {
    pub universe_id: usize,
    pub structured_ratio: f64,
    pub n_rules: usize,
    pub n_structured: usize,
    pub rule_names: Vec<String>,
    pub persistence: f64,
    pub storage: f64,
    pub memory: f64,
}

/// Serialized hypothesis record for the research output.
#[derive(Debug, Clone)]
pub struct HypothesisRecord {
    pub name: String,
    pub condition_desc: String,
    pub property_name: String,
    pub complexity: f64,
    pub accuracy: f64,
    pub score: f64,
    pub survives: bool,
}

impl From<&Hypothesis> for HypothesisRecord {
    fn from(h: &Hypothesis) -> Self {
        Self {
            name: h.name.clone(),
            condition_desc: h.condition_desc.clone(),
            property_name: h.property_name.clone(),
            complexity: h.complexity,
            accuracy: h.accuracy,
            score: h.score,
            survives: h.survives(),
        }
    }
}

impl ResearchRecord {
    /// Create a new empty research record.
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            elapsed_seconds: 0.0,
            config: HashMap::new(),
            thresholds: HashMap::new(),
            results: Vec::new(),
            hypotheses: Vec::new(),
            boolean_discoveries: HashMap::new(),
            failure_conditions: Vec::new(),
        }
    }

    /// Number of universes above the persistence threshold.
    pub fn n_persistent(&self) -> usize {
        let threshold = self
            .thresholds
            .get("persistence")
            .copied()
            .unwrap_or(f64::MAX);
        self.results
            .iter()
            .filter(|r| r.persistence > threshold)
            .count()
    }

    /// Number of universes above the storage threshold.
    pub fn n_storage(&self) -> usize {
        let threshold = self.thresholds.get("storage").copied().unwrap_or(f64::MAX);
        self.results
            .iter()
            .filter(|r| r.storage > threshold)
            .count()
    }

    /// Number of universes above the memory threshold.
    pub fn n_memory(&self) -> usize {
        let threshold = self.thresholds.get("memory").copied().unwrap_or(f64::MAX);
        self.results.iter().filter(|r| r.memory > threshold).count()
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("ARCO v{} — Scientific Cycle Report", self.version));
        lines.push("=".repeat(60));
        lines.push(format!("Universes: {}", self.results.len()));
        lines.push(format!("Duration:  {:.1}s", self.elapsed_seconds));
        lines.push(String::new());
        lines.push("Emergence (above calibrated thresholds):".to_string());
        lines.push(format!(
            "  Storage: {}/{} ({:.1}%)",
            self.n_storage(),
            self.results.len(),
            100.0 * self.n_storage() as f64 / self.results.len() as f64,
        ));
        lines.push(format!(
            "  Memory:  {}/{} ({:.1}%)",
            self.n_memory(),
            self.results.len(),
            100.0 * self.n_memory() as f64 / self.results.len() as f64,
        ));
        lines.push(String::new());
        lines.push(format!("Hypotheses tested: {}", self.hypotheses.len()));
        let surviving: Vec<&HypothesisRecord> =
            self.hypotheses.iter().filter(|h| h.survives).collect();
        lines.push(format!("Hypotheses survived: {}", surviving.len()));

        if !surviving.is_empty() {
            lines.push(String::new());
            lines.push("Surviving hypotheses:".to_string());
            for h in surviving {
                lines.push(format!(
                    "  {}: {} (acc={:.3}, score={:.3})",
                    h.name, h.condition_desc, h.accuracy, h.score,
                ));
            }
        }

        if !self.boolean_discoveries.is_empty() {
            lines.push(String::new());
            lines.push("Boolean functions rediscovered:".to_string());
            let mut gates: Vec<(&String, &usize)> = self.boolean_discoveries.iter().collect();
            gates.sort_by(|a, b| a.0.cmp(b.0));
            for (gate, count) in gates {
                lines.push(format!("  {}: {}", gate, count));
            }
        }

        if !self.failure_conditions.is_empty() {
            lines.push(String::new());
            lines.push("FAILURE CONDITIONS TRIGGERED:".to_string());
            for fc in &self.failure_conditions {
                lines.push(format!("  ! {}", fc));
            }
        }

        lines.join("\n")
    }
}

impl Default for ResearchRecord {
    fn default() -> Self {
        Self::new()
    }
}

// ===================================================================
// Spectrum analysis
// ===================================================================

/// Group results by structured-ratio bracket.
fn compute_spectrum(results: &[UniverseResult], threshold: f64) -> HashMap<String, SpectrumBucket> {
    let brackets: &[(&str, f64, f64)] = &[
        ("Noise", 0.00, 0.15),
        ("Noise-dominated", 0.15, 0.40),
        ("Balanced", 0.40, 0.60),
        ("Structure-dominated", 0.60, 0.85),
        ("Structured", 0.85, 1.01),
    ];

    let mut spectrum = HashMap::new();

    for (label, low, high) in brackets {
        let group: Vec<&UniverseResult> = results
            .iter()
            .filter(|r| r.structured_ratio >= *low && r.structured_ratio < *high)
            .collect();

        if group.is_empty() {
            continue;
        }

        let n = group.len();
        let storage_pct =
            100.0 * group.iter().filter(|r| r.storage > threshold).count() as f64 / n as f64;
        let mean_storage = group.iter().map(|r| r.storage).sum::<f64>() / n as f64;

        spectrum.insert(
            label.to_string(),
            SpectrumBucket {
                n,
                storage_pct,
                mean_storage,
            },
        );
    }

    spectrum
}

#[derive(Debug, Clone)]
struct SpectrumBucket {
    n: usize,
    storage_pct: f64,
    mean_storage: f64,
}

// ===================================================================
// Main scientific cycle
// ===================================================================

/// Configuration for a scientific cycle run.
pub struct CycleConfig {
    pub n_train: usize,
    pub n_test: usize,
    pub n_vertices: usize,
    pub n_ensemble: usize,
    pub steps: usize,
    pub window_size: usize,
    pub obs_name: String,
    pub max_delta: usize,
    pub n_shuffles: usize,
    pub n_null_universes: usize,
    pub seed: u64,
}

impl Default for CycleConfig {
    fn default() -> Self {
        Self {
            n_train: 300,
            n_test: 100,
            n_vertices: 3,
            n_ensemble: 10,
            steps: 60,
            window_size: 1,
            obs_name: "compound".to_string(),
            max_delta: 15,
            n_shuffles: 10,
            n_null_universes: 30,
            seed: 42,
        }
    }
}

/// Execute the full ARCO scientific cycle.
///
/// Steps:
/// 1. GENERATE — create spectrum universes
/// 2. CALIBRATE — compute thresholds from destructive null
/// 3. OBSERVE — compute emergence metrics on all universes
/// 4. HYPOTHESIZE & TEST — evaluate hypotheses on held-out data
/// 5. REVISE — check failure conditions, compile spectrum,
///    test Boolean rediscovery, report surviving laws
pub fn run_cycle(config: &CycleConfig) -> ResearchRecord {
    let t0 = Instant::now();
    let mut record = ResearchRecord::new();

    // Store config
    record
        .config
        .insert("n_train".to_string(), config.n_train.to_string());
    record
        .config
        .insert("n_test".to_string(), config.n_test.to_string());
    record
        .config
        .insert("n_vertices".to_string(), config.n_vertices.to_string());
    record
        .config
        .insert("n_ensemble".to_string(), config.n_ensemble.to_string());
    record
        .config
        .insert("steps".to_string(), config.steps.to_string());
    record
        .config
        .insert("window_size".to_string(), config.window_size.to_string());
    record
        .config
        .insert("obs_name".to_string(), config.obs_name.clone());
    record
        .config
        .insert("seed".to_string(), config.seed.to_string());

    let mut rng = StdRng::seed_from_u64(config.seed);

    // ================================================================
    // STEP 1: GENERATE
    // ================================================================
    let structured_pool = create_structured_rules();
    let destructive_pool = create_destructive_rules();

    let ratios = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
    let all_subsets = generate_mixed_rule_subsets(
        &structured_pool,
        &destructive_pool,
        config.n_train + config.n_test,
        5,
        &ratios,
        &mut rng,
    );

    // Shuffle and split
    let mut indices: Vec<usize> = (0..all_subsets.len()).collect();
    for i in (1..indices.len()).rev() {
        let j = rng.random_range(0..=i);
        indices.swap(i, j);
    }

    let train_subsets: Vec<_> = indices[..config.n_train]
        .iter()
        .map(|&i| all_subsets[i].clone())
        .collect();
    let test_subsets: Vec<_> = indices[config.n_train..config.n_train + config.n_test]
        .iter()
        .map(|&i| all_subsets[i].clone())
        .collect();

    // Generate state pool
    let state_pool: Vec<BinaryGraphState> = (0..500)
        .map(|_| BinaryGraphState::random(config.n_vertices, &mut rng))
        .collect();

    // ================================================================
    // STEP 2: CALIBRATE
    // ================================================================
    let calibration = calibrate(
        config.n_null_universes,
        config.n_vertices,
        config.n_ensemble,
        config.steps,
        config.window_size,
        5, // max_rules_per_subset
        &observe_windowed,
        95.0,
        config.max_delta,
        config.n_shuffles,
        config.seed,
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
    let storage_threshold = calibration.storage_threshold;

    for (i, (rules, ratio)) in train_subsets.iter().enumerate() {
        // Select initial states
        let initial_states: Vec<BinaryGraphState> =
            state_pool.iter().take(config.n_ensemble).cloned().collect();

        // Generate ensemble
        let ensemble = generate_ensemble(
            &initial_states,
            rules,
            config.steps,
            config.n_ensemble,
            config.window_size,
            &DEFAULT_SCHEDULE,
            &observe_windowed,
            config.seed + i as u64 * 137,
        );

        // Compute metrics
        let storage = compute_storage(&ensemble, config.max_delta, config.n_shuffles, config.seed);
        let memory = compute_memory(&ensemble, config.max_delta, config.n_shuffles, config.seed);
        // Persistence at Δ=1 (documented as unreliable with small ensembles)
        let persistence =
            crate::metrics::compute_persistence(&ensemble, 1, config.n_shuffles, config.seed);

        record.results.push(UniverseResult {
            universe_id: i,
            structured_ratio: *ratio,
            n_rules: rules.len(),
            n_structured: rules
                .iter()
                .filter(|r| r.rule_type() == "structured")
                .count(),
            rule_names: rules.iter().map(|r| r.name().to_string()).collect(),
            persistence,
            storage,
            memory,
        });
    }

    // ================================================================
    // STEP 4: HYPOTHESIZE & TEST
    // ================================================================
    let mut hypotheses = generate_standard_hypotheses();

    // Prepare test data
    let mut test_data = Vec::new();
    for (rules, _ratio) in &test_subsets {
        let initial_states: Vec<BinaryGraphState> =
            state_pool.iter().take(config.n_ensemble).cloned().collect();

        let ensemble = generate_ensemble(
            &initial_states,
            rules,
            config.steps,
            config.n_ensemble,
            config.window_size,
            &DEFAULT_SCHEDULE,
            &observe_windowed,
            config.seed + 10000 + test_data.len() as u64 * 137,
        );

        test_data.push((rules.as_slice(), ensemble));
    }

    // Build typed test data references
    let test_refs: Vec<TestDataEntry> = test_data
        .iter()
        .map(|(rules, ensemble)| (*rules, ensemble.as_ref()))
        .collect();

    // Metric map
    let mut metric_map: HashMap<String, Box<MetricFn>> = HashMap::new();
    let max_delta = config.max_delta;
    let n_shuffles = config.n_shuffles;
    let seed = config.seed;

    metric_map.insert(
        "persistence".to_string(),
        Box::new(move |trajs: &[Vec<Vec<u8>>]| -> f64 {
            crate::metrics::compute_persistence(trajs, 1, n_shuffles, seed)
        }),
    );
    metric_map.insert(
        "storage".to_string(),
        Box::new(move |trajs: &[Vec<Vec<u8>>]| -> f64 {
            compute_storage(trajs, max_delta, n_shuffles, seed)
        }),
    );
    metric_map.insert(
        "memory".to_string(),
        Box::new(move |trajs: &[Vec<Vec<u8>>]| -> f64 {
            compute_memory(trajs, max_delta, n_shuffles, seed)
        }),
    );

    test_all_hypotheses(&mut hypotheses, &test_refs, &metric_map, &record.thresholds);

    record.hypotheses = hypotheses.iter().map(HypothesisRecord::from).collect();

    // ================================================================
    // STEP 5: BOOLEAN REDISCOVERY
    // ================================================================
    let target_functions: HashMap<&str, TruthTable> = HashMap::from([
        (
            "NAND",
            vec![((0, 0), 1), ((0, 1), 1), ((1, 0), 1), ((1, 1), 0)],
        ),
        (
            "NOR",
            vec![((0, 0), 1), ((0, 1), 0), ((1, 0), 0), ((1, 1), 0)],
        ),
        (
            "AND",
            vec![((0, 0), 0), ((0, 1), 0), ((1, 0), 0), ((1, 1), 1)],
        ),
        (
            "OR",
            vec![((0, 0), 0), ((0, 1), 1), ((1, 0), 1), ((1, 1), 1)],
        ),
        (
            "XOR",
            vec![((0, 0), 0), ((0, 1), 1), ((1, 0), 1), ((1, 1), 0)],
        ),
    ]);

    // Test high-structure universes (ratio >= 0.4)
    for (rules, _ratio) in train_subsets.iter().filter(|(_, r)| *r >= 0.4).take(200) {
        for (gate_name, truth_table) in &target_functions {
            if test_boolean_function(rules, truth_table, 8, 5, &DEFAULT_SCHEDULE) {
                *record
                    .boolean_discoveries
                    .entry(gate_name.to_string())
                    .or_insert(0) += 1;
            }
        }
    }

    // ================================================================
    // STEP 6: FAILURE CONDITION CHECK
    // ================================================================
    let nand_count = record.boolean_discoveries.get("NAND").copied().unwrap_or(0);

    if record.n_storage() == 0 {
        record
            .failure_conditions
            .push("F-1 (NULL): No storage universes found.".to_string());
    }

    if nand_count == 0 {
        record
            .failure_conditions
            .push("F-1 (NULL): NAND not rediscovered.".to_string());
    }

    let surviving = surviving_hypotheses(&hypotheses);
    if surviving.is_empty() {
        record
            .failure_conditions
            .push("F-6 (DISCONFIRMATION): No hypotheses survived.".to_string());
    }

    // ================================================================
    // FINALIZE
    // ================================================================
    record.elapsed_seconds = t0.elapsed().as_secs_f64();

    // Print summary
    println!("{}", record.summary());

    // Print spectrum if we have storage results
    if record.n_storage() > 0 {
        let spectrum = compute_spectrum(&record.results, storage_threshold);
        println!("\nStorage Spectrum:");
        for label in &[
            "Noise",
            "Noise-dominated",
            "Balanced",
            "Structure-dominated",
            "Structured",
        ] {
            if let Some(bucket) = spectrum.get(*label) {
                println!(
                    "  {:<20} n={:<4} storage={:<6.1}% mean={:.4}",
                    label, bucket.n, bucket.storage_pct, bucket.mean_storage,
                );
            }
        }
    }

    record
}

// ===================================================================
// Quick-start
// ===================================================================

/// Run the scientific cycle with default parameters.
pub fn quick_start() -> ResearchRecord {
    run_cycle(&CycleConfig::default())
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_record_default() {
        let record = ResearchRecord::default();
        assert!(!record.version.is_empty());
        assert_eq!(record.results.len(), 0);
        assert_eq!(record.hypotheses.len(), 0);
    }

    #[test]
    fn test_record_counts() {
        let mut record = ResearchRecord::default();
        record.thresholds.insert("storage".to_string(), 0.5);

        record.results.push(UniverseResult {
            universe_id: 0,
            structured_ratio: 1.0,
            n_rules: 3,
            n_structured: 3,
            rule_names: vec!["R0".to_string()],
            persistence: 0.0,
            storage: 0.8,
            memory: 0.8,
        });
        record.results.push(UniverseResult {
            universe_id: 1,
            structured_ratio: 0.0,
            n_rules: 2,
            n_structured: 0,
            rule_names: vec!["D0".to_string()],
            persistence: 0.0,
            storage: 0.1,
            memory: 0.1,
        });

        assert_eq!(record.n_storage(), 1);
    }

    #[test]
    fn test_cycle_config_default() {
        let config = CycleConfig::default();
        assert_eq!(config.n_train, 300);
        assert_eq!(config.n_vertices, 3);
        assert_eq!(config.n_ensemble, 10);
    }

    #[test]
    fn test_quick_start_runs() {
        // Override config for fast test
        let config = CycleConfig {
            n_train: 20,
            n_test: 5,
            n_vertices: 3,
            n_ensemble: 4,
            steps: 10,
            window_size: 1,
            obs_name: "compound".to_string(),
            max_delta: 5,
            n_shuffles: 3,
            n_null_universes: 5,
            seed: 42,
        };
        let record = run_cycle(&config);
        assert!(record.results.len() >= 20);
        assert!(!record.thresholds.is_empty());
        assert!(!record.hypotheses.is_empty());
    }
}
