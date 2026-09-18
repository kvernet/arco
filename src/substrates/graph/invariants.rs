//! Standard invariants for the Binary Graph substrate.

use crate::invariants::Invariant;
use crate::substrates::graph::state::BinaryGraphState;

// ===================================================================
// EdgeCountInvariant
// ===================================================================

/// `I(s) = edge_count(s)` — the total number of directed edges.
///
/// Exactly conserved by every rule shipped with the Binary Graph
/// Universe: every structured and destructive rule generator in
/// `substrates::graph::rules` mutates vertex labels only
/// (`mutate_label`/`mutate_labels`) and never the adjacency matrix
/// (`mutate_adj`). This makes the invariant trivially true by
/// construction for this rule set — a validation-substrate-style
/// demonstration (per Constitution), not a discovered structural
/// fact. A substrate whose rules can rewrite edges would need this
/// invariant to do real work distinguishing edge-preserving from
/// edge-mutating rule sets.
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeCountInvariant;

impl Invariant<BinaryGraphState> for EdgeCountInvariant {
    fn name(&self) -> &str {
        "edge_count"
    }

    fn evaluate(&self, state: &BinaryGraphState) -> f64 {
        state.edge_count() as f64
    }
}

// ===================================================================
// LabelSumInvariant
// ===================================================================

/// `I(s) = label_sum(s)` — the number of vertices labeled `1`.
///
/// Included as a contrast case, not a genuine invariant: most
/// structured rules (`NAND`, `AND`, `OR`, `XOR`, `TOGGLE`, ...)
/// deliberately change vertex labels, so this quantity is *not*
/// conserved by most rule sets. It demonstrates that not every
/// candidate real-valued function of state is an invariant — and
/// that ARCO's mechanism for telling the difference is exactly
/// [`Invariant::is_conserved`] returning `false`.
#[derive(Debug, Clone, Copy, Default)]
pub struct LabelSumInvariant;

impl Invariant<BinaryGraphState> for LabelSumInvariant {
    fn name(&self) -> &str {
        "label_sum"
    }

    fn evaluate(&self, state: &BinaryGraphState) -> f64 {
        state.label_sum() as f64
    }
}

// ===================================================================
// Standard invariant set
// ===================================================================

/// Generate the standard set of invariants for the Binary Graph
/// Universe: one genuinely conserved quantity ([`EdgeCountInvariant`])
/// and one contrast case that most rule sets violate
/// ([`LabelSumInvariant`]).
pub fn generate_standard_invariants() -> Vec<Box<dyn Invariant<BinaryGraphState>>> {
    vec![Box::new(EdgeCountInvariant), Box::new(LabelSumInvariant)]
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rules::Rule, substrates::graph::rules::create_structured_rules};
    use ndarray::{arr1, arr2};
    use rand::{SeedableRng, rngs::StdRng};

    #[test]
    fn edge_count_conserved_under_label_only_mutation() {
        let adj = arr2(&[[0, 1], [1, 0]]);
        let labels = arr1(&[0, 0]);
        let before = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
        let after = before.mutate_label(0, 1).unwrap();

        let inv = EdgeCountInvariant;
        assert!(inv.is_conserved(&before, &after));
    }

    #[test]
    fn edge_count_violated_under_adjacency_mutation() {
        let adj = arr2(&[[0, 1], [0, 0]]);
        let labels = arr1(&[0, 0]);
        let before = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
        let after = before.mutate_adj(1, 0, 1).unwrap();

        let inv = EdgeCountInvariant;
        assert!(!inv.is_conserved(&before, &after));
    }

    #[test]
    fn label_sum_violated_by_toggle() {
        // Every structured rule generator only mutates labels, so
        // toggling a label is a representative example of the kind
        // of change that breaks label_sum conservation.
        let adj = arr2(&[[0, 0], [0, 0]]);
        let labels = arr1(&[0, 0]);
        let before = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
        let after = before.mutate_label(0, 1).unwrap();

        let inv = LabelSumInvariant;
        assert!(!inv.is_conserved(&before, &after));
    }

    #[test]
    fn standard_invariants_has_both() {
        let invariants = generate_standard_invariants();
        assert_eq!(invariants.len(), 2);
        let names: Vec<&str> = invariants.iter().map(|i| i.name()).collect();
        assert!(names.contains(&"edge_count"));
        assert!(names.contains(&"label_sum"));
    }

    #[test]
    fn structured_rules_never_touch_adjacency() {
        // Cross-check the doc claim: apply every structured rule to a
        // handful of states and confirm edge_count never changes.
        let rules = create_structured_rules();
        let adj = arr2(&[[1, 0], [0, 1]]);
        let labels = arr1(&[0, 1]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
        let inv = EdgeCountInvariant;

        let mut rng = StdRng::seed_from_u64(0);

        for rule in &rules {
            if let Some(info) = rule.matches(&state, 0) {
                let next = rule.apply(&state, &info, &mut rng);
                assert!(
                    inv.is_conserved(&state, &next),
                    "rule {} changed edge_count",
                    rule.name()
                );
            }
        }
    }
}
