//! Information Universe container and factories.
//!
//! Per Constitution:
//!     An Information Universe is a 6-tuple U = (S, T, O, R, I, K).
//!     This module provides the container that binds these components
//!     together.
//!
//! Design commitments:
//! - The universe is a lightweight container, not a god object.
//! - Universes are configured via factory functions.
//! - Spectrum universes span structured/destructive rule ratios.

use crate::rules::{
    RewriteRule, Rule, create_destructive_rules, create_structured_rules,
    generate_mixed_rule_subsets,
};
use crate::state::BinaryGraphState;

// ===================================================================
// Information Universe
// ===================================================================

/// An Information Universe per Constitution.
///
/// U = (S, T, O, R, I, K)
///
/// This is a container that binds the state space, rule set, observation
/// operator name, resources, invariants, and schedule.
#[derive(Debug, Clone)]
pub struct InformationUniverse {
    /// Human-readable identifier.
    pub name: String,
    /// State space sample (S).
    pub state_space: Vec<BinaryGraphState>,
    /// Transformation set (T).
    pub rules: Vec<RewriteRule>,
    /// Observation operator name (key in the observation registry).
    pub obs_name: String,
    /// Fraction of rules that are structured (for spectrum analysis).
    pub structured_ratio: f64,
    /// Resource constraints (R) — placeholder.
    pub resources: std::collections::HashMap<String, String>,
    /// Invariant structure (I) — placeholder.
    pub invariants: Vec<String>,
    /// Number of vertices in the state space.
    pub n_vertices: usize,
    /// Number of structured rules.
    pub n_structured: usize,
    /// Number of destructive rules.
    pub n_destructive: usize,
}

impl InformationUniverse {
    /// Create a new InformationUniverse.
    pub fn new(
        name: impl Into<String>,
        state_space: Vec<BinaryGraphState>,
        rules: Vec<RewriteRule>,
        obs_name: impl Into<String>,
        n_vertices: usize,
    ) -> Self {
        let n_structured = rules
            .iter()
            .filter(|r| r.rule_type() == "structured")
            .count();
        let n_destructive = rules.len() - n_structured;
        let structured_ratio = if rules.is_empty() {
            0.0
        } else {
            n_structured as f64 / rules.len() as f64
        };

        let mut resources = std::collections::HashMap::new();
        resources.insert("time".to_string(), "steps".to_string());
        resources.insert("space".to_string(), format!("{} vertices", n_vertices));
        resources.insert("locality".to_string(), "1-hop".to_string());

        Self {
            name: name.into(),
            state_space,
            rules,
            obs_name: obs_name.into(),
            structured_ratio,
            resources,
            invariants: Vec::new(),
            n_vertices,
            n_structured,
            n_destructive,
        }
    }

    /// Total number of rules.
    pub fn n_rules(&self) -> usize {
        self.rules.len()
    }
}

impl std::fmt::Display for InformationUniverse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InformationUniverse({}, rules={} (S:{}/D:{}), obs={}, ratio={:.2})",
            self.name,
            self.n_rules(),
            self.n_structured,
            self.n_destructive,
            self.obs_name,
            self.structured_ratio,
        )
    }
}

// ===================================================================
// Factory functions
// ===================================================================

/// Create a Binary Graph Information Universe.
///
/// Factory function that assembles a universe with the standard
/// Binary Graph state space, the given rules, and observation operator.
///
/// # Parameters
/// * `rules` — The rule set.
/// * `name` — Universe identifier.
/// * `n_vertices` — Number of vertices per state.
/// * `obs_name` — Observation operator name.
/// * `seed` — Seed for state space generation.
pub fn create_binary_graph_universe(
    rules: Vec<RewriteRule>,
    name: impl Into<String>,
    n_vertices: usize,
    obs_name: impl Into<String>,
    seed: u64,
) -> InformationUniverse {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    let mut rng = StdRng::seed_from_u64(seed);
    let state_space: Vec<BinaryGraphState> = (0..500)
        .map(|_| BinaryGraphState::random(n_vertices, &mut rng))
        .collect();

    InformationUniverse::new(name, state_space, rules, obs_name, n_vertices)
}

/// Generate universes spanning the structured/destructive spectrum.
///
/// Creates a set of universes with varying fractions of structured
/// rules, enabling the spectrum analysis that produced the
/// Structure-Storage Gradient.
///
/// # Parameters
/// * `n_universes` — Number of universes to generate.
/// * `n_vertices` — Number of vertices per state.
/// * `obs_name` — Observation operator name.
/// * `seed` — Random seed.
pub fn generate_spectrum_universes(
    n_universes: usize,
    n_vertices: usize,
    obs_name: impl Into<String>,
    seed: u64,
) -> Vec<InformationUniverse> {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    let mut rng = StdRng::seed_from_u64(seed);
    let structured_pool = create_structured_rules();
    let destructive_pool = create_destructive_rules();

    let ratios = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
    let subsets = generate_mixed_rule_subsets(
        &structured_pool,
        &destructive_pool,
        n_universes,
        5, // max_size
        &ratios,
        &mut rng,
    );

    let obs_name = obs_name.into();
    subsets
        .into_iter()
        .enumerate()
        .map(|(i, (rules, _ratio))| {
            create_binary_graph_universe(
                rules,
                format!("spectrum_{}", i),
                n_vertices,
                &obs_name,
                seed + i as u64,
            )
        })
        .collect()
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_binary_graph_universe() {
        let rules = create_structured_rules();
        let universe = create_binary_graph_universe(rules, "test", 3, "compound", 42);

        assert_eq!(universe.name, "test");
        assert_eq!(universe.n_vertices, 3);
        assert_eq!(universe.state_space.len(), 500);
        assert!(universe.n_structured > 0);
        assert_eq!(universe.n_destructive, 0);
        assert!((universe.structured_ratio - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_create_binary_graph_universe_with_destructive() {
        let mut rules = create_structured_rules();
        rules.extend(create_destructive_rules());
        let universe = create_binary_graph_universe(rules, "mixed", 3, "compound", 42);

        assert!(universe.n_structured > 0);
        assert!(universe.n_destructive > 0);
        assert!(universe.structured_ratio < 1.0);
    }

    #[test]
    fn test_generate_spectrum_universes() {
        let universes = generate_spectrum_universes(30, 3, "compound", 42);

        assert_eq!(universes.len(), 30);

        // Should have some all-structured and some all-destructive
        let has_all_structured = universes
            .iter()
            .any(|u| (u.structured_ratio - 1.0).abs() < 1e-10);
        let has_all_destructive = universes.iter().any(|u| u.structured_ratio == 0.0);

        assert!(has_all_structured);
        assert!(has_all_destructive);
    }

    #[test]
    fn test_universe_display() {
        let rules = create_structured_rules();
        let universe = create_binary_graph_universe(rules, "display_test", 3, "compound", 42);
        let display = format!("{}", universe);
        assert!(display.contains("display_test"));
        assert!(display.contains("S:16"));
        assert!(display.contains("D:0"));
        assert!(display.contains("ratio=1.00"));
    }
}
