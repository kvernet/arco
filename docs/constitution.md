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

---

## 1.6 The Invariant Structure $\mathcal{I}$

$\mathcal{I}$ is a set of functions $I: \mathcal{S} \to \mathbb{R}$ that are conserved (exactly or approximately) under all $\tau \in \mathcal{T}$. All invariants must be computable in finite time.

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

---

## 2.1 Storage Universe

A universe exhibits **storage** if:

$$
\text{Store}(\mathcal{U}) > \theta_{\text{stor}}
$$

where storage is the maximum shuffle-corrected NMI across all timescales $\Delta \in [1, \Delta_{\text{max}}]$, computed using pooled estimation.

---

## 2.2 Memory Universe

A universe exhibits **memory** if it exhibits storage. Memory is the capacity to preserve information about past observations such that it can be recovered later — measured via $I(O_t; O_{t+\Delta})$.

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

**Limitation**: The plugin MI estimator has known small-sample bias when the observation alphabet is large relative to sample size. Shuffle correction subtracts the mean baseline but does not eliminate all bias. Global shuffling assumes no long-range temporal autocorrelation in the null distribution.

---

## 3.3 Storage

$$
\boxed{\text{Store}(\mathcal{U}) = \max_{\Delta \in [1, \Delta_{\text{max}}]} \text{NMI}_{\text{corr}}\left(
\bigcup_{i,t} \{o(s_t^{(i)})\},
\bigcup_{i,t} \{o(s_{t+\Delta}^{(i)})\}
\right)}
$$

Storage uses **pooled estimation**: all observation pairs from all ensemble members and all timesteps are pooled before computing NMI.

---

## 3.4 Persistence

$$
\boxed{\text{Persist}(\mathcal{U}, \Delta) = \frac{1}{T-\Delta} \sum_{t=0}^{T-\Delta-1} \text{NMI}_{\text{corr}}\left(
\{o(s_t^{(i)})\}_{i=1}^n,
\{o(s_{t+\Delta}^{(i)})\}_{i=1}^n
\right)}
$$

Per-timestep persistence at $\Delta=1$ with small ensembles rarely exceeds the shuffle baseline. Use storage instead.

---

## 3.5 Memory

Memory is an alias for storage.

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
> **Success Criterion**: Metric exceeds threshold on $\ge 50\%$ of universes.
> **Falsification**: If the criterion is not met, the hypothesis is rejected.

---

## 9.2 Hypothesis Scoring

$$
\boxed{\text{Score}(H) = \text{Accuracy}(H) - \lambda \cdot \text{Complexity}(H)}
$$

where $\lambda = 0.1$. A hypothesis **survives** if $\text{Score}(H) > 0$ and $\text{Accuracy}(H) \ge 0.5$.

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