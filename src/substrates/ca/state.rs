//! Cellular automaton state implementation.
//!
//! This module provides [`CAState`], a 1D binary cellular automaton
//! with periodic boundary conditions. It implements the core [`State`]
//! trait and serves as the state type for the CA substrate.
//!
//! # Type parameters
//!
//! - `N`: Number of cells. The state space has 2^N configurations.
//! - `R`: Neighborhood radius. Each cell sees 2R+1 neighbors.
//!   Default is 1 (elementary CA with 3-bit neighborhoods).
//!
//! # Examples
//!
//! ```rust
//! use arco::substrates::ca::CAState;
//!
//! // Elementary CA: 8 cells, radius 1
//! let state = CAState::<8, 1>::new([0, 1, 0, 1, 0, 0, 0, 0]);
//! assert_eq!(state.cells(), &[0, 1, 0, 1, 0, 0, 0, 0]);
//! assert_eq!(state.cell(0), 0);
//! assert_eq!(state.cell(-1), 0); // periodic: wraps to last cell
//! ```

use rand::{Rng, RngExt};
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::state::State;

/// Number of cells in the neighborhood: 2R + 1.
pub const fn neighborhood_size<const R: usize>() -> usize {
    2 * R + 1
}

/// A state in a 1D binary cellular automaton with periodic boundaries.
///
/// # Type parameters
/// - `N`: Number of cells. State space size = 2^N.
/// - `R`: Neighborhood radius (default 1). Each cell's next state
///   depends on itself and R neighbors on each side.
#[derive(Clone)]
pub struct CAState<const N: usize, const R: usize = 1> {
    cells: [u8; N],
}

impl<const N: usize, const R: usize> CAState<N, R> {
    /// Create a new CAState from an array of binary values.
    ///
    /// # Panics
    /// Panics if any value is not 0 or 1.
    pub fn new(cells: [u8; N]) -> Self {
        for &c in &cells {
            assert!(c <= 1, "Cell values must be 0 or 1, got {}", c);
        }
        Self { cells }
    }

    /// Generate a random state.
    pub fn random(rng: &mut impl Rng) -> Self {
        let mut cells = [0u8; N];
        for c in cells.iter_mut() {
            *c = rng.random_range(0..=1);
        }
        Self { cells }
    }

    /// Get the value of a cell at the given index with periodic wrapping.
    /// Negative indices wrap from the end.
    pub fn cell(&self, index: i32) -> u8 {
        let i = index.rem_euclid(N as i32) as usize;
        self.cells[i]
    }

    /// Get a reference to the cell array.
    pub fn cells(&self) -> &[u8; N] {
        &self.cells
    }

    /// Get the neighborhood pattern at position `i` as an integer.
    ///
    /// Returns a value in 0..num_patterns::<R>() where the bits
    /// are ordered from leftmost neighbor to rightmost neighbor.
    ///
    /// # Example
    /// For R=1, the 3-bit pattern is: (left << 2) | (center << 1) | right.
    pub fn neighborhood(&self, i: usize) -> usize {
        let mut pattern = 0usize;
        for r in 0..neighborhood_size::<R>() {
            let offset = (r as i32) - (R as i32);
            let bit = self.cell(i as i32 + offset) as usize;
            pattern |= bit << (neighborhood_size::<R>() - 1 - r);
        }
        pattern
    }

    /// Create a new state with the given cells.
    pub fn with_cells(&self, cells: [u8; N]) -> Self {
        Self { cells }
    }
}

impl<const N: usize, const R: usize> State for CAState<N, R> {
    type Encoding = Vec<u8>;

    fn canonical_encoding(&self) -> Self::Encoding {
        self.cells.to_vec()
    }

    fn distance(&self, other: &Self) -> u32 {
        self.cells
            .iter()
            .zip(other.cells.iter())
            .map(|(a, b)| if a != b { 1 } else { 0 })
            .sum()
    }
}

impl<const N: usize, const R: usize> PartialEq for CAState<N, R> {
    fn eq(&self, other: &Self) -> bool {
        self.cells == other.cells
    }
}

impl<const N: usize, const R: usize> Eq for CAState<N, R> {}

impl<const N: usize, const R: usize> Hash for CAState<N, R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cells.hash(state);
    }
}

impl<const N: usize, const R: usize> fmt::Debug for CAState<N, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CAState<{}, {}>({:?})", N, R, self.cells)
    }
}

impl<const N: usize, const R: usize> fmt::Display for CAState<N, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &c in &self.cells {
            write!(f, "{}", if c == 1 { '■' } else { '□' })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighborhood_r1() {
        // State: [1, 0, 1, 0, 0, 0, 0, 0]
        let state = CAState::<8, 1>::new([1, 0, 1, 0, 0, 0, 0, 0]);
        // At i=0: left=state[7]=0, center=1, right=state[1]=0 → 010 = 2
        assert_eq!(state.neighborhood(0), 2);
        // At i=1: left=1, center=0, right=1 → 101 = 5
        assert_eq!(state.neighborhood(1), 5);
    }

    #[test]
    fn test_periodic_boundary() {
        let state = CAState::<4, 1>::new([1, 0, 0, 0]);
        assert_eq!(state.cell(-1), 0); // wraps to index 3
        assert_eq!(state.cell(4), 1); // wraps to index 0
    }

    #[test]
    fn test_distance() {
        let s1 = CAState::<4, 1>::new([0, 0, 0, 0]);
        let s2 = CAState::<4, 1>::new([1, 0, 0, 0]);
        assert_eq!(s1.distance(&s2), 1);
        assert_eq!(s1.distance(&s1), 0);
    }
}
