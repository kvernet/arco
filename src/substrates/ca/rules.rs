//! Cellular automaton rule implementation.
//!
//! This module provides [`CARule`], a lookup-table-based rule for
//! 1D binary cellular automata. It implements the core [`Rule`]
//! trait and provides measurable properties for hypothesis
//! generation without human labels.
//!
//! # Type parameters
//!
//! - `N`: Number of cells (must match [`CAState<N, R>`]).
//! - `R`: Neighborhood radius (default 1).

use crate::rules::{NoContext, Rule};
use crate::state::State;
use crate::substrates::ca::state::CAState;
use rand::{Rng, RngExt};
use std::fmt;

/// Number of cells in the neighborhood.
const fn neighborhood_size<const R: usize>() -> usize {
    2 * R + 1
}

/// Total number of possible states for N cells.
const fn num_states<const N: usize>() -> usize {
    1usize << N
}

/// Convert a state index (0..2^N) into a cell array.
#[allow(clippy::needless_range_loop)]
fn cells_from_bits<const N: usize>(bits: usize) -> [u8; N] {
    let mut cells = [0u8; N];
    for i in 0..N {
        cells[i] = ((bits >> i) & 1) as u8;
    }
    cells
}

// ===================================================================
// CARule
// ===================================================================

/// A cellular automaton rule with configurable radius.
///
/// The rule is defined by a lookup table mapping each possible
/// neighborhood pattern (0..2^(2R+1)) to an output bit (0 or 1).
///
/// # Type parameters
/// - `N`: Number of cells.
/// - `R`: Neighborhood radius (default 1). For R=1, this is an
///   elementary CA rule (Wolfram rules 0–255).
///
/// # Examples
///
/// ```rust
/// use arco::substrates::ca::CARule;
///
/// // Create Rule 110 (elementary CA, R=1)
/// let rule = CARule::<8, 1>::from_wolfram_number(110);
/// assert_eq!(rule.wolfram_number(), Some(110));
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct CARule<const N: usize, const R: usize = 1> {
    name: String,
    wolfram_number: Option<u64>,
    table: Vec<u8>,
}

impl<const N: usize, const R: usize> CARule<N, R> {
    /// Number of possible neighborhood patterns.
    const NUM_PATTERNS: usize = 1 << neighborhood_size::<R>();

    /// Maximum Wolfram rule number for this radius.
    const MAX_RULE: Option<u64> = if Self::NUM_PATTERNS < 64 {
        Some(1u64 << Self::NUM_PATTERNS)
    } else {
        None
    };

    /// Create a rule from a Wolfram rule number.
    ///
    /// Only valid when NUM_PATTERNS < 64 (R ≤ 5). For larger radii,
    /// use [`from_table`] instead.
    ///
    /// # Panics
    /// Panics if NUM_PATTERNS ≥ 64 or if the rule number exceeds
    /// the maximum for this radius.
    pub fn from_wolfram_number(rule_number: u64) -> Self {
        let max_rule = Self::MAX_RULE.unwrap_or_else(|| {
            panic!(
                "NUM_PATTERNS = {} exceeds u64 range; use from_table() instead",
                Self::NUM_PATTERNS
            )
        });
        assert!(
            rule_number < max_rule,
            "Rule number {} exceeds maximum {} for R={}",
            rule_number,
            max_rule - 1,
            R
        );

        let mut table = vec![0u8; Self::NUM_PATTERNS];
        for (i, entry) in table.iter_mut().enumerate() {
            *entry = ((rule_number >> i) & 1) as u8;
        }
        Self {
            name: format!("Rule {}", rule_number),
            wolfram_number: Some(rule_number),
            table,
        }
    }

    /// Create a rule from an explicit lookup table.
    ///
    /// # Panics
    /// Panics if the table length doesn't match NUM_PATTERNS.
    pub fn from_table(table: Vec<u8>) -> Self {
        assert_eq!(
            table.len(),
            Self::NUM_PATTERNS,
            "Table length {} doesn't match NUM_PATTERNS {} for R={}",
            table.len(),
            Self::NUM_PATTERNS,
            R
        );
        Self {
            name: format!("CA Rule ({} entries)", table.len()),
            wolfram_number: None,
            table,
        }
    }

    /// Generate a random rule.
    pub fn random(rng: &mut (impl Rng + ?Sized)) -> Self {
        let table: Vec<u8> = (0..Self::NUM_PATTERNS)
            .map(|_| rng.random_range(0..=1))
            .collect();
        let id = rng.random_range::<u64, _>(0..1_000_000);
        Self {
            name: format!("Random CA #{}", id),
            wolfram_number: None,
            table,
        }
    }

    /// The Wolfram rule number, if applicable (R=1 only).
    pub fn wolfram_number(&self) -> Option<u64> {
        self.wolfram_number
    }

    /// The lookup table.
    pub fn table(&self) -> &[u8] {
        &self.table
    }

    /// Apply the rule to a single neighborhood pattern.
    pub fn apply_to_neighborhood(&self, pattern: usize) -> u8 {
        self.table[pattern]
    }

    /// Apply the rule synchronously to an entire state.
    #[allow(clippy::needless_range_loop)]
    pub fn apply_sync(&self, state: &CAState<N, R>) -> CAState<N, R> {
        let mut new_cells = [0u8; N];
        for i in 0..N {
            let pattern = state.neighborhood(i);
            new_cells[i] = self.table[pattern];
        }
        CAState::new(new_cells)
    }

    // ================================================================
    // Measurable properties (for hypothesis generation)
    // ================================================================

    /// Is this rule reversible?
    ///
    /// Checks exhaustively that every state has a unique successor.
    /// Only feasible for small N (N ≤ 10, 2^N ≤ 1024).
    pub fn is_reversible(&self) -> bool {
        let total = num_states::<N>();
        if total > 1024 {
            return false;
        }
        let mut seen = vec![false; total];
        for bits in 0..total {
            let state = CAState::<N, R>::new(cells_from_bits::<N>(bits));
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

    /// Does this rule conserve parity (sum mod 2)?
    pub fn conserves_parity(&self) -> bool {
        let total = num_states::<N>();
        if total > 1024 {
            return false;
        }
        for bits in 0..total {
            let state = CAState::<N, R>::new(cells_from_bits::<N>(bits));
            let next = self.apply_sync(&state);
            let parity_before: u32 = state.cells().iter().map(|&c| c as u32).sum::<u32>() % 2;
            let parity_after: u32 = next.cells().iter().map(|&c| c as u32).sum::<u32>() % 2;
            if parity_before != parity_after {
                return false;
            }
        }
        true
    }

    /// Sensitivity to initial conditions.
    ///
    /// Average number of cells that differ after one timestep when
    /// starting from states that differ by a single bit.
    pub fn sensitivity(&self) -> f64 {
        let total = num_states::<N>();
        if total > 1024 {
            return 0.0;
        }
        let mut total_diff = 0u32;
        let mut count = 0u32;

        for bits in 0..total {
            let cells = cells_from_bits::<N>(bits);
            let state = CAState::<N, R>::new(cells);

            for flip in 0..N {
                let mut flipped_cells = cells;
                flipped_cells[flip] = 1 - flipped_cells[flip];
                let flipped_state = CAState::<N, R>::new(flipped_cells);

                let next_orig = self.apply_sync(&state);
                let next_flip = self.apply_sync(&flipped_state);

                total_diff += next_orig.distance(&next_flip);
                count += 1;
            }
        }

        if count == 0 {
            0.0
        } else {
            total_diff as f64 / count as f64
        }
    }

    /// Lambda parameter (fraction of non-zero outputs).
    ///
    /// Langton's lambda: the fraction of neighborhood patterns that
    /// produce a 1. High lambda correlates with chaotic behavior.
    pub fn lambda(&self) -> f64 {
        let ones = self.table.iter().filter(|&&b| b == 1).count();
        ones as f64 / self.table.len() as f64
    }
}

impl<const N: usize, const R: usize> Rule<CAState<N, R>> for CARule<N, R> {
    type Context = NoContext;

    fn name(&self) -> &str {
        &self.name
    }

    fn apply(
        &self,
        state: &CAState<N, R>,
        _context: &NoContext,
        _rng: &mut dyn Rng,
    ) -> CAState<N, R> {
        self.apply_sync(state)
    }
}

impl<const N: usize, const R: usize> fmt::Debug for CARule<N, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(wn) = self.wolfram_number {
            write!(f, "CARule<{}, {}>(Rule {})", N, R, wn)
        } else {
            write!(f, "CARule<{}, {}>({} entries)", N, R, self.table.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn test_rule110_table() {
        let rule = CARule::<8, 1>::from_wolfram_number(110);
        assert_eq!(rule.table(), &[0, 1, 1, 1, 0, 1, 1, 0]);
        assert_eq!(rule.wolfram_number(), Some(110));
        assert_eq!(rule.name(), "Rule 110");
    }

    #[test]
    fn test_rule110_not_reversible() {
        let rule = CARule::<4, 1>::from_wolfram_number(110);
        assert!(!rule.is_reversible());
    }

    #[test]
    fn test_rule0_lambda_zero() {
        let rule = CARule::<8, 1>::from_wolfram_number(0);
        assert!((rule.lambda() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_rule255_lambda_one() {
        let rule = CARule::<8, 1>::from_wolfram_number(255);
        assert!((rule.lambda() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_sensitivity_rule30_high() {
        let rule = CARule::<4, 1>::from_wolfram_number(30);
        let s = rule.sensitivity();
        assert!(s > 1.0, "Rule 30 sensitivity should be > 1.0, got {}", s);
    }

    #[test]
    fn test_name_is_descriptive() {
        let wolfram = CARule::<8, 1>::from_wolfram_number(110);
        assert_eq!(wolfram.name(), "Rule 110");

        let custom = CARule::<8, 1>::from_table(vec![0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(custom.name().contains("CA Rule"));

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let random = CARule::<8, 1>::random(&mut rng);
        assert!(random.name().contains("Random CA"));
    }
}
