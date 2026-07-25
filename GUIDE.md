# ARCO User Guide

This guide walks through using ARCO to study emergent computation. You should have read the [Mathematical Constitution](docs/constitution.md) first — it defines the concepts. This guide shows how to use them.

## 1. Quick Start

### Installation

```bash
git clone https://github.com/kvernet/arco.git
cd arco
cargo build --release
```

Requires Rust 1.85 or later.

### Your First Run

```bash
./target/release/arco
```

This executes the full scientific cycle with default parameters: 300 training universes, 100 test universes, 3-vertex binary graphs, 10 ensemble members, 60 timesteps. On a modern machine it completes in under a minute.

### What Just Happened

ARCO performed six steps:

1. **Generate**: Created 400 rule subsets spanning the spectrum from purely destructive (information-scrambling) to purely structured (logic gates, transport rules). Each subset was applied to a small graph universe.

2. **Calibrate**: Ran 30 purely destructive universes to establish a baseline — "this is what storage looks like when nothing is being preserved." Set the detection threshold at the 95th percentile of that baseline.

3. **Observe**: For each of the 300 training universes, generated 10 trajectories from different random initial states. Computed storage (how much information about the past remains recoverable) for each one.

4. **Hypothesize & Test**: Generated 8 structural hypotheses — for example, "rule sets containing transport operations (PROPAGATE, SWAP, COPY) exhibit storage above threshold." Evaluated each hypothesis on the 100 held-out test universes. Computed accuracy (what fraction of qualifying universes actually exhibited storage) and applied a complexity penalty.

5. **Boolean `validation`**: Validated Boolean logic gates.

6. **Revise**: Checked failure conditions, and compiled a research record.

### The Output

You'll see something like:

```
ARCO v0.2.1 — Scientific Cycle Report
============================================================
Universes: 300
Duration:  4.1s

Emergence (above calibrated thresholds):
  Storage: 131/300 (43.7%)
  Memory:  131/300 (43.7%)

Hypotheses tested: 8
Hypotheses survived: 4

Surviving hypotheses:
  H2_MAJORITY_STRUCTURED: Majority of rules are structured (acc=0.700, score=0.550)
  H3_LOGIC_GATE: Rule set contains a logic gate (acc=0.600, score=0.450)
  H5_TRANSPORT: Rule set contains an information transport rule (acc=0.600, score=0.500)
  H7_MULTIPLE_LOGIC: Rule set contains at least 2 logic gates (acc=0.800, score=0.600)

Boolean functions validated:
  AND: 5
  NAND: 4
  NOR: 3
  OR: 9
  XOR: 5

Storage Spectrum:
  Noise                n=84   storage=15.5  % mean=0.1994
  Noise-dominated      n=40   storage=25.0  % mean=0.1424
  Balanced             n=50   storage=30.0  % mean=0.2330
  Structure-dominated  n=47   storage=40.4  % mean=0.2133
  Structured           n=79   storage=93.7  % mean=0.6438
```

The spectrum shows the Structure-Storage Gradient: as the fraction of structured rules increases, so does the probability of detecting storage. Purely destructive universes preserve information ~15% of the time. Purely structured universes preserve it ~93% of the time.

The Transport Law (H5) survived: rule sets containing PROPAGATE, SWAP, or COPY rules exhibit storage at rates above the calibrated threshold. The accuracy (60%) means it correctly predicted storage for 60% of qualifying test universes — above the 50% chance baseline.

### Trying Different Configurations

```bash
# Larger experiment (10,000 universes)
./target/release/arco --train 10000 --test 2000

# Different random seed (results will vary)
./target/release/arco --seed 99

# Fast test run
./target/release/arco --quick

# Use a coarser observation operator
./target/release/arco --obs label_sum

# See all options
./target/release/arco --help
```

### What the Numbers Mean

- **Storage above 50% at large sample sizes**: The hypothesis predicts better than chance. This is a real structural regularity.
- **Storage at 90%+**: A strong effect. The Structure-Storage Gradient is ARCO's most robust finding.
- **Storage below 50% or failing the complexity penalty**: The hypothesis doesn't hold. Either the condition is wrong or the effect is too weak.
- **Hypothesis accuracy varies across seeds**: At n=300, accuracies can swing ±20 points. At n=10,000, they stabilize to within ±5 points. For reliable estimates, run larger experiments.

### Next Steps

- Read Section 2 to understand the output in detail
- Read Section 3 to build your own universe
- Read the [examples/](https://github.com/kvernet/arco/tree/main/examples) directory for runnable code

---

## 2. Understanding the Output

Every ARCO run produces a research record. This section explains each part of the output and how to interpret it.

### The Summary Block

```
ARCO v0.2.1 — Scientific Cycle Report
============================================================
Universes: 300
Duration:  4.1s

Emergence (above calibrated thresholds):
  Storage: 131/300 (43.7%)
  Memory:  131/300 (43.7%)
```

**Universes**: How many rule sets were tested. More universes = more reliable statistics.

**Duration**: Wall-clock time. Roughly linear in the number of universes.

**Storage**: How many universes scored above the calibrated storage threshold. Storage measures whether information about past states remains recoverable after multiple timesteps. A universe with 43.7% storage means about 44% of tested rule sets preserve information above the noise baseline.

**Memory**: Always equal to storage. Memory is an alias — the capacity to preserve information about the past is what storage measures.

### Hypotheses

```
Hypotheses tested: 8
Hypotheses survived: 4

Surviving hypotheses:
  H2_MAJORITY_STRUCTURED: Majority of rules are structured (acc=0.700, score=0.550)
  H3_LOGIC_GATE: Rule set contains a logic gate (acc=0.600, score=0.450)
  H5_TRANSPORT: Rule set contains an information transport rule (acc=0.600, score=0.500)
  ...
```

Each hypothesis is a structural claim: "rule sets with property X exhibit storage above threshold."

**Accuracy**: The fraction of qualifying test universes where the prediction was correct. An accuracy of 0.600 means the hypothesis was right 60% of the time. The baseline is 50% (random guessing). Anything consistently above 50% at large sample sizes is a real effect.

**Score**: Accuracy minus a complexity penalty (0.1 × number of conditions in the hypothesis). A hypothesis with accuracy 0.600 and complexity 1.0 scores 0.500. The penalty prevents overfitting — a hypothesis with 20 conditions might be 90% accurate but would score poorly because it's essentially memorizing the training data.

**Survival**: A hypothesis survives if score > 0 AND accuracy ≥ 50%. A hypothesis can be above chance but still fail if it's too complex (negative score). Or it can be simple but wrong (low accuracy).

**Why some hypotheses fail**: H1 (has any structured rule → storage) fails because a single structured rule among destructive rules rarely preserves information. H4 (all structured → storage) sometimes fails because fully structured rule sets can still destroy information if they contain only constants and no transport mechanisms. H6 (all destructive → storage) is a negative control — it *should* fail, and its failure validates the calibration.

### The Storage Spectrum

```
Storage Spectrum:
  Noise                n=84   storage=15.5  % mean=0.1994
  Noise-dominated      n=40   storage=25.0  % mean=0.1424
  Balanced             n=50   storage=30.0  % mean=0.2330
  Structure-dominated  n=47   storage=40.4  % mean=0.2133
  Structured           n=79   storage=93.7  % mean=0.6438
```

This is the most important part of the output. It groups universes by their structured-rule fraction and shows what percentage of each group exhibits storage.

**Brackets**:
- **Noise** (0–15% structured): Almost entirely destructive rules. Information is scrambled every timestep.
- **Noise-dominated** (15–40%): Mostly destructive, a few structured rules mixed in.
- **Balanced** (40–60%): Roughly equal mix.
- **Structure-dominated** (60–85%): Mostly structured rules.
- **Structured** (85–100%): Almost entirely structured rules.

**What to look for**: A monotonic gradient — storage increases as structured rules increase. In healthy results, the Noise bracket is low (10–25%) and the Structured bracket is high (90%+). If the gradient is flat (all brackets similar), either the metrics aren't discriminating or the rule sets aren't sufficiently different.

**What the gradient means**: Structured rules (logic gates, transport operations, identity) tend to preserve information across timesteps. Destructive rules (random scramblers, constant assignments) tend to destroy it. The spectrum confirms that ARCO's storage metric detects this difference. This is the Structure-Storage Gradient — ARCO's most robust finding.

### Boolean Function Verification

```
Boolean functions validated:
  AND: 5
  NAND: 4
  NOR: 3
  OR: 9
  XOR: 5
```

ARCO tests whether any high-structure rule sets (≥40% structured) can implement basic logic gates. A rule set "implements NAND" if, when given two input bits on designated vertices, it reliably produces the correct NAND output after multiple timesteps of stochastic, asynchronous rule application.

**This is a verification, not a discovery**: The logic gates are hand-coded in the current Binary Graph Universe. This test confirms they function correctly despite random vertex ordering and rule competition. It answers: "can a NAND gate survive in a noisy, asynchronous environment?" The answer is yes — multiple rule sets containing NAND rules successfully implement the NAND truth table.

In future discovery substrates, this section would report genuinely discovered computational primitives.

### Failure Conditions

```
FAILURE CONDITIONS TRIGGERED:
  ! F-6 (DISCONFIRMATION): No hypotheses survived.
```

ARCO has seven failure conditions defined in the Constitution. If any trigger, the framework itself may need revision. The most important:

- **F-1 (Null)**: No known computation detected at all. If triggered, the metrics or calibration are broken.
- **F-6 (Disconfirmation)**: No hypotheses survive. The central claim — that structural properties predict emergence — is not supported.
- **F-7 (Overfitting)**: Only overly complex hypotheses survive. The framework is memorizing, not discovering.

If you see failure conditions, don't ignore them. They mean something is wrong with the experimental setup or the framework's assumptions.

### Variability Across Runs

Run the same command twice with different seeds:

```bash
./target/release/arco --train 300 --test 100 --seed 42
./target/release/arco --train 300 --test 100 --seed 99
```

The numbers will differ. At n=300, hypothesis accuracies can swing ±20 percentage points. The spectrum is more stable — structured storage usually stays above 85%. For reliable hypothesis estimates, use larger experiments (n=10,000+). For quick exploration, n=300 is sufficient to see the gradient.

### When Results Look Wrong

**All brackets show similar storage (flat spectrum)**: The destructive rules aren't destructive enough. Try increasing the weight of SCRAMBLE_ALL rules in `create_destructive_rules()`.

**No hypotheses survive**: Sample size too small, or the null threshold is too high. Try increasing n_train and n_test, or check that null universes contain at least one scrambler.

**Structured storage below 80%**: The structured rules may be too weak. Ensure transport rules (PROPAGATE, SWAP) are present in the pool. Without multi-write operations, information doesn't spread far enough to survive multiple timesteps.

**NAND not validated**: The logic gate rules require incoming edges to fire. If the test graph doesn't have edges 0→2 and 1→2, the gates can't trigger. This is set up correctly by `test_boolean_function()` but may fail if NAND shares a rule set with unconditional rules like IDENTITY that fire first.

---

## 3. Your First Custom Universe

The default run uses a pre-built Binary Graph Universe. This section shows how to create your own — choose rules, configure parameters, and interpret the results.

### Creating a Custom Rule Set

ARCO ships with two rule pools: `create_structured_rules()` (16 rules: logic gates, transport, identity) and `create_destructive_rules()` (8 rules: scramblers, constants). You can mix them however you want.

We'll use the following function to find a rule by name in a pool.

```rust
/// Find a rule by name in a pool. Panics if not found — useful for examples.
fn find_rule<'a>(pool: &'a [RewriteRule], name: &str) -> &'a RewriteRule {
    pool.iter().find(|r| r.name() == name)
        .unwrap_or_else(|| panic!("Rule '{}' not found in pool", name))
}
```

```rust
use arco::cycle::{CycleConfig, run_cycle};
use arco::rules::{create_structured_rules, create_destructive_rules, RewriteRule, Rule};

fn main() {
    // Get the standard rule pools
    let structured = create_structured_rules();
    let destructive = create_destructive_rules();

    // Build a custom rule set: 3 structured + 1 destructive
    let my_rules: Vec<RewriteRule> = vec![
        find_rule(&structured, "IDENTITY").clone(),
        find_rule(&structured, "NAND").clone(),
        find_rule(&structured, "SWAP").clone(),
        find_rule(&destructive, "DESTROY_SCRAMBLE_ALL_0").clone(),
    ];

    // Print what we built
    for rule in &my_rules {
        println!("  {} ({})", rule.name(), rule.rule_type());
    }

    // Run a cycle with default config
    let config = CycleConfig::default();
    let record = run_cycle(&config);
    println!("{}", record.summary());
}
```

This creates a single rule set and runs the full cycle. But `run_cycle` generates its own rule sets — it doesn't use `my_rules`. To test a specific rule set, we need to bypass the cycle and work directly with the lower-level APIs.

### Testing a Single Rule Set

To see what one rule set does without running the full cycle:

```rust
use arco::dynamics::{DEFAULT_SCHEDULE, generate_ensemble};
use arco::metrics::compute_storage;
use arco::observation::observe_full_state;
use arco::rules::{create_structured_rules, create_destructive_rules, RewriteRule, Rule};
use arco::state::BinaryGraphState;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    let n_vertices = 3;
    let n_ensemble = 10;
    let steps = 60;
    let seed = 42;

    // Build rule set
    let structured = create_structured_rules();
    let destructive = create_destructive_rules();
    let rules = vec![
    	// IDENTITY
        find_rule(&structured, "IDENTITY").clone(),
        // NAND
        find_rule(&structured, "NAND").clone(),
        // DESTROY_SCRAMBLE_ALL_0
        find_rule(&destructive, "DESTROY_SCRAMBLE_ALL_0").clone(),
    ];

    // Generate random initial states
    let mut rng = StdRng::seed_from_u64(seed);
    let initial_states: Vec<BinaryGraphState> = (0..n_ensemble)
        .map(|_| BinaryGraphState::random(n_vertices, &mut rng))
        .collect();

    // Generate ensemble
    let ensemble = generate_ensemble(
        &initial_states, &rules, steps, n_ensemble,
        1,                  // window_size
        &DEFAULT_SCHEDULE,  // all-vertices asynchronous
        &|window| observe_full_state(&window[0]),  // observe most recent state
        seed,
    );

    // Compute storage
    let storage = compute_storage(&ensemble, 15, 10, seed);
    println!("Storage: {:.4}", storage);

    // Compare to a typical threshold
    let threshold = 0.12;
    if storage > threshold {
        println!("Storage DETECTED (above {:.2})", threshold);
    } else {
        println!("No storage detected (below {:.2})", threshold);
    }
}
```

This is the minimal experiment: one rule set, one ensemble, one metric. Everything in the scientific cycle is built on these primitives.

### Understanding the Pieces

**State**: `BinaryGraphState::random(n_vertices, &mut rng)` creates a random directed graph with binary vertex labels and binary edges. For n=3, there are 2^(9+3) = 4096 possible states.

For a directed graph with `n` vertices:

- **Edges**: There are `n × n` possible directed edges (including self-loops). Each edge can be 0 or 1, so edges contribute `2^(n²)` possibilities. For n=3: `2^9 = 512`.

- **Labels**: Each vertex has a binary label. So labels contribute `2^n` possibilities. For n=3: `2^3 = 8`.

- **Total**: `2^(n²) × 2^n = 2^(n² + n)`. For n=3: `2^(9+3) = 2^12 = 4096`.

**Rules**: Each rule has a condition (when does it fire?) and an action (what does it do?). IDENTITY always fires and does nothing. NAND fires when a vertex has at least 2 incoming edges and replaces its label with the NAND of the source labels. DESTROY_SCRAMBLE_ALL_0 always fires and randomizes every vertex label.

**Ensemble**: `generate_ensemble` runs multiple trajectories from different initial states. This is required because single-trajectory information estimates are unreliable — you need statistical power across different starting conditions.

**Schedule**: `DEFAULT_SCHEDULE` is the all-vertices asynchronous schedule. Every timestep, each vertex is visited once in random order. At each vertex, rules are tried in random order and the first match fires. This randomness means the same rule set can produce different outcomes — the metrics measure whether information preservation survives this stochasticity.

**Observation**: `observe_full_state` returns the complete state encoding (labels + edges). This is the identity observation — it distinguishes every state. Coarser observations (like `observe_label_sum`) ignore some details. The observation operator determines what "information preservation" means.

**Storage**: `compute_storage` measures the maximum information about the past that remains recoverable, across timescales from 1 to max_delta. It uses pooled estimation (all timesteps combined) with shuffle correction to remove estimator bias.

### Experimenting

Try changing the rule set and see how storage changes:

- Remove DESTROY_SCRAMBLE_ALL_0 → storage should increase (no information destruction)
- Replace IDENTITY with TOGGLE → storage may decrease (labels flip every step)
- Add PROPAGATE → storage should increase (information spreads to multiple vertices)
- Use only DESTROY_ZERO → storage may spike artificially (constant state has perfect but trivial storage — this is why null calibration is important)

### When to Write a Custom Universe vs. Using the Cycle

- **Use the cycle** (`run_cycle`) when you want to test hypotheses across many rule sets, calibrate thresholds, and produce a research record.
- **Use the low-level API** (`generate_ensemble` + `compute_storage`) when you want to understand a single rule set, debug behavior, or prototype new ideas.
- **Write a custom state space** (implement the `State` trait) when you want to study something other than binary graphs — cellular automata, string rewriting, tensor networks.

The cycle is built on the low-level API. Everything the cycle does, you can do by hand with more control.

---

## 4. Writing Rules

Rules are the heart of ARCO. They define how a universe evolves. This section covers writing your own rules, understanding `MatchInfo`, and composing rules together.

### Rule Anatomy

Every rule has two parts: a **condition** (when does it fire?) and an **action** (what does it do?).

```rust
use arco::rules::{RewriteRule, MatchInfo};

let my_rule = RewriteRule::new(
    "MY_RULE",          // name — used in hypotheses and display
    "structured",       // type — "structured" or "destructive"
    
    // Condition: returns Some(MatchInfo) if the rule can fire, None otherwise
    |state, vertex| {
        if state.label(vertex) == 1 {
            Some(MatchInfo::Unconditional { vertex })
        } else {
            None
        }
    },
    
    // Action: transforms the state using the match info
    |state, info, _rng| {
        state.mutate_label(info.vertex(), 0).unwrap()
    },
    
    true,  // deterministic — same (state, info) always produces same result
    0,     // locality_radius — 0 = only affects the target vertex
);
```

This rule fires when a vertex has label 1, and sets it to 0. It only affects the target vertex (radius 0) and always produces the same result (deterministic).

### Condition Functions

The condition receives the current state and a vertex index. It returns `Some(MatchInfo)` if the rule can fire at that vertex, or `None` if it can't.

**Unconditional rules** always fire:

```rust
|_, vertex| Some(MatchInfo::Unconditional { vertex })
```

**Conditional rules** check the state:

```rust
// Only fire if the vertex has at least one outgoing edge
|state, vertex| {
    let n = state.n_vertices();
    for j in 0..n {
        if state.edge(vertex, j) == 1 {
            return Some(MatchInfo::Unconditional { vertex });
        }
    }
    None
}
```

**Rules that need match context** return richer `MatchInfo` variants:

```rust
// Find the first incoming neighbor and record it
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
}
```

### MatchInfo Variants

`MatchInfo` carries information from the condition to the action, avoiding a second scan of the state.

| Variant | When to use | What it carries |
|---------|------------|-----------------|
| `Unconditional { vertex }` | Rule always fires, or only needs the vertex index | Just the vertex |
| `Incoming { vertex, sources }` | Rule reads from incoming neighbors | Which neighbors triggered the match |
| `Outgoing { vertex, targets }` | Rule writes to outgoing neighbors | Which neighbors to write to |
| `Swap { vertex, other }` | Rule exchanges labels with a neighbor | Which neighbor to swap with |

### Action Functions

The action receives the state, the match info from the condition, and an RNG for stochastic rules. It returns a new state — never modifies in place.

**Using match info safely**:

```rust
|state, info, _rng| {
    match info {
        MatchInfo::Incoming { vertex, sources } => {
            let src_label = state.label(sources[0]);
            state.mutate_label(*vertex, src_label).unwrap()
        }
        _ => state.clone(),  // safe fallback: no-op if wrong variant
    }
}
```

The action should handle any `MatchInfo` variant gracefully. The schedule guarantees that `apply()` is only called after a successful `matches()`, so the correct variant will be passed — but defensive coding prevents panics if the rule is used incorrectly in tests.

### Deterministic vs. Stochastic Rules

**Deterministic** (`true`): Same (state, match_info) always produces the same next state. The RNG is ignored.

```rust
|state, info, _rng| {
    state.mutate_label(info.vertex(), 1 - state.label(info.vertex())).unwrap()
}
```

**Stochastic** (`false`): The RNG influences the outcome. Used for noise and destructive rules.

```rust
|state, info, rng| {
    if rng.random_bool(0.1) {  // 10% chance
        state.mutate_label(info.vertex(), 1 - state.label(info.vertex())).unwrap()
    } else {
        state.clone()
    }
}
```

Stochastic rules produce different trajectories from the same initial state. The ensemble approach (multiple trajectories) averages over this randomness.

### Locality Radius

The `locality_radius` parameter documents how far a rule's effects reach:

- **0**: Only affects the target vertex (IDENTITY, TOGGLE, CONST)
- **1**: Reads from or writes to immediate neighbors (COPY, SWAP, logic gates)
- **`usize::MAX`**: Can affect the entire graph (SCRAMBLE_ALL)

This is currently metadata — it doesn't change behavior. Future work will use it for causal analysis and parallel scheduling.

### Structured vs. Destructive

The `rule_type` field determines how the rule is treated in experiments:

- **`"structured"`**: Counts toward the structured-rule fraction in spectrum analysis. These rules are assumed to do something computationally meaningful.
- **`"destructive"`**: Counts toward the destructive fraction. Used for null-distribution calibration. These rules should destroy information.

The distinction matters for hypothesis testing: `has_structured` and `all_destructive` predicates depend on `rule_type`, not on what the rule actually does. A rule named `"CONST_0"` with type `"structured"` will count as structured even though it destroys information. A rule named `"DESTROY_ZERO"` with type `"destructive"` will count as destructive. Same operation, different experimental role.

### Rule Composition

Rules form a semigroup: you can compose two rules into a new rule that applies them sequentially.

```rust
use arco::rules::compose;

let toggle = /* ... */;
let identity = /* ... */;

// Apply toggle, then identity (no net effect)
let composed = compose(toggle, identity);

// The name follows mathematical notation:
// compose(a, b) is named "(b∘a)" — "apply a first, then b"
```

Composition is used to verify the semigroup property. In practice, the schedule handles sequential application by visiting vertices and trying rules in order.

### Writing Rules: Checklist

- [ ] Condition returns `Some(MatchInfo)` with the right variant for the action's needs
- [ ] Action handles its expected `MatchInfo` variant and falls back safely for others
- [ ] Action returns a new state (uses `mutate_label`, `mutate_labels`, `mutate_adj`, or `clone`)
- [ ] Action never panics — defensive `_ => state.clone()` fallback
- [ ] `is_deterministic` is `true` unless the rule uses `rng`
- [ ] `locality_radius` accurately describes the rule's reach
- [ ] `rule_type` matches the rule's experimental role (validation substrate) or is assigned systematically (discovery substrate)
- [ ] Rule name is unique within its pool (names are used for equality and hashing)

### Example: Writing a Custom Transport Rule

```rust
// COPY_TO_ALL: copies the vertex's label to every other vertex.
// Multi-write, radius n (global), deterministic.
RewriteRule::new(
    "COPY_TO_ALL",
    "structured",
    |_, vertex| Some(MatchInfo::Unconditional { vertex }),
    |state, info, _rng| {
        let src = state.label(info.vertex());
        let n = state.n_vertices();
        let mut labels: Vec<u8> = (0..n).map(|i| state.label(i)).collect();
        for i in 0..n {
            labels[i] = src;
        }
        state.mutate_labels(&labels).unwrap()
    },
    true,
    usize::MAX,  // affects entire graph
);
```

This rule would be a strong predictor of storage — it copies information everywhere, creating massive redundancy. Adding it to the structured pool and testing the Transport Law would likely increase H5 accuracy.

---

## 5. Observation Operators

Observation operators determine what ARCO can "see" when measuring emergence. The same universe can appear to have high or low storage depending on how you observe it. This section covers choosing, writing, and validating observation operators.

### Why Observation Matters

Consider a universe where information is preserved entirely in the graph's edge structure — labels are scrambled every step, but edges remain stable. If you observe only labels (`observe_label_vector`), storage will appear to be zero. If you observe only edges (`observe_edge_vector`), storage will appear to be high. The choice of observation determines what "information preservation" means.

This is not a bug. It's the **observer-relative nature of emergence**. ARCO makes this explicit: the observation operator is a first-class component of the experimental design.

### Built-in Observers

ARCO ships with single-state observers in `observation.rs`. Each maps a state to a `Vec<u8>`.

| Observer | What it captures | Size for n=3 |
|----------|-----------------|--------------|
| `observe_full_state` | Labels + edges (identity) | 12 bytes |
| `observe_label_vector` | Vertex labels only | 3 bytes |
| `observe_label_sum` | Count of 1-labels | 1 byte |
| `observe_root_label` | Label of vertex 0 only | 1 byte |
| `observe_edge_vector` | Flattened adjacency matrix | 9 bytes |
| `observe_edge_count` | Total number of edges | 1 byte |
| `observe_compound` | Labels followed by edges | 12 bytes |

**Choosing granularity**:

- **Identity** (`observe_full_state`): Use when you want to detect any information preservation, no matter how subtle. Most sensitive, but largest observation alphabet (4096 possible values for n=3). This is the default.

- **Coarse** (`observe_label_sum`): Use when you want to test whether aggregate properties are preserved. Less sensitive, smaller alphabet (4 possible values). Good for detecting whether a system preserves the *number* of active vertices without caring which ones.

- **Minimal** (`observe_root_label`): Use as a negative control. If storage is detected with this observer, information preservation is extremely robust — it survives even when you ignore almost everything.

### The Dynamic Sufficiency Axiom

An observation operator is **dynamically sufficient** if switching from the identity observation to this operator doesn't qualitatively change the storage metrics. If `observe_full_state` shows storage but `observe_label_vector` shows zero, the label vector is insufficient — the information being preserved lives in the edges.

**How to test sufficiency**:

```bash
# Run with full state (default)
./target/release/arco --train 300 --test 100 --seed 42

# Run with a coarser observer
./target/release/arco --train 300 --test 100 --seed 42 --obs label_vector
```

If the storage spectrum looks similar (structured universes still show high storage), the coarser observer is sufficient. If storage collapses to near-zero everywhere, the coarser observer is missing the information that structured rules preserve.

In the Binary Graph Universe, `observe_label_vector` is sufficient because most structured rules operate on labels. `observe_root_label` is not — it misses too much.

### Windowed Observers

Windowed observers see multiple consecutive states, enabling detection of temporal patterns invisible at the single-step level.

```rust
use arco::observation::observe_windowed_deltas;

// This observer encodes which labels changed between consecutive states.
// For a window of size 2: [state_t, state_{t+1}] → vector of 0s (no change) and 1s (changed)
```

The `observe_windowed_deltas` observer is useful for detecting propagation: if a label change at vertex 0 always precedes a change at vertex 1, the delta pattern will show `[1, 0]` followed by `[0, 1]`. Single-state observers would see two different states but miss the causal relationship.

**Window size tradeoff**: Larger windows capture longer temporal patterns but increase the observation alphabet exponentially. A window of size w with an alphabet of size A produces A^w possible observation values. For w=3 and full state (4096 values), that's 68 billion — far too many for reliable MI estimation with small ensembles. Start with w=1 or w=2.

### Writing a Custom Observer

Observers are plain functions. To write one:

```rust
use arco::state::BinaryGraphState;

/// Observe the parity of vertex labels (0 if even number of 1s, 1 if odd).
pub fn observe_label_parity(state: &BinaryGraphState) -> Vec<u8> {
    let n = state.n_vertices();
    let sum: u8 = (0..n).map(|i| state.label(i)).sum();
    vec![sum % 2]
}
```

Register it so the CLI can use it:

```rust
// In your code, add to the registry or pass directly
let my_obs = observe_label_parity;
```

For now, custom observers must be passed programmatically — the CLI only supports observers in the built-in registry. To add yours to the CLI, add an entry to `STATE_OBSERVERS` in `observation.rs`.

### Observer Design Guidelines

- **Return `Vec<u8>`**: This is the `Observation` type used throughout ARCO. Must be hashable and comparable for MI estimation.

- **Be deterministic**: Same state always produces the same observation. No RNG, no timestamps, no memory addresses.

- **Match granularity to your question**: If you're studying label dynamics, observe labels. If you're studying graph topology, observe edges. If you're studying both, use compound or full state.

- **Beware the alphabet explosion**: An observer that returns 12 bytes has 2^96 possible values. Even though only a tiny fraction appear in practice, the *potential* alphabet size affects MI estimator bias. The shuffle correction helps, but very fine-grained observers need larger ensembles.

- **Test sufficiency before drawing conclusions**: Always compare your custom observer against the identity observation. If they disagree qualitatively, your observer is missing something.

### Observation and the Scientific Cycle

The cycle uses the observer you specify (via `--obs` or `CycleConfig.obs_name`) for both calibration and measurement. The null distribution is computed with the same observer as the training data. If you change observers, thresholds must be recalibrated — a threshold calibrated for full state observation may not be valid for label sum observation.

This is why calibration is per-universe-class and per-observer. Run calibration whenever you change the observation operator.

---

## 6. Hypotheses

Hypotheses are how ARCO formalizes claims about emergence. A hypothesis says: "rule sets with property X exhibit emergent property Y above threshold." This section covers writing, testing, and interpreting hypotheses.

### Hypothesis Anatomy

A hypothesis has five parts:

```rust
use arco::hypotheses::Hypothesis;
use arco::rules::RewriteRule;

let my_hypothesis = Hypothesis::new(
    "H_MY_RULE",        // name — unique identifier
    |rules: &[RewriteRule]| -> bool {
        // condition: does this rule set qualify?
        rules.iter().any(|r| r.name() == "PROPAGATE")
    },
    "storage",           // property — "persistence", "storage", or "memory"
    "Contains PROPAGATE rule",  // human-readable description
    1.0,                 // complexity — penalty weight
);
```

**Name**: Used in output and for comparing hypotheses across runs. Must be unique within a hypothesis set.

**Condition**: A predicate on rule sets. Returns `true` if the hypothesis applies to this rule set. This is where you encode your scientific claim — "transport rules matter," "multiple logic gates matter," "structured majority matters."

**Property**: Which emergence metric the hypothesis predicts. Always one of `"persistence"`, `"storage"`, or `"memory"`. In practice, use `"storage"` or `"memory"` (they're equivalent). `"persistence"` is unreliable with small ensembles.

**Description**: Human-readable text for research records and display.

**Complexity**: The MDL penalty weight. Simple conditions (one clause) get 0.5–1.0. Compound conditions (multiple clauses, specific combinations) get 1.5–2.0. The penalty is 0.1 × complexity, subtracted from accuracy.

### How Testing Works

The cycle splits universes into training (75%) and test (25%) sets.

1. **Training set**: Used for calibration (computing thresholds) and observation (computing metrics). The hypotheses are *not* fitted to the training data — conditions are written by hand.

2. **Test set**: For each hypothesis, ARCO finds all test universes that satisfy the condition. It computes the predicted metric on each. It counts what fraction exceed the calibrated threshold.

3. **Scoring**: `score = accuracy - 0.1 × complexity`. A hypothesis survives if `score > 0` and `accuracy >= 0.5`.

### Writing Good Hypotheses

**Start simple**: Single-clause conditions are easier to interpret and less prone to overfitting.

```rust
// Good: one clear condition
|rules| rules.iter().any(|r| r.name() == "PROPAGATE")

// Avoid: many specific clauses
|rules| {
    rules.len() >= 3
        && rules.iter().filter(|r| r.name() == "NAND").count() >= 1
        && rules.iter().filter(|r| r.name() == "SWAP").count() >= 2
        && !rules.iter().any(|r| r.name() == "DESTROY_ZERO")
}
```

The second condition might be highly accurate on your test set, but it's memorizing specific rule combinations rather than capturing a general principle. The complexity penalty helps, but it's better to write simple conditions.

**Include negative controls**: A hypothesis that should *always* fail validates your calibration.

```rust
// H6: all destructive → storage. This should fail.
|rules| !rules.is_empty() && rules.iter().all(|r| r.rule_type() == "destructive")
```

If H6 survives, your null distribution is wrong — destructive rules are scoring above threshold when they shouldn't.

**Test complementary claims**: If "has transport → storage" survives, also test "no transport → no storage." The second may fail (transport isn't necessary for storage), but the comparison tells you whether transport is sufficient, necessary, both, or neither.

**Use rule_type, not rule names, for general claims**: `r.rule_type() == "structured"` generalizes to any validation substrate. `r.name() == "PROPAGATE"` only works if PROPAGATE exists. In discovery substrates (where rules have no human labels), conditions must be based on measurable properties — locality radius, determinism, number of inputs read — not semantic names.

### Interpreting Results

**High accuracy (80%+) with moderate complexity**: A strong finding. The condition reliably predicts emergence.

**Moderate accuracy (55–70%) with low complexity**: A real but weak effect. The condition predicts better than chance, but many counterexamples exist. The Transport Law falls here — transport rules help, but they're neither necessary nor sufficient.

**Accuracy near 50%**: No predictive power. The condition is irrelevant.

**High accuracy with high complexity**: Suspicious. Likely overfitting. Check whether the condition is just memorizing specific rule sets from the training data.

**Accuracy below 50% but hypothesis survives (negative score)**: Won't happen — the survival check requires score > 0.

### Why Hypotheses Fail

The standard hypothesis set includes several failures by design:

- **H1 (has any structured rule → persistence)**: Fails because a single structured rule among destructive chaos can't preserve information. Structure must reach a critical mass.

- **H4 (all structured → persistence)**: Sometimes fails because "all structured" includes rule sets with only constants (CONST_0, CONST_1) that actively destroy information. Structure is necessary but not sufficient.

- **H6 (all destructive → persistence)**: Should always fail. Its failure validates calibration.

- **H8 (mixed rules → persistence)**: Fails because "mixed" (30–70% structured) is too broad — it includes both information-preserving and information-destroying mixes.

These failures are informative. They constrain theories of emergence: structure alone isn't enough, destruction alone prevents emergence, and the middle ground is unpredictable.

### Adding Hypotheses to the Cycle

The cycle uses `generate_standard_hypotheses()` to get the eight built-in hypotheses. To add your own:

```rust
use arco::cycle::{CycleConfig, run_cycle};
use arco::hypotheses::{Hypothesis, test_all_hypotheses, surviving_hypotheses};
use std::collections::HashMap;

fn main() {
    let config = CycleConfig::default();
    let mut record = run_cycle(&config);

    // Define a custom hypothesis
    let my_h = Hypothesis::new(
        "H_MY_RULE",
        |rules| rules.iter().any(|r| r.locality_radius() > 1),
        "storage",
        "Contains non-local rule (radius > 1)",
        1.0,
    );

    // You'd need test data and metric maps to test it.
    // See the examples/ directory for the full pattern.
}
```

For now, custom hypotheses require working with the cycle internals. Future versions will support hypothesis plugins via configuration.

### Hypothesis Design Checklist

- [ ] Condition is a pure function of the rule set (no RNG, no external state)
- [ ] Condition uses `rule_type()` or measurable properties, not just hand-labeled names (for generality)
- [ ] Property is `"storage"` or `"memory"` (not `"persistence"` unless you have large ensembles)
- [ ] Complexity reflects the number of clauses (0.5 for trivial, 1.0 for simple, 2.0 for compound)
- [ ] At least one negative control hypothesis is included
- [ ] Accuracy is interpreted in context of sample size (n=100: ±10 points; n=2000: ±2 points)

---

## 7. Calibration and Null Models

Not every universe that preserves information is doing something meaningful. A constant universe — where every state is identical — has perfect storage (the past is trivially recoverable) but no computation. Calibration ensures ARCO only flags universes that preserve information *above what noise would produce*.

### Why Calibration Matters

Without calibration, you might claim "emergence" for a universe that just happens to have high mutual information by chance, or because its state space is so small that random trajectories overlap frequently. Calibration answers: "Is this storage value higher than what purely destructive dynamics would produce?"

The null hypothesis is: *this universe's storage is indistinguishable from information scrambling.* ARCO rejects this null when storage exceeds the 95th percentile of the destructive null distribution.

### How Calibration Works

1. **Generate null universes**: 30 rule sets composed entirely of destructive rules. Each contains at least one SCRAMBLE_ALL rule (guaranteed by the calibration code) to prevent degenerate constant universes.

2. **Run them**: Each null universe gets the same treatment as a real universe — same ensemble size, same number of steps, same observation operator.

3. **Compute storage**: For each null universe, compute storage just as you would for a real universe.

4. **Set the threshold**: The threshold is the 95th percentile of the null storage distribution. Only 5% of purely destructive universes would score this high by chance.

5. **Apply a floor**: If the null distribution is extremely compressed (all null universes score near zero), the threshold is raised to a minimum of 0.01. This prevents pathological cases where the threshold is 0.0001 and everything passes.

### Reading Calibration Output

When you run the cycle in verbose mode (or check `record.thresholds`), you'll see:

```
CALIBRATION: storage_threshold=0.1241
CALIBRATION: null_storage_mean=0.1013
CALIBRATION: null_storage_std=0.0368
```

**Interpretation**:
- Null storage mean = 0.10: purely destructive rule sets still show some apparent information preservation (~0.10 on the NMI scale). This is the estimator's bias — the shuffle correction removes most but not all of it.
- Threshold = 0.124: a universe needs to score above 0.124 to be considered emergent. This is the 95th percentile.
- Standard deviation = 0.037: the null distribution has moderate spread. If std were very large, the threshold would be higher and fewer universes would pass.

### What Makes a Good Null Distribution

**Good**: Mean near zero, moderate standard deviation, threshold well below typical structured-universe storage values.

```
Null mean: 0.04, threshold: 0.11, structured storage: 0.65 → clean separation
```

**Bad**: Mean high, threshold near or above structured-universe values.

```
Null mean: 0.35, threshold: 0.62, structured storage: 0.64 → no separation
```

If your null distribution is too hot (high mean), check:
- Are destructive rules actually destructive? SCRAMBLE_ALL should randomize everything.
- Is the observation alphabet too large relative to ensemble size? Try a coarser observer.
- Are there degenerate constant universes in the null set? Ensure every null universe has at least one scrambler.

### The Calibration Pipeline

The `calibration` module provides three levels of access:

**One-liner** (what the cycle uses):
```rust
use arco::calibration::calibrate;

let result = calibrate(
    30,     // n_null_universes
    3,      // n_vertices
    10,     // n_ensemble
    60,     // steps
    1,      // window_size
    5,      // max_rules_per_subset
    &obs_fn, // observation operator
    95.0,   // percentile
    15,     // max_delta
    10,     // n_shuffles
    42,     // seed
);

println!("Threshold: {:.4}", result.storage_threshold);
println!("Null mean: {:.4}", result.null_storage.mean);
println!("Empirical p for 0.5: {:.4}", result.null_storage.empirical_p(0.5));
```

**Generate + calibrate separately** (for debugging):
```rust
use arco::calibration::{generate_null_trajectories, calibrate_thresholds};

let null_ensembles = generate_null_trajectories(
    30, 3, 10, 60, 1, 5, &obs_fn, 42,
);

// Inspect individual null universes
for (i, ensemble) in null_ensembles.iter().enumerate() {
    let storage = compute_storage(ensemble, 15, 10, 42);
    println!("Null {}: storage={:.4}", i, storage);
}

let result = calibrate_thresholds(&null_ensembles, 95.0, 0.01, 0.01, 0.01, 15, 10, 42);
```

**Manual calibration** (for full control):
```rust
// Generate your own destructive rule sets, run them, collect storage values,
// compute the percentile yourself. The calibration module is a convenience,
// not a requirement.
```

### Empirical P-Values

The `NullStats` struct provides an `empirical_p()` method:

```rust
let p = result.null_storage.empirical_p(0.5);
// p = fraction of null universes with storage >= 0.5
```

If p = 0.03, only 3% of destructive universes score 0.5 or higher — strong evidence that 0.5 is not noise. If p = 0.40, 40% of destructive universes score that high — weak evidence.

Empirical p-values are more intuitive than raw threshold comparisons. They answer: "How surprising is this storage value, given what pure noise produces?"

### Calibration and Observer Choice

Calibration is observer-specific. If you change `--obs`, thresholds change:

```bash
# Full state: large alphabet, moderate null mean
./target/release/arco --obs compound --train 300 --seed 42

# Label sum: tiny alphabet, different null distribution
./target/release/arco --obs label_sum --train 300 --seed 42
```

These will produce different thresholds because the null distribution depends on the observation granularity. Always recalibrate when changing observers.

### Calibration and Sample Size

With n_null=30 (the default), the 95th percentile is estimated from only 30 values. The confidence interval on a percentile from 30 samples is wide — the "true" 95th percentile could be several points higher or lower.

For publication-quality thresholds, use n_null=100 or more. For exploration, n_null=30 is sufficient to detect large effects.

### Common Calibration Problems

**Threshold equals the floor (0.01)**: The null distribution is compressed near zero. This is actually good — it means destructive rules reliably destroy information. But check that the floor isn't masking a problem by running with `floor_storage=0.0` temporarily.

**Threshold exceeds structured storage**: Either the destructive rules aren't destructive enough, or the structured rules aren't preserving enough information. Check for degenerate null universes (two ZERO rules making a constant state).

**Threshold varies wildly across seeds**: n_null is too small. Increase to 100.

**Negative control (H6) survives**: Calibration has failed. The null distribution doesn't represent true destructive dynamics. Check that null universes contain scramblers.

---

## 8. The Scientific Cycle

The scientific cycle is ARCO's top-level orchestrator. It runs the six-step loop: Generate → Calibrate → Observe → Hypothesize → Predict → Test → Revise. This section covers how to configure, run, and extend the cycle.

### What the Cycle Does

Each call to `run_cycle()` executes a complete experiment:

1. **Generate**: Creates rule subsets spanning the structured/destructive spectrum. Each subset is applied to a small graph universe with random initial states. The subsets are split into training (75%) and test (25%) sets.

2. **Calibrate**: Runs 30 purely destructive universes to establish the null distribution. Computes the 95th percentile threshold for storage and memory.

3. **Observe**: For each training universe, generates an ensemble of trajectories and computes storage, memory, and persistence.

4. **Hypothesize**: Loads the standard hypothesis set (H1–H8). These are structural predicates — "has transport rules," "majority structured," etc. — that predict storage or memory.

5. **Predict & Test**: Evaluates each hypothesis on the held-out test universes. Computes accuracy (fraction of qualifying universes where the prediction held) and applies the MDL complexity penalty.

6. **Revise**: Checks failure conditions, searches for Boolean logic gates in high-structure universes, compiles the storage spectrum, and produces a research record.

### Configuring the Cycle

All parameters live in `CycleConfig`:

```rust
use arco::cycle::{CycleConfig, run_cycle};

let config = CycleConfig {
    n_train: 300,           // training universes
    n_test: 100,            // held-out test universes
    n_vertices: 3,          // vertices per state
    n_ensemble: 10,         // trajectories per ensemble
    steps: 60,              // timesteps per trajectory
    window_size: 1,         // observation window
    obs_name: "compound".to_string(),  // observation operator
    max_delta: 15,          // max timescale for storage
    n_shuffles: 10,         // shuffle iterations for bias correction
    n_null_universes: 30,   // null universes for calibration
    seed: 42,               // random seed
};

let record = run_cycle(&config);
```

Or via CLI:

```bash
./target/release/arco \
    --train 1000 \
    --test 300 \
    --vertices 3 \
    --ensemble 10 \
    --steps 60 \
    --obs compound \
    --max-delta 15 \
    --shuffles 10 \
    --null 30 \
    --seed 42
```

### Choosing Parameters

**Sample size (n_train, n_test)**:
- n=300/100: Quick exploration. Hypothesis accuracies ±20 points. Good for prototyping.
- n=1,000/300: Moderate reliability. Accuracies ±10 points.
- n=10,000/2,000: Publication quality. Accuracies ±2–3 points.
- n=50,000/5,000: High precision. Accuracies ±1–2 points. Overkill for most uses.

**Ensemble size (n_ensemble)**:
- n=10: Default. Sufficient for storage detection with pooled estimation.
- n=20–50: Better for per-timestep persistence. Increases runtime proportionally.
- n<10: Not recommended. MI estimates become unreliable.

**Steps**: 60 is sufficient for 3-vertex graphs. Larger state spaces may need more steps for information to propagate. If storage is near zero for structured universes, try increasing steps.

**Observation (obs_name)**: Start with `"compound"` (the default). If you want to test whether a coarser observation still detects emergence, try `"label_vector"` or `"label_sum"`. Always recalibrate when changing observers.

**Seed**: Any u64. Different seeds produce different random universes. For publication, run multiple seeds and report ranges. For exploration, any seed works.

### Understanding the Research Record

`run_cycle()` returns a `ResearchRecord` with everything you need:

```rust
let record = run_cycle(&config);

// Access results
println!("{}", record.summary());           // human-readable overview
println!("{:?}", record.thresholds);        // calibrated thresholds
println!("{:?}", record.failure_conditions); // any triggered failures

// Inspect individual universes
for result in &record.results {
    if result.storage > record.thresholds["storage"] {
        println!("Universe {}: storage={:.4}, rules={:?}",
            result.universe_id, result.storage, result.rule_names);
    }
}

// Check hypotheses
for h in &record.hypotheses {
    if h.survives {
        println!("{}: acc={:.3}, score={:.3}", h.name, h.accuracy, h.score);
    }
}

// Boolean validation
for (gate, count) in &record.boolean_validation {
    println!("{}: {} universes", gate, count);
}
```

### Extending the Cycle

The cycle is designed to be replaced piece by piece as your needs grow.

**Custom rule pools**: Edit `create_structured_rules()` and `create_destructive_rules()` in `rules.rs` to add or remove rules.

**Custom hypotheses**: Edit `generate_standard_hypotheses()` in `hypotheses.rs` to add your own structural predicates.

**Custom state space**: Implement the `State` trait for a new state type (cellular automata, string rewriting, tensor networks). Then write a new `run_cycle` variant that uses your state type.

**Custom schedule**: Implement the `Schedule` trait for a new update rule (synchronous, block-parallel, priority-ordered). Pass it to `generate_ensemble` instead of `DEFAULT_SCHEDULE`.

**Custom calibration**: Replace `calibrate()` with your own null model. The cycle just needs thresholds — how you compute them is up to you.

### The Cycle as a Library

The cycle is a convenience, not a framework. Every step can be called independently:

```rust
// Generate your own universes
let my_universes = generate_spectrum_universes(300, 3, "compound", 42);

// Calibrate your own thresholds
let my_calibration = calibrate(30, 3, 10, 60, 1, 5, &obs_fn, 95.0, 15, 10, 42);

// Observe with your own metrics
for universe in &my_universes {
    let ensemble = universe.run_ensemble(10, 60, 42);
    let storage = compute_storage(&ensemble, 15, 10, 42);
    // ...
}

// Test your own hypotheses
let my_hypothesis = Hypothesis::new(/* ... */);
// ...
```

The cycle ties these steps together for convenience. When you outgrow it, use the pieces directly.

---

## 9. Limitations and Caveats

ARCO is a research instrument, not a finished product. This section documents known limitations, statistical caveats, and design tradeoffs. Understanding these is essential for interpreting results correctly.

### Estimator Bias

ARCO uses a plugin (empirical) estimator for mutual information. This estimator is known to be biased upward when the observation alphabet is large relative to the sample size.

**The problem**: With 3 vertices and compound observation, there are 4096 possible observation values. With 10 ensemble members and 60 steps, you have at most a few hundred samples per timescale. Many observation values appear only once. The plugin estimator overestimates mutual information in this sparse regime.

**What ARCO does about it**: Shuffle correction subtracts the mean NMI of temporally permuted data. This removes the baseline bias — destructive rule sets show near-zero corrected NMI. But it doesn't eliminate all bias, and the variance of the correction with only 10 shuffles is not negligible.

**What you should do**: For publication-quality results on larger state spaces, replace the plugin estimator with a Bayesian (NSB) estimator, a Miller-Madow correction, or a k-NN estimator. The estimator boundary is the `shuffle_corrected_nmi` function in `metrics.rs` — swap it out without changing anything else.

**Practical impact**: The bias means small differences in storage (e.g., 0.12 vs 0.15) may not be meaningful. Focus on large effects (the 5× difference in the Structure-Storage Gradient) rather than precise threshold values.

### Sample Size and Variance

Hypothesis accuracies are proportions estimated from finite test sets. The standard error is sqrt(p(1-p)/n).

| Test set size | 95% CI for 60% accuracy |
|--------------|------------------------|
| 100 | 60% ± 9.6% |
| 300 | 60% ± 5.5% |
| 1,000 | 60% ± 3.0% |
| 2,000 | 60% ± 2.1% |
| 5,000 | 60% ± 1.4% |

At n=300 (the default), a reported accuracy of 60% could be anywhere from 50% to 70% in a different random sample. At n=2,000, it's reliably 58–62%.

**What you should do**: Run multiple seeds and report ranges, not single values. The README does this for n=10,000 across 10 seeds. For your own experiments, use at least n=1,000 test universes before drawing conclusions.

### Validation Substrate vs. Discovery Substrate

The current Binary Graph Universe is a **validation substrate**: rules are hand-coded with human-assigned semantic labels (NAND, PROPAGATE, SWAP). Hypotheses reference these labels. The Boolean "validation" test checks whether hand-coded NAND rules function under stochastic scheduling — it verifies robustness, not discovery.

This is appropriate for calibrating the measurement apparatus. It is not evidence that ARCO can discover novel computational structures in unlabeled rule spaces.

**What would change this**: A **discovery substrate** where rules are generated algorithmically without human semantic labels. ARCO would need to identify computational structure from measurable properties (locality, determinism, input/output relationships) rather than from names. This is the next major research milestone.

### Single State Space

All experiments to date use 3-vertex binary graphs. The six-tuple formalism supports arbitrary state spaces, but only one has been tested. Claims about "paradigm-neutrality" are aspirational until validated on multiple substrates.

The Python reference included a Symbolic Universe (binary tuple rewriting) that showed weaker but consistent results. Porting this to Rust and reproducing the Structure-Storage Gradient would provide the first cross-substrate validation.

### Graph Isomorphism

`BinaryGraphState` is vertex-order dependent. Two graphs that are isomorphic (same structure, different vertex numbering) are treated as distinct states. This inflates the apparent state space size and means some "different" universes in the results are structurally identical.

This is documented in the state module and the Constitution. For the Binary Graph Universe, it doesn't affect the storage metric (which measures information preservation in the representation, not the abstract graph). For future universes where isomorphism matters, implement canonical graph labeling in the state's `canonical_encoding` method.

### Schedule Dependence

All experiments use the all-vertices asynchronous schedule. Changing the schedule can dramatically change results — earlier experiments showed that switching from random-vertex to all-vertices altered persistence from 0% to 98%.

The discovered regularities may be schedule-specific. Testing the Transport Law under a synchronous schedule, a block-parallel schedule, or a priority schedule would establish whether it's a property of the rules or an artifact of the update order.

### Static Causal Structure

In the current Binary Graph Universe, the adjacency matrix is static — only vertex labels evolve. Edges are never created or destroyed by structured rules. This means the causal graph (which vertices can influence which others) is fixed at initialization.

This is a simplification for the validation substrate, not a fundamental limitation. The `mutate_adj` method exists and works. Future universes may include rules that rewire edges, making the causal structure itself an emergent property.

### Memory Metric

`compute_memory` is an alias for `compute_storage`. This is a deliberate simplification: the earlier memory metric (initial condition separation) was found to measure sensitivity to initial conditions, not memory. The current definition — memory = recoverable information about the past — is exactly what storage measures.

The old metric is preserved as `compute_initial_condition_separation` for diagnostic use. It can identify chaotic systems (high separation) vs. ordered systems (low separation), which is useful for classification even though it's not memory.

### Determinism and Reproducibility

ARCO is deterministic given a seed. Same seed, same code, same machine → same results. Different machines may produce different results due to floating-point differences or RNG implementation details (Python's MT19937 vs. Rust's ChaCha12).

The Python and Rust implementations produce different individual trajectories from the same seed because their RNGs differ. Their statistical distributions should match, and the cross-validation test confirmed that storage values agree to within 0.001 for identical inputs. But exact bit-for-bit reproducibility across implementations is not guaranteed.

### Performance Scaling

Runtime scales roughly linearly with the number of universes. Memory usage is dominated by trajectory storage (n_ensemble × steps × observation size). For default parameters, memory is negligible. For n=50,000 universes, the research record can be several gigabytes if all trajectories are stored.

The current implementation stores all results in memory. For very large experiments, stream results to disk or use a database.

### When Not to Use ARCO

ARCO is designed for one question: "under what conditions does computation emerge in arbitrary information systems?" It is not:

- A general-purpose complex systems simulator
- A tool for analyzing specific real-world networks
- A replacement for theoretical computer science
- A platform for engineering practical algorithms

If your question is "does this specific biological network exhibit emergent computation?", ARCO's framework could be adapted, but the current implementation targets abstract, generative universes — not empirical data.

### Reporting Results Honestly

When publishing ARCO results:

1. Report ranges across multiple seeds, not single values.
2. State the sample size and note the confidence interval.
3. Disclose whether the substrate is validation (hand-labeled) or discovery (unlabeled).
4. Acknowledge the plugin estimator's bias.
5. Distinguish between "storage was detected" (above threshold) and "the hypothesis survived" (above chance with complexity penalty).
6. Report negative results — hypotheses that failed are as informative as those that survived.

---

## 10. Extending ARCO

ARCO is designed to be extended. This section covers adding new state spaces, schedules, rule types, and generators. Each subsection is a recipe — follow it to add a capability, test it, and integrate it with the cycle.

### Adding a New State Space

ARCO's `State` trait abstracts over any representation. To add a new kind of state:

**Step 1: Implement the trait.**

```rust
use arco::state::State;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct MyState {
    // your fields here
    data: Vec<f64>,
}

impl State for MyState {
    type Encoding = Vec<u8>;

    fn canonical_encoding(&self) -> Self::Encoding {
        // Convert your state to a deterministic, hashable byte sequence
        self.data.iter()
            .flat_map(|f| f.to_le_bytes())
            .collect()
    }

    fn distance(&self, other: &Self) -> u32 {
        // Hamming distance or whatever metric makes sense
        self.data.iter()
            .zip(other.data.iter())
            .map(|(a, b)| if a != b { 1 } else { 0 })
            .sum()
    }
}

// Implement Eq, Hash, Debug, Display for your type
impl PartialEq for MyState {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl Eq for MyState {}
impl Hash for MyState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.iter().for_each(|f| f.to_le_bytes().hash(state));
    }
}
```

**Step 2: Write rules for your state.** If you're using `RewriteRule`, your state must be `BinaryGraphState`. For custom states, define your own rule type implementing the `Rule` trait pattern. The key contract: `matches()` returns match context, `apply()` consumes it and returns a new state.

**Step 3: Write an observation operator.** A function `&MyState -> Vec<u8>`. Register it or pass it directly.

**Step 4: Write a schedule.** Implement `Schedule` with a `step()` method that knows how to apply your rules to your state.

**Step 5: Write a cycle variant.** Copy `run_cycle()` and replace `BinaryGraphState` with `MyState`. The cycle is not generic yet — this is manual for now.

### Adding a New Schedule

Schedules determine the order and concurrency of rule application. To add one:

```rust
use arco::dynamics::Schedule;
use arco::rules::{RewriteRule, Rule};
use arco::state::BinaryGraphState;
use rand::Rng;

pub struct SynchronousSchedule;

impl Schedule for SynchronousSchedule {
    fn name(&self) -> &str { "synchronous" }
    fn timing(&self) -> &str { "synchronous" }
    fn selection(&self) -> &str { "exhaustive" }

    fn step(
        &self,
        state: &BinaryGraphState,
        rules: &[RewriteRule],
        rng: &mut impl Rng,
    ) -> BinaryGraphState {
        // Compute all updates from the same pre-timestep state,
        // then apply them simultaneously.
        let n = state.n_vertices();
        let mut new_state = state.clone();

        for vertex in 0..n {
            // Try rules in random order
            let mut rule_indices: Vec<usize> = (0..rules.len()).collect();
            rng.shuffle(&mut rule_indices);

            for &ri in &rule_indices {
                if let Some(info) = rules[ri].matches(state, vertex) {
                    // Apply to new_state, but match against original state
                    new_state = rules[ri].apply(&new_state, &info, rng);
                    break;
                }
            }
        }

        new_state
    }
}
```

Key difference from `AllVerticesSchedule`: all vertices match against the *original* state, not the progressively updated one. This is the synchronous vs. asynchronous distinction.

To use it: pass `&SynchronousSchedule` instead of `&DEFAULT_SCHEDULE` to `generate_ensemble`.

### Adding a New Rule Pool

The structured and destructive pools are just functions that return `Vec<RewriteRule>`. To add your own:

```rust
pub fn create_my_rules() -> Vec<RewriteRule> {
    let mut rules = Vec::new();

    rules.push(RewriteRule::new(
        "MY_RULE",
        "structured",
        |state, vertex| {
            // condition
            if state.label(vertex) == 1 {
                Some(MatchInfo::Unconditional { vertex })
            } else {
                None
            }
        },
        |state, info, _rng| {
            // action
            state.mutate_label(info.vertex(), 0).unwrap()
        },
        true,  // deterministic
        0,     // locality radius
    ));

    rules
}
```

Then use `generate_mixed_rule_subsets` with your pools instead of the defaults. Or bypass subset generation entirely and test specific rule combinations.

### Adding a New Generator

The universe generator `generate_spectrum_universes` creates universes at fixed structured ratios. For different generation strategies:

**Random search**: Generate rule sets by sampling rules randomly from a large pool. No fixed ratios.

```rust
fn random_search(n: usize, pool: &[RewriteRule], rng: &mut impl Rng) -> Vec<Vec<RewriteRule>> {
    (0..n).map(|_| {
        let size = rng.random_range(1..=5);
        (0..size).map(|_| pool[rng.random_range(0..pool.len())].clone()).collect()
    }).collect()
}
```

**Evolutionary search**: Mutate rule sets that score high on storage. Select, crossover, repeat.

**Constraint-based search**: Generate rule sets that satisfy specific properties (e.g., exactly one transport rule, at least two logic gates, no constants).

**Grid search**: Enumerate all combinations of a small rule pool up to a fixed size.

Each generation strategy tests a different hypothesis about where computation lives in the rule space. Random search is unbiased but sparse. Evolutionary search exploits structure but may miss isolated peaks. Grid search is exhaustive but explodes combinatorially.

### Adding a New Emergence Metric

Storage and memory are not the only ways to measure emergence. To add your own:

```rust
/// Measure the diversity of states visited in an ensemble.
/// High diversity suggests complex dynamics.
pub fn compute_state_diversity(
    trajectories: &[Vec<Vec<u8>>],
) -> f64 {
    use std::collections::HashSet;

    let mut seen: HashSet<&Vec<u8>> = HashSet::new();
    for traj in trajectories {
        for obs in traj {
            seen.insert(obs);
        }
    }

    seen.len() as f64
}
```

Then calibrate it against a null distribution, write hypotheses that predict it, and add it to the cycle's metric map. Any computable function of trajectories can be an emergence metric — the framework doesn't care what it measures, only that it's calibrated and falsifiable.

### Adding a New Hypothesis Predicate

Hypothesis conditions are functions `&[RewriteRule] -> bool`. They can use any property of rules:

```rust
// By locality
|rules| rules.iter().any(|r| r.locality_radius() > 1)

// By determinism
|rules| rules.iter().filter(|r| !r.is_deterministic()).count() >= 2

// By rule count
|rules| rules.len() >= 3

// By composition (rules that can chain)
|rules| {
    // A rule's output condition matches another rule's input condition
    // ... implement your logic
}
```

In discovery substrates (no human labels), predicates must be based on measurable properties — locality, determinism, arity, reversibility. This is where ARCO transitions from validating known structure to discovering unknown structure.

### Integrating with the Cycle

The cycle is not extensible through configuration (yet). To add your extensions:

1. Fork `run_cycle()` into your own function.
2. Replace the pieces you want to change (rule pools, hypotheses, metrics).
3. Call your custom cycle instead of the built-in one.

This is deliberate: the cycle is a reference implementation, not a plugin architecture. As ARCO matures, the cycle will become more configurable. For now, copying and modifying is the supported extension mechanism.

### What's Stable vs. What's in Flux

**Stable (safe to build on)**:
- `State` trait and `BinaryGraphState`
- `Rule` trait and `RewriteRule`
- `MatchInfo` enum
- `Schedule` trait and `AllVerticesSchedule`
- Observation operators (function signatures)
- `compute_storage`, `compute_memory`
- `Hypothesis` struct and scoring
- `ResearchRecord` and `CycleConfig`

**In flux (may change)**:
- `run_cycle()` internals
- `generate_standard_hypotheses()` (the set may grow)
- `calibrate()` convenience function
- `generate_spectrum_universes()`
- CLI interface (`main.rs`)

**Experimental (use at your own risk)**:
- `compute_persistence` (unreliable with small ensembles)
- `compute_initial_condition_separation` (diagnostic, not memory)
- `observe_windowed_deltas` (API may change)
- Rule composition (`compose`) — works but not integrated into the cycle

### Where to Go Next

- Read the [Mathematical Constitution](https://github.com/kvernet/arco/blob/main/docs/constitution.md) for the formal specification
- Study the [examples](https://github.com/kvernet/arco/tree/main/examples) for runnable code
- Read the source of `cycle.rs` to understand how the pieces fit together
- Read the source of `metrics.rs` for the estimator implementation details
- Open an issue on GitHub if you find bugs or have questions

---