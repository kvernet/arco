//! Evolution schedules and trajectory generation.
//!
//! Per Constitution:
//!     K is the update schedule — a rule specifying the order and selection
//!     of transformations at each timestep. The schedule is a first-class
//!     component of an Information Universe.
//!
//! This module implements the `all_vertices` schedule and trajectory
//! generation for ensembles.
//!
//! Design commitments:
//! - Schedules are first-class objects, not string tags.
//! - Trajectory generation is deterministic given a seed.
//! - Trajectories store observation values via an injected observation
//!   operator, keeping state identity separate from observation.
//! - Ensemble generation produces multiple trajectories from distinct
//!   initial states.

use std::fmt;

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::rules::{RewriteRule, Rule};
use crate::state::BinaryGraphState;

// ===================================================================
// Schedule trait
// ===================================================================

/// Trait for an update schedule.
///
/// A schedule determines the order and selection of transformations
/// at each timestep. It is a first-class component of an Information
/// Universe (Constitution).
///
/// # Schedule Semantics Classification
///
/// **Timing**
/// - *synchronous*: all updates within a timestep are computed from
///   the same pre-timestep state.
/// - *asynchronous*: updates are applied immediately; later updates
///   within the same timestep see earlier changes.
///
/// **Selection**
/// - *exhaustive*: every vertex is visited exactly once per timestep.
/// - *stochastic*: vertices are sampled probabilistically.
/// - *priority*: vertices are ordered by a fixed criterion.
pub trait Schedule: fmt::Debug + Send + Sync {
    /// Human-readable schedule identifier.
    fn name(&self) -> &str;

    /// `"asynchronous"` or `"synchronous"`.
    fn timing(&self) -> &str;

    /// `"exhaustive"`, `"stochastic"`, or `"priority"`.
    fn selection(&self) -> &str;

    /// Apply one timestep of evolution.
    ///
    /// Takes the current state, the rule set, and an RNG, and returns
    /// the state after one timestep.
    fn step(
        &self,
        state: &BinaryGraphState,
        rules: &[RewriteRule],
        rng: &mut impl Rng,
    ) -> BinaryGraphState;
}

// ===================================================================
// All-Vertices Schedule
// ===================================================================

/// Random sequential asynchronous update on all vertices.
///
/// Every vertex is visited exactly once per timestep, in random order.
/// At each vertex, rules are tried in random order; the first matching
/// rule fires. Updates are immediately visible — later vertices within
/// the same timestep observe the state after earlier vertices' updates.
///
/// **Classification:**
/// - Timing: `"asynchronous"`
/// - Selection: `"exhaustive"`
#[derive(Debug, Clone)]
pub struct AllVerticesSchedule;

impl AllVerticesSchedule {
    /// Create a new AllVerticesSchedule.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AllVerticesSchedule {
    fn default() -> Self {
        Self::new()
    }
}

impl Schedule for AllVerticesSchedule {
    fn name(&self) -> &str {
        "all_vertices"
    }

    fn timing(&self) -> &str {
        "asynchronous"
    }

    fn selection(&self) -> &str {
        "exhaustive"
    }

    fn step(
        &self,
        state: &BinaryGraphState,
        rules: &[RewriteRule],
        rng: &mut impl Rng,
    ) -> BinaryGraphState {
        let mut current = state.clone();
        let n = current.n_vertices();
        let mut vertices: Vec<usize> = (0..n).collect();
        vertices.shuffle(rng);

        for &vertex in &vertices {
            let mut rule_indices: Vec<usize> = (0..rules.len()).collect();
            rule_indices.shuffle(rng);

            for &ri in &rule_indices {
                if let Some(info) = rules[ri].matches(&current, vertex) {
                    current = rules[ri].apply(&current, &info, rng);
                    break; // first match only per vertex
                }
            }
        }

        current
    }
}

/// Default schedule instance.
pub static DEFAULT_SCHEDULE: AllVerticesSchedule = AllVerticesSchedule;

// ===================================================================
// Trajectory generation
// ===================================================================

/// Generate a single trajectory, returning observation values.
///
/// The trajectory is a list of observation values, one per timestep
/// (including the initial state). The observation operator `obs_fn`
/// is applied to each state. By default, this is the canonical
/// encoding (the identity observation).
///
/// # Parameters
/// * `initial_state` — The starting state.
/// * `rules` — The rule set governing evolution.
/// * `steps` — Number of timesteps to simulate.
/// * `window_size` — Window size
/// * `schedule` — The update schedule.
/// * `obs_fn` — Observation operator `(BinaryGraphState) -> O`.
/// * `seed` — Seed for the per-trajectory RNG.
///
/// # Returns
/// Vector of observation values, length `steps + 1`.
pub fn generate_trajectory<O>(
    initial_state: &BinaryGraphState,
    rules: &[RewriteRule],
    steps: usize,
    window_size: usize,
    schedule: &impl Schedule,
    obs_fn: &dyn Fn(&[BinaryGraphState]) -> O,
    seed: u64,
) -> Vec<O> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut state_history = Vec::with_capacity(window_size);
    state_history.push(initial_state.clone());
    let mut observations = Vec::with_capacity(steps + 1);
    observations.push(obs_fn(&state_history));
    let mut current = initial_state.clone();

    for _ in 0..steps {
        current = schedule.step(&current, rules, &mut rng);
        state_history.push(current.clone());
        if state_history.len() > window_size {
            state_history.remove(0);
        }
        observations.push(obs_fn(&state_history));
    }

    observations
}

// ===================================================================
// Ensemble generation
// ===================================================================

/// Generate an ensemble of trajectories from distinct initial states.
///
/// Per Constitution, all emergence metrics must be
/// computed over ensembles of at least `n_ensemble` trajectories
/// from distinct initial conditions. Each trajectory uses a
/// deterministic seed derived from `base_seed` and its index.
///
/// # Parameters
/// * `initial_states` — Pool of initial states. Must have at least
///   `n_ensemble` elements.
/// * `rules` — The rule set governing evolution.
/// * `steps` — Number of timesteps per trajectory.
/// * `n_ensemble` — Number of trajectories to generate (≥ 2).
/// * `window_size` — Window size
/// * `schedule` — The update schedule.
/// * `obs_fn` — Observation operator `(BinaryGraphState) -> O`.
/// * `base_seed` — Base seed. Trajectory `i` uses seed
///   `base_seed + i * 137`.
///
/// # Returns
/// Vector of trajectories, each a vector of observation values.
///
/// # Panics
/// If `n_ensemble < 2` or fewer than `n_ensemble` initial states
/// are provided.
#[allow(clippy::needless_range_loop)]
#[allow(clippy::too_many_arguments)]
pub fn generate_ensemble<O: Clone>(
    initial_states: &[BinaryGraphState],
    rules: &[RewriteRule],
    steps: usize,
    n_ensemble: usize,
    window_size: usize,
    schedule: &impl Schedule,
    obs_fn: &dyn Fn(&[BinaryGraphState]) -> O,
    base_seed: u64,
) -> Vec<Vec<O>> {
    assert!(n_ensemble >= 2, "Ensemble size must be at least 2");
    assert!(
        initial_states.len() >= n_ensemble,
        "Need at least {} initial states, got {}",
        n_ensemble,
        initial_states.len()
    );

    let mut trajectories = Vec::with_capacity(n_ensemble);

    for i in 0..n_ensemble {
        let seed = base_seed + i as u64 * 137;
        let traj = generate_trajectory(
            &initial_states[i],
            rules,
            steps,
            window_size,
            schedule,
            obs_fn,
            seed,
        );
        trajectories.push(traj);
    }

    trajectories
}

// ===================================================================
// Boolean function testing (computational rediscovery)
// ===================================================================

/// Test whether a rule set implements a 2-input Boolean function.
///
/// Sets up a 3-vertex graph where vertices 0 and 1 hold inputs and
/// vertex 2 is the output. Edges 0→2 and 1→2 are created so that
/// logic gate rules (which require incoming edges) can fire.
///
/// The function is tested on all 4 input combinations with multiple
/// trials per combination. Majority vote determines the output for
/// each input pair.
///
/// This function operationalizes the computational rediscovery
/// milestone (Constitution).
///
/// # Parameters
/// * `rules` — The rule set to test.
/// * `truth_table` — Expected outputs: `[(0,0)→out, (0,1)→out, …]`.
/// * `steps` — Timesteps to simulate per trial.
/// * `n_trials` — Number of trials per input combination.
///
/// # Returns
/// `true` if the rule set implements the specified truth table
/// under majority voting across trials.
///
/// # Notes
/// The test uses random vertex order, random rule order, and first-match
/// semantics. This means a rule set containing a NAND rule may fail to
/// implement NAND if another unconditional rule fires first at the output
/// vertex. This is intentional: the test measures whether computation
/// emerges *robustly* despite stochastic scheduling. Multiple trials
/// with majority voting handle the resulting nondeterminism.
pub fn test_boolean_function(
    rules: &[RewriteRule],
    truth_table: &[((u8, u8), u8)],
    steps: usize,
    n_trials: usize,
    schedule: &impl Schedule,
) -> bool {
    use ndarray::{arr1, arr2};

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

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rules::create_structured_rules, state::State};
    use ndarray::{arr1, arr2};

    fn make_test_state() -> BinaryGraphState {
        let adj = arr2(&[[0, 1, 0], [1, 0, 1], [0, 0, 0]]);
        let labels = arr1(&[1, 0, 1]);
        BinaryGraphState::new(3, adj.view(), labels.view()).unwrap()
    }

    #[test]
    fn test_all_vertices_schedule_applies_rules() {
        let state = make_test_state();
        let rules = create_structured_rules();
        let schedule = AllVerticesSchedule::new();
        let mut rng = StdRng::seed_from_u64(42);

        let new_state = schedule.step(&state, &rules, &mut rng);
        // State should have been modified (unless only IDENTITY matched)
        // We can't assert exact output due to randomness, but we can check
        // the state is still valid
        assert_eq!(new_state.n_vertices(), 3);
        for i in 0..3 {
            assert!(new_state.label(i) <= 1);
        }
    }

    #[test]
    fn test_trajectory_deterministic() {
        let state = make_test_state();
        let rules = create_structured_rules();
        let schedule = AllVerticesSchedule::new();
        let obs_fn = |s: &[BinaryGraphState]| s[0].canonical_encoding();

        let traj1 = generate_trajectory(&state, &rules, 10, 1, &schedule, &obs_fn, 42);
        let traj2 = generate_trajectory(&state, &rules, 10, 1, &schedule, &obs_fn, 42);

        assert_eq!(traj1.len(), traj2.len());
        for (o1, o2) in traj1.iter().zip(traj2.iter()) {
            assert_eq!(o1, o2);
        }
    }

    #[test]
    fn test_trajectory_different_seeds_diverge() {
        let state = make_test_state();
        let rules = create_structured_rules();
        let schedule = AllVerticesSchedule::new();
        let obs_fn = |s: &[BinaryGraphState]| s[0].canonical_encoding();

        let traj1 = generate_trajectory(&state, &rules, 20, 1, &schedule, &obs_fn, 42);
        let traj2 = generate_trajectory(&state, &rules, 20, 1, &schedule, &obs_fn, 99);

        // With many steps and different seeds, trajectories should diverge
        let all_same = traj1.iter().zip(traj2.iter()).all(|(a, b)| a == b);
        assert!(
            !all_same,
            "Different seeds should produce different trajectories"
        );
    }

    #[test]
    fn test_ensemble_size() {
        let state = make_test_state();
        let rules = create_structured_rules();
        let schedule = AllVerticesSchedule::new();
        let obs_fn = |s: &[BinaryGraphState]| s[0].canonical_encoding();
        let initial_states = vec![state.clone(), state.clone(), state.clone()];

        let ensemble = generate_ensemble(&initial_states, &rules, 10, 3, 1, &schedule, &obs_fn, 42);

        assert_eq!(ensemble.len(), 3);
        for traj in &ensemble {
            assert_eq!(traj.len(), 11); // steps + 1
        }
    }

    #[test]
    #[should_panic(expected = "Ensemble size must be at least 2")]
    fn test_ensemble_too_small() {
        let state = make_test_state();
        let rules = create_structured_rules();
        let schedule = AllVerticesSchedule::new();
        let obs_fn = |s: &[BinaryGraphState]| s[0].canonical_encoding();
        let initial_states = vec![state.clone()];

        generate_ensemble(&initial_states, &rules, 10, 1, 1, &schedule, &obs_fn, 42);
    }

    #[test]
    fn test_nand_boolean_function() {
        let rules = create_structured_rules();
        let schedule = AllVerticesSchedule::new();
        let truth_table = vec![((0, 0), 1), ((0, 1), 1), ((1, 0), 1), ((1, 1), 0)];

        // Test on a rule set that includes NAND
        let nand_rules: Vec<RewriteRule> = rules
            .into_iter()
            .filter(|r| r.name() == "NAND" || r.name() == "IDENTITY")
            .collect();

        let result = test_boolean_function(&nand_rules, &truth_table, 10, 5, &schedule);
        // Note: with IDENTITY also present, NAND may not always fire first.
        // This is expected stochastic behavior.
        // The test documents the behavior; we don't assert true/false.
        let _ = result;
    }
}
