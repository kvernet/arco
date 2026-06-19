//! Observation operators for Information Universes.
//!
//! Per Constitution:
//!     O is a set of functions from S to Y, where Y is an observation
//!     space equipped with a metric. Observation operators must be
//!     dynamically sufficient for the transformation dynamics.
//!
//! This module provides observation operators at varying granularities,
//! from the identity observation (full canonical encoding) to coarse
//! aggregate observations (label sums, edge counts).
//!
//! # Single-state vs windowed observation
//!
//! Single-state operators observe one state at a time. Windowed operators
//! observe a sliding window of recent states, enabling detection of
//! temporal patterns (oscillations, propagation delays) that are invisible
//! at the single-step level. Window size 1 recovers single-state behavior.
//!
//! Design commitments:
//! - Observation operators are pure functions.
//! - Observation values must be hashable and comparable (for MI estimation).
//! - The identity observation serves as the dynamically sufficient baseline.

use crate::state::{BinaryGraphState, State};

// ===================================================================
// Type aliases
// ===================================================================

/// An observation value. Must be hashable and comparable.
pub type Observation = Vec<u8>;

/// A single-state observation operator.
pub type StateObserver = dyn Fn(&BinaryGraphState) -> Observation + Send + Sync;

/// A windowed observation operator.
pub type WindowObserver = dyn Fn(&[BinaryGraphState]) -> Observation;

// ===================================================================
// Single-state observers
// ===================================================================

/// Identity observation — the full canonical encoding.
///
/// This is the maximally dynamically sufficient observation operator
/// (Constitution). No two distinct states produce the same observation.
pub fn observe_full_state(state: &BinaryGraphState) -> Observation {
    let (adj, labels) = state.canonical_encoding();
    let mut result = adj;
    result.extend(labels);
    result
}

/// Full label vector.
pub fn observe_label_vector(state: &BinaryGraphState) -> Observation {
    let n = state.n_vertices();
    (0..n).map(|i| state.label(i)).collect()
}

/// Sum of vertex labels.
pub fn observe_label_sum(state: &BinaryGraphState) -> Observation {
    vec![state.label_sum() as u8]
}

/// Label of vertex 0 only.
pub fn observe_root_label(state: &BinaryGraphState) -> Observation {
    vec![state.label(0)]
}

/// Flattened adjacency matrix.
pub fn observe_edge_vector(state: &BinaryGraphState) -> Observation {
    let n = state.n_vertices();
    (0..n)
        .flat_map(|i| (0..n).map(move |j| state.edge(i, j)))
        .collect()
}

/// Total edge count.
pub fn observe_edge_count(state: &BinaryGraphState) -> Observation {
    vec![state.edge_count() as u8]
}

/// Compound observation: labels followed by edges.
pub fn observe_compound(state: &BinaryGraphState) -> Observation {
    let mut result = observe_label_vector(state);
    result.extend(observe_edge_vector(state));
    result
}

// ===================================================================
// Windowed observers
// ===================================================================

/// Windowed identity: concatenation of canonical encodings.
///
/// With window_size=1, equivalent to `observe_full_state`.
pub fn observe_windowed(window: &[BinaryGraphState]) -> Observation {
    let mut result = Vec::new();
    for state in window {
        result.extend(observe_full_state(state));
    }
    result
}

/// Windowed label vector: concatenation of label vectors.
pub fn observe_windowed_labels(window: &[BinaryGraphState]) -> Observation {
    let mut result = Vec::new();
    for state in window {
        result.extend(observe_label_vector(state));
    }
    result
}

/// Windowed delta: encodes changes between consecutive states.
///
/// For each adjacent pair in the window, records which labels changed.
/// Useful for detecting propagation and oscillation.
pub fn observe_windowed_deltas(window: &[BinaryGraphState]) -> Observation {
    if window.len() < 2 {
        return observe_label_vector(&window[0]);
    }
    let mut result = Vec::new();
    for pair in window.windows(2) {
        let prev = &pair[0];
        let curr = &pair[1];
        let n = prev.n_vertices();
        for i in 0..n {
            result.push(if prev.label(i) != curr.label(i) { 1 } else { 0 });
        }
    }
    result
}

// ===================================================================
// Observation registry
// ===================================================================

/// Mapping from observation name to single-state observer function.
pub static STATE_OBSERVERS: &[(&str, &StateObserver)] = &[
    ("full_state", &observe_full_state),
    ("label_vector", &observe_label_vector),
    ("label_sum", &observe_label_sum),
    ("root_label", &observe_root_label),
    ("edge_vector", &observe_edge_vector),
    ("edge_count", &observe_edge_count),
    ("compound", &observe_compound),
];

/// Look up a single-state observer by name.
pub fn get_state_observer(name: &str) -> Option<&'static StateObserver> {
    STATE_OBSERVERS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, f)| *f)
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{arr1, arr2};

    fn make_state() -> BinaryGraphState {
        let adj = arr2(&[[0, 1], [0, 0]]);
        let labels = arr1(&[1, 0]);
        BinaryGraphState::new(2, adj.view(), labels.view()).unwrap()
    }

    #[test]
    fn test_full_state_is_complete() {
        let s1 = make_state();
        let s2 = s1.mutate_label(1, 1).unwrap();
        assert_ne!(observe_full_state(&s1), observe_full_state(&s2));
    }

    #[test]
    fn test_label_vector() {
        let s = make_state();
        assert_eq!(observe_label_vector(&s), vec![1, 0]);
    }

    #[test]
    fn test_label_sum() {
        let s = make_state();
        assert_eq!(observe_label_sum(&s), vec![1]);
    }

    #[test]
    fn test_root_label() {
        let s = make_state();
        assert_eq!(observe_root_label(&s), vec![1]);
    }

    #[test]
    fn test_edge_vector() {
        let s = make_state();
        // 2x2 matrix: [0,1; 0,0] flattened
        assert_eq!(observe_edge_vector(&s), vec![0, 1, 0, 0]);
    }

    #[test]
    fn test_edge_count() {
        let s = make_state();
        assert_eq!(observe_edge_count(&s), vec![1]);
    }

    #[test]
    fn test_compound_length() {
        let s = make_state();
        // 2 labels + 4 edges = 6
        assert_eq!(observe_compound(&s).len(), 6);
    }

    #[test]
    fn test_windowed_deltas_detects_change() {
        let s1 = make_state();
        let s2 = s1.mutate_label(0, 0).unwrap();
        let window = vec![s1, s2];
        let deltas = observe_windowed_deltas(&window);
        // One pair: label 0 changed (1), label 1 didn't (0)
        assert_eq!(deltas, vec![1, 0]);
    }

    #[test]
    fn test_windowed_deltas_no_change() {
        let s = make_state();
        let window = vec![s.clone(), s.clone()];
        let deltas = observe_windowed_deltas(&window);
        assert_eq!(deltas, vec![0, 0]);
    }

    #[test]
    fn test_registry_lookup() {
        let obs = get_state_observer("compound");
        assert!(obs.is_some());
        let s = make_state();
        assert_eq!(obs.unwrap()(&s), observe_compound(&s));
    }

    #[test]
    fn test_registry_missing() {
        assert!(get_state_observer("nonexistent").is_none());
    }

    #[test]
    fn test_windowed_observer_window_size_one() {
        let s = make_state();
        let window = vec![s.clone()];
        assert_eq!(observe_windowed(&window), observe_full_state(&s));
    }
}
