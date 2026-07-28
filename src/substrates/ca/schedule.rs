//! Cellular automaton schedule implementation.
//!
//! This module provides [`SynchronousCASchedule`], a synchronous
//! update schedule where all cells are updated simultaneously from
//! the same pre-timestep state. This is the standard schedule for
//! cellular automata.
//!
//! # Comparison with graph schedules
//!
//! Unlike the Binary Graph Universe's [`AllVerticesSchedule`]
//! (asynchronous, vertex-by-vertex, random order), CA schedules
//! are synchronous and deterministic given the rule. There is no
//! vertex ordering or rule competition — the rule applies to
//! every cell simultaneously.
//!
//! # Quick start
//!
//! ```rust
//! use arco::substrates::ca::state::CAState;
//! use arco::substrates::ca::rules::CARule;
//! use arco::substrates::ca::schedule::SynchronousCASchedule;
//! use arco::schedule::Schedule;
//! use rand::rngs::StdRng;
//! use rand::SeedableRng;
//!
//! let state = CAState::<8, 1>::new([0; 8]);
//! let rule = CARule::<8, 1>::from_wolfram_number(110);
//! let schedule = SynchronousCASchedule::new();
//! let mut rng = StdRng::seed_from_u64(42);
//!
//! let next = schedule.step(&state, &[rule], &mut rng);
//! // Rule 110 on all zeros produces all zeros
//! assert_eq!(next.cells(), &[0; 8]);
//! ```

use crate::schedule::Schedule;
use crate::substrates::ca::rules::CARule;
use crate::substrates::ca::state::CAState;
use rand::Rng;

// ===================================================================
// Synchronous Schedule
// ===================================================================

/// A synchronous update schedule for cellular automata.
///
/// All cells are updated simultaneously from the same pre-timestep
/// state. If multiple rules are provided, only the first one is
/// applied. This matches the standard CA semantics: one rule
/// governs the entire lattice.
///
/// # Semantics
/// - Timing: `"synchronous"`
/// - Selection: `"exhaustive"` (all cells updated at once)
///
/// # Why synchronous?
///
/// CA are traditionally synchronous — all cells compute their next
/// state from the current state simultaneously. An asynchronous CA
/// would be a different (and interesting) object of study, but this
/// schedule implements the standard semantics.
#[derive(Debug, Clone, Default)]
pub struct SynchronousCASchedule;

impl SynchronousCASchedule {
    /// Create a new synchronous CA schedule.
    pub fn new() -> Self {
        Self
    }
}

impl<const N: usize, const R: usize> Schedule<CAState<N, R>, CARule<N, R>>
    for SynchronousCASchedule
{
    fn name(&self) -> &str {
        "synchronous_ca"
    }

    fn timing(&self) -> &str {
        "synchronous"
    }

    fn selection(&self) -> &str {
        "exhaustive"
    }

    fn step(
        &self,
        state: &CAState<N, R>,
        rules: &[CARule<N, R>],
        _rng: &mut dyn Rng,
    ) -> CAState<N, R> {
        // Apply the first rule (standard CA: one rule governs all cells).
        // The RNG is ignored — CA rules are deterministic.
        if let Some(rule) = rules.first() {
            rule.apply_sync(state)
        } else {
            state.clone()
        }
    }
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;
    use rand::{SeedableRng, rngs::StdRng};

    #[test]
    fn test_synchronous_schedule_applies_rule() {
        // Rule 110: single 1 at position 0 → should propagate
        let state = CAState::<8, 1>::new([1, 0, 0, 0, 0, 0, 0, 0]);
        let rule = CARule::<8, 1>::from_wolfram_number(110);
        let schedule = SynchronousCASchedule::new();
        let mut rng = StdRng::seed_from_u64(42);

        let next = schedule.step(&state, &[rule], &mut rng);
        // Rule 110 on [1,0,0,0,0,0,0,0]:
        // cell 0: neighbors (0,1,0)=010 → rule[2]=1
        // cell 1: neighbors (1,0,0)=100 → rule[4]=0
        assert_eq!(next.cells()[0], 1);
        assert_eq!(next.cells()[1], 0); // rule 110 propagates right
    }

    #[test]
    fn test_empty_rules_returns_clone() {
        let state = CAState::<8, 1>::new([1, 0, 0, 0, 0, 0, 0, 0]);
        let schedule = SynchronousCASchedule::new();
        let mut rng = StdRng::seed_from_u64(42);

        let next = schedule.step(&state, &[], &mut rng);
        assert_eq!(next.canonical_encoding(), state.canonical_encoding());
    }
}
