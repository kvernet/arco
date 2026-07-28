//! Observation operators for the Cellular Automaton substrate.
//!
//! This module provides observation operators for [`CAState`] at
//! varying granularities. All observers implement the core
//! [`Observation`] trait.
//!
//! # Available observers
//!
//! | Observer | What it captures | Output size |
//! |----------|-----------------|-------------|
//! | `FullStateObserver` | All cells (identity) | N bytes |
//! | `DensityObserver` | Fraction of 1s | 1 byte |
//! | `ParityObserver` | Sum mod 2 | 1 byte |
//!
//! # Quick start
//!
//! ```rust
//! use arco::substrates::ca::state::CAState;
//! use arco::substrates::ca::observation::{FullStateObserver, DensityObserver, ParityObserver};
//! use arco::observation::Observation;
//!
//! let state = CAState::<8, 1>::new([1, 0, 1, 0, 0, 0, 0, 0]);
//!
//! // Identity observation — all 8 cells
//! let full = FullStateObserver;
//! assert_eq!(full.observe(&state), vec![1, 0, 1, 0, 0, 0, 0, 0]);
//!
//! // Coarse aggregate — just the count of 1s
//! let density = DensityObserver;
//! assert_eq!(density.observe(&state), vec![2]);
//!
//! // Parity — sum mod 2
//! let parity = ParityObserver;
//! assert_eq!(parity.observe(&state), vec![0]);
//! ```

use crate::observation::Observation;
use crate::state::State;
use crate::substrates::ca::state::CAState;

// ===================================================================
// Observer structs
// ===================================================================

/// Identity observation — the full cell array.
///
/// Distinguishes every distinct state. Maximally dynamically sufficient.
#[derive(Debug, Clone, Default)]
pub struct FullStateObserver;

impl<const N: usize, const R: usize> Observation<CAState<N, R>> for FullStateObserver {
    type Output = Vec<u8>;

    fn observe(&self, state: &CAState<N, R>) -> Self::Output {
        state.canonical_encoding()
    }
}

/// Density observation — the fraction of cells that are 1.
///
/// Maps the count of 1s to a single byte in 0..=N.
/// Coarse but useful for detecting conservation laws.
#[derive(Debug, Clone, Default)]
pub struct DensityObserver;

impl<const N: usize, const R: usize> Observation<CAState<N, R>> for DensityObserver {
    type Output = Vec<u8>;

    fn observe(&self, state: &CAState<N, R>) -> Self::Output {
        let count: u8 = state.cells().iter().copied().sum();
        vec![count]
    }
}

/// Parity observation — the sum of cells modulo 2.
///
/// Useful for detecting parity-conserving rules.
#[derive(Debug, Clone, Default)]
pub struct ParityObserver;

impl<const N: usize, const R: usize> Observation<CAState<N, R>> for ParityObserver {
    type Output = Vec<u8>;

    fn observe(&self, state: &CAState<N, R>) -> Self::Output {
        let parity: u8 = state.cells().iter().copied().sum::<u8>() % 2;
        vec![parity]
    }
}

#[derive(Debug, Clone)]
pub enum CAObserver {
    FullState,
    Density,
    Parity,
}

impl CAObserver {
    pub fn from_name(name: &str) -> Self {
        match name {
            "full_state" => Self::FullState,
            "density" => Self::Density,
            "parity" => Self::Parity,
            _ => Self::FullState,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::FullState => "full_state",
            Self::Density => "density",
            Self::Parity => "parity",
        }
    }
}

impl<const N: usize, const R: usize> Observation<CAState<N, R>> for CAObserver {
    type Output = Vec<u8>;

    fn observe(&self, state: &CAState<N, R>) -> Self::Output {
        match self {
            Self::FullState => FullStateObserver.observe(state),
            Self::Density => DensityObserver.observe(state),
            Self::Parity => ParityObserver.observe(state),
        }
    }
}

// ===================================================================
// Observer registry
// ===================================================================

/// Mapping from observation name to a function constructing the observer.
/// All CA observers are stateless, so we return references to static instances.
pub fn get_observer<const N: usize, const R: usize>(
    name: &str,
) -> Option<Box<dyn Observation<CAState<N, R>, Output = Vec<u8>> + Send + Sync>> {
    match name {
        "full_state" => Some(Box::new(FullStateObserver)),
        "density" => Some(Box::new(DensityObserver)),
        "parity" => Some(Box::new(ParityObserver)),
        _ => None,
    }
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_state_observer() {
        let state = CAState::<8, 1>::new([1, 0, 1, 0, 0, 0, 0, 0]);
        let obs = FullStateObserver;
        assert_eq!(obs.observe(&state), vec![1, 0, 1, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_density_observer() {
        let state = CAState::<8, 1>::new([1, 0, 1, 0, 0, 0, 0, 0]);
        let obs = DensityObserver;
        assert_eq!(obs.observe(&state), vec![2]); // two 1s
    }

    #[test]
    fn test_parity_observer() {
        let state = CAState::<8, 1>::new([1, 0, 1, 0, 0, 0, 0, 0]);
        let obs = ParityObserver;
        assert_eq!(obs.observe(&state), vec![0]); // 2 % 2 = 0
    }

    #[test]
    fn test_full_state_distinguishes() {
        let s1 = CAState::<8, 1>::new([0; 8]);
        let s2 = CAState::<8, 1>::new([1, 0, 0, 0, 0, 0, 0, 0]);
        let obs = FullStateObserver;
        assert_ne!(obs.observe(&s1), obs.observe(&s2));
    }

    #[test]
    fn test_get_observer() {
        let obs = get_observer::<8, 1>("density");
        assert!(obs.is_some());
        let obs = get_observer::<8, 1>("nonexistent");
        assert!(obs.is_none());
    }
}
