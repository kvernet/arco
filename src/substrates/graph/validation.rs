//! Boolean function validation for the Binary Graph Universe.
//!
//! This module tests whether rule sets can implement basic logic
//! gates (NAND, AND, OR, NOR, XOR) under stochastic, asynchronous,
//! first-match scheduling. This is a verification, not a discovery —
//! the gates are hand-coded in the structured rule pool.
//!
//! The test measures whether computation emerges *robustly* despite
//! random vertex ordering and rule competition.

use ndarray::{arr1, arr2};
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::schedule::Schedule;
use crate::substrates::graph::rules::RewriteRule;
use crate::substrates::graph::schedule::AllVerticesSchedule;
use crate::substrates::graph::state::BinaryGraphState;
use crate::types::TruthTable;

/// Test whether a rule set implements a 2-input Boolean function.
///
/// Sets up a 3-vertex graph where vertices 0 and 1 hold inputs and
/// vertex 2 is the output. Edges 0→2 and 1→2 are created so logic
/// gate rules (which require incoming edges) can fire.
///
/// The function is tested on all 4 input combinations with multiple
/// trials per combination. Majority vote determines the output.
///
/// # Arguments
/// * `rules` — The rule set to test.
/// * `truth_table` — Expected outputs: `&[((0,0), 1), ((0,1), 1), ...]`.
/// * `steps` — Timesteps per trial.
/// * `n_trials` — Trials per input combination.
///
/// # Returns
/// `true` if the rule set implements the truth table under majority
/// voting across trials.
///
/// # Notes
/// Random vertex order, random rule order, and first-match semantics
/// mean a rule set containing NAND may fail to implement NAND if
/// another unconditional rule fires first. Multiple trials with
/// majority voting handle this nondeterminism.
pub fn test_boolean_function(
    rules: &[RewriteRule],
    truth_table: &TruthTable,
    steps: usize,
    n_trials: usize,
) -> bool {
    let schedule = AllVerticesSchedule::new();

    for &((a, b), expected) in truth_table {
        let mut results = Vec::new();

        for trial in 0..n_trials {
            let adj = arr2(&[[0, 0, 1], [0, 0, 1], [0, 0, 0]]);
            let labels = arr1(&[a as i8, b as i8, 0i8]);
            let mut state = BinaryGraphState::new(3, adj.view(), labels.view())
                .expect("Boolean test state creation");

            let mut rng = StdRng::seed_from_u64((trial * 100) as u64);

            for _ in 0..steps {
                state = schedule.step(&state, rules, &mut rng);
            }

            results.push(state.label(2));
        }

        // Majority vote
        let ones = results.iter().filter(|&&x| x == 1).count();
        let observed = if ones > results.len() / 2 { 1 } else { 0 };

        if observed != expected {
            return false;
        }
    }

    true
}

/// Verify which of the five basic Boolean functions a rule set implements.
///
/// Tests NAND, NOR, AND, OR, and XOR under stochastic scheduling.
/// Returns the names of gates that passed verification.
///
/// This is a validation, not a discovery — the gates are hand-coded
/// in the structured rule pool. The test confirms they function
/// correctly despite random vertex ordering and rule competition.
pub fn verify_boolean_functions(
    rules: &[RewriteRule],
    steps: usize,
    n_trials: usize,
) -> Vec<String> {
    let gates: &[(&str, &TruthTable)] = &[
        (
            "NAND",
            &[((0, 0), 1), ((0, 1), 1), ((1, 0), 1), ((1, 1), 0)],
        ),
        ("NOR", &[((0, 0), 1), ((0, 1), 0), ((1, 0), 0), ((1, 1), 0)]),
        ("AND", &[((0, 0), 0), ((0, 1), 0), ((1, 0), 0), ((1, 1), 1)]),
        ("OR", &[((0, 0), 0), ((0, 1), 1), ((1, 0), 1), ((1, 1), 1)]),
        ("XOR", &[((0, 0), 0), ((0, 1), 1), ((1, 0), 1), ((1, 1), 0)]),
    ];

    let mut verified = Vec::new();
    for (name, truth_table) in gates {
        if test_boolean_function(rules, truth_table, steps, n_trials) {
            verified.push(name.to_string());
        }
    }
    verified
}
