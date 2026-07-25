//! Observation operators for the Binary Graph Universe.
//!
//! This module provides observation operators at varying granularities,
//! from the identity observation (full canonical encoding) to coarse
//! aggregate observations (label sums, edge counts).
//!
//! All observers implement the core [`Observation`] trait for
//! [`BinaryGraphState`]. They are registered in [`STATE_OBSERVERS`]
//! for name-based lookup.
//!
//! # Available observers
//!
//! | Observer | What it captures | Output size for n=3 |
//! |----------|-----------------|---------------------|
//! | `observe_full_state` | Labels + edges (identity) | 12 bytes |
//! | `observe_compound` | Labels then edges | 12 bytes |
//! | `observe_label_vector` | Vertex labels only | 3 bytes |
//! | `observe_label_sum` | Count of 1-labels | 1 byte |
//! | `observe_root_label` | Label of vertex 0 only | 1 byte |
//! | `observe_edge_vector` | Flattened adjacency matrix | 9 bytes |
//! | `observe_edge_count` | Total number of edges | 1 byte |

use crate::observation::Observation;
use crate::state::State;
use crate::substrates::graph::state::BinaryGraphState;
use crate::types::GraphObserverFn;

// ===================================================================
// Single-state observers
// ===================================================================

/// Identity observation — the full canonical encoding.
///
/// This is the maximally dynamically sufficient observation operator.
/// No two distinct states produce the same observation.
pub fn observe_full_state(state: &BinaryGraphState) -> Vec<u8> {
    let (adj, labels) = state.canonical_encoding();
    let mut result = adj;
    result.extend(labels);
    result
}

/// Full label vector.
pub fn observe_label_vector(state: &BinaryGraphState) -> Vec<u8> {
    let n = state.n_vertices();
    (0..n).map(|i| state.label(i)).collect()
}

/// Sum of vertex labels.
pub fn observe_label_sum(state: &BinaryGraphState) -> Vec<u8> {
    vec![state.label_sum() as u8]
}

/// Label of vertex 0 only.
pub fn observe_root_label(state: &BinaryGraphState) -> Vec<u8> {
    vec![state.label(0)]
}

/// Flattened adjacency matrix.
pub fn observe_edge_vector(state: &BinaryGraphState) -> Vec<u8> {
    let n = state.n_vertices();
    (0..n)
        .flat_map(|i| (0..n).map(move |j| state.edge(i, j)))
        .collect()
}

/// Total edge count.
pub fn observe_edge_count(state: &BinaryGraphState) -> Vec<u8> {
    vec![state.edge_count() as u8]
}

/// Compound observation: labels followed by edges.
pub fn observe_compound(state: &BinaryGraphState) -> Vec<u8> {
    let mut result = observe_label_vector(state);
    result.extend(observe_edge_vector(state));
    result
}

// ===================================================================
// Observation structs (implementing the Observation trait)
// ===================================================================

macro_rules! impl_observer {
    ($name:ident, $fn:ident, $desc:literal) => {
        #[doc = $desc]
        #[derive(Debug, Clone, Default)]
        pub struct $name;

        impl Observation<BinaryGraphState> for $name {
            type Output = Vec<u8>;

            fn observe(&self, state: &BinaryGraphState) -> Self::Output {
                $fn(state)
            }
        }
    };
}

impl_observer!(
    FullStateObserver,
    observe_full_state,
    "Identity observation — the full canonical encoding."
);
impl_observer!(
    LabelVectorObserver,
    observe_label_vector,
    "Full label vector."
);
impl_observer!(LabelSumObserver, observe_label_sum, "Sum of vertex labels.");
impl_observer!(
    RootLabelObserver,
    observe_root_label,
    "Label of vertex 0 only."
);
impl_observer!(
    EdgeVectorObserver,
    observe_edge_vector,
    "Flattened adjacency matrix."
);
impl_observer!(EdgeCountObserver, observe_edge_count, "Total edge count.");
impl_observer!(
    CompoundObserver,
    observe_compound,
    "Compound: labels followed by edges."
);

// ===================================================================
// Observer registry
// ===================================================================

/// Mapping from observation name to observer.
pub static STATE_OBSERVERS: &[(&str, &GraphObserverFn)] = &[
    ("full_state", &observe_full_state),
    ("label_vector", &observe_label_vector),
    ("label_sum", &observe_label_sum),
    ("root_label", &observe_root_label),
    ("edge_vector", &observe_edge_vector),
    ("edge_count", &observe_edge_count),
    ("compound", &observe_compound),
];

pub fn get_state_observer(name: &str) -> Option<&'static GraphObserverFn> {
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
        assert_eq!(observe_compound(&s).len(), 6);
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
    fn test_trait_implementation() {
        let observer = CompoundObserver;
        let s = make_state();
        assert_eq!(observer.observe(&s), observe_compound(&s));
    }
}
