//! Graph-specific schedules for the Binary Graph Universe.
//!
//! This module provides [`AllVerticesSchedule`], the asynchronous
//! exhaustive schedule used throughout the Binary Graph experiments.
//! It implements the core [`Schedule`] trait.

use rand::Rng;
use rand::seq::SliceRandom;

use crate::rules::Rule;
use crate::schedule::Schedule;
use crate::substrates::graph::rules::RewriteRule;
use crate::substrates::graph::state::BinaryGraphState;

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
/// # Semantics
/// - Timing: `"asynchronous"`
/// - Selection: `"exhaustive"`
///
/// This is the schedule used throughout the Binary Graph Universe
/// experiments and the Python reference implementation.
///
/// # Context
///
/// This schedule creates [`MatchInfo`] contexts internally by calling
/// `rule.matches()` before `rule.apply()`. Users never need to
/// construct contexts manually.
///
/// # Examples
///
/// ```rust
/// use arco::schedule::Schedule;
/// use arco::substrates::graph::schedule::AllVerticesSchedule;
/// use arco::substrates::graph::{
///     BinaryGraphState, create_destructive_rules, create_structured_rules,
///     rules::generate_mixed_rule_subsets,
/// };
/// use ndarray::{arr1, arr2};
/// use rand::{SeedableRng, rngs::StdRng};
///
/// fn main() {
///     let mut rng = StdRng::seed_from_u64(42);
///
///     let mut state = BinaryGraphState::new(
///         3,
///         arr2(&[[0, 0, 1], [0, 0, 1], [0, 0, 0]]).view(),
///         arr1(&[1, 1, 0]).view(),
///     )
///     .unwrap();
///
///     println!("{}", state);
///
///     let structured = create_structured_rules();
///     let destructive = create_destructive_rules();
///     let n_subsets = 7;
///     let max_size = 5;
///     let ratios = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
///
///     let subsets = generate_mixed_rule_subsets(
///         &structured,
///         &destructive,
///         n_subsets,
///         max_size,
///         &ratios,
///         &mut rng,
///     );
///
///     for (rules, _) in &subsets {
///         let schedule = AllVerticesSchedule::new();
///
///         state = schedule.step(&state, rules, &mut rng);
///     }
///
///     println!("{}", state);
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct AllVerticesSchedule;

impl AllVerticesSchedule {
    /// Create a new AllVerticesSchedule.
    pub fn new() -> Self {
        Self
    }
}

impl Schedule<BinaryGraphState, RewriteRule> for AllVerticesSchedule {
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
        rng: &mut dyn Rng,
    ) -> BinaryGraphState {
        let mut current = state.clone();
        let n = state.n_vertices();
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
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;
    use crate::substrates::graph::rules::create_structured_rules;
    use ndarray::{arr1, arr2};
    use rand::SeedableRng;
    use rand::rngs::StdRng;

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
        assert_eq!(new_state.n_vertices(), 3);
        for i in 0..3 {
            assert!(new_state.label(i) <= 1);
        }
    }

    #[test]
    fn test_schedule_is_deterministic_given_seed() {
        let state = make_test_state();
        let rules = create_structured_rules();
        let schedule = AllVerticesSchedule::new();

        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);

        let result1 = schedule.step(&state, &rules, &mut rng1);
        let result2 = schedule.step(&state, &rules, &mut rng2);

        assert_eq!(result1.canonical_encoding(), result2.canonical_encoding());
    }
}
