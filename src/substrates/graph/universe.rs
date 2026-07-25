//! Binary Graph Universe — the complete validation substrate.

use rand::{Rng, RngExt};

use crate::observation::Observation;
use crate::rules::Rule;
use crate::substrates::graph::observation::{
    observe_compound, observe_edge_count, observe_edge_vector, observe_full_state,
    observe_label_sum, observe_label_vector, observe_root_label,
};
use crate::substrates::graph::rules::{
    RewriteRule, create_destructive_rules, create_structured_rules, generate_mixed_rule_subsets,
};
use crate::substrates::graph::schedule::AllVerticesSchedule;
use crate::substrates::graph::state::BinaryGraphState;
use crate::universe::InformationUniverse;

// ===================================================================
// Observer enum
// ===================================================================

/// An observation operator for the Binary Graph Universe.
///
/// This enum allows runtime selection of observation granularity
/// without dynamic dispatch.
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
    /// Create an observer from a name string.
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

    /// The name of this observer.
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
// BinaryGraphUniverse
// ===================================================================

/// The Binary Graph Universe — ARCO's validation substrate.
#[derive(Debug, Clone)]
pub struct BinaryGraphUniverse {
    state_space: Vec<BinaryGraphState>,
    observer: GraphObserver,
    schedule: AllVerticesSchedule,
    n_vertices: usize,
}

impl BinaryGraphUniverse {
    /// Create a new BinaryGraphUniverse.
    ///
    /// # Arguments
    /// * `n_vertices` — Number of vertices per state (typically 3).
    /// * `obs_name` — Observation operator name. One of: `"full_state"`,
    ///   `"compound"`, `"label_vector"`, `"label_sum"`, `"root_label"`,
    ///   `"edge_vector"`, `"edge_count"`.
    /// * `seed` — Random seed for state space generation.
    pub fn new(n_vertices: usize, obs_name: &str, rng: &mut impl Rng) -> Self {
        let state_space: Vec<BinaryGraphState> = (0..500)
            .map(|_| BinaryGraphState::random(n_vertices, rng))
            .collect();

        Self {
            state_space,
            observer: GraphObserver::from_name(obs_name),
            schedule: AllVerticesSchedule::new(),
            n_vertices,
        }
    }

    /// Number of vertices per state.
    pub fn n_vertices(&self) -> usize {
        self.n_vertices
    }

    /// Name of the observation operator.
    pub fn obs_name(&self) -> &str {
        self.observer.name()
    }
}

impl InformationUniverse for BinaryGraphUniverse {
    type State = BinaryGraphState;
    type Rule = RewriteRule;
    type Observation = GraphObserver;
    type Schedule = AllVerticesSchedule;

    fn state_space(&self) -> &[Self::State] {
        &self.state_space
    }

    fn rules(&self) -> &[Self::Rule] {
        &[]
    }

    fn observation(&self) -> &Self::Observation {
        &self.observer
    }

    fn schedule(&self) -> &Self::Schedule {
        &self.schedule
    }

    fn null_rules(&self, rng: &mut dyn Rng) -> Vec<Self::Rule> {
        let destructive_pool = create_destructive_rules();
        let scramblers: Vec<&RewriteRule> = destructive_pool
            .iter()
            .filter(|r| r.name().starts_with("DESTROY_SCRAMBLE_ALL"))
            .collect();

        let size = rng.random_range(1..=5);
        let mut rules = Vec::with_capacity(size);

        // Make sure to have at least one scrambler (ref. experience)
        let scrambler_idx = rng.random_range(0..scramblers.len());
        rules.push(scramblers[scrambler_idx].clone());

        for _ in 1..size {
            let idx = rng.random_range(0..destructive_pool.len());
            rules.push(destructive_pool[idx].clone());
        }

        for i in (1..rules.len()).rev() {
            let j = rng.random_range(0..=i);
            rules.swap(i, j);
        }

        rules
    }
}

// ===================================================================
// Rule generator
// ===================================================================

/// Generate a rule set with a given structured ratio.
///
/// Pre-generates subsets for reproducibility, then cycles through them.
pub fn spectrum_rule_generator(
    n_subsets: usize,
    seed: u64,
) -> impl FnMut(&mut dyn Rng) -> (Vec<RewriteRule>, f64) {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    let structured_pool = create_structured_rules();
    let destructive_pool = create_destructive_rules();
    let ratios = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];

    let mut rng = StdRng::seed_from_u64(seed);
    let subsets = generate_mixed_rule_subsets(
        &structured_pool,
        &destructive_pool,
        n_subsets,
        5,
        &ratios,
        &mut rng,
    );

    let mut index = 0usize;
    move |_rng: &mut dyn Rng| {
        let subset = subsets[index % subsets.len()].clone();
        index += 1;
        subset
    }
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rule;
    use rand::{SeedableRng, rngs::StdRng};

    #[test]
    fn test_create_universe() {
        let mut rng = StdRng::seed_from_u64(42);
        let universe = BinaryGraphUniverse::new(3, "compound", &mut rng);
        assert_eq!(universe.n_vertices(), 3);
        assert_eq!(universe.obs_name(), "compound");
        assert_eq!(universe.state_space().len(), 500);
    }

    #[test]
    fn test_null_rules_contain_scrambler() {
        let mut rng = StdRng::seed_from_u64(42);
        let universe = BinaryGraphUniverse::new(3, "compound", &mut rng);
        let mut rng = StdRng::seed_from_u64(99);

        for _ in 0..20 {
            let rules = universe.null_rules(&mut rng);
            assert!(
                rules
                    .iter()
                    .any(|r| r.name().starts_with("DESTROY_SCRAMBLE_ALL")),
                "Null rules must contain at least one scrambler"
            );
        }
    }

    #[test]
    fn test_observer_from_name() {
        assert!(matches!(
            GraphObserver::from_name("compound"),
            GraphObserver::Compound
        ));
        assert!(matches!(
            GraphObserver::from_name("label_sum"),
            GraphObserver::LabelSum
        ));
        assert!(matches!(
            GraphObserver::from_name("unknown"),
            GraphObserver::FullState
        ));
    }
}
