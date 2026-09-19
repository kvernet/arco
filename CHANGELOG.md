# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and version numbers follow [Semantic Versioning](https://semver.org/)
(pre-1.0: a minor version bump may include breaking changes).

## Yanked versions

**All versions prior to 0.5.0 have been yanked from crates.io.** They
exposed non-default MI estimators (`--estimator mm`, `--estimator nsb`)
with correctness bugs described under 0.5.0 below. If you depended on
one of these versions:

- If you only used the default estimator (plugin with shuffle
  correction — i.e. you never passed `--estimator mm` or
  `--estimator nsb`, and never constructed a `MetricConfig` with a
  non-default `estimator` field), your results are **not** affected
  by either bug.
- If you used `mm` or `nsb`, treat those results as unreliable and
  re-run under 0.5.0 or later.
- In those yanked versions `--estimator nsb` selected the Quadratic
  Extrapolation estimator (see 0.5.0, *Changed*). From 0.6.0,
  `--estimator nsb` selects the genuine Nemenman–Shafee–Bialek
  estimator. Never compare `nsb` results across that boundary.

## [Unreleased]

### Added

- `Resources` trait and `UnitResources` (`src/resources.rs`), and the
  `Invariant` trait (`src/invariants.rs`), with CA and graph substrate
  implementations. Both are re-exported from the prelude.
- `InformationUniverse` now exposes `resources()` and `invariants()`,
  completing the six components of the Constitution's definition
  (state space, transformations, observations, resources, invariants,
  schedule).
- Configurable survival criteria:
  `Hypothesis::with_survival_criterion(criterion, description)`
  replaces the survival predicate for a single hypothesis.
  `Hypothesis::new()` keeps the historical default,
  `balanced_accuracy >= 0.5 AND score > 0`.
- `CycleConfig` lets callers define the complete cycle configuration.

### Changed

- **`memory()` is no longer an alias for `storage()`.** Both are
  computed from the same per-timescale pooled, shuffle-corrected NMI
  profile: `storage` is its maximum over Δ, `memory` its mean over Δ,
  so `memory <= storage`. Results produced with `memory()` in 0.6.0 and
  earlier are storage results.
- Hypothesis evaluation results live in
  `Hypothesis::classification_metrics` (`ClassificationMetrics`,
  including `balanced_accuracy` and `score`); the score is
  `balanced_accuracy - 0.1 * complexity`.
- Constitution §3.2.1 (estimator variants) and §9.2 (scoring and
  survival) updated accordingly.

## [0.6.0] - 2026-08-29

### Added

- **`Estimator::NSB`**: the genuine Nemenman–Shafee–Bialek
  (Nemenman, Shafee & Bialek, 2002) entropy / mutual-information
  estimator (`src/metrics/nsb.rs`), a Bayesian mixture of symmetric
  Dirichlet priors, selectable with `--estimator nsb`.
  `MetricConfig::cardinality` (`NsbCardinality::Observed` |
  `Explicit`) sets the alphabet cardinalities NSB requires.
- Estimator consistency benchmark with exact ground truth
  (`docs/benchmarks/estimator_consistency.md`) and validation results
  (`docs/RESULTS.md`).
- `ClassificationMetrics` (`accuracy`, `precision`, `recall`,
  `specificity`, `balanced_accuracy`, `coverage`), computed for every
  hypothesis in `run_cycle()` and stored on `HypothesisRecord`,
  replacing the previous single `correct / positive` ratio. The
  previous ratio conflated "is this condition predictive" with "how
  common is this condition in the test set" into one number with no
  way to separate them; the full breakdown makes that decomposable.
- `scripts/analyze.py`: pooled TP/FP/TN/FN aggregation across
  seed-records, reconstructed exactly from `coverage`, `precision`, and
  `accuracy` plus each record's `n_test` (exact, not approximate —
  those ratios were themselves computed from integer counts over a
  known total). `hypothesis_summary_pooled()` sums counts across all
  seeds *before* computing precision/recall/balanced_accuracy, instead
  of averaging each seed's independently-computed ratio.
- `scripts/analyze.py`: 95% Wilson score confidence interval on pooled
  `balanced_accuracy`. Verdicts whose interval excludes 50% by less
  than 2 points are flagged (🔶, "near-chance") rather than reported as
  a clean pass; verdicts whose interval still contains 50% are flagged
  separately (❓, "inconclusive").

### Changed

- **`Estimator::MillerMadow` renamed to `Estimator::MM`** (breaking
  for library users; the CLI value `--estimator mm` is unchanged).
- `Hypothesis::survives()` now checks `balanced_accuracy >= 0.5`
  rather than plain `accuracy >= 0.5`. Plain accuracy is dominated by
  whichever class is more common in the test set, independent of
  whether a hypothesis's condition is actually predictive — for a
  low-coverage condition (e.g. `H6_ALL_DESTRUCTIVE`, a negative
  control whose "positive" class is a small minority of any realistic
  test population), this let true-negative dominance alone push
  `accuracy` above 0.5 regardless of whether the condition predicted
  anything. Caught during development of the `ClassificationMetrics`
  change above, before it shipped: an interim version scoring
  `hypothesis.accuracy` from the new `accuracy` field (rather than
  `balanced_accuracy`) reproduced this exact failure mode —
  `H6_ALL_DESTRUCTIVE` surviving on a 1000-universe train / 300-universe
  test sweep despite showing zero true positives. Scoring from
  `balanced_accuracy` instead — already computed, same struct — closes
  it: pooled across a 30-seed sweep at the same scale,
  `H6_ALL_DESTRUCTIVE` correctly fails (8/30 per-seed survival, pooled
  balanced_accuracy 45.0%, 95% CI [41.5, 49.7]) and `H8_MIXED`
  similarly (6/30, 41.5%, CI [38.1, 46.1]).
- `scripts/analyze.py`'s hypothesis survival table now reports a
  single pooled verdict per hypothesis as the primary result, with the
  old per-seed "X/N survived" count kept as secondary context rather
  than the headline — the per-seed count is itself noisy for
  low-true-positive-count hypotheses (a hypothesis can show a majority
  of individual seeds crossing 0.5 by chance while the pooled evidence
  clearly says no; the reverse also occurs), and reporting it alongside
  a rigorously pooled verdict makes that visible instead of hiding it
  behind a single ambiguous fraction.

### Fixed

- `scripts/analyze.py`'s original `hypothesis_summary()` computed
  "Mean"/"Range" as an arithmetic mean and min/max of per-seed ratios.
  For hypotheses with small per-seed true-positive counts, this is
  statistically unsound: a single lucky or unlucky seed can swing the
  mean by 50+ points even when the pooled signal across all seeds is
  stable. An unusually wide naive range (50+ points) was the
  reproducible tell for exactly this failure mode across multiple
  hypotheses during validation.

## [0.5.0] - 2026-08-03

### Added

- `Estimator` enum (`Plugin`, `MillerMadow`, `QE`) and `MetricConfig`,
  threaded consistently through `calibrate_thresholds` and the main
  scoring loop in `run_cycle`, so calibration and observed-universe
  scoring always use the same estimator.
- `--estimator <plugin|mm|qe>` CLI flag. Default remains `plugin`.
- Miller-Madow bias-corrected entropy/MI estimator (`src/metrics/mm.rs`).
- Quadratic extrapolation (QE) bias-corrected MI estimator
  (`src/metrics/qe.rs`), following Strong, Koberle, de Ruyter van
  Steveninck, Bialek (1998).
- `src/metrics/` split into focused submodules: `entropy`, `mm`, `qe`,
  `persistence`, `separation`, `shuffle`, `storage`.

### Fixed

- **Miller-Madow formula.** The MI correction term used
  `(m_x - 1)(m_y - 1) / (2N ln 2)`, where `m_x`, `m_y` are the number
  of *observed* distinct values — the degrees-of-freedom term for a
  fully dense contingency table. The standard correction for mutual
  information is `(K_xy - K_x - K_y + 1) / (2N ln 2)`, where `K_xy` is
  the number of *observed* joint pairs. The two are only equal when
  every possible (x, y) combination has been observed. In the
  undersampled, large-alphabet regime this crate operates in, the old
  formula massively over-corrected — on the graph substrate
  (4096-symbol alphabet) it drove hypothesis survival to 0/10 across
  all seeds. Fixed to use the standard formula; graph-substrate MM
  survival is now 10/10, in line with plugin and QE.
- **QE subsampling independence.** The subsampling RNG seed was
  derived from the caller-supplied `seed` alone. `storage()` passes
  `config.seed + delta`, identical for every universe scored at a
  given timescale — so every distinct dataset sharing a sequence
  length received the *same* "random" subsample rather than an
  independent draw, undermining the extrapolation's statistical
  basis. Now derives the subsampling seed from a hash of the actual
  `(x_seq, y_seq)` content combined with the caller's seed, so
  distinct datasets get independent draws regardless of what seed the
  caller passes; identical inputs remain fully reproducible. Also now
  averages `N_RESAMPLES = 3` independent subsamples at each fraction
  below 1.0, rather than a single arbitrary draw. Graph-substrate
  survival under this estimator went from 9/10 to 10/10; cross-
  estimator agreement (plugin / MM / QE) tightened from within 3
  points to within 2 points across both substrates and all 10 seeds.

### Changed

- **`Estimator::Nsb` renamed to `Estimator::QE`** (`--estimator nsb`
  is now `--estimator qe`). The implementation is Quadratic
  Extrapolation (Strong et al., 1998), not the Nemenman-Shafee-Bialek
  estimator (Nemenman, Shafee, Bialek, 2002) it was previously
  labeled as and cited against. Version 0.5.0 does not implement NSB
  (a genuine implementation was added in 0.6.0). Do not cite results
  computed with the mislabeled estimator as NSB.

## Prior to 0.5.0

Not individually documented — see "Yanked versions" above.
