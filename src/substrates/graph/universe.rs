//! Binary Graph Universe — the complete validation substrate.

use std::cell::Cell;

use rand::{Rng, RngExt};

use crate::rules::Rule;
use crate::substrates::graph::observation::GraphObserver;
use crate::substrates::graph::rules::{
    RewriteRule, create_destructive_rules, create_structured_rules, generate_mixed_rule_subsets,
};
use crate::substrates::graph::schedule::AllVerticesSchedule;
use crate::substrates::graph::state::BinaryGraphState;
use crate::universe::InformationUniverse;

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
    destructive_pool: Vec<RewriteRule>,
    // Pre-generated rule subsets for reproducible spectrum experiments
    subsets: Vec<(Vec<RewriteRule>, f64)>,
    subset_index: Cell<usize>,
}

impl BinaryGraphUniverse {
    /// Create a new BinaryGraphUniverse.
    ///
    /// # Arguments
    /// * `n_vertices` — Number of vertices per state (typically 3).
    /// * `obs_name` — Observation operator name.
    /// * `rng` — Random number generator for state space and rule generation.
    /// * `n_subsets` — Number of rule subsets to pre-generate for the spectrum.
    pub fn new(n_vertices: usize, obs_name: &str, rng: &mut impl Rng, n_subsets: usize) -> Self {
        let state_space: Vec<BinaryGraphState> = (0..500)
            .map(|_| BinaryGraphState::random(n_vertices, rng))
            .collect();

        let structured_pool = create_structured_rules();
        let destructive_pool = create_destructive_rules();

        let ratios = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
        let subsets = generate_mixed_rule_subsets(
            &structured_pool,
            &destructive_pool,
            n_subsets,
            5,
            &ratios,
            rng,
        );

        Self {
            state_space,
            observer: GraphObserver::from_name(obs_name),
            schedule: AllVerticesSchedule::new(),
            n_vertices,
            destructive_pool,
            subsets,
            subset_index: std::cell::Cell::new(0),
        }
    }

    pub fn n_vertices(&self) -> usize {
        self.n_vertices
    }

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

    fn observation(&self) -> &Self::Observation {
        &self.observer
    }

    fn schedule(&self) -> &Self::Schedule {
        &self.schedule
    }

    fn generate_rules(&self, _rng: &mut dyn Rng) -> (Vec<Self::Rule>, f64) {
        let index = self.subset_index.get();
        let subset = self.subsets[index % self.subsets.len()].clone();
        self.subset_index.set(index + 1);
        subset
    }

    fn null_rules(&self, rng: &mut dyn Rng) -> Vec<Self::Rule> {
        let scramblers: Vec<&RewriteRule> = self
            .destructive_pool
            .iter()
            .filter(|r| r.name().starts_with("DESTROY_SCRAMBLE_ALL"))
            .collect();

        let size = rng.random_range(1..=5);
        let mut rules = Vec::with_capacity(size);

        let scrambler_idx = rng.random_range(0..scramblers.len());
        rules.push(scramblers[scrambler_idx].clone());

        for _ in 1..size {
            let idx = rng.random_range(0..self.destructive_pool.len());
            rules.push(self.destructive_pool[idx].clone());
        }

        for i in (1..rules.len()).rev() {
            let j = rng.random_range(0..=i);
            rules.swap(i, j);
        }

        rules
    }
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use crate::rules::Rule;

    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_create_universe() {
        let mut rng = StdRng::seed_from_u64(42);
        let universe = BinaryGraphUniverse::new(3, "compound", &mut rng, 100);
        assert_eq!(universe.n_vertices(), 3);
        assert_eq!(universe.obs_name(), "compound");
        assert_eq!(universe.state_space().len(), 500);
    }

    #[test]
    fn test_null_rules_contain_scrambler() {
        let mut rng = StdRng::seed_from_u64(42);
        let universe = BinaryGraphUniverse::new(3, "compound", &mut rng, 100);
        let mut test_rng = StdRng::seed_from_u64(99);

        for _ in 0..20 {
            let rules = universe.null_rules(&mut test_rng);
            assert!(
                rules
                    .iter()
                    .any(|r| r.name().starts_with("DESTROY_SCRAMBLE_ALL")),
                "Null rules must contain at least one scrambler"
            );
        }
    }

    #[test]
    fn test_generate_rules_produces_spectrum() {
        let mut rng = StdRng::seed_from_u64(42);
        let universe = BinaryGraphUniverse::new(3, "compound", &mut rng, 30);
        let mut test_rng = StdRng::seed_from_u64(0);

        let mut seen_structured = false;
        let mut seen_destructive = false;

        for _ in 0..30 {
            let (_rules, ratio) = universe.generate_rules(&mut test_rng);
            if ratio > 0.9 {
                seen_structured = true;
            }
            if ratio < 0.1 {
                seen_destructive = true;
            }
        }

        assert!(
            seen_structured,
            "Should generate some all-structured subsets"
        );
        assert!(
            seen_destructive,
            "Should generate some all-destructive subsets"
        );
    }
}
