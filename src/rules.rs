//! Rule trait and rewrite rule implementations.
//!
//! Per Constitution:
//!     T is a set of maps from S to S. T must form a semigroup under
//!     composition. Rules have a condition (matches) and an action (apply),
//!     returning new states.
//!
//! Rules are classified as:
//! - **structured**: semantically meaningful information-processing operations
//! - **destructive**: entropy-increasing operations for null-distribution calibration
//!
//! # MatchInfo
//!
//! When a rule matches, it returns a [`MatchInfo`] carrying the specific
//! context needed by the action (e.g., which neighbors triggered the match).
//! This avoids double-scanning the state and guarantees that `apply()` uses
//! the same match data that `matches()` found.
//!
//! Design contracts:
//! - Rules are immutable and stateless. Randomness comes from an externally
//!   provided RNG, not from rule state.
//! - Condition and action callables must be pure functions.
//! - Rule equality is based on (name, rule_type), not function identity.
//! - Rules use Arc for shared ownership, enabling Clone via reference counting.
//! - `apply()` is safe to call with any `MatchInfo` — it will not panic,
//!   though the result may be a no-op if the info doesn't match the rule.

use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use rand::{Rng, RngExt};
use sha2::{Digest, Sha256};

use crate::state::BinaryGraphState;

// ===================================================================
// Type aliases for complex function signatures
// ===================================================================

type ConditionFn = Arc<dyn Fn(&BinaryGraphState, usize) -> Option<MatchInfo> + Send + Sync>;
type ActionFn =
    Arc<dyn Fn(&BinaryGraphState, &MatchInfo, &mut dyn Rng) -> BinaryGraphState + Send + Sync>;

// ===================================================================
// MatchInfo
// ===================================================================

/// Context captured during rule matching, passed to `apply()`.
///
/// Carries the vertex where the rule fires and any additional context
/// the action needs (incoming neighbors, outgoing neighbors, swap target).
/// This avoids double-scanning the state and guarantees that `apply()`
/// uses the same match data that `matches()` found.
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

// ===================================================================
// Rule trait
// ===================================================================

/// Trait for a graph rewrite rule.
///
/// A rule has a human-readable name, a semantic type, a condition predicate
/// that returns [`MatchInfo`] on success, and an action function that consumes
/// that match info.
pub trait Rule: fmt::Debug + Send + Sync {
    /// Human-readable rule identifier.
    fn name(&self) -> &str;

    /// Either `"structured"` or `"destructive"`.
    fn rule_type(&self) -> &str;

    /// Whether the rule always produces the same output for the same
    /// (state, match_info) pair, independent of the RNG.
    fn is_deterministic(&self) -> bool;

    /// Maximum graph distance affected by this rule (0 = self only,
    /// 1 = neighbors, `usize::MAX` = global).
    fn locality_radius(&self) -> usize;

    /// Test whether this rule can fire at the given vertex.
    ///
    /// Returns `Some(MatchInfo)` with the match context if the rule applies,
    /// or `None` if it does not.
    fn matches(&self, state: &BinaryGraphState, vertex: usize) -> Option<MatchInfo>;

    /// Apply this rule using the match info from a successful match.
    ///
    /// # Safety
    ///
    /// This method is safe to call with any `MatchInfo`. If the info
    /// does not match the rule's expected variant, the call is a no-op
    /// (returns a clone of the input state). It will not panic.
    fn apply(
        &self,
        state: &BinaryGraphState,
        info: &MatchInfo,
        rng: &mut dyn Rng,
    ) -> BinaryGraphState;

    /// Stable digest of the rule's structural identity (name + type).
    fn stable_digest(&self) -> String {
        let raw = format!("{}:{}", self.name(), self.rule_type());
        let hash = Sha256::digest(raw.as_bytes());
        let mut hex = String::new();
        for byte in hash.as_slice() {
            use std::fmt::Write;
            write!(&mut hex, "{:02x}", byte).unwrap();
        }
        hex[..16].to_string()
    }
}

// ===================================================================
// Concrete rule implementation
// ===================================================================

/// A concrete graph rewrite rule with explicit semantics.
///
/// Uses `Arc` internally for function fields, enabling cheap cloning
/// via reference counting. This is required for [`compose`] to capture
/// rules in closures without unsafe code.
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
    /// * `condition_fn` — Pure function `(state, vertex) -> Option<MatchInfo>`.
    /// * `action_fn` — Pure function `(state, match_info, rng) -> BinaryGraphState`.
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

        if rule_type != "structured" && rule_type != "destructive" {
            panic!("rule_type must be 'structured' or 'destructive', got '{rule_type}'");
        }

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
}

impl Rule for RewriteRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn rule_type(&self) -> &str {
        &self.rule_type
    }

    fn is_deterministic(&self) -> bool {
        self.deterministic
    }

    fn locality_radius(&self) -> usize {
        self.locality
    }

    fn matches(&self, state: &BinaryGraphState, vertex: usize) -> Option<MatchInfo> {
        (self.condition_fn)(state, vertex)
    }

    fn apply(
        &self,
        state: &BinaryGraphState,
        info: &MatchInfo,
        rng: &mut dyn Rng,
    ) -> BinaryGraphState {
        (self.action_fn)(state, info, rng)
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
// Rule composition (semigroup requirement)
// ===================================================================

/// Compose two rules sequentially: apply `r1`, then `r2`.
///
/// The composed rule matches if `r1` matches, and its action is `r2`
/// applied after `r1`. The name follows standard mathematical notation:
/// `compose(r1, r2)` produces the rule named `(r2∘r1)` — "apply r1
/// first, then r2."
///
/// This satisfies the semigroup requirement of the Constitution.
///
/// Uses `Arc` internally — cloning the source rules is cheap (reference
/// count increment).
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
        move |state: &BinaryGraphState, vertex: usize| -> Option<MatchInfo> {
            r1a.matches(state, vertex)
        },
        move |state: &BinaryGraphState, info: &MatchInfo, rng: &mut dyn Rng| -> BinaryGraphState {
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
/// - **Pointwise** (radius 0): IDENTITY, TOGGLE, PRESERVE_NOISY, CONST_0, CONST_1
/// - **Neighborhood-read** (radius 1): COPY_FROM_IN, NAND, NOR, AND, OR, XOR, NOT, MAJORITY
/// - **Multi-write** (radius 1, multiple writes): COPY_TO_OUT, SWAP, PROPAGATE
#[allow(clippy::needless_range_loop)]
pub fn create_structured_rules() -> Vec<RewriteRule> {
    let mut rules = Vec::new();

    // ================================================================
    // Pointwise rules
    // ================================================================

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

    // R12: PRESERVE_NOISY (90% preserve, 10% flip)
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

    // ================================================================
    // Neighborhood-read rules
    // ================================================================

    // R2: COPY_FROM_IN — copies label from first incoming neighbor
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

    // R4-R8: Logic gates (NAND, NOR, AND, OR, XOR)
    // All share the same condition: at least 2 incoming edges.
    // The match info carries exactly the first two sources.

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

    // NAND
    rules.push(RewriteRule::new(
        "NAND",
        "structured",
        gate_condition,
        |state, info, _| {
            if let MatchInfo::Incoming { vertex, sources } = info {
                let a = state.label(sources[0]);
                let b = state.label(sources[1]);
                state.mutate_label(*vertex, 1 - (a & b)).unwrap()
            } else {
                state.clone()
            }
        },
        true,
        1,
    ));

    // NOR
    rules.push(RewriteRule::new(
        "NOR",
        "structured",
        gate_condition,
        |state, info, _| {
            if let MatchInfo::Incoming { vertex, sources } = info {
                let a = state.label(sources[0]);
                let b = state.label(sources[1]);
                state.mutate_label(*vertex, 1 - (a | b)).unwrap()
            } else {
                state.clone()
            }
        },
        true,
        1,
    ));

    // AND
    rules.push(RewriteRule::new(
        "AND",
        "structured",
        gate_condition,
        |state, info, _| {
            if let MatchInfo::Incoming { vertex, sources } = info {
                let a = state.label(sources[0]);
                let b = state.label(sources[1]);
                state.mutate_label(*vertex, a & b).unwrap()
            } else {
                state.clone()
            }
        },
        true,
        1,
    ));

    // OR
    rules.push(RewriteRule::new(
        "OR",
        "structured",
        gate_condition,
        |state, info, _| {
            if let MatchInfo::Incoming { vertex, sources } = info {
                let a = state.label(sources[0]);
                let b = state.label(sources[1]);
                state.mutate_label(*vertex, a | b).unwrap()
            } else {
                state.clone()
            }
        },
        true,
        1,
    ));

    // XOR
    rules.push(RewriteRule::new(
        "XOR",
        "structured",
        gate_condition,
        |state, info, _| {
            if let MatchInfo::Incoming { vertex, sources } = info {
                let a = state.label(sources[0]);
                let b = state.label(sources[1]);
                state.mutate_label(*vertex, a ^ b).unwrap()
            } else {
                state.clone()
            }
        },
        true,
        1,
    ));

    // R9: NOT — requires at least 1 incoming edge
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

    // R15: MAJORITY — requires at least 3 incoming edges
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

    // ================================================================
    // Multi-write rules
    // ================================================================

    // R3: COPY_TO_OUT — copies label to first outgoing neighbor
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

    // R10: SWAP — exchanges labels with first neighbor (outgoing preferred)
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

    // R11: PROPAGATE — copies label to ALL outgoing neighbors (multi-write)
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

    rules
}

// ===================================================================
// Destructive rule generator
// ===================================================================

/// Create destructive rules for null-distribution calibration.
///
/// Destructive rules are biased toward entropy-increasing operations
/// (randomize_label, scramble_all) to ensure the null distribution
/// represents genuine information scrambling.
///
/// The 5:1 weighting of scramble_all ensures the null distribution
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
            usize::MAX, // global
        ));
    }

    // RANDOMIZE — randomize a single label
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

    // ZERO_OUT — set a single label to 0
    rules.push(RewriteRule::new(
        "DESTROY_ZERO",
        "destructive",
        |_, vertex| Some(MatchInfo::Unconditional { vertex }),
        |state, info, _| state.mutate_label(info.vertex(), 0).unwrap(),
        true,
        0,
    ));

    // ONE_OUT — set a single label to 1
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
///
/// Returns rule subsets at specified structured-fraction ratios.
/// Rules are cloned via Arc reference counting — cheap.
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
                for _ in 0..n_struct {
                    let idx = rng.random_range(0..structured_rules.len());
                    selected.push(structured_rules[idx].clone());
                }
            }

            if n_destr > 0 && !destructive_rules.is_empty() {
                for _ in 0..n_destr {
                    let idx = rng.random_range(0..destructive_rules.len());
                    selected.push(destructive_rules[idx].clone());
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
// Rule set categorization (for hypothesis generation)
// ===================================================================

/// Extract structural predicates from a rule subset for hypothesis testing.
pub fn categorize_subset(subset: &[RewriteRule]) -> Categorization {
    let names: Vec<&str> = subset.iter().map(|r| r.name()).collect();
    let types: Vec<&str> = subset.iter().map(|r| r.rule_type()).collect();

    let n_struct = types.iter().filter(|&&t| t == "structured").count();
    let ratio = if subset.is_empty() {
        0.0
    } else {
        n_struct as f64 / subset.len() as f64
    };

    let logic_gates = ["NAND", "NOR", "AND", "OR", "XOR", "NOT"];
    let transport = ["PROPAGATE", "SWAP", "COPY_TO_OUT", "COPY_FROM_IN"];

    Categorization {
        has_structured: types.contains(&"structured"),
        all_structured: !subset.is_empty() && types.iter().all(|&t| t == "structured"),
        all_destructive: !subset.is_empty() && types.iter().all(|&t| t == "destructive"),
        majority_structured: ratio >= 0.5,
        has_logic_gate: names.iter().any(|n| logic_gates.contains(n)),
        has_transport: names.iter().any(|n| transport.contains(n)),
        has_multiple_logic: names.iter().filter(|n| logic_gates.contains(n)).count() >= 2,
        structured_ratio: ratio,
        n_structured: n_struct,
        n_rules: subset.len(),
    }
}

/// Structural categorization of a rule subset.
#[derive(Debug, Clone)]
pub struct Categorization {
    pub has_structured: bool,
    pub all_structured: bool,
    pub all_destructive: bool,
    pub majority_structured: bool,
    pub has_logic_gate: bool,
    pub has_transport: bool,
    pub has_multiple_logic: bool,
    pub structured_ratio: f64,
    pub n_structured: usize,
    pub n_rules: usize,
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;

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
        use crate::state::BinaryGraphState;
        use ndarray::{arr1, arr2};
        use rand::SeedableRng;
        use rand::rngs::StdRng;

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
        use crate::state::BinaryGraphState;
        use ndarray::{arr1, arr2};
        use rand::SeedableRng;
        use rand::rngs::StdRng;

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
        use crate::state::BinaryGraphState;
        use ndarray::{arr1, arr2};
        use rand::SeedableRng;
        use rand::rngs::StdRng;

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
        use crate::state::BinaryGraphState;
        use ndarray::{arr1, arr2};

        let rules = create_structured_rules();
        let nand = rules.iter().find(|r| r.name() == "NAND").unwrap();

        // Only 1 incoming edge — should not match
        let adj = arr2(&[[0, 0, 1], [0, 0, 0], [0, 0, 0]]);
        let labels = arr1(&[0, 0, 0]);
        let state = BinaryGraphState::new(3, adj.view(), labels.view()).unwrap();

        assert!(nand.matches(&state, 2).is_none());
    }

    #[test]
    fn test_apply_safe_with_wrong_match_info() {
        use crate::state::BinaryGraphState;
        use ndarray::{arr1, arr2};
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let rules = create_structured_rules();
        let nand = rules.iter().find(|r| r.name() == "NAND").unwrap();

        let adj = arr2(&[[0, 0, 0], [0, 0, 0], [0, 0, 0]]);
        let labels = arr1(&[0, 0, 0]);
        let state = BinaryGraphState::new(3, adj.view(), labels.view()).unwrap();

        let mut rng = StdRng::seed_from_u64(42);
        // Pass Unconditional info to a gate rule — should not panic
        let wrong_info = MatchInfo::Unconditional { vertex: 2 };
        let result = nand.apply(&state, &wrong_info, &mut rng);
        // Should return a clone (no-op) since info variant doesn't match
        assert_eq!(state.canonical_encoding(), result.canonical_encoding());
    }

    #[test]
    fn test_compose_applies_both_rules() {
        use crate::state::BinaryGraphState;
        use ndarray::{arr1, arr2};
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let adj = arr2(&[[0, 0], [0, 0]]);
        let labels = arr1(&[0, 0]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();

        let rules = create_structured_rules();
        let toggle = rules.iter().find(|r| r.name() == "TOGGLE").unwrap();

        // toggle ∘ toggle = identity (two flips = no change)
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

        // compose(nand, xor) applies NAND first, then XOR
        // Notation: (XOR∘NAND) reads "XOR after NAND"
        let composed = compose(nand, xor);
        assert_eq!(composed.name(), "(XOR∘NAND)");
    }

    #[test]
    fn test_mixed_rule_subsets_span_spectrum() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let structured = create_structured_rules();
        let destructive = create_destructive_rules();
        let ratios = vec![0.0, 0.5, 1.0];
        let mut rng = StdRng::seed_from_u64(42);

        let subsets =
            generate_mixed_rule_subsets(&structured, &destructive, 30, 5, &ratios, &mut rng);

        assert_eq!(subsets.len(), 30);

        let cats: Vec<Categorization> = subsets
            .iter()
            .map(|(rules, _)| categorize_subset(rules))
            .collect();

        assert!(cats.iter().any(|c| c.all_destructive));
        assert!(cats.iter().any(|c| c.all_structured));
    }

    #[test]
    fn test_categorize_subset() {
        let rules = create_structured_rules();
        let subset: Vec<RewriteRule> = rules.into_iter().take(3).collect();
        let cat = categorize_subset(&subset);

        assert!(cat.has_structured);
        assert!(cat.all_structured);
        assert!(!cat.all_destructive);
        assert!(cat.majority_structured);
        assert_eq!(cat.n_rules, 3);
        assert_eq!(cat.n_structured, 3);
        assert!((cat.structured_ratio - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rule_stable_digest_deterministic() {
        let rules1 = create_structured_rules();
        let rules2 = create_structured_rules();
        let d1 = rules1[0].stable_digest();
        let d2 = rules2[0].stable_digest();
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_rule_equality() {
        let rules1 = create_structured_rules();
        let rules2 = create_structured_rules();
        assert_eq!(rules1[0], rules2[0]);
        assert_ne!(rules1[0], rules1[1]);
    }

    #[test]
    fn test_clone_is_cheap() {
        let rules = create_structured_rules();
        let original = &rules[0];
        let cloned = original.clone();
        assert_eq!(original, &cloned);

        use crate::state::BinaryGraphState;
        use ndarray::{arr1, arr2};
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let adj = arr2(&[[0, 0], [0, 0]]);
        let labels = arr1(&[0, 0]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();

        let mut rng = StdRng::seed_from_u64(42);
        let info = original.matches(&state, 0).unwrap();
        let r1 = original.apply(&state, &info, &mut rng);
        let r2 = cloned.apply(&state, &info, &mut rng);
        assert_eq!(r1.canonical_encoding(), r2.canonical_encoding());
    }
}
