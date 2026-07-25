//! Graph rewrite rules for the Binary Graph Universe.
//!
//! This module provides [`RewriteRule`], a local graph rewrite rule
//! with pattern matching via [`MatchInfo`]. It includes generators
//! for structured rules (logic gates, transport, identity) and
//! destructive rules (scramblers, constants) used for null-
//! distribution calibration.
//!
//! # Rule classification
//!
//! - **Structured**: semantically meaningful information-processing
//!   operations (IDENTITY, NAND, PROPAGATE, SWAP, etc.).
//! - **Destructive**: entropy-increasing operations for null-
//!   distribution calibration (SCRAMBLE_ALL, RANDOMIZE, ZERO, ONE).
//!
//! # Design
//!
//! Rules are immutable and stateless. Randomness comes from an
//! externally provided RNG. Rules implement the core [`Rule`] trait
//! for [`BinaryGraphState`].

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use rand::{Rng, RngExt};

use crate::rules::{Rule, RuleContext};
use crate::substrates::graph::state::BinaryGraphState;
use crate::types::{ActionFn, ConditionFn};

// ===================================================================
// MatchInfo
// ===================================================================

/// Context captured during rule matching, passed to `apply()`.
///
/// Carries the vertex where the rule fires and any additional context
/// the action needs (incoming neighbors, outgoing neighbors, swap
/// target). This avoids double-scanning the state and guarantees
/// that `apply()` uses the same match data that `matches()` found.
#[derive(Debug, Clone)]
pub enum MatchInfo {
    /// Rule fires unconditionally at this vertex.
    Unconditional { vertex: usize },
    /// Rule fires at vertex, using these specific incoming neighbors.
    Incoming { vertex: usize, sources: Vec<usize> },
    /// Rule fires at vertex, using these specific outgoing neighbors.
    Outgoing { vertex: usize, targets: Vec<usize> },
    /// Rule fires at vertex, swapping with this specific neighbor.
    Swap { vertex: usize, other: usize },
}

impl MatchInfo {
    /// The vertex where the rule fires.
    pub fn vertex(&self) -> usize {
        match self {
            MatchInfo::Unconditional { vertex } => *vertex,
            MatchInfo::Incoming { vertex, .. } => *vertex,
            MatchInfo::Outgoing { vertex, .. } => *vertex,
            MatchInfo::Swap { vertex, .. } => *vertex,
        }
    }
}

impl RuleContext for MatchInfo {}

// ===================================================================
// RewriteRule
// ===================================================================

/// A concrete graph rewrite rule with explicit semantics.
///
/// Uses `Arc` internally for function fields, enabling cheap cloning
/// via reference counting. This is required for [`compose`] to
/// capture rules in closures.
///
/// Implements the core [`Rule`] trait for [`BinaryGraphState`].
///
/// # Examples
///
/// ```rust
/// use arco::{
///     rules::Rule,
///     substrates::graph::{BinaryGraphState, MatchInfo, RewriteRule},
/// };
/// use ndarray::{arr1, arr2};
/// use rand::{SeedableRng, rngs::StdRng};
///
/// fn main() {
///     let mut rng = StdRng::seed_from_u64(42);
///     let rule = RewriteRule::new(
///         "NAND",
///         "structured",
///         |state, vertex| {
///             let mut sources = Vec::new();
///             for src in 0..state.n_vertices() {
///                 if state.edge(src, vertex) == 1 {
///                     sources.push(src);
///                     if sources.len() == 2 {
///                         return Some(MatchInfo::Incoming { vertex, sources });
///                     }
///                 }
///             }
///             None
///         },
///         |state, info, _rng| {
///             if let MatchInfo::Incoming { vertex, sources } = info {
///                 let a = state.label(sources[0]);
///                 let b = state.label(sources[1]);
///                 let value = 1 - (a & b);
///                 return state.mutate_label(*vertex, value).unwrap();
///             }
///             state.clone()
///         },
///         true,
///         1,
///     );
///
///     println!("\n\n==== {} ====", rule.name());
///
///     let truth_table = [((0, 0), 1), ((0, 1), 1), ((1, 0), 1), ((1, 1), 0)];
///     let vertex = 2;
///     for ((a, b), expected) in truth_table {
///         let mut state = BinaryGraphState::new(
///             3,
///             arr2(&[[0, 0, 1], [0, 0, 1], [0, 0, 0]]).view(),
///             arr1(&[a, b, 0]).view(),
///         )
///         .unwrap();
///         let context = rule.matches(&state, vertex).unwrap();
///         state = rule.apply(&state, &context, &mut rng);
///
///         assert_eq!(state.label(vertex), expected);
///
///         println!(" {}    {}    {}", a, b, state.label(vertex));
///     }
///     println!("\n");
/// }
/// ```
#[derive(Clone)]
pub struct RewriteRule {
    name: String,
    rule_type: String,
    deterministic: bool,
    locality: usize,
    condition_fn: ConditionFn,
    action_fn: ActionFn,
    _id: (String, String),
}

impl RewriteRule {
    /// Create a new RewriteRule.
    ///
    /// # Arguments
    /// * `name` — Unique rule identifier.
    /// * `rule_type` — `"structured"` or `"destructive"`.
    /// * `condition_fn` — `(state, vertex) -> Option<MatchInfo>`.
    /// * `action_fn` — `(state, match_info, rng) -> BinaryGraphState`.
    /// * `deterministic` — Whether the rule ignores the RNG.
    /// * `locality_radius` — Maximum graph distance affected.
    pub fn new(
        name: impl Into<String>,
        rule_type: impl Into<String>,
        condition_fn: impl Fn(&BinaryGraphState, usize) -> Option<MatchInfo> + Send + Sync + 'static,
        action_fn: impl Fn(&BinaryGraphState, &MatchInfo, &mut dyn Rng) -> BinaryGraphState
        + Send
        + Sync
        + 'static,
        deterministic: bool,
        locality_radius: usize,
    ) -> Self {
        let name = name.into();
        let rule_type = rule_type.into();

        assert!(
            rule_type == "structured" || rule_type == "destructive",
            "rule_type must be 'structured' or 'destructive', got '{rule_type}'"
        );

        Self {
            _id: (name.clone(), rule_type.clone()),
            name,
            rule_type,
            deterministic,
            locality: locality_radius,
            condition_fn: Arc::new(condition_fn),
            action_fn: Arc::new(action_fn),
        }
    }

    /// `"structured"` or `"destructive"`.
    pub fn rule_type(&self) -> &str {
        &self.rule_type
    }

    /// Whether the rule always produces the same output for the same
    /// (state, match_info) pair.
    pub fn is_deterministic(&self) -> bool {
        self.deterministic
    }

    /// Maximum graph distance affected by this rule.
    pub fn locality_radius(&self) -> usize {
        self.locality
    }

    /// Test whether this rule can fire at the given vertex.
    /// Returns `Some(MatchInfo)` with match context, or `None`.
    pub fn matches(&self, state: &BinaryGraphState, vertex: usize) -> Option<MatchInfo> {
        (self.condition_fn)(state, vertex)
    }
}

impl Rule<BinaryGraphState> for RewriteRule {
    type Context = MatchInfo;

    fn name(&self) -> &str {
        &self.name
    }

    fn apply(
        &self,
        state: &BinaryGraphState,
        context: &Self::Context,
        rng: &mut dyn Rng,
    ) -> BinaryGraphState {
        (self.action_fn)(state, context, rng)
    }
}

impl fmt::Debug for RewriteRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RewriteRule({}, type={}, det={}, r={})",
            self.name, self.rule_type, self.deterministic, self.locality
        )
    }
}

impl PartialEq for RewriteRule {
    fn eq(&self, other: &Self) -> bool {
        self._id == other._id
    }
}

impl Eq for RewriteRule {}

impl Hash for RewriteRule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self._id.hash(state);
    }
}

// ===================================================================
// Rule composition
// ===================================================================

/// Compose two rules sequentially: apply `r1`, then `r2`.
///
/// The composed rule matches if `r1` matches, and its action is `r2`
/// applied after `r1`. The name follows mathematical notation:
/// `compose(r1, r2)` produces `(r2∘r1)` — "apply r1 first, then r2."
///
/// Uses `Arc` internally — cloning the source rules is cheap.
pub fn compose(r1: &RewriteRule, r2: &RewriteRule) -> RewriteRule {
    let r1a = r1.clone();
    let r1b = r1.clone();
    let r2a = r2.clone();
    let r2b = r2.clone();

    let det = r1.is_deterministic() && r2.is_deterministic();
    let loc = r1.locality_radius().max(r2.locality_radius());

    RewriteRule::new(
        format!("({}∘{})", r2.name(), r1.name()),
        r1.rule_type().to_string(),
        move |state, vertex| r1a.matches(state, vertex),
        move |state, info, rng| {
            let intermediate = r1b.apply(state, info, rng);
            let v = info.vertex();
            if let Some(info2) = r2a.matches(&intermediate, v) {
                r2b.apply(&intermediate, &info2, rng)
            } else {
                intermediate
            }
        },
        det,
        loc,
    )
}

// ===================================================================
// Structured rule generators
// ===================================================================

/// Create the standard set of 16 semantically meaningful rewrite rules.
///
/// Rule classification by locality class:
/// - **Pointwise** (radius 0): IDENTITY, TOGGLE, PRESERVE_NOISY,
///   CONST_0, CONST_1
/// - **Neighborhood-read** (radius 1): COPY_FROM_IN, NAND, NOR, AND,
///   OR, XOR, NOT, MAJORITY
/// - **Multi-write** (radius 1, multiple writes): COPY_TO_OUT, SWAP,
///   PROPAGATE
#[allow(clippy::needless_range_loop)]
pub fn create_structured_rules() -> Vec<RewriteRule> {
    let mut rules = Vec::new();

    // R0: IDENTITY
    rules.push(RewriteRule::new(
        "IDENTITY",
        "structured",
        |_, vertex| Some(MatchInfo::Unconditional { vertex }),
        |state, _, _| state.clone(),
        true,
        0,
    ));

    // R1: TOGGLE
    rules.push(RewriteRule::new(
        "TOGGLE",
        "structured",
        |_, vertex| Some(MatchInfo::Unconditional { vertex }),
        |state, info, _| {
            let val = state.label(info.vertex());
            state.mutate_label(info.vertex(), 1 - val).unwrap()
        },
        true,
        0,
    ));

    // R2: COPY_FROM_IN
    rules.push(RewriteRule::new(
        "COPY_FROM_IN",
        "structured",
        |state, vertex| {
            let n = state.n_vertices();
            for i in 0..n {
                if state.edge(i, vertex) == 1 {
                    return Some(MatchInfo::Incoming {
                        vertex,
                        sources: vec![i],
                    });
                }
            }
            None
        },
        |state, info, _| {
            if let MatchInfo::Incoming { vertex, sources } = info {
                let src_label = state.label(sources[0]);
                state.mutate_label(*vertex, src_label).unwrap()
            } else {
                state.clone()
            }
        },
        true,
        1,
    ));

    // R3: COPY_TO_OUT
    rules.push(RewriteRule::new(
        "COPY_TO_OUT",
        "structured",
        |state, vertex| {
            let n = state.n_vertices();
            for j in 0..n {
                if state.edge(vertex, j) == 1 {
                    return Some(MatchInfo::Outgoing {
                        vertex,
                        targets: vec![j],
                    });
                }
            }
            None
        },
        |state, info, _| {
            if let MatchInfo::Outgoing { vertex, targets } = info {
                let src_label = state.label(*vertex);
                state.mutate_label(targets[0], src_label).unwrap()
            } else {
                state.clone()
            }
        },
        true,
        1,
    ));

    // R4-R8: Logic gates (NAND, NOR, AND, OR, XOR)
    fn build_logic_gate(name: &str, op: fn(u8, u8) -> u8) -> RewriteRule {
        let gate_condition = |state: &BinaryGraphState, vertex: usize| -> Option<MatchInfo> {
            let n = state.n_vertices();
            let mut sources = Vec::new();
            for i in 0..n {
                if state.edge(i, vertex) == 1 {
                    sources.push(i);
                    if sources.len() == 2 {
                        return Some(MatchInfo::Incoming { vertex, sources });
                    }
                }
            }
            None
        };

        RewriteRule::new(
            name,
            "structured",
            gate_condition,
            move |state, info, _| {
                if let MatchInfo::Incoming { vertex, sources } = info {
                    let a = state.label(sources[0]);
                    let b = state.label(sources[1]);
                    state.mutate_label(*vertex, op(a, b)).unwrap()
                } else {
                    state.clone()
                }
            },
            true,
            1,
        )
    }

    // NAND
    rules.push(build_logic_gate("NAND", |a, b| 1 - (a & b)));
    // NOR
    rules.push(build_logic_gate("NOR", |a, b| 1 - (a | b)));
    // AND
    rules.push(build_logic_gate("AND", |a, b| a & b));
    // OR
    rules.push(build_logic_gate("OR", |a, b| a | b));
    // XOR
    rules.push(build_logic_gate("XOR", |a, b| a ^ b));

    // R9: NOT
    rules.push(RewriteRule::new(
        "NOT",
        "structured",
        |state, vertex| {
            let n = state.n_vertices();
            for i in 0..n {
                if state.edge(i, vertex) == 1 {
                    return Some(MatchInfo::Incoming {
                        vertex,
                        sources: vec![i],
                    });
                }
            }
            None
        },
        |state, info, _| {
            if let MatchInfo::Incoming { vertex, sources } = info {
                let src_label = state.label(sources[0]);
                state.mutate_label(*vertex, 1 - src_label).unwrap()
            } else {
                state.clone()
            }
        },
        true,
        1,
    ));

    // R10: SWAP
    rules.push(RewriteRule::new(
        "SWAP",
        "structured",
        |state, vertex| {
            let n = state.n_vertices();
            for j in 0..n {
                if state.edge(vertex, j) == 1 {
                    return Some(MatchInfo::Swap { vertex, other: j });
                }
            }
            for j in 0..n {
                if state.edge(j, vertex) == 1 {
                    return Some(MatchInfo::Swap { vertex, other: j });
                }
            }
            None
        },
        |state, info, _| {
            if let MatchInfo::Swap { vertex, other } = info {
                let a = state.label(*vertex);
                let b = state.label(*other);
                let n = state.n_vertices();
                let mut labels: Vec<u8> = (0..n).map(|i| state.label(i)).collect();
                labels[*vertex] = b;
                labels[*other] = a;
                state.mutate_labels(&labels).unwrap()
            } else {
                state.clone()
            }
        },
        true,
        1,
    ));

    // R11: PROPAGATE (multi-write)
    rules.push(RewriteRule::new(
        "PROPAGATE",
        "structured",
        |state, vertex| {
            let n = state.n_vertices();
            let mut targets = Vec::new();
            for j in 0..n {
                if state.edge(vertex, j) == 1 {
                    targets.push(j);
                }
            }
            if targets.is_empty() {
                None
            } else {
                Some(MatchInfo::Outgoing { vertex, targets })
            }
        },
        |state, info, _| {
            if let MatchInfo::Outgoing { vertex, targets } = info {
                let src_label = state.label(*vertex);
                let n = state.n_vertices();
                let mut labels: Vec<u8> = (0..n).map(|i| state.label(i)).collect();
                for &t in targets {
                    labels[t] = src_label;
                }
                state.mutate_labels(&labels).unwrap()
            } else {
                state.clone()
            }
        },
        true,
        1,
    ));

    // R12: PRESERVE_NOISY
    rules.push(RewriteRule::new(
        "PRESERVE_NOISY",
        "structured",
        |_, vertex| Some(MatchInfo::Unconditional { vertex }),
        |state, info, rng| {
            if rng.random_bool(0.1) {
                let val = state.label(info.vertex());
                state.mutate_label(info.vertex(), 1 - val).unwrap()
            } else {
                state.clone()
            }
        },
        false,
        0,
    ));

    // R13: CONST_0
    rules.push(RewriteRule::new(
        "CONST_0",
        "structured",
        |_, vertex| Some(MatchInfo::Unconditional { vertex }),
        |state, info, _| state.mutate_label(info.vertex(), 0).unwrap(),
        true,
        0,
    ));

    // R14: CONST_1
    rules.push(RewriteRule::new(
        "CONST_1",
        "structured",
        |_, vertex| Some(MatchInfo::Unconditional { vertex }),
        |state, info, _| state.mutate_label(info.vertex(), 1).unwrap(),
        true,
        0,
    ));

    // R15: MAJORITY
    rules.push(RewriteRule::new(
        "MAJORITY",
        "structured",
        |state, vertex| {
            let n = state.n_vertices();
            let mut sources = Vec::new();
            for i in 0..n {
                if state.edge(i, vertex) == 1 {
                    sources.push(i);
                }
            }
            if sources.len() >= 3 {
                Some(MatchInfo::Incoming { vertex, sources })
            } else {
                None
            }
        },
        |state, info, _| {
            if let MatchInfo::Incoming { vertex, sources } = info {
                let mut ones = 0u32;
                let total = sources.len() as u32;
                for &src in sources {
                    ones += state.label(src) as u32;
                }
                let result = if ones > total - ones { 1 } else { 0 };
                state.mutate_label(*vertex, result).unwrap()
            } else {
                state.clone()
            }
        },
        true,
        1,
    ));

    rules
}

// ===================================================================
// Destructive rule generator
// ===================================================================

/// Create destructive rules for null-distribution calibration.
///
/// The 5:1 weighting of SCRAMBLE_ALL ensures the null distribution
/// is properly destructive.
pub fn create_destructive_rules() -> Vec<RewriteRule> {
    let mut rules = Vec::new();

    // SCRAMBLE_ALL (5 copies for weighting)
    for k in 0..5 {
        rules.push(RewriteRule::new(
            format!("DESTROY_SCRAMBLE_ALL_{}", k),
            "destructive",
            |_, vertex| Some(MatchInfo::Unconditional { vertex }),
            |state, _, rng| {
                let n = state.n_vertices();
                let new_labels: Vec<u8> = (0..n).map(|_| rng.random_range(0..=1)).collect();
                state.mutate_labels(&new_labels).unwrap()
            },
            false,
            usize::MAX,
        ));
    }

    // RANDOMIZE
    rules.push(RewriteRule::new(
        "DESTROY_RANDOMIZE",
        "destructive",
        |_, vertex| Some(MatchInfo::Unconditional { vertex }),
        |state, info, rng| {
            state
                .mutate_label(info.vertex(), rng.random_range(0..=1))
                .unwrap()
        },
        false,
        0,
    ));

    // ZERO_OUT
    rules.push(RewriteRule::new(
        "DESTROY_ZERO",
        "destructive",
        |_, vertex| Some(MatchInfo::Unconditional { vertex }),
        |state, info, _| state.mutate_label(info.vertex(), 0).unwrap(),
        true,
        0,
    ));

    // ONE_OUT
    rules.push(RewriteRule::new(
        "DESTROY_ONE",
        "destructive",
        |_, vertex| Some(MatchInfo::Unconditional { vertex }),
        |state, info, _| state.mutate_label(info.vertex(), 1).unwrap(),
        true,
        0,
    ));

    rules
}

// ===================================================================
// Rule set generation for spectrum experiments
// ===================================================================

/// Generate rule subsets spanning the structured/destructive spectrum.
pub fn generate_mixed_rule_subsets(
    structured_rules: &[RewriteRule],
    destructive_rules: &[RewriteRule],
    n_subsets: usize,
    max_size: usize,
    ratios: &[f64],
    rng: &mut impl Rng,
) -> Vec<(Vec<RewriteRule>, f64)> {
    let mut subsets = Vec::new();
    let per_ratio = n_subsets / ratios.len() + 1;

    for &target_ratio in ratios {
        for _ in 0..per_ratio {
            let size = rng.random_range(1..=max_size);

            let (n_struct, n_destr) = if target_ratio == 0.0 {
                (0, size)
            } else if target_ratio == 1.0 {
                (size, 0)
            } else {
                let ns = (size as f64 * target_ratio).round() as usize;
                let ns = ns.min(size);
                (ns, size - ns)
            };

            let mut selected = Vec::new();

            if n_struct > 0 && !structured_rules.is_empty() {
                let n_available = structured_rules.len();
                if n_struct <= n_available {
                    let mut indices: Vec<usize> = (0..n_available).collect();
                    for i in 0..n_struct {
                        let j = rng.random_range(i..n_available);
                        indices.swap(i, j);
                        selected.push(structured_rules[indices[i]].clone());
                    }
                } else {
                    for rule in structured_rules {
                        selected.push(rule.clone());
                    }
                    for _ in 0..(n_struct - n_available) {
                        let idx = rng.random_range(0..n_available);
                        selected.push(structured_rules[idx].clone());
                    }
                }
            }

            if n_destr > 0 && !destructive_rules.is_empty() {
                let n_available = destructive_rules.len();
                if n_destr <= n_available {
                    let mut indices: Vec<usize> = (0..n_available).collect();
                    for i in 0..n_destr {
                        let j = rng.random_range(i..n_available);
                        indices.swap(i, j);
                        selected.push(destructive_rules[indices[i]].clone());
                    }
                } else {
                    for rule in destructive_rules {
                        selected.push(rule.clone());
                    }
                    for _ in 0..(n_destr - n_available) {
                        let idx = rng.random_range(0..n_available);
                        selected.push(destructive_rules[idx].clone());
                    }
                }
            }

            if selected.is_empty() {
                continue;
            }

            // Fisher-Yates shuffle
            for i in (1..selected.len()).rev() {
                let j = rng.random_range(0..=i);
                selected.swap(i, j);
            }

            let actual_ratio = selected
                .iter()
                .filter(|r| r.rule_type() == "structured")
                .count() as f64
                / selected.len() as f64;

            subsets.push((selected, actual_ratio));
        }
    }

    // Shuffle the subsets themselves
    for i in (1..subsets.len()).rev() {
        let j = rng.random_range(0..=i);
        subsets.swap(i, j);
    }

    subsets.truncate(n_subsets);
    subsets
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;
    use ndarray::{arr1, arr2};
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_structured_rules_count() {
        let rules = create_structured_rules();
        assert_eq!(rules.len(), 16);
    }

    #[test]
    fn test_destructive_rules_count() {
        let rules = create_destructive_rules();
        assert_eq!(rules.len(), 8);
    }

    #[test]
    fn test_identity_rule_preserves_state() {
        let adj = arr2(&[[0, 1], [0, 0]]);
        let labels = arr1(&[1, 0]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();

        let rules = create_structured_rules();
        let identity = rules.iter().find(|r| r.name() == "IDENTITY").unwrap();

        let mut rng = StdRng::seed_from_u64(42);
        let info = identity.matches(&state, 0).unwrap();
        let result = identity.apply(&state, &info, &mut rng);
        assert_eq!(state.canonical_encoding(), result.canonical_encoding());
    }

    #[test]
    fn test_toggle_flips_label() {
        let adj = arr2(&[[0, 0], [0, 0]]);
        let labels = arr1(&[0, 0]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();

        let rules = create_structured_rules();
        let toggle = rules.iter().find(|r| r.name() == "TOGGLE").unwrap();

        let mut rng = StdRng::seed_from_u64(42);
        let info = toggle.matches(&state, 0).unwrap();
        let result = toggle.apply(&state, &info, &mut rng);
        assert_eq!(result.label(0), 1);
        assert_eq!(state.label(0), 0);
    }

    #[test]
    fn test_nand_truth_table() {
        let rules = create_structured_rules();
        let nand = rules.iter().find(|r| r.name() == "NAND").unwrap();

        let truth_table = vec![((0, 0), 1), ((0, 1), 1), ((1, 0), 1), ((1, 1), 0)];

        let mut rng = StdRng::seed_from_u64(42);

        for ((a, b), expected) in truth_table {
            let adj = arr2(&[[0, 0, 1], [0, 0, 1], [0, 0, 0]]);
            let labels = arr1(&[a, b, 0]);
            let state = BinaryGraphState::new(3, adj.view(), labels.view()).unwrap();

            let info = nand.matches(&state, 2).unwrap();
            let result = nand.apply(&state, &info, &mut rng);
            assert_eq!(
                result.label(2),
                expected,
                "NAND({}, {}) should be {}",
                a,
                b,
                expected
            );
        }
    }

    #[test]
    fn test_nand_no_match_on_few_inputs() {
        let rules = create_structured_rules();
        let nand = rules.iter().find(|r| r.name() == "NAND").unwrap();

        let adj = arr2(&[[0, 0, 1], [0, 0, 0], [0, 0, 0]]);
        let labels = arr1(&[0, 0, 0]);
        let state = BinaryGraphState::new(3, adj.view(), labels.view()).unwrap();

        assert!(nand.matches(&state, 2).is_none());
    }

    #[test]
    fn test_apply_safe_with_wrong_match_info() {
        let rules = create_structured_rules();
        let nand = rules.iter().find(|r| r.name() == "NAND").unwrap();

        let adj = arr2(&[[0, 0, 0], [0, 0, 0], [0, 0, 0]]);
        let labels = arr1(&[0, 0, 0]);
        let state = BinaryGraphState::new(3, adj.view(), labels.view()).unwrap();

        let mut rng = StdRng::seed_from_u64(42);
        let wrong_info = MatchInfo::Unconditional { vertex: 2 };
        let result = nand.apply(&state, &wrong_info, &mut rng);
        assert_eq!(state.canonical_encoding(), result.canonical_encoding());
    }

    #[test]
    fn test_compose_applies_both_rules() {
        let adj = arr2(&[[0, 0], [0, 0]]);
        let labels = arr1(&[0, 0]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();

        let rules = create_structured_rules();
        let toggle = rules.iter().find(|r| r.name() == "TOGGLE").unwrap();

        let composed = compose(toggle, toggle);

        let mut rng = StdRng::seed_from_u64(42);
        let info = composed.matches(&state, 0).unwrap();
        let result = composed.apply(&state, &info, &mut rng);
        assert_eq!(result.label(0), state.label(0));
    }

    #[test]
    fn test_compose_name_notation() {
        let rules = create_structured_rules();
        let nand = rules.iter().find(|r| r.name() == "NAND").unwrap();
        let xor = rules.iter().find(|r| r.name() == "XOR").unwrap();

        let composed = compose(nand, xor);
        assert_eq!(composed.name(), "(XOR∘NAND)");
    }

    #[test]
    fn test_mixed_rule_subsets_span_spectrum() {
        let structured = create_structured_rules();
        let destructive = create_destructive_rules();
        let ratios = vec![0.0, 0.5, 1.0];
        let mut rng = StdRng::seed_from_u64(42);

        let subsets =
            generate_mixed_rule_subsets(&structured, &destructive, 30, 5, &ratios, &mut rng);

        assert_eq!(subsets.len(), 30);

        let all_destructive = subsets
            .iter()
            .any(|(rules, _)| rules.iter().all(|r| r.rule_type() == "destructive"));
        let all_structured = subsets
            .iter()
            .any(|(rules, _)| rules.iter().all(|r| r.rule_type() == "structured"));

        assert!(all_destructive);
        assert!(all_structured);
    }
}
