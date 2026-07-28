//! Observation operators for the Binary Graph Universe.
//!
//! This module provides the [`GraphObserver`] enum for runtime
//! selection of observation granularity, backed by pure functions.

use crate::observation::Observation;
use crate::state::State;
use crate::substrates::graph::state::BinaryGraphState;

// ===================================================================
// Observer functions
// ===================================================================

pub fn observe_full_state(state: &BinaryGraphState) -> Vec<u8> {
    let (adj, labels) = state.canonical_encoding();
    let mut result = adj;
    result.extend(labels);
    result
}

pub fn observe_label_vector(state: &BinaryGraphState) -> Vec<u8> {
    let n = state.n_vertices();
    (0..n).map(|i| state.label(i)).collect()
}

pub fn observe_label_sum(state: &BinaryGraphState) -> Vec<u8> {
    vec![state.label_sum() as u8]
}

pub fn observe_root_label(state: &BinaryGraphState) -> Vec<u8> {
    vec![state.label(0)]
}

pub fn observe_edge_vector(state: &BinaryGraphState) -> Vec<u8> {
    let n = state.n_vertices();
    (0..n)
        .flat_map(|i| (0..n).map(move |j| state.edge(i, j)))
        .collect()
}

pub fn observe_edge_count(state: &BinaryGraphState) -> Vec<u8> {
    vec![state.edge_count() as u8]
}

pub fn observe_compound(state: &BinaryGraphState) -> Vec<u8> {
    let mut result = observe_label_vector(state);
    result.extend(observe_edge_vector(state));
    result
}

// ===================================================================
// Observer enum
// ===================================================================

#[derive(Debug, Clone)]
pub enum GraphObserver {
    FullState,
    Compound,
    LabelVector,
    LabelSum,
    RootLabel,
    EdgeVector,
    EdgeCount,
}

impl GraphObserver {
    pub fn from_name(name: &str) -> Self {
        match name {
            "full_state" => Self::FullState,
            "compound" => Self::Compound,
            "label_vector" => Self::LabelVector,
            "label_sum" => Self::LabelSum,
            "root_label" => Self::RootLabel,
            "edge_vector" => Self::EdgeVector,
            "edge_count" => Self::EdgeCount,
            _ => Self::FullState,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::FullState => "full_state",
            Self::Compound => "compound",
            Self::LabelVector => "label_vector",
            Self::LabelSum => "label_sum",
            Self::RootLabel => "root_label",
            Self::EdgeVector => "edge_vector",
            Self::EdgeCount => "edge_count",
        }
    }
}

impl Observation<BinaryGraphState> for GraphObserver {
    type Output = Vec<u8>;

    fn observe(&self, state: &BinaryGraphState) -> Self::Output {
        match self {
            Self::FullState => observe_full_state(state),
            Self::Compound => observe_compound(state),
            Self::LabelVector => observe_label_vector(state),
            Self::LabelSum => observe_label_sum(state),
            Self::RootLabel => observe_root_label(state),
            Self::EdgeVector => observe_edge_vector(state),
            Self::EdgeCount => observe_edge_count(state),
        }
    }
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
    fn test_observer_enum_compound() {
        let s = make_state();
        let obs = GraphObserver::Compound;
        assert_eq!(obs.observe(&s), observe_compound(&s));
    }

    #[test]
    fn test_observer_from_name() {
        assert!(matches!(
            GraphObserver::from_name("compound"),
            GraphObserver::Compound
        ));
        assert!(matches!(
            GraphObserver::from_name("unknown"),
            GraphObserver::FullState
        ));
    }
}
