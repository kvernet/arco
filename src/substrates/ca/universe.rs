//! Cellular Automaton Universe — the CA substrate.
//!
//! This module provides [`CAUniverse`], which bundles the CA state
//! space, rules, observation operators, and schedule into a single
//! type implementing [`InformationUniverse`].
//!
//! # Type parameters
//!
//! - `N`: Number of cells (typically 8 for elementary CA).
//! - `R`: Neighborhood radius (default 1).

use rand::{Rng, RngExt};

use crate::substrates::ca::observation::CAObserver;
use crate::substrates::ca::rules::CARule;
use crate::substrates::ca::schedule::SynchronousCASchedule;
use crate::substrates::ca::state::CAState;
use crate::universe::InformationUniverse;

// ===================================================================
// CAUniverse
// ===================================================================

/// The Cellular Automaton Universe.
///
/// Bundles 1D binary cellular automata with periodic boundaries,
/// lookup-table rules, observation operators, and synchronous
/// schedule into a complete Information Universe.
///
/// # Type parameters
/// - `N`: Number of cells. State space = 2^N.
/// - `R`: Neighborhood radius (default 1). Rule space = 2^(2^(2R+1)).
///
/// # Usage
///
/// ```rust,no_run
/// use arco::substrates::ca::CAUniverse;
/// use arco::cycle::{CycleConfig, run_cycle};
/// use rand::SeedableRng;
/// use rand::rngs::StdRng;
///
/// let mut rng = StdRng::seed_from_u64(42);
/// let universe = CAUniverse::<8, 1>::new("full_state", &mut rng, 400);
/// let config = CycleConfig::default();
/// let mut hypotheses = vec![];
/// let record = run_cycle(&universe, &config, &mut hypotheses, None);
/// ```

#[derive(Debug, Clone)]
pub struct CAUniverse<const N: usize, const R: usize = 1> {
    state_space: Vec<CAState<N, R>>,
    /// Pre-generated rule sets for reproducible experiments
    rules: Vec<CARule<N, R>>,
    observer: CAObserver,
    schedule: SynchronousCASchedule,
}

impl<const N: usize, const R: usize> CAUniverse<N, R> {
    /// Create a new CAUniverse.
    ///
    /// # Arguments
    /// * `obs_name` — Observation operator name. One of: `"full_state"`,
    ///   `"density"`, `"parity"`.
    /// * `rng` — Random number generator for state space and rule generation.
    /// * `n_rules` — Number of rules to pre-generate.
    pub fn new(obs_name: &str, rng: &mut impl Rng, n_rules: usize) -> Self {
        let state_space: Vec<CAState<N, R>> =
            (0..500).map(|_| CAState::<N, R>::random(rng)).collect();

        // Generate rules across the spectrum: some Wolfram, some random
        let mut rules = Vec::with_capacity(n_rules);
        if R == 1 {
            // For elementary CA, include all 256 Wolfram rules
            for wn in 0..=255u64 {
                rules.push(CARule::<N, R>::from_wolfram_number(wn));
            }
        }
        // Fill remaining with random rules
        while rules.len() < n_rules {
            rules.push(CARule::<N, R>::random(rng));
        }

        Self {
            state_space,
            rules,
            observer: CAObserver::from_name(obs_name),
            schedule: SynchronousCASchedule::new(),
        }
    }

    /// Name of the observation operator.
    pub fn obs_name(&self) -> &str {
        self.observer.name()
    }
}

impl<const N: usize, const R: usize> InformationUniverse for CAUniverse<N, R> {
    type State = CAState<N, R>;
    type Rule = CARule<N, R>;
    type Observation = CAObserver;
    type Schedule = SynchronousCASchedule;

    fn state_space(&self) -> &[Self::State] {
        &self.state_space
    }

    fn observation(&self) -> &Self::Observation {
        &self.observer
    }

    fn schedule(&self) -> &Self::Schedule {
        &self.schedule
    }

    fn generate_rules(&self, rng: &mut dyn Rng) -> (Vec<Self::Rule>, f64) {
        // Sample uniformly from the Wolfram-numbered rules.
        // This ensures both train and test sets see the full distribution
        // of rule numbers, avoiding the sequential cycling bug where test
        // data could structurally exclude hypothesis-positive cases.
        let wolfram_rules: Vec<&CARule<N, R>> = self
            .rules
            .iter()
            .filter(|r| r.wolfram_number().is_some())
            .collect();

        if wolfram_rules.is_empty() {
            // Fallback: pick any rule
            let idx = rng.random_range(0..self.rules.len());
            let rule = self.rules[idx].clone();
            let lambda = rule.lambda();
            let structured_ratio = if !(0.2..=0.8).contains(&lambda) {
                0.2
            } else {
                0.8
            };
            return (vec![rule], structured_ratio);
        }

        let idx = rng.random_range(0..wolfram_rules.len());
        let rule = wolfram_rules[idx].clone();
        let lambda = rule.lambda();
        let structured_ratio = if !(0.2..=0.8).contains(&lambda) {
            0.2
        } else {
            0.8
        };

        (vec![rule], structured_ratio)
    }

    fn null_rules(&self, rng: &mut dyn Rng) -> Vec<Self::Rule> {
        // Pool of known chaotic rules for R=1
        let chaotic: &[u64] = &[30, 45, 86, 106, 135, 149];
        if R == 1 {
            let size = rng.random_range(1..=3);
            let mut rules = Vec::with_capacity(size);
            for _ in 0..size {
                let idx = rng.random_range(0..chaotic.len());
                rules.push(CARule::<N, R>::from_wolfram_number(chaotic[idx]));
            }
            rules
        } else {
            let size = rng.random_range(1..=3);
            (0..size).map(|_| CARule::<N, R>::random(rng)).collect()
        }
    }
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rule;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_create_universe() {
        let mut rng = StdRng::seed_from_u64(42);
        let universe = CAUniverse::<8, 1>::new("full_state", &mut rng, 300);
        assert_eq!(universe.obs_name(), "full_state");
        assert_eq!(universe.state_space().len(), 500);
    }

    #[test]
    fn test_generate_rules_produces_valid_rule() {
        let mut rng = StdRng::seed_from_u64(42);
        let universe = CAUniverse::<8, 1>::new("full_state", &mut rng, 300);
        let mut test_rng = StdRng::seed_from_u64(0);

        let (rules, ratio) = universe.generate_rules(&mut test_rng);
        assert_eq!(rules.len(), 1);
        assert!(ratio >= 0.0 && ratio <= 1.0);
        assert!(rules[0].name().contains("Rule"));
    }

    #[test]
    fn test_null_rules_samples_from_chaotic_pool() {
        let mut rng = StdRng::seed_from_u64(42);
        let universe = CAUniverse::<8, 1>::new("full_state", &mut rng, 300);
        let mut test_rng = StdRng::seed_from_u64(0);

        let rules = universe.null_rules(&mut test_rng);
        assert!(!rules.is_empty());
        // All rules should be from the chaotic pool
        let chaotic: &[u64] = &[30, 45, 86, 106, 135, 149];
        for rule in &rules {
            let wn = rule.wolfram_number().unwrap();
            assert!(chaotic.contains(&wn), "Rule {} not in chaotic pool", wn);
        }
    }
}
