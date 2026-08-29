## [0.6.0] - 2026-08-29

### Added

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
