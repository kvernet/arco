//! Resource accounting for the Binary Graph Universe.

use crate::resources::Resources;
use crate::substrates::graph::rules::RewriteRule;
use crate::substrates::graph::state::BinaryGraphState;

/// Resource accounting for [`crate::substrates::graph::BinaryGraphUniverse`].
///
/// - **Space** (`R_space`): total binary cells in the canonical
///   encoding — `n² (adjacency matrix) + n (vertex labels)`.
/// - **Time** (`R_time`): `1.0` per rule application. The Binary
///   Graph Universe does not distinguish rule cost by type.
/// - **Locality** (`R_local`): [`RewriteRule::locality_radius`] — this
///   substrate already tracks maximum interaction radius per rule for
///   `compose()`'s bookkeeping; `GraphResources` exposes the same
///   quantity through the generic [`Resources`] trait.
#[derive(Debug, Clone, Copy, Default)]
pub struct GraphResources;

impl Resources<BinaryGraphState, RewriteRule> for GraphResources {
    fn name(&self) -> &str {
        "graph_resources"
    }

    fn space(&self, state: &BinaryGraphState) -> f64 {
        let n = state.n_vertices() as f64;
        n * n + n
    }

    fn time(&self, _rule: &RewriteRule) -> f64 {
        1.0
    }

    fn locality(&self, rule: &RewriteRule) -> f64 {
        rule.locality_radius() as f64
    }
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::satisfies_resource_algebra;
    use crate::rules::Rule;
    use crate::substrates::graph::rules::{compose, create_structured_rules};
    use ndarray::{arr1, arr2};

    #[test]
    fn space_matches_canonical_encoding_size() {
        let adj = arr2(&[[0, 1], [0, 0]]);
        let labels = arr1(&[1, 0]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
        let model = GraphResources;
        // n=2: 2*2 (adjacency) + 2 (labels) = 6
        assert_eq!(model.space(&state), 6.0);
    }

    #[test]
    fn locality_matches_rule_locality_radius() {
        let rules = create_structured_rules();
        let model = GraphResources;
        for rule in &rules {
            assert_eq!(model.locality(rule), rule.locality_radius() as f64);
        }
    }

    #[test]
    fn compose_respects_resource_algebra() {
        // IDENTITY has locality 0, PROPAGATE has locality 1 --
        // composing them exercises the max-based composition already
        // used by `compose()`, and this checks it against the
        // Resource Algebra (per Constitution) via the generic
        // Resources interface.
        let rules = create_structured_rules();
        let model = GraphResources;
        let r1 = rules.iter().find(|r| r.name() == "IDENTITY").unwrap();
        let r2 = rules.iter().find(|r| r.name() == "PROPAGATE").unwrap();
        let composed = compose(r1, r2);

        assert!(satisfies_resource_algebra(
            (model.time(r1), model.locality(r1)),
            (model.time(r2), model.locality(r2)),
            (model.time(&composed), model.locality(&composed)),
        ));
    }
}
