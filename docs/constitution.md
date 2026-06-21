# ARCO

# Mathematical Constitution

## Experimental Validation

---

> *This document defines the mathematical objects, operations, and criteria that constitute ARCO.*

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

**Status**: Not triggered. Storage detected in 60.7% of spectrum universes in the Python reference. All five basic Boolean functions (NAND, AND, OR, NOR, XOR) rediscovered. Confirmed in Rust implementation at n=50,000 (43.2% storage rate, 94.3% structured storage).

---

## F-2: The Triviality Condition

**Statement**: Every invariant discovered by ARCO is a logical consequence of the resource algebra or state space axioms alone.

**Status**: Not yet testable. Awaiting invariant discovery infrastructure.

---

## F-3: The Vacuous Novelty Condition

**Statement**: Every universe ARCO discovers is bisimilar to a known computational model in the Taxonomy.

**Status**: Not triggered. The Binary Graph Universe was designed to rediscover known models, not novel ones. This condition applies when ARCO searches for novel computation.

---

## F-4: The Scalability Condition

**Statement**: ARCO's search procedure cannot explore state spaces beyond size $10^3$ within reasonable resources.

**Status**: Not triggered. Python reference processes 300 universes in ~36 seconds. Rust implementation processes 50,000 universes in ~7 minutes on 20 cores (110 universes/second), demonstrating linear scaling.

---

## F-5: The Incomputability Condition

**Statement**: Any emergence metric requires computing a quantity that is provably uncomputable.

**Status**: Not triggered. All metrics are computable (shuffle-corrected plugin NMI, total variation distance).

---

## F-6: The Disconfirmation Condition

**Statement**: No predictive laws of the form "Condition A ∧ Condition B ⇒ Emergent Property C" generalize to unseen universes.

**Status**: Not triggered. Four hypotheses survived cross-validation in both Python and Rust implementations. The Transport Law (H5) achieved 91.2% accuracy in the Python reference and ~53-61% accuracy at n=50,000 in Rust. The Logic Gate Law (H3), Majority Structure Law (H2), and Multiple Logic Law (H7) also survive at all scales.

---

## F-7: The Overfitting Condition

**Statement**: Surviving hypotheses have complexity exceeding a threshold relative to their predictive accuracy.

**Status**: Not triggered. Surviving hypotheses have complexity ≤ 2.0 and accuracy ≥ 52.7%, yielding positive scores after the MDL penalty across all experimental scales.

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

*The schedule $\mathcal{K}$ determines which transformations apply when, and is not reducible to the transformation set. Changing the schedule from random-vertex to all-vertices altered persistence from 0% to 97.7% in experiments.*

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
- `structured`: semantically meaningful, information-processing operations (logic gates, transport, identity).
- `destructive`: entropy-increasing operations for null-distribution calibration. Not "random" — they are deliberately biased toward information destruction.

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

An observation set $\mathcal{O}$ is **dynamically sufficient** for $\mathcal{T}$ if replacing $\mathcal{O}$ with the identity observation (canonical encoding) does not qualitatively change emergence metric values. If a coarser observation yields zero metrics while the identity observation yields nonzero metrics, $\mathcal{O}$ is insufficient.

**Experimental basis**: `observe_root_label` (single vertex) yielded zero persistence in early experiments. `observe_label_vector` (full label tuple) and `observe_compound` (labels + edges) yielded the storage spectrum.

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

Schedules are classified along two axes:

- **Timing**: *synchronous* (all updates computed from the same pre-timestep state) vs *asynchronous* (updates immediately visible to later operations within the same timestep).
- **Selection**: *exhaustive* (every update site visited once), *stochastic* (sites sampled probabilistically), or *priority* (sites ordered by a fixed criterion).

### 1.7.3 Standard Schedules

| Schedule | Timing | Selection | Description |
|----------|--------|-----------|-------------|
| All-vertices | Asynchronous | Exhaustive | Every vertex updated once per timestep in random order; first matching rule fires; later vertices see earlier updates |

The all-vertices schedule was used in all Binary Graph Universe experiments. Rule ordering is randomized per vertex, making rule competition probabilistic even with deterministic rules.

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

*"Persistent Universe" is not included in the hierarchy. Step-to-step persistence (Δ=1) is not reliably measurable with current ensemble sizes and estimators. Storage (maximum persistence across all Δ) is the primary emergence signal.*

---

## 2.1 Storage Universe

A universe exhibits **storage** if:

$$
\text{Store}(\mathcal{U}) > \theta_{\text{stor}}
$$

where storage is the maximum shuffle-corrected NMI across all timescales Δ ∈ [1, Δ_max], computed using **pooled estimation** (all timesteps and ensemble members pooled before MI computation).

**Experimental basis**: Storage was the primary discriminating metric in the Binary Graph Universe spectrum, ranging from ~18% (Noise) to ~95% (Structured) across both Python and Rust implementations at scales up to n=50,000.

---

## 2.2 Memory Universe

A universe exhibits **memory** if it exhibits storage. Memory is the capacity to preserve information about past observations such that it can be recovered later — which is exactly what storage measures via $I(O_t; O_{t+\Delta})$.

*Memory was previously defined as trajectory separation (distinguishability of futures given different initial conditions). That metric measures sensitivity to initial conditions, not memory. Trajectory separation is preserved as a diagnostic metric but is not memory.*

---

# Part Three: Emergence Metrics

---

## 3.1 Ensemble Requirement

All emergence metrics must be computed over ensembles of trajectories from distinct initial conditions.

An **ensemble** is a set of $n$ trajectories generated from distinct initial states. The minimum $n$ depends on the observation space cardinality and the effect size being measured. For the Binary Graph Universe with compound observation, $n = 10$ was sufficient for storage detection with pooled estimation.

---

## 3.2 Shuffle-Corrected Normalized Mutual Information

All MI-based metrics must use bias correction via temporal shuffling.

$$
\text{NMI}_{\text{corr}}(X, Y) = \text{NMI}(X, Y) - \mathbb{E}[\text{NMI}(X, Y_{\text{shuf}})]
$$

where $Y_{\text{shuf}}$ is $Y$ with temporal order randomly permuted. The expectation is over $k \ge 5$ shuffles. The result is clamped to $[0, 1]$.

**Experimental basis**: Raw NMI gave a null-distribution mean of ~0.89 for destructive rule sets. Shuffle correction reduced this to ~0.00, restoring proper discrimination between structured and destructive universes.

**Limitation**: Global shuffling assumes no long-range temporal autocorrelation in the null distribution. For periodic or strongly autocorrelated systems, use block shuffling or circular phase randomization.

---

## 3.3 Storage

$$
\boxed{\text{Store}(\mathcal{U}) = \max_{\Delta \in [1, \Delta_{\text{max}}]} \text{NMI}_{\text{corr}}\left(
\bigcup_{i,t} \{o(s_t^{(i)})\},
\bigcup_{i,t} \{o(s_{t+\Delta}^{(i)})\}
\right)}
$$

Storage uses **pooled estimation**: all observation pairs from all ensemble members and all timesteps are pooled into two vectors before computing NMI. This gives the estimator sufficient samples to distinguish signal from shuffle baseline.

*Per-timestep averaging was found to be unreliable with small ensembles (n=10). Pooled estimation is the canonical method for storage.*

---

## 3.4 Persistence

$$
\boxed{\text{Persist}(\mathcal{U}, \Delta) = \frac{1}{T-\Delta} \sum_{t=0}^{T-\Delta-1} \text{NMI}_{\text{corr}}\left(
\{o(s_t^{(i)})\}_{i=1}^n,
\{o(s_{t+\Delta}^{(i)})\}_{i=1}^n
\right)}
$$

Persistence is the per-timestep average of ensemble NMI. For Δ=1 with small ensembles (n=10), the per-timestep estimator rarely exceeds the shuffle baseline. **Persistence at Δ=1 is not a reliable emergence indicator at current ensemble sizes.** Use storage instead.

*Persistence is documented as requiring larger ensembles (n ≥ 50) or a different estimator to be reliable. It is preserved for completeness and for use with larger-scale experiments.*

---

## 3.5 Memory

Memory is an alias for storage. See Section 2.2.

---

## 3.6 Trajectory Separation (Diagnostic)

Trajectory separation measures how distinguishable futures are given different initial conditions, using total variation distance between conditional output distributions. High values indicate sensitivity to initial conditions, **not memory**. This metric is preserved for diagnostic purposes.

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
- **D2**: Novel computational organization (primitives, tradeoffs, invariants, composition).
- **D3**: Novel error resilience.
- **D4**: Novel universality.
- **D5**: Novel invariant.

---

# Part Six: Computational Taxonomy and Equivalence

---

## 6.1 The Taxonomy

A versioned, peer-reviewed catalogue of known computational models. The current version includes finite automata, Turing machines, Boolean circuits, quantum circuits, cellular automata, neural networks, rewriting systems, lambda calculi, and categorical quantum mechanics models.

---

## 6.2 Simulation and Bisimulation

$\mathcal{U}_A$ simulates $\mathcal{U}_B$ if there exist encoding, decoding, and transformation correspondence maps such that the dynamics of $\mathcal{U}_B$ are reproduced up to observational equivalence. Bisimulation is mutual simulation. Genuine novelty requires that no known model in the Taxonomy is bisimilar to the candidate.

---

# Part Seven: The Universe Generator $\mathcal{G}$

$$
\mathcal{G}: \Theta \to \mathbb{U}
$$

Generator classes include grammar, categorical, rewrite, evolutionary, constraint, random, compositional, and mixed generators. The generator parameter space $\Theta$ is itself searchable (meta-search).

---

# Part Eight: The Scientific Cycle

```
GENERATE → CALIBRATE → OBSERVE → HYPOTHESIZE → PREDICT → TEST → REVISE
```

---

## 8.1 Threshold Calibration

Thresholds are calibrated per universe class.

**Procedure**:

1. Generate $m \ge 30$ null universes using purely destructive rule sets, each guaranteed to contain at least one information-scrambling rule.
2. Compute the emergence metric for each null universe.
3. Set threshold $\theta$ to the 95th percentile of the null distribution.
4. Apply engineering floors (minimum thresholds) to prevent degenerate cases. These floors are safeguards against statistical artifacts, not scientific priors.

The calibration returns null distribution statistics (mean, standard deviation) and an empirical p-value function for effect size estimation.

**Experimental basis**: Degenerate null universes (e.g., two DESTROY_ZERO rules producing a constant state) were found to inflate the null distribution. Forcing at least one SCRAMBLE_ALL rule per null universe resolved this.

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

# Part Ten: Discovered Laws

*Laws that have survived cross-validation in both the Python reference and Rust implementations.*

---

## Law 1: The Transport Law

**Statement**: Rule sets containing information transport operations (PROPAGATE, SWAP, COPY_TO_OUT, COPY_FROM_IN) exhibit storage at significantly higher rates than those without.

**Evidence**:
- Python reference (n=300): 91.2% accuracy, score 0.812
- Rust implementation (n=50,000): ~53-61% accuracy, score 0.43-0.51
- Status: **SURVIVED** across all experimental scales

**Interpretation**: Transport rules create persistent correlations and delayed dependencies across the graph, enabling information to survive multiple timesteps. This is the strongest and most replicated finding in ARCO.

---

## Law 2: The Logic Gate Law

**Statement**: Rule sets containing at least one logic gate (NAND, NOR, AND, OR, XOR, NOT) exhibit memory above threshold.

**Evidence**:
- Python reference (n=300): 76.9% accuracy, score 0.619
- Rust implementation (n=50,000): ~53% accuracy, score 0.38-0.41
- Status: **SURVIVED**

---

## Law 3: The Majority Structure Law

**Statement**: Rule sets where the majority of rules are structured exhibit memory above threshold.

**Evidence**:
- Python reference (n=300): 84.5% accuracy, score 0.695
- Rust implementation (n=50,000): ~62-64% accuracy, score 0.47-0.49
- Status: **SURVIVED**

---

## Law 4: The Multiple Logic Law

**Statement**: Rule sets containing at least two logic gates exhibit memory above threshold.

**Evidence**:
- Python reference (n=300): 84.6% accuracy, score 0.646
- Rust implementation (n=50,000): ~56-63% accuracy, score 0.36-0.43
- Status: **SURVIVED**

---

## Law 5: The Structure-Storage Gradient

**Statement**: The probability that a universe exhibits storage increases monotonically with the fraction of structured rules.

**Evidence (Rust implementation, n=50,000)**:

| Structured Ratio | Storage Above Threshold |
|------------------|------------------------|
| 0.00 – 0.15 | ~18% |
| 0.15 – 0.40 | ~16% |
| 0.40 – 0.60 | ~26% |
| 0.60 – 0.85 | ~40% |
| 0.85 – 1.00 | **~95%** |

---

## Negative Results

The following hypotheses were tested and **failed**:

- **H1 (Has structured rule → persistence)**: The presence of a single structured rule is insufficient to guarantee emergence.
- **H4 (All structured → persistence)**: Even fully-structured rule sets often fail to produce detectable persistence. This is a constraint on any theory of emergent computation.
- **H6 (All destructive → persistence)**: Correctly fails — validates the null distribution calibration.
- **H8 (Mixed rules → persistence)**: The condition (0.3 < ratio < 0.7) is too broad or persistence is the wrong metric.

---

# Part Eleven: Meta-Theorems

ARCO discovers not only laws about universes but laws about discovery. Meta-hypotheses concern the generator parameter space $\Theta$, the effectiveness of inductive biases, and the predictive power of different metrics.

---

# Part Twelve: Inductive Biases

Configurable biases guide the search: locality, compositionality, stability, and resource monotonicity. The set of active biases is itself searchable (bias variation meta-search).

---

# Part Thirteen: The Hierarchical Search Space

```
Level 0: INFORMATION UNIVERSES
Level 1: STRUCTURED INFORMATION UNIVERSES
Level 2: INFORMATION-BEARING UNIVERSES
Level 3: STORAGE UNIVERSES
Level 4: MEMORY UNIVERSES (= STORAGE)
Level 5: COMPUTATIONAL UNIVERSES
Level 6: UNIVERSAL COMPUTATIONAL UNIVERSES
Level 7: NOVEL COMPUTATIONAL UNIVERSES
```

---

# Part Fourteen: What ARCO Is Not

- Not a proposal to build a better computer.
- Not an attempt to discover practical algorithms.
- Not a replacement for human mathematicians.
- Not a claim that discovered structures will be useful for any applied purpose.
- Not an attempt to simulate physical reality.
- Not a theory of everything.

ARCO is a precisely scoped scientific instrument for exploring the space of possible information universes, discovering the conditions under which computation emerges, and identifying genuinely novel computational structures and the laws that govern them.

---

# Part Fifteen: Implementations

## Python Reference

The Python reference implementation (`arco` package, Python 3.10+) first validated every definition in this Constitution. It reproduces all five discovered laws, the Structure-Storage Gradient, and Boolean rediscovery.

## Rust Production Implementation

The Rust crate (`arco` on crates.io, v0.2+) provides a parallelized, high-performance implementation capable of processing 50,000 universes in ~7 minutes on 20 cores. It reproduces all four surviving hypotheses and the Structure-Storage Gradient, confirming the methodology generalizes across independent implementations.

---