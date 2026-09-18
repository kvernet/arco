//! Standard invariants for the Cellular Automaton substrate.
//!
//! These generalize the ad-hoc exhaustive checks the CA substrate
//! already performs: [`CARule::conserves_parity`] tests, for every
//! state in a `2^N`-state space, whether population parity agrees
//! before and after a rule application. That is exactly an
//! [`Invariant`] evaluation (`I(s) = popcount(s) mod 2`) checked for
//! conservation across the whole state space.

use crate::invariants::Invariant;
use crate::substrates::ca::state::CAState;

// ===================================================================
// PopulationInvariant
// ===================================================================

/// `I(s) = popcount(s)` — the total number of live (`1`) cells.
///
/// Unlike a quantity that is conserved by construction, this one is
/// genuinely rule-dependent: it is exactly conserved by
/// number-conserving elementary CA rules — famously Wolfram Rule 184,
/// the traffic-flow rule — and violated by almost every other rule.
/// Use [`crate::invariants::exhaustively_conserved`] with a
/// universe's state space to check whether a specific rule qualifies.
#[derive(Debug, Clone, Copy, Default)]
pub struct PopulationInvariant;

impl<const N: usize, const R: usize> Invariant<CAState<N, R>> for PopulationInvariant {
    fn name(&self) -> &str {
        "population"
    }

    fn evaluate(&self, state: &CAState<N, R>) -> f64 {
        state.cells().iter().map(|&c| c as f64).sum()
    }
}

// ===================================================================
// ParityInvariant
// ===================================================================

/// `I(s) = popcount(s) mod 2` — population parity.
///
/// Generalizes [`CARule::conserves_parity`](crate::substrates::ca::CARule::conserves_parity)
/// into the standard [`Invariant`] interface. `evaluate` already
/// reduces to `{0, 1}`, so the default exact-conservation check
/// (`tolerance() == 0.0`) correctly captures the mod-2 congruence —
/// no override of `is_conserved` is needed.
#[derive(Debug, Clone, Copy, Default)]
pub struct ParityInvariant;

impl<const N: usize, const R: usize> Invariant<CAState<N, R>> for ParityInvariant {
    fn name(&self) -> &str {
        "parity"
    }

    fn evaluate(&self, state: &CAState<N, R>) -> f64 {
        (state.cells().iter().map(|&c| c as u32).sum::<u32>() % 2) as f64
    }
}

// ===================================================================
// Standard invariant set
// ===================================================================

/// Generate the standard set of invariants for the CA substrate:
/// population count and population parity. Neither is claimed to
/// hold for every rule — whether they do is rule-dependent and
/// checkable via [`crate::invariants::exhaustively_conserved`] or
/// [`crate::invariants::violation_rate`].
pub fn generate_ca_invariants<const N: usize, const R: usize>()
-> Vec<Box<dyn Invariant<CAState<N, R>>>> {
    vec![Box::new(PopulationInvariant), Box::new(ParityInvariant)]
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invariants::exhaustively_conserved;
    use crate::substrates::ca::rules::CARule;

    /// All 2^8 states for an 8-cell CA.
    fn full_state_space_8() -> Vec<CAState<8, 1>> {
        (0u32..256)
            .map(|bits| {
                let mut cells = [0u8; 8];
                for (i, c) in cells.iter_mut().enumerate() {
                    *c = ((bits >> i) & 1) as u8;
                }
                CAState::new(cells)
            })
            .collect()
    }

    #[test]
    fn rule_184_conserves_population() {
        let rule = CARule::<8, 1>::from_wolfram_number(184);
        let state_space = full_state_space_8();
        let inv = PopulationInvariant;
        assert!(exhaustively_conserved(&rule, &inv, &state_space));
    }

    #[test]
    fn rule_30_does_not_conserve_population() {
        // Rule 30 is chaotic and not number-conserving.
        let rule = CARule::<8, 1>::from_wolfram_number(30);
        let state_space = full_state_space_8();
        let inv = PopulationInvariant;
        assert!(!exhaustively_conserved(&rule, &inv, &state_space));
    }

    #[test]
    fn parity_invariant_matches_conserves_parity() {
        let state_space = full_state_space_8();
        let inv = ParityInvariant;
        for wn in [30u64, 90, 110, 184, 232] {
            let rule = CARule::<8, 1>::from_wolfram_number(wn);
            assert_eq!(
                exhaustively_conserved(&rule, &inv, &state_space),
                rule.conserves_parity(),
                "mismatch for rule {wn}"
            );
        }
    }

    #[test]
    fn standard_invariants_has_both() {
        let invariants = generate_ca_invariants::<8, 1>();
        assert_eq!(invariants.len(), 2);
        let names: Vec<&str> = invariants.iter().map(|i| i.name()).collect();
        assert!(names.contains(&"population"));
        assert!(names.contains(&"parity"));
    }
}
