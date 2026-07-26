//! Cellular Automata Discovery Experiment
//! ======================================
//!
//! This example demonstrates ARCO's paradigm-neutrality by applying
//! the same emergence metrics used on graph rewriting systems to
//! elementary cellular automata (Wolfram rules 0–255).
//!
//! # What this experiment tests
//!
//! ARCO was validated on a hand-labeled Binary Graph Universe where
//! rules had human-assigned semantic types ("structured" vs
//! "destructive"). This experiment removes human labels entirely.
//! Rules are identified only by their Wolfram rule number (0–255).
//! Structural hypotheses are based on measurable properties
//! (reversibility, parity conservation, sensitivity) — not on names.
//!
//! # The question
//!
//! Does ARCO's storage metric independently recover the known
//! computational taxonomy of elementary cellular automata?
//!
//! # Running
//!
//! ```bash
//! cargo run --example ca_discovery --release
//! ```

use std::{
    fmt,
    hash::{Hash, Hasher},
};

use arco::{
    calibration::generate_trajectories,
    metrics::compute_storage,
    observation::Observation,
    rules::{NoContext, Rule},
    schedule::Schedule,
    state::State,
};
use rand::{Rng, RngExt, SeedableRng, rngs::StdRng};

/// Number of cells in the 1D automaton.
pub const N_CELLS: usize = 8;

fn main() {
    println!("{}", "=".repeat(70));
    println!("ARCO Cellular Automata Discovery Experiment");
    println!("{}", "=".repeat(70));
    println!();
    println!("Evaluating all 256 elementary cellular automaton rules");
    println!("State size: {} cells ({} states)", N_CELLS, 1 << N_CELLS);
    println!("Ensemble: 10 trajectories, 30 steps each");
    println!();

    // Read seed from command line, default to 42
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    let n_ensemble = 10;
    let steps = 30;
    let max_delta = 10;
    let n_shuffles = 10;

    // Phase 1: Compute storage for all 256 rules
    println!("Phase 1: Computing storage for all 256 rules...");

    let mut results: Vec<RuleResult> = Vec::with_capacity(256);

    for rule_number in 0..=255u8 {
        let rule = CARule::new(rule_number);
        let ensemble = generate_ca_ensemble(&rule, steps, n_ensemble, seed);

        let storage = compute_storage(&ensemble, max_delta, n_shuffles, seed);

        let reversible = rule.is_reversible();
        let conserves_parity = rule.conserves_parity();
        let sensitivity = rule.sensitivity();

        results.push(RuleResult {
            rule_number,
            storage,
            reversible,
            conserves_parity,
            sensitivity,
        });

        if rule_number % 32 == 0 {
            println!("  Progress: {}/256 rules evaluated", rule_number);
        }
    }

    println!("  Complete: 256/256 rules evaluated");
    println!();

    // Phase 2: Calibrate threshold
    println!("Phase 2: Calibrating storage threshold...");

    let mut sorted_by_storage: Vec<&RuleResult> = results.iter().collect();
    sorted_by_storage.sort_by(|a, b| a.storage.partial_cmp(&b.storage).unwrap());

    let null_storage: Vec<f64> = sorted_by_storage
        .iter()
        .take(30)
        .map(|r| r.storage)
        .collect();

    let null_mean = null_storage.iter().sum::<f64>() / null_storage.len() as f64;
    let null_std = (null_storage
        .iter()
        .map(|s| (s - null_mean).powi(2))
        .sum::<f64>()
        / (null_storage.len() - 1) as f64)
        .sqrt();

    let mut sorted_null = null_storage.clone();
    sorted_null.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let threshold_idx = ((95.0 / 100.0) * (sorted_null.len() - 1) as f64).round() as usize;
    let threshold = sorted_null[threshold_idx].max(0.01);

    println!("  Null mean:   {:.4}", null_mean);
    println!("  Null std:    {:.4}", null_std);
    println!("  Threshold:   {:.4}", threshold);
    println!();

    // Phase 3: Classify rules
    println!("Phase 3: Classifying rules by storage...");
    println!();

    let high_storage: Vec<&RuleResult> = results.iter().filter(|r| r.storage > threshold).collect();
    let low_storage: Vec<&RuleResult> = results.iter().filter(|r| r.storage <= threshold).collect();

    println!(
        "  High storage (above {:.3}): {} rules",
        threshold,
        high_storage.len()
    );
    println!(
        "  Low storage (below {:.3}):  {} rules",
        threshold,
        low_storage.len()
    );
    println!();

    // Show famous rules
    println!("  Storage for well-known rules:");
    println!(
        "  {:<10} {:<12} {:<15} {}",
        "Rule", "Storage", "Wolfram Class", "Known Property"
    );
    println!(
        "  {:<10} {:<12} {:<15} {}",
        "----", "-------", "-------------", "---------------"
    );

    let famous = [
        (0, "Class 1", "Fixed point (all 0)"),
        (30, "Class 3", "Chaotic"),
        (54, "Class 4", "Complex, Turing-complete"),
        (90, "Class 2", "Sierpinski triangle"),
        (110, "Class 4", "Turing-complete"),
        (184, "Class 2", "Traffic flow model"),
        (255, "Class 1", "Fixed point (all 1)"),
    ];

    for &(rule_num, wolfram_class, property) in &famous {
        if let Some(result) = results.iter().find(|r| r.rule_number == rule_num) {
            println!(
                "  Rule {:<3}   {:<12.4} {:<15} {}",
                rule_num, result.storage, wolfram_class, property
            );
        }
    }
    println!();

    // Phase 4: Test structural hypotheses
    println!("Phase 4: Testing structural hypotheses...");
    println!();
    println!(
        "  {:<35} {:<10} {:<10} {}",
        "Hypothesis", "Accuracy", "Survives?", "Description"
    );
    println!(
        "  {:<35} {:<10} {:<10} {}",
        "----------", "--------", "---------", "-----------"
    );

    test_hypothesis("H1: Reversible → Storage", &results, threshold, |r| {
        r.reversible
    });
    test_hypothesis(
        "H2: Parity-conserving → Storage",
        &results,
        threshold,
        |r| r.conserves_parity,
    );
    test_hypothesis(
        "H3: Low sensitivity (< 2.0) → Storage",
        &results,
        threshold,
        |r| r.sensitivity < 2.0,
    );
    test_hypothesis(
        "H4: Even rule number → Storage",
        &results,
        threshold,
        |r| r.rule_number % 2 == 0,
    );
    test_hypothesis(
        "H5: Not Rule 0 → Storage (weak)",
        &results,
        threshold,
        |r| r.rule_number != 0,
    );

    println!();

    // Phase 5: Visualize trajectories
    println!("Phase 5: Visualizing example trajectories...");
    println!();

    for &rule_num in &[110, 30, 90, 184] {
        if results.iter().any(|r| r.rule_number == rule_num) {
            let rule = CARule::new(rule_num);
            let mut cells = vec![0u8; N_CELLS];
            cells[N_CELLS / 2] = 1;
            let mut state = CAState::new(cells);

            let storage = results
                .iter()
                .find(|r| r.rule_number == rule_num)
                .unwrap()
                .storage;
            println!("  Rule {} (storage: {:.4}):", rule_num, storage);
            for _step in 0..10 {
                println!("    {}", state);
                state = rule.apply_sync(&state);
            }
            println!();
        }
    }

    println!("{}", "=".repeat(70));
    println!("Experiment complete.");
    println!("{}", "=".repeat(70));
}

// ===================================================================
// CA State
// ===================================================================

/// A state in a 1D binary cellular automaton with periodic boundaries.
#[derive(Clone)]
pub struct CAState {
    cells: Vec<u8>,
}

impl CAState {
    pub fn new(cells: Vec<u8>) -> Self {
        assert_eq!(cells.len(), N_CELLS);
        for &c in &cells {
            assert!(c <= 1);
        }
        Self { cells }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        let cells: Vec<u8> = (0..N_CELLS).map(|_| rng.random_range(0..=1)).collect();
        Self { cells }
    }

    pub fn cell(&self, index: i32) -> u8 {
        let i = index.rem_euclid(N_CELLS as i32) as usize;
        self.cells[i]
    }

    pub fn cells(&self) -> &[u8] {
        &self.cells
    }

    pub fn neighborhood(&self, i: usize) -> usize {
        let left = self.cell(i as i32 - 1) as usize;
        let center = self.cells[i] as usize;
        let right = self.cell(i as i32 + 1) as usize;
        (left << 2) | (center << 1) | right
    }
}

impl State for CAState {
    type Encoding = Vec<u8>;

    fn canonical_encoding(&self) -> Self::Encoding {
        self.cells.clone()
    }

    fn distance(&self, other: &Self) -> u32 {
        self.cells
            .iter()
            .zip(other.cells.iter())
            .map(|(a, b)| if a != b { 1 } else { 0 })
            .sum()
    }
}

impl PartialEq for CAState {
    fn eq(&self, other: &Self) -> bool {
        self.cells == other.cells
    }
}

impl Eq for CAState {}

impl Hash for CAState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cells.hash(state);
    }
}

impl fmt::Debug for CAState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CAState({})", self)
    }
}

impl fmt::Display for CAState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &c in &self.cells {
            write!(f, "{}", if c == 1 { '█' } else { ' ' })?;
        }
        Ok(())
    }
}

// ===================================================================
// CA Rule
// ===================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CARule {
    rule_number: u8,
    table: [u8; 8],
}

impl CARule {
    pub fn new(rule_number: u8) -> Self {
        let mut table = [0u8; 8];
        for (i, entry) in table.iter_mut().enumerate() {
            *entry = (rule_number >> i) & 1;
        }
        Self { rule_number, table }
    }

    pub fn rule_number(&self) -> u8 {
        self.rule_number
    }

    pub fn apply_to_neighborhood(&self, neighborhood: usize) -> u8 {
        self.table[neighborhood]
    }

    pub fn apply_sync(&self, state: &CAState) -> CAState {
        let mut new_cells = Vec::with_capacity(N_CELLS);
        for i in 0..N_CELLS {
            let neighborhood = state.neighborhood(i);
            new_cells.push(self.apply_to_neighborhood(neighborhood));
        }
        CAState::new(new_cells)
    }

    // Measurable properties
    pub fn is_reversible(&self) -> bool {
        let mut seen = vec![false; 256];
        for bits in 0..256u16 {
            let cells: Vec<u8> = (0..N_CELLS).map(|i| ((bits >> i) & 1) as u8).collect();
            let state = CAState::new(cells);
            let next = self.apply_sync(&state);
            let next_bits: usize = next
                .cells()
                .iter()
                .enumerate()
                .map(|(i, &c)| (c as usize) << i)
                .sum();
            if seen[next_bits] {
                return false;
            }
            seen[next_bits] = true;
        }
        true
    }

    pub fn conserves_parity(&self) -> bool {
        for bits in 0..256u16 {
            let cells: Vec<u8> = (0..N_CELLS).map(|i| ((bits >> i) & 1) as u8).collect();
            let state = CAState::new(cells);
            let next = self.apply_sync(&state);
            let parity_before: u32 = state.cells().iter().map(|&c| c as u32).sum::<u32>() % 2;
            let parity_after: u32 = next.cells().iter().map(|&c| c as u32).sum::<u32>() % 2;
            if parity_before != parity_after {
                return false;
            }
        }
        true
    }

    pub fn sensitivity(&self) -> f64 {
        let mut total_diff = 0u32;
        let mut count = 0u32;
        for bits in 0..256u16 {
            let cells: Vec<u8> = (0..N_CELLS).map(|i| ((bits >> i) & 1) as u8).collect();
            let state = CAState::new(cells);
            for flip in 0..N_CELLS {
                let mut flipped_cells = state.cells().to_vec();
                flipped_cells[flip] = 1 - flipped_cells[flip];
                let flipped_state = CAState::new(flipped_cells);
                let next_orig = self.apply_sync(&state);
                let next_flip = self.apply_sync(&flipped_state);
                total_diff += next_orig.distance(&next_flip);
                count += 1;
            }
        }
        total_diff as f64 / count as f64
    }
}

impl Rule<CAState> for CARule {
    type Context = NoContext;

    fn name(&self) -> &str {
        // Return a static string for each rule number — we can't return
        // a formatted string from a reference. Use a fixed string and
        // identify rules by their number separately.
        "CA Rule"
    }

    fn apply(&self, state: &CAState, _context: &NoContext, _rng: &mut dyn Rng) -> CAState {
        self.apply_sync(state)
    }
}

// ===================================================================
// CA Observer
// ===================================================================

#[derive(Debug, Clone, Default)]
pub struct CAObserver;

impl Observation<CAState> for CAObserver {
    type Output = Vec<u8>;

    fn observe(&self, state: &CAState) -> Self::Output {
        state.canonical_encoding()
    }
}

// ===================================================================
// Synchronous Schedule
// ===================================================================

#[derive(Debug, Clone)]
pub struct SynchronousCASchedule;

impl SynchronousCASchedule {
    pub fn new() -> Self {
        Self
    }
}

impl Schedule<CAState, CARule> for SynchronousCASchedule {
    fn name(&self) -> &str {
        "synchronous_ca"
    }
    fn timing(&self) -> &str {
        "synchronous"
    }
    fn selection(&self) -> &str {
        "exhaustive"
    }

    fn step(&self, state: &CAState, rules: &[CARule], _rng: &mut dyn Rng) -> CAState {
        // In CA, there's typically one rule applied to all cells.
        // If multiple rules are provided, apply the first one.
        if let Some(rule) = rules.first() {
            rule.apply_sync(state)
        } else {
            state.clone()
        }
    }
}

// ===================================================================
// Trajectory generation using ARCO's generic pipeline
// ===================================================================

pub fn generate_ca_ensemble(
    rule: &CARule,
    steps: usize,
    n_ensemble: usize,
    seed: u64,
) -> Vec<Vec<Vec<u8>>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let schedule = SynchronousCASchedule::new();
    let observer = CAObserver;

    let initial_states: Vec<CAState> = (0..n_ensemble).map(|_| CAState::random(&mut rng)).collect();

    let rules = vec![rule.clone()];

    generate_trajectories(&initial_states, &rules, steps, &schedule, &observer, seed)
}

// ===================================================================
// Data Structures
// ===================================================================

pub struct RuleResult {
    pub rule_number: u8,
    pub storage: f64,
    pub reversible: bool,
    pub conserves_parity: bool,
    pub sensitivity: f64,
}

pub fn test_hypothesis(
    name: &str,
    results: &[RuleResult],
    threshold: f64,
    condition: impl Fn(&RuleResult) -> bool,
) {
    let qualifying: Vec<&RuleResult> = results.iter().filter(|r| condition(r)).collect();
    if qualifying.is_empty() {
        println!(
            "  {:<35} {:<10} {:<10} (no qualifying rules)",
            name, "N/A", "N/A"
        );
        return;
    }

    let correct = qualifying.iter().filter(|r| r.storage > threshold).count();
    let accuracy = correct as f64 / qualifying.len() as f64;
    let survives = accuracy > 0.5;

    println!(
        "  {:<35} {:<10.1}% {:<10} {}",
        name,
        accuracy * 100.0,
        if survives { "✓ YES" } else { "✗ NO" },
        format!("({}/{})", correct, qualifying.len()),
    );
}
