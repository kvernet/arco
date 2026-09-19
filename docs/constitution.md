# ARCO

# Mathematical Constitution

---

> *This document defines the mathematical objects, operations, and criteria that constitute ARCO. It is a stable specification. Experimental findings are reported in the project [README](https://github.com/kvernet/arco) and associated publications.*

---

# Foundational Principle

> **ARCO Principle 1 — Computational Neutrality**
>
> No representation, computational model, or information-processing paradigm shall be assumed fundamental. Classical circuits, quantum circuits, neural networks, Turing machines, and all other known computational frameworks are treated as phenomena to be explained, not primitives to be assumed. All such structures must emerge from properties of Information Universes and be evaluated using the same operational criteria.

---

# Part Zero: Failure Conditions

Before defining what ARCO is, we define how it can fail. These conditions are decision criteria. If any condition triggers, the framework must be revised or abandoned.

---

## F-1: The Null Condition

**Statement**: No known computational system scores above threshold on ARCO's emergence metrics.

---

## F-2: The Triviality Condition

**Statement**: Every invariant discovered by ARCO is a logical consequence of the resource algebra or state space axioms alone.

---

## F-3: The Vacuous Novelty Condition

**Statement**: Every universe ARCO discovers is bisimilar to a known computational model in the Taxonomy.

---

## F-4: The Scalability Condition

**Statement**: ARCO's search procedure cannot explore state spaces beyond size $10^3$ within reasonable resources.

---

## F-5: The Incomputability Condition

**Statement**: Any emergence metric requires computing a quantity that is provably uncomputable.

---

## F-6: The Disconfirmation Condition

**Statement**: No predictive laws of the form "Condition A ∧ Condition B ⇒ Emergent Property C" generalize to unseen universes.

---

## F-7: The Overfitting Condition

**Statement**: Surviving hypotheses have complexity exceeding a threshold relative to their predictive accuracy.

---

# Part One: Information Universes

---

## 1.1 Definition

An **Information Universe** is a 6-tuple:

$$
\boxed{\mathcal{U} = (\mathcal{S}, \mathcal{T}, \mathcal{O}, \mathcal{R}, \mathcal{I}, \mathcal{K})}
$$

where:

- $\mathcal{S}$ = state space
- $\mathcal{T}$ = transformation set
- $\mathcal{O}$ = observation operators
- $\mathcal{R}$ = resource constraints
- $\mathcal{I}$ = invariant structure
- $\mathcal{K}$ = update schedule

---

## 1.2 The State Space $\mathcal{S}$

### 1.2.1 Core Definition

$\mathcal{S}$ is a set equipped with:

- A **canonical encoding** function $c: \mathcal{S} \to \mathcal{E}$ where $\mathcal{E}$ is a set of hashable, immutable values that uniquely identify each state. The canonical encoding is deterministic and independent of runtime concerns.
- A **distance function** $d_{\mathcal{S}}: \mathcal{S} \times \mathcal{S} \to \mathbb{R}_{\ge 0}$ satisfying the metric axioms.
- A **cardinality bound** $|\mathcal{S}| \le \aleph_0$ for any effectively explorable universe.

### 1.2.2 Admissible State Classes

| Class | Structure | Example |
|-------|-----------|---------|
| $\mathcal{S}_{\text{graph}}$ | Finite directed graphs with labeled vertices and edges | $G = (V, E, \ell_V, \ell_E)$ |
| $\mathcal{S}_{\text{tensor}}$ | Tensors over a fixed field | $T \in \mathbb{F}^{d_1 \times \cdots \times d_k}$ |
| $\mathcal{S}_{\text{symbolic}}$ | Well-formed expressions in a formal language | $\lambda x. f(g(x))$ |
| $\mathcal{S}_{\text{simplicial}}$ | Finite abstract simplicial complexes | $\Delta \subseteq 2^V$ |
| $\mathcal{S}_{\text{categorical}}$ | Objects in a specified category | $A \in \text{Ob}(\mathcal{C})$ |

### 1.2.3 Axiom: Distinguishability

For any two distinct states $s_1, s_2 \in \mathcal{S}$, their canonical encodings must differ: $c(s_1) \neq c(s_2)$. If this fails, the states are observationally equivalent under the identity observation.

---

## 1.3 The Transformation Set $\mathcal{T}$

### 1.3.1 Core Definition

$\mathcal{T}$ is a set of maps:

$$
\mathcal{T} \subseteq \{ \tau : \mathcal{S} \to \mathcal{S} \}
$$

or, for nondeterministic systems:

$$
\mathcal{T} \subseteq \{ \tau : \mathcal{S} \to \mathcal{P}(\mathcal{S}) \}
$$

### 1.3.2 Required Structure

$\mathcal{T}$ must form a **semigroup under composition**: for $\tau_1, \tau_2 \in \mathcal{T}$, there exists a rule implementing $\tau_1 \circ \tau_2$, and composition is associative.

### 1.3.3 Rule Classification

Rules are classified along two axes:

**Semantic type**:
- `structured`: semantically meaningful, information-processing operations.
- `destructive`: entropy-increasing operations for null-distribution calibration. Deliberately biased toward information destruction.

**Locality class**:
- *Pointwise*: affects only the target vertex.
- *Neighborhood-read*: reads from neighbors, writes to target.
- *Multi-write*: writes to multiple vertices simultaneously.

### 1.3.4 Axiom: Nontriviality

There exists $\tau \in \mathcal{T}$ and $s \in \mathcal{S}$ such that $\tau(s) \neq s$.

---

## 1.4 The Observation Operators $\mathcal{O}$

### 1.4.1 Core Definition

$\mathcal{O}$ is a set of functions:

$$
\mathcal{O} \subseteq \{ o : \mathcal{S} \to \mathcal{Y} \}
$$

where $\mathcal{Y}$ is an observation space. Observation values must be hashable and immutable.

### 1.4.2 Observation Granularity

| Level | Example | Distinguishing Power |
|-------|---------|---------------------|
| Identity | Canonical encoding | Distinguishes all states |
| Full compound | Labels + edges | Distinguishes all label/edge configurations |
| Label vector | Vertex labels only | Ignores edge structure |
| Scalar aggregate | Label sum, edge count | Coarse-grained |

### 1.4.3 Axiom: Dynamic Sufficiency

An observation set $\mathcal{O}$ is **dynamically sufficient** for $\mathcal{T}$ if replacing $\mathcal{O}$ with the identity observation does not qualitatively change emergence metric values. A coarser observation that yields zero metrics while the identity observation yields nonzero metrics is insufficient.

---

## 1.5 The Resource Constraints $\mathcal{R}$

### 1.5.1 Required Resources

| Resource | Notation | Meaning |
|----------|----------|---------|
| **Time** | $R_{\text{time}}$ | Transformation steps |
| **Space** | $R_{\text{space}}$ | State representation size |
| **Locality** | $R_{\text{local}}$ | Maximum interaction radius |

### 1.5.2 Resource Algebra

Resources are subadditive under composition: $R_i(\tau_1 \circ \tau_2) \le R_i(\tau_1) + R_i(\tau_2)$.

### 1.5.3 Implementations

Every Information Universe provides exactly one resource model realizing $R_{\text{time}}$, $R_{\text{space}}$, and $R_{\text{local}}$ for its own states and rules.

| Universe | $R_{\text{space}}$ | $R_{\text{time}}$ | $R_{\text{local}}$ |
|----------|---------------------|---------------------|----------------------|
| Binary Graph | $n^2 + n$ (adjacency + labels) | $1$ per application | Rule's maximum graph distance |
| Cellular Automaton | $N$ (cell count) | $1$ per synchronous update | $R$ (neighborhood radius) |

The Resource Algebra (1.5.2) is checked directly rather than assumed: composing two rules must not manufacture cost the parts did not already require. In the Binary Graph Universe, composing two rules combines locality via $\max(R_{\text{local}}(\tau_1), R_{\text{local}}(\tau_2))$, which is a tighter bound that still satisfies subadditivity, since $\max(a,b) \le a+b$ for non-negative costs.

A universe with no resource structure of its own may use the trivial model $R_{\text{time}} \equiv 1$, $R_{\text{space}} \equiv 0$, $R_{\text{local}} \equiv 0$ rather than leaving $\mathcal{R}$ unspecified — the tuple is not considered complete without an explicit resource model, even a trivial one.

---

## 1.6 The Invariant Structure $\mathcal{I}$

$\mathcal{I}$ is a set of functions $I: \mathcal{S} \to \mathbb{R}$ that are conserved (exactly or approximately) under all $\tau \in \mathcal{T}$. All invariants must be computable in finite time.

### 1.6.1 Conservation and Tolerance

$I$ is conserved between states $s$ and $\tau(s)$ within tolerance $\epsilon \ge 0$ if $|I(s) - I(\tau(s))| \le \epsilon$. $\epsilon = 0$ is exact conservation. Approximate invariants ($\epsilon > 0$) are permitted but must state $\epsilon$ explicitly.

### 1.6.2 Verification

"Computable in finite time" (1.6) is discharged one of two ways:

- **Exhaustive verification**: for a finite $\mathcal{S}$, evaluate conservation for every $s \in \mathcal{S}$ under a given $\tau$. Decisive, but only tractable for small state spaces.
- **Trajectory violation rate**: for larger or infinite $\mathcal{S}$, evaluate conservation across the states visited along sampled trajectories and report the fraction of transitions at which it failed. Not decisive, but scales.

### 1.6.3 Triviality (Failure Condition F-2)

Not every conserved quantity is a discovery. If a rule set is constructed so that it structurally cannot touch some part of the state, conservation of a quantity depending only on that part is a restatement of how the rules were built, not new information — exactly the situation F-2 warns against. The Binary Graph Universe's edge-count invariant (1.6.4) is a worked example of this: it is exactly conserved only because no shipped rule ever mutates the adjacency matrix, which is a fact about the rule set's construction, not a discovery about computation. Triviality is a research judgment about *why* a quantity is conserved, not a property this document mechanizes; 1.6.2's verification procedures only establish *whether* it is conserved.

### 1.6.4 Implementations

| Universe | Invariant | Status |
|----------|-----------|--------|
| Binary Graph | Edge count | Exactly conserved by every shipped rule (trivial by construction — see 1.6.3) |
| Binary Graph | Label sum | Not conserved by most structured rules — a contrast case, not a claim |
| Cellular Automaton | Population count | Exactly conserved only by number-conserving rules (e.g. Wolfram Rule 184); violated by most others — genuinely rule-dependent |
| Cellular Automaton | Population parity | Conserved iff the rule is parity-conserving; generalizes an exhaustive check the CA substrate already performed informally before $\mathcal{I}$ was implemented |

A universe that claims no invariants provides the empty set rather than leaving $\mathcal{I}$ unspecified.

---

## 1.7 The Update Schedule $\mathcal{K}$

### 1.7.1 Core Definition

$\mathcal{K}$ specifies the order and selection of transformations at each timestep. Two universes differing only in schedule are distinct objects of study.

### 1.7.2 Schedule Classification

- **Timing**: *synchronous* vs *asynchronous*
- **Selection**: *exhaustive*, *stochastic*, or *priority*

### 1.7.3 Standard Schedules

| Schedule | Timing | Selection | Description |
|----------|--------|-----------|-------------|
| All-vertices | Asynchronous | Exhaustive | Every vertex updated once per timestep in random order; first matching rule fires; later vertices see earlier updates |

---

## 1.8 Validation Substrates and Discovery Substrates

The Binary Graph Universe shipped with ARCO is a **validation substrate**: it uses hand-coded computational primitives (logic gates, transport rules) with human-assigned semantic labels to verify that ARCO's metrics, calibration, and hypothesis-testing pipeline function correctly. This is analogous to using a known chemical reaction to calibrate a spectrometer.

Validation substrates do not violate Computational Neutrality because the *framework* (Information Universes, emergence metrics, calibrated thresholds) is paradigm-neutral. The *current universe instance* uses known primitives as a bootstrap. Future **discovery substrates** will generate rules without human semantic labels, requiring ARCO to identify computational structure without knowing what "NAND" or "PROPAGATE" means in advance.

---

# Part Two: The Information Processing Hierarchy

---

$$
\begin{array}{c}
\text{Information Universe} \\
\downarrow \\
\text{Structured Information Universe} \\
\downarrow \\
\text{Information-Bearing Universe} \\
\downarrow \\
\text{Storage Universe} \\
\downarrow \\
\text{Computational Universe} \\
\downarrow \\
\text{Universal Computational Universe} \\
\downarrow \\
\text{Novel Computational Universe}
\end{array}
$$

Step-to-step persistence ($\Delta=1$) is not included in the hierarchy. It is not reliably measurable with current ensemble sizes. Storage (maximum persistence across all $\Delta$) is the primary emergence signal.

Storage and Memory (2.1, 2.2) are drawn as sequential levels below for continuity with earlier versions of this document, but they are calibrated independently and neither implies the other in general (2.2). Read the diagram as "in the order these conditions are typically checked," not as strict logical containment between those two levels specifically — every other adjacent pair in the hierarchy is a strengthening of the one below it.

---

## 2.1 Storage Universe

A universe exhibits **storage** if:

$$
\text{Store}(\mathcal{U}) > \theta_{\text{stor}}
$$

where storage is the maximum shuffle-corrected NMI across all timescales $\Delta \in [1, \Delta_{\text{max}}]$, computed using pooled estimation.

---

## 2.2 Memory Universe

A universe exhibits **memory** if:

$$
\text{Mem}(\mathcal{U}) > \theta_{\text{mem}}
$$

where memory is the *mean* shuffle-corrected NMI across all timescales $\Delta \in [1, \Delta_{\text{max}}]$ that had enough pooled samples to estimate, computed from the same pooled per-$\Delta$ profile as storage (2.1, 3.3) but averaged rather than maximized.

Storage and Memory Universe are related but logically distinct classifications, not nested levels of the same condition. Since the mean of a profile never exceeds its max, $\text{Mem}(\mathcal{U}) \le \text{Store}(\mathcal{U})$ pointwise, always. But $\theta_{\text{stor}}$ and $\theta_{\text{mem}}$ are calibrated independently against their own null distributions (8.1), so satisfying one condition does not imply satisfying the other:

- **Storage without memory**: information survives at one specific timescale and nowhere else — a sharp resonance, not durable retention.
- **Storage and memory together**: information persists broadly and roughly equally across timescales.

Memory is the capacity to preserve information about past observations such that it can be recovered later, evaluated as an *average* over how far back that recovery still works, rather than the *best case* storage reports.

---

# Part Three: Emergence Metrics

---

## 3.1 Ensemble Requirement

All emergence metrics are computed over ensembles of $n \ge 2$ trajectories from distinct initial states.

---

## 3.2 Shuffle-Corrected Normalized Mutual Information

$$
\text{NMI}_{\text{corr}}(X, Y) = \text{NMI}(X, Y) - \mathbb{E}[\text{NMI}(X, Y_{\text{shuf}})]
$$

where $Y_{\text{shuf}}$ is $Y$ with temporal order randomly permuted. The expectation is over $k \ge 5$ shuffles. The result is clamped to $[0, 1]$.

**Limitation**: The plugin MI estimator has known small-sample bias when the observation alphabet is large relative to sample size. Shuffle correction subtracts the mean baseline but does not eliminate all bias; the bias-corrected estimators of 3.2.1 reduce it without removing it. Global shuffling assumes no long-range temporal autocorrelation in the null distribution.

---

## 3.2.1 Estimator Variants

The mutual information in 3.2 is estimated from finite samples by one of four estimators, selected through the metric configuration:

| Estimator | Method |
|---|---|
| Plugin (default) | Empirical-frequency estimate |
| Miller–Madow (MM) | Plugin plus the first-order bias correction $(K_{xy} - K_x - K_y + 1) / (2N \ln 2)$, with $K$ the numbers of *observed* distinct values |
| Quadratic Extrapolation (QE) | Plugin MI at data fractions $1, 1/2, 1/4$, extrapolated to $1/N \to 0$ (Strong et al., 1998) |
| Nemenman–Shafee–Bialek (NSB) | Bayesian mixture of symmetric Dirichlet priors (Nemenman, Shafee & Bialek, 2002); requires alphabet cardinalities, supplied by an *observed* or *explicit* cardinality policy |

Shuffle correction applies to every estimator. Null calibration (8.1) and scoring must use the same estimator, and calibrated thresholds are not comparable across estimators. The estimator, and the NSB cardinality policy, are part of the experimental configuration and must be reported with every result. QE and NSB are distinct methods and must be cited as such.

---

## 3.3 Persistence

$$
\boxed{\text{Persist}(\mathcal{U}, \Delta) = \frac{1}{T-\Delta} \sum_{t=0}^{T-\Delta-1} \text{NMI}_{\text{corr}}\left(
\{o(s_t^{(i)})\}_{i=1}^n,
\{o(s_{t+\Delta}^{(i)})\}_{i=1}^n
\right)}
$$

Per-timestep persistence at $\Delta=1$ with small ensembles rarely exceeds the shuffle baseline. Use storage or memory instead — both use pooled estimation across the full $\Delta$ range.

---

## 3.4 Storage

$$
\boxed{\text{Store}(\mathcal{U}) = \max_{\Delta \in [1, \Delta_{\text{max}}]} \text{NMI}_{\text{corr}}\left(
\bigcup_{i,t} \{o(s_t^{(i)})\},
\bigcup_{i,t} \{o(s_{t+\Delta}^{(i)})\}
\right)}
$$

Storage uses **pooled estimation**: all observation pairs from all ensemble members and all timesteps are pooled before computing NMI.

---

## 3.5 Memory

$$
\boxed{\text{Mem}(\mathcal{U}) = \frac{1}{|\Delta_{\text{valid}}|} \sum_{\Delta \in \Delta_{\text{valid}}} \text{NMI}_{\text{corr}}\left(
\bigcup_{i,t} \{o(s_t^{(i)})\},
\bigcup_{i,t} \{o(s_{t+\Delta}^{(i)})\}
\right)}
$$

where $\Delta_{\text{valid}} \subseteq [1, \Delta_{\text{max}}]$ is the set of timescales with enough pooled pairs. Memory and storage are computed from the *same* per-$\Delta$ pooled profile — storage takes its max, memory takes its mean — so the two numbers are always directly comparable on $[0,1]$ and $\text{Mem}(\mathcal{U}) \le \text{Store}(\mathcal{U})$ by construction.

This is not "active information storage" (Lizier et al.), which conditions on the entire past history rather than a single lagged pair. That estimator requires ensembles much larger than ARCO's calibration currently uses (8.1) to avoid being dominated by small-sample bias (3.2). Memory as defined here differs from storage only in aggregation — mean vs. max across $\Delta$ — not in what information-theoretic quantity is estimated at each $\Delta$.

---

## 3.6 Trajectory Separation (Diagnostic)

Measures distinguishability of futures given different initial conditions via total variation distance. High values indicate sensitivity to initial conditions, **not memory**. Preserved for diagnostic use.

---

# Part Four: Computational Criteria

A universe is **computational** if and only if it satisfies:

- **C1 (Representation)**: Information can be encoded in states.
- **C2 (Transformation)**: Encoded information can be manipulated nontrivially.
- **C3 (Observation)**: Encoded information can be recovered.
- **C4 (Reliability)**: Computational behavior survives small perturbations.
- **C5 (Compositionality)**: Computational processes can be combined.

---

# Part Five: Discovery Criteria

A candidate universe must satisfy at least one of:

- **D1**: Novel representation efficiency.
- **D2**: Novel computational organization.
- **D3**: Novel error resilience.
- **D4**: Novel universality.
- **D5**: Novel invariant.

---

# Part Six: Computational Taxonomy and Equivalence

---

## 6.1 The Taxonomy

A versioned catalogue of known computational models including finite automata, Turing machines, Boolean circuits, quantum circuits, cellular automata, neural networks, rewriting systems, lambda calculi, and categorical quantum mechanics models.

---

## 6.2 Simulation and Bisimulation

$\mathcal{U}_A$ simulates $\mathcal{U}_B$ if there exist encoding, decoding, and transformation correspondence maps such that the dynamics of $\mathcal{U}_B$ are reproduced up to observational equivalence. Bisimulation is mutual simulation. Genuine novelty requires that no known model in the Taxonomy is bisimilar to the candidate.

---

# Part Seven: The Universe Generator $\mathcal{G}$

$$
\mathcal{G}: \Theta \to \mathbb{U}
$$

Generator classes include grammar, categorical, rewrite, evolutionary, constraint, random, compositional, and mixed generators. The generator parameter space $\Theta$ is itself searchable.

---

# Part Eight: The Scientific Cycle

```
GENERATE → CALIBRATE → OBSERVE → HYPOTHESIZE → PREDICT → TEST → REVISE
```

---

## 8.1 Threshold Calibration

**Procedure**:

1. Generate $m \ge 30$ null universes using purely destructive rule sets, each containing at least one information-scrambling rule.
2. Compute the emergence metric for each null universe.
3. Set threshold $\theta$ to the 95th percentile of the null distribution.
4. Apply engineering floors to prevent degenerate cases.

---

# Part Nine: Formal Hypotheses

---

## 9.1 Hypothesis Template

> **Hypothesis $H_{\text{id}}$**
>
> **Conditions**: Formal predicates on universe structure.
> **Claim**: Any universe satisfying Conditions exhibits Property $P$.
> **Prediction**: For any $\mathcal{U}$ satisfying Conditions, Metric $M(\mathcal{U}) > \theta$.
> **Test**: Evaluate Metric on $n$ held-out universes satisfying Conditions.
> **Success Criterion**: The hypothesis's survival criterion (9.2) is met on the held-out universes.
> **Falsification**: If the criterion is not met, the hypothesis is rejected.

---

## 9.2 Hypothesis Scoring and Survival

$$
\boxed{\text{Score}(H) = \text{BalancedAccuracy}(H) - \lambda \cdot \text{Complexity}(H)}
$$

where $\lambda = 0.1$ and $\text{BalancedAccuracy} = (\text{Recall} + \text{Specificity}) / 2$, computed on held-out universes from the confusion counts of the condition's prediction against the observed property. Balanced accuracy is used because plain accuracy is dominated by the more common class of the test set, whether or not the condition is predictive.

**Survival.** Every hypothesis carries a *survival criterion*: a predicate over its classification metrics (accuracy, precision, recall, specificity, balanced accuracy, coverage, score) and its complexity. The **default** criterion is

$$
\text{BalancedAccuracy}(H) \ge 0.5 \;\wedge\; \text{Score}(H) > 0.
$$

A different criterion may replace the default for a given hypothesis. It must be declared, with a human-readable description, **before** evaluation and reported with the result; changing it after seeing test outcomes defines a new hypothesis.

---

# Part Ten: Meta-Theorems

ARCO discovers not only regularities about universes but regularities about discovery. Meta-hypotheses concern the generator parameter space, the effectiveness of inductive biases, and the predictive power of different metrics.

---

# Part Eleven: Inductive Biases

Configurable biases guide the search: locality, compositionality, stability, and resource monotonicity. The set of active biases is itself searchable.

---

# Part Twelve: The Hierarchical Search Space

```
Level 0: INFORMATION UNIVERSES
Level 1: STRUCTURED INFORMATION UNIVERSES
Level 2: INFORMATION-BEARING UNIVERSES
Level 3: STORAGE UNIVERSES
Level 4: MEMORY UNIVERSES
Level 5: COMPUTATIONAL UNIVERSES
Level 6: UNIVERSAL COMPUTATIONAL UNIVERSES
Level 7: NOVEL COMPUTATIONAL UNIVERSES
```

Levels 3 and 4 are the one place in this list that is not strict containment: Storage Universe (max over timescales) and Memory Universe (mean over timescales) are independently calibrated conditions on the same underlying profile, not nested requirements — see 2.2. A universe can occupy Level 3 without Level 4, or be evaluated against Level 4's criterion without having first been screened at Level 3. Every other adjacent pair strengthens the one below it.

---

# Part Thirteen: What ARCO Is Not

- Not a proposal to build a better computer.
- Not an attempt to discover practical algorithms.
- Not a replacement for human mathematicians.
- Not a claim that discovered structures will be useful for any applied purpose.
- Not an attempt to simulate physical reality.
- Not a theory of everything.

ARCO is a precisely scoped scientific instrument for exploring the space of possible information universes and discovering the conditions under which computation emerges.

---

# Part Fourteen: Implementations

- **Python reference** ([arco-python](https://github.com/kvernet/arco-python)): First validated the methodology.
- **Rust production** ([arco](https://crates.io/crates/arco)): Parallelized, high-performance implementation. Experimental findings are reported in the [project README](https://github.com/kvernet/arco).