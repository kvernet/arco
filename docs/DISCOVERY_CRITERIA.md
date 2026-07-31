# Discovery Substrate Criteria

This document defines the success criteria for a candidate discovery
substrate. It must be written **before** experiments begin. Results
are evaluated against these criteria regardless of outcome.

---

## 1. Null Model

- [ ] The null distribution is calibrated against a diverse pool of
  destructive/chaotic rules, not a single fixed rule.
- [ ] At least one negative control hypothesis is included (a condition
  expected to fail). Its failure is reported alongside successes.
- [ ] The null distribution and observed distribution are reported.
  The median observed storage across all test universes must exceed
  the 95th percentile of the null distribution by a margin of at
  least 0.1. (This tests separation, not absolute scale.)
- [ ] Alternatively, `NullStats::empirical_p` may be used: the median
  observed storage must have an empirical p-value < 0.05 against the
  null.

## 2. Test-Set Integrity

- [ ] Verified by construction that held-out test cases can satisfy
  every hypothesis's precondition. (The H4 failure mode: test data
  structurally excluding positive cases.)
- [ ] Train/test split is random and independent. For substrates where
  the rule space is small enough to enumerate (e.g., 256 CA rules),
  train and test must be a **disjoint partition**, not independently-
  drawn samples. This prevents the same rule from appearing in both
  sets.
- [ ] For substrates with a pre-generated rule pool, the pool must be
  large enough that train and test draw from non-overlapping
  regions, or sampling must be done per-call rather than via
  sequential cycling.

## 3. Statistical Robustness

- [ ] Results reported across ≥ 10 independent seeds with ranges
  (min–max), not single-seed point estimates.
- [ ] At least one bias-corrected estimator variant tested (Miller-Madow
  or NSB) alongside the plugin estimator with shuffle correction.
- [ ] Sample size (n_train, n_test) is documented and justified.
- [ ] Hypothesis accuracy confidence intervals reported (binomial
  proportion, 95% CI).

## 4. Hypothesis Design

- [ ] All hypotheses use measurable properties computed from rule
  definitions. No human semantic labels (no "structured," no rule
  names).
- [ ] **Exemption for validation substrates**: Substrates whose purpose
  is calibrating the measurement apparatus (not discovering novel
  computation) are exempt from the measurable-properties requirement.
  Validation substrates may use human-assigned rule names and semantic
  categories. This exemption applies to the Binary Graph Universe
  (which uses hand-coded NAND, PROPAGATE, etc. to verify the metrics
  work) but not to discovery substrates. A substrate claiming this
  exemption must explicitly document itself as a validation substrate
  and state what it is validating.

- [ ] At least one compound hypothesis (two properties combined) is
  included. The specific properties and combination logic must be
  named in this pre-registration document. Post-hoc compound
  hypotheses are not eligible for promotion.
- [ ] Hypothesis complexity is documented and the MDL penalty (λ = 0.1)
  is applied consistently.
- [ ] The total number of hypotheses tested is reported. If multiple
  hypothesis sets are tested, a correction for multiple comparisons
  (e.g., Bonferroni) is applied or the uncorrected results are
  explicitly flagged as exploratory.

## 5. Documentation

- [ ] Substrate design documented: state space, rule space, schedule,
  observation operators, measurable properties.
- [ ] All hypotheses listed with their conditions, predicted properties,
  and complexity scores — written before experiments.
- [ ] Null model construction documented, including the pool of
  destructive/chaotic rules and why they represent the null
  hypothesis for this substrate.

## 6. Promotion Bar (substrate)

A candidate clears the substrate bar if:
- All null model checks pass (Section 1)
- All test-set integrity checks pass (Section 2)
- At least one hypothesis survives at ≥ 8/10 seeds
- Negative control fails at ≥ 8/10 seeds
- The substrate compiles and runs via `arco <substrate>` CLI
- The substrate's tests and any feature flags it depends on
  (e.g., `serialize`) run in CI
- **Re-certification trigger**: If `metrics.rs`, `calibration.rs`,
  or the estimator pipeline changes, substrates that previously
  cleared this bar must be re-evaluated under the new pipeline
  version.
- Validation substrates are exempt from §4.1 (measurable properties
  only). All other criteria apply. A validation substrate that passes
  all applicable criteria is eligible for inclusion in ARCO core as a
  calibration instrument.

## 7. Publication Bar (paper)

A candidate clears the publication bar if it additionally:
- Has been stress-tested via independent reimplementation or
  external review
- Includes effect sizes, not just accuracy
- Connects findings to existing literature, including:
  - Wolfram's CA classification
  - Langton's λ and edge of chaos
  - Kauffman's Random Boolean Networks
  - Crutchfield's computational mechanics / ε-machines — the
    established framework for measuring information storage and
    processing in dynamical systems. ARCO must differentiate its
    approach from computational mechanics.
- The paper addresses one of these narratives, matching what the
  data actually shows:
  - **Discovery**: "ARCO discovered X without being told about X"
  - **Calibration**: "This measurable property tracks established
    ground truth, and here's where it diverges"
  - **Limitation**: "This substrate reveals a boundary condition
    for the estimator, informing future work"

---

*Version 1.0. Last updated: 2025-07-30.*