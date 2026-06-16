//! Rule trait and rewrite rule implementations.
//!
//! Per Constitution v0.4, Part 1.3:
//!     T is a set of maps from S to S. T must form a semigroup under
//!     composition. Rules have a condition (matches) and an action (apply),
//!     returning new states.
//!
//! Rules are classified as:
//! - **structured**: semantically meaningful information-processing operations
//! - **destructive**: entropy-increasing operations for null-distribution calibration
//!
//! Design contracts:
//! - Rules are immutable and stateless. Randomness comes from an externally
//!   provided RNG, not from rule state.
//! - Condition and action callables must be pure functions.
//! - Rule equality is based on (name, rule_type), not function identity.
//! - Rules use Arc for shared ownership, enabling Clone via reference counting.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use rand::{Rng, RngExt};
use sha2::{Digest, Sha256};

use crate::state::BinaryGraphState;

type ConditionFn = Arc<dyn Fn(&BinaryGraphState, usize) -> bool + Send + Sync>;
type ActionFn =
    Arc<dyn Fn(&BinaryGraphState, usize, &mut dyn Rng) -> BinaryGraphState + Send + Sync>;

// ===================================================================
// Rule trait
// ===================================================================

/// Trait for a graph rewrite rule.
///
/// A rule has a human-readable name, a semantic type, a condition predicate,
/// and an action function.
pub trait Rule: fmt::Debug + Send + Sync {
    /// Human-readable rule identifier.
    fn name(&self) -> &str;

    /// Either `"structured"` or `"destructive"`.
    fn rule_type(&self) -> &str;

    /// Whether the rule always produces the same output for the same
    /// (state, vertex) pair, independent of the RNG.
    fn is_deterministic(&self) -> bool;

    /// Maximum graph distance affected by this rule (0 = self only,
    /// 1 = neighbors, n = global).
    fn locality_radius(&self) -> usize;

    /// Test whether this rule can fire at the given vertex.
    fn matches(&self, state: &BinaryGraphState, vertex: usize) -> bool;

    /// Apply this rule at the given vertex, returning a new state.
    fn apply(&self, state: &BinaryGraphState, vertex: usize, rng: &mut dyn Rng)
    -> BinaryGraphState;

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
/// via reference counting. This is required for `compose()` to capture
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
    /// * `condition_fn` — Pure function `(state, vertex) -> bool`.
    /// * `action_fn` — Pure function `(state, vertex, rng) -> BinaryGraphState`.
    /// * `deterministic` — Whether the rule ignores the RNG.
    /// * `locality_radius` — Maximum graph distance affected.
    pub fn new(
        name: impl Into<String>,
        rule_type: impl Into<String>,
        condition_fn: impl Fn(&BinaryGraphState, usize) -> bool + Send + Sync + 'static,
        action_fn: impl Fn(&BinaryGraphState, usize, &mut dyn Rng) -> BinaryGraphState
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

    fn matches(&self, state: &BinaryGraphState, vertex: usize) -> bool {
        (self.condition_fn)(state, vertex)
    }

    fn apply(
        &self,
        state: &BinaryGraphState,
        vertex: usize,
        rng: &mut dyn Rng,
    ) -> BinaryGraphState {
        (self.action_fn)(state, vertex, rng)
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

/// Compose two rules sequentially: apply r1, then r2.
///
/// The composed rule matches if r1 matches, and its action is r2 applied
/// after r1. This satisfies the semigroup requirement of Constitution
/// Part 1.3.2.
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
        format!("({}∘{})", r1.name(), r2.name()),
        r1.rule_type().to_string(),
        move |state, vertex| r1a.matches(state, vertex),
        move |state, vertex, rng| {
            let intermediate = r1b.apply(state, vertex, rng);
            if r2a.matches(&intermediate, vertex) {
                r2b.apply(&intermediate, vertex, rng)
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

    // R0: IDENTITY
    rules.push(RewriteRule::new(
        "IDENTITY",
        "structured",
        |_, _| true,
        |state, _, _| state.clone(),
        true,
        0,
    ));

    // R1: TOGGLE
    rules.push(RewriteRule::new(
        "TOGGLE",
        "structured",
        |_, _| true,
        |state, vertex, _| {
            let val = state.label(vertex);
            state.mutate_label(vertex, 1 - val).unwrap()
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
                    return true;
                }
            }
            false
        },
        |state, vertex, _| {
            let n = state.n_vertices();
            for i in 0..n {
                if state.edge(i, vertex) == 1 {
                    let src_label = state.label(i);
                    return state.mutate_label(vertex, src_label).unwrap();
                }
            }
            state.clone()
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
                    return true;
                }
            }
            false
        },
        |state, vertex, _| {
            let n = state.n_vertices();
            for j in 0..n {
                if state.edge(vertex, j) == 1 {
                    let src_label = state.label(vertex);
                    return state.mutate_label(j, src_label).unwrap();
                }
            }
            state.clone()
        },
        true,
        1,
    ));

    // R4-R8: Logic gates
    let gate_condition = |state: &BinaryGraphState, vertex: usize| -> bool {
        let n = state.n_vertices();
        let mut count = 0;
        for i in 0..n {
            if state.edge(i, vertex) == 1 {
                count += 1;
            }
        }
        count >= 2
    };

    let get_two = |state: &BinaryGraphState, vertex: usize| -> (u8, u8) {
        let n = state.n_vertices();
        let mut inputs = Vec::new();
        for i in 0..n {
            if state.edge(i, vertex) == 1 {
                inputs.push(state.label(i));
                if inputs.len() == 2 {
                    break;
                }
            }
        }
        (inputs[0], inputs[1])
    };

    // NAND
    rules.push(RewriteRule::new(
        "NAND",
        "structured",
        gate_condition,
        move |state, vertex, _| {
            let (a, b) = get_two(state, vertex);
            state.mutate_label(vertex, 1 - (a & b)).unwrap()
        },
        true,
        1,
    ));

    // NOR
    rules.push(RewriteRule::new(
        "NOR",
        "structured",
        gate_condition,
        move |state, vertex, _| {
            let (a, b) = get_two(state, vertex);
            state.mutate_label(vertex, 1 - (a | b)).unwrap()
        },
        true,
        1,
    ));

    // AND
    rules.push(RewriteRule::new(
        "AND",
        "structured",
        gate_condition,
        move |state, vertex, _| {
            let (a, b) = get_two(state, vertex);
            state.mutate_label(vertex, a & b).unwrap()
        },
        true,
        1,
    ));

    // OR
    rules.push(RewriteRule::new(
        "OR",
        "structured",
        gate_condition,
        move |state, vertex, _| {
            let (a, b) = get_two(state, vertex);
            state.mutate_label(vertex, a | b).unwrap()
        },
        true,
        1,
    ));

    // XOR
    rules.push(RewriteRule::new(
        "XOR",
        "structured",
        gate_condition,
        move |state, vertex, _| {
            let (a, b) = get_two(state, vertex);
            state.mutate_label(vertex, a ^ b).unwrap()
        },
        true,
        1,
    ));

    // R9: NOT
    rules.push(RewriteRule::new(
        "NOT",
        "structured",
        |state, vertex| {
            let n = state.n_vertices();
            for i in 0..n {
                if state.edge(i, vertex) == 1 {
                    return true;
                }
            }
            false
        },
        |state, vertex, _| {
            let n = state.n_vertices();
            for i in 0..n {
                if state.edge(i, vertex) == 1 {
                    let src_label = state.label(i);
                    return state.mutate_label(vertex, 1 - src_label).unwrap();
                }
            }
            state.clone()
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
                if state.edge(vertex, j) == 1 || state.edge(j, vertex) == 1 {
                    return true;
                }
            }
            false
        },
        |state, vertex, _| {
            let n = state.n_vertices();
            for j in 0..n {
                if state.edge(vertex, j) == 1 {
                    let a = state.label(vertex);
                    let b = state.label(j);
                    let mut labels: Vec<u8> = (0..n).map(|i| state.label(i)).collect();
                    labels[vertex] = b;
                    labels[j] = a;
                    return state.mutate_labels(&labels).unwrap();
                }
            }
            for j in 0..n {
                if state.edge(j, vertex) == 1 {
                    let a = state.label(vertex);
                    let b = state.label(j);
                    let mut labels: Vec<u8> = (0..n).map(|i| state.label(i)).collect();
                    labels[vertex] = b;
                    labels[j] = a;
                    return state.mutate_labels(&labels).unwrap();
                }
            }
            state.clone()
        },
        true,
        1,
    ));

    // R11: PROPAGATE (multi-write — writes to all outgoing neighbors)
    rules.push(RewriteRule::new(
        "PROPAGATE",
        "structured",
        |state, vertex| {
            let n = state.n_vertices();
            for j in 0..n {
                if state.edge(vertex, j) == 1 {
                    return true;
                }
            }
            false
        },
        |state, vertex, _| {
            let n = state.n_vertices();
            let src_label = state.label(vertex);
            let mut labels: Vec<u8> = (0..n).map(|i| state.label(i)).collect();

            for j in 0..n {
                if state.edge(vertex, j) == 1 {
                    labels[j] = src_label;
                }
            }
            state.mutate_labels(&labels).unwrap()
        },
        true,
        1,
    ));

    // R12: PRESERVE_NOISY (90% preserve, 10% flip)
    rules.push(RewriteRule::new(
        "PRESERVE_NOISY",
        "structured",
        |_, _| true,
        |state, vertex, rng| {
            if rng.random_bool(0.1) {
                let val = state.label(vertex);
                state.mutate_label(vertex, 1 - val).unwrap()
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
        |_, _| true,
        |state, vertex, _| state.mutate_label(vertex, 0).unwrap(),
        true,
        0,
    ));

    // R14: CONST_1
    rules.push(RewriteRule::new(
        "CONST_1",
        "structured",
        |_, _| true,
        |state, vertex, _| state.mutate_label(vertex, 1).unwrap(),
        true,
        0,
    ));

    // R15: MAJORITY
    rules.push(RewriteRule::new(
        "MAJORITY",
        "structured",
        |state, vertex| {
            let n = state.n_vertices();
            let mut count = 0;
            for i in 0..n {
                if state.edge(i, vertex) == 1 {
                    count += 1;
                }
            }
            count >= 3
        },
        |state, vertex, _| {
            let n = state.n_vertices();
            let mut ones = 0u32;
            let mut total = 0u32;
            for i in 0..n {
                if state.edge(i, vertex) == 1 {
                    total += 1;
                    ones += state.label(i) as u32;
                }
            }
            let result = if ones > total - ones { 1 } else { 0 };
            state.mutate_label(vertex, result).unwrap()
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
/// is properly destructive, matching the Python reference implementation.
pub fn create_destructive_rules() -> Vec<RewriteRule> {
    let mut rules = Vec::new();

    // SCRAMBLE_ALL (5 copies for weighting)
    for k in 0..5 {
        rules.push(RewriteRule::new(
            format!("DESTROY_SCRAMBLE_ALL_{}", k),
            "destructive",
            |_, _| true,
            |state, _, rng| {
                let n = state.n_vertices();
                let new_labels: Vec<u8> = (0..n).map(|_| rng.random_range(0..=1)).collect();
                state.mutate_labels(&new_labels).unwrap()
            },
            false,
            usize::MAX, // global
        ));
    }

    // RANDOMIZE
    rules.push(RewriteRule::new(
        "DESTROY_RANDOMIZE",
        "destructive",
        |_, _| true,
        |state, vertex, rng| state.mutate_label(vertex, rng.random_range(0..=1)).unwrap(),
        false,
        0,
    ));

    // ZERO_OUT
    rules.push(RewriteRule::new(
        "DESTROY_ZERO",
        "destructive",
        |_, _| true,
        |state, vertex, _| state.mutate_label(vertex, 0).unwrap(),
        true,
        0,
    ));

    // ONE_OUT
    rules.push(RewriteRule::new(
        "DESTROY_ONE",
        "destructive",
        |_, _| true,
        |state, vertex, _| state.mutate_label(vertex, 1).unwrap(),
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
        let result = identity.apply(&state, 0, &mut rng);
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
        let result = toggle.apply(&state, 0, &mut rng);
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

            let result = nand.apply(&state, 2, &mut rng);
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
        let result = composed.apply(&state, 0, &mut rng);
        assert_eq!(result.label(0), state.label(0));
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
        // Equality preserved
        assert_eq!(original, &cloned);
        // Applying both produces same result
        use crate::state::BinaryGraphState;
        use ndarray::{arr1, arr2};
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let adj = arr2(&[[0, 0], [0, 0]]);
        let labels = arr1(&[0, 0]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();

        let mut rng = StdRng::seed_from_u64(42);
        let r1 = original.apply(&state, 0, &mut rng);
        let r2 = cloned.apply(&state, 0, &mut rng);
        assert_eq!(r1.canonical_encoding(), r2.canonical_encoding());
    }
}
