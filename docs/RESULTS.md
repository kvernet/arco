# ARCO Validation Results

This document records a specific, reproducible validation sweep using
`ClassificationMetrics`-based hypothesis scoring (`accuracy`,
`precision`, `recall`, `specificity`, `balanced_accuracy`), with
`hypothesis.accuracy = balanced_accuracy` as the field used for the
survival decision, and a pooled-count aggregation script in place of a
naive per-seed mean.

**Config:** `n_train=1000, n_test=300, n_ensemble=10, steps=60`, binary
graph (3 vertices, compound observer) and CA (N=8, R=1, full_state
observer) substrates, plugin estimator unless otherwise noted.

**Seeds (30):**
```
SEEDS=(0 1 7 42 69 123 256 512 999 1024 1337 2024 4096 8192 12345 54321 65535 65536 100000 123456 271828 314159 424242 999999 1000000 8675309 2147483647 2147483648 3735928559 4294967295)
```

## Methodology

`run_cycle()`'s hypothesis-testing step is scored using full
`ClassificationMetrics` (accuracy, precision, recall, specificity,
balanced_accuracy, coverage) rather than a single correct/positive
ratio, with `balanced_accuracy` as the field a hypothesis's survival is
judged on. `scripts/analyze.py` pools raw TP/FP/TN/FN across all
seed-records for a hypothesis before computing precision/recall/
balanced_accuracy, rather than averaging per-seed ratios — this matters
for hypotheses with small per-seed true-positive counts, where a
naive mean of ratios can differ from the pooled result by 50+ points
even when the pooled signal is stable. It also reports a 95% Wilson
confidence interval on the pooled balanced_accuracy, so "is this
finding real" has a statistical answer rather than a single point
estimate.

Reproduce with:
```bash
./scripts/sweep.sh
python3 scripts/analyze.py sweep_data
```

## How to read the tables

- **Verdict (pooled)** — `pooled_balanced_accuracy >= 50%`, computed
  once from counts summed across all seeds. This is the number to cite.
- **95% CI** — Wilson interval on pooled balanced_accuracy. A verdict
  can be "✅ survives" and still be flagged:
  - **❓** — the interval crosses 50%. Genuinely inconclusive at this
    seed count; don't cite this row's verdict as settled.
  - **🔶** — the interval excludes 50%, but the nearer bound is within
    2 points of it. Statistically significant, but barely — report
    these with a hedge ("weakly confirmed"), not alongside a clean
    result.
  - *(no marker)* — excludes 50% with real margin.
- **Per-seed (X/N)** — how many individual seeds crossed 0.5 on their
  own. Kept for transparency; disagrees with the pooled verdict for
  low-count hypotheses (e.g. Graph H1: 21/30 per-seed vs. a pooled
  verdict that only barely survives) and should not be read as the
  primary signal.
- **⚠** — naive per-seed mean and pooled balanced_accuracy disagree by
  >10 points, usually meaning the naive mean (and the raw per-seed
  count) shouldn't be trusted for that row. None of the results below
  are flagged — the pooled method and the naive method agree
  everywhere, which is itself a useful sanity check.

## Estimator Validation

Four independent MI estimators (plugin with shuffle correction,
Miller-Madow, quadratic extrapolation, Nemenman-Shafee-Bialek) compared
across all 30 seeds on both substrates.

| Substrate | Estimator | Storage Rate | Structured Storage | H5_TRANSPORT Acc (pooled) | Survival |
|-----------|-----------|-------------|-------------------|----------|----------|
| Graph     | plugin    | 37.1–59.2% (46.6) | 90.5–98.9% (94.9) | 61.2 | ✅ (30/30 per-seed) |
| Graph     | mm        | 37.0–59.3% (46.5) | 90.5–98.9% (94.9) | 61.1 | ✅ (30/30 per-seed) |
| Graph     | qe        | 36.1–60.7% (46.0) | 90.5–98.5% (94.5) | 61.0 | ✅ (30/30 per-seed) |
| Graph     | nsb       | 36.9–59.7% (46.0) | 90.5–98.9% (94.8) | 60.8 | ✅ (30/30 per-seed) |


| Substrate | Estimator | Storage Rate | Structured Storage | H3_LOW_SENSITIVITY Acc (pooled) | Survival |
|-----------|-----------|-------------|-------------------|----------|----------|
| Ca        | plugin    | 72.9–90.4% (83.4) | 73.8–90.7% (84.3) | 58.8 | ✅ (30/30 per-seed) |
| Ca        | mm        | 79.2–90.0% (85.1) | 80.6–90.9% (86.8) | 60.2 | ✅ (30/30 per-seed) |
| Ca        | qe        | 78.1–89.3% (84.4) | 79.0–90.9% (85.7) | 59.8 | ✅ (30/30 per-seed) |
| Ca        | nsb       | 68.5–90.1% (81.4) | 69.2–90.4% (82.5) | 57.5 | ✅ (29/30 per-seed) |

Graph estimators agree within 0.4 points on the key-hypothesis column (H5_TRANSPORT Acc);
CA within 2.7 points (H3_LOW_SENSITIVITY Acc). Both comfortably support "estimators agree" as a
qualitative claim; CA's spread is the wider of the two and worth
keeping in mind if a tighter bound is ever claimed in prose.

## Graph substrate

### Structure-Storage Gradient

| Bracket | Storage Rate Range | Mean |
|---------|-------------------|------|
| Noise (0.00–0.15) | 11.0–28.5% | 18.7% |
| Balanced (0.40–0.60) | 16.5–51.6% | 32.1% |
| Structured (0.85–1.00) | 90.5–98.9% | 94.9% |

**94.9% / 18.7% ≈ 5.1×** difference. This is ARCO's robust finding.

### Hypothesis Survival

| ID | Condition | Verdict (pooled) | 95% CI | Per-seed | Naive Range | Naive Mean | Pooled Bal.Acc | ⚠ |
|----|-----------|-------------------|--------|----------|-------------|------------|----------------|---|
| H2_MAJORITY_STRUCTURED | Majority of rules are structured | ✅ survives | [72.8–75.3] | 30/30 | 67.8–78.3% | 74.5% | 74.1% |  |
| H5_TRANSPORT | Rule set contains an information transport rule | ✅ survives | [59.9–62.5] | 30/30 | 55.9–68.6% | 61.4% | 61.2% |  |
| H3_LOGIC_GATE | Rule set contains a logic gate | ✅ survives | [57.6–60.5] | 30/30 | 54.6–63.5% | 59.2% | 59.1% |  |
| H7_MULTIPLE_LOGIC | Rule set contains at least 2 logic gates | ✅ survives | [52.9–54.8] | 30/30 | 50.9–57.2% | 53.9% | 53.9% |  |
| H4_ALL_STRUCTURED | All rules are structured | ✅ survives | [62.4–72.5] | 27/30 | 35.6–87.7% | 66.3% | 67.7% |  |
| H1_HAS_STRUCTURED | Rule set contains at least one structured rule | ✅ survives | [50.3–58.5] 🔶 | 21/30 | 14.2–64.8% | 53.6% | 55.0% |  |
| H6_ALL_DESTRUCTIVE | All rules are destructive (negative control) | ❌ fails | [41.5–49.7] 🔶 | 8/30 | 35.2–71.5% | 44.7% | 45.0% |  |
| H8_MIXED | Mixed structured and destructive rules | ❌ fails | [38.1–46.1] | 6/30 | 30.1–58.7% | 41.1% | 41.5% |  |

Note: `H6_ALL_DESTRUCTIVE` and `H8_MIXED` are both registered against
`property_name="persistence"`, not `storage` — correcting an earlier
draft that described H6 as "all-destructive → storage."

**Reading this table:** H2/H3/H5/H7 are solid, comfortably-above-chance
findings. H4 survives clearly but its point estimate isn't fully
stable yet — it moved from 77.2% to 67.7% between a 10-seed and this
30-seed run, still comfortably above 50% but a reminder that "no ⚠
flag" doesn't mean "converged," only "internally consistent." H1 and H6
are the two to hedge in any prose summary: both pass the 95%
significance bar, but only barely (0.3 points of margin on the nearer
CI bound in both cases) — real effects, not strong ones. H6 failing at
all (even if not by a wide margin) is the important result: as a
negative control, its failure is what validates that calibration is
discriminating real structure from noise, however this specific
number should not be oversold as "1/8 or 6/8 hypotheses solidly
confirm" without naming which two are borderline.

## CA substrate

### Hypothesis Survival

| ID | Condition | Verdict (pooled) | 95% CI | Per-seed | Naive Range | Naive Mean | Pooled Bal.Acc | ⚠ |
|----|-----------|-------------------|--------|----------|-------------|------------|----------------|---|
| H5_NOT_RULE_0 | Rule is not the zero rule | ✅ survives | [50.9–51.7] 🔶 | 30/30 | 50.0–53.7% | 51.3% | 51.2% |  |
| H3_LOW_SENSITIVITY | Rule has low sensitivity (< 2.0) | ✅ survives | [57.2–60.4] | 30/30 | 50.1–66.0% | 59.0% | 58.8% |  |
| H6_MID_LAMBDA | Rule has mid-range lambda (edge of chaos) | ✅ survives | [51.5–53.6] 🔶 | 22/30 | 46.6–59.3% | 52.6% | 52.5% |  |
| H4_EVEN_RULE | Rule has even Wolfram number | ✅ survives | [51.3–55.0] 🔶 | 22/30 | 46.8–62.9% | 53.4% | 53.1% |  |
| H2_PARITY | Rule conserves parity | ❌ fails | [46.7–48.8] 🔶 | 3/30 | 43.3–50.6% | 47.7% | 47.8% |  |
| H1_REVERSIBLE | Rule is reversible | ❌ fails | [45.6–47.3] | 0/30 | 42.1–49.8% | 46.4% | 46.5% |  |

**This substrate's findings are, as a set, much closer to chance than
Graph's.** Only `H3_LOW_SENSITIVITY` clears the 🔶 margin cleanly.
Four of six hypotheses (H4, H5, H6 survives; H2 fails) are flagged
borderline — real per the 95% test, but weakly so. This is worth
stating as an actual finding, not just a methodology caveat: CA rule
properties appear to be substantially weaker predictors of storage
than Graph rule composition is, at this scale. Whether that's a
property of CA specifically or an artifact of this particular
hypothesis set / rule pool (`{30, 45, 86, 106, 135, 149}`, the "known
chaotic rules" null) isn't something this sweep can distinguish.

`H4_EVEN_RULE`'s condition corresponds to Wolfram's quiescent-rule
classification (even rule number ⟺ all-0 input maps to 0). That's a
real, legitimate structural connection — but with a 🔶-flagged, 51.3%
lower CI bound, "ARCO recovered the connection to quiescence" overstates
what a barely-significant result supports; "consistent with, though not
strongly confirming" is the more accurate framing until more seeds
narrow the interval.

## Limitations

- `persistence` (used by H1, H4, H6, H8 on the graph substrate) is
  documented in `persistence.rs` as unreliable at small per-timestep
  sample sizes; the pooled/CI methodology here is a mitigation, not a
  fix to the metric itself.
- CA's overall weaker, more borderline results (above) may reflect a
  genuine substrate property or may reflect the specific hypothesis
  set / null rule pool chosen — not distinguishable from this data
  alone.
- All results are from small state spaces (3-vertex graphs, 8-cell
  automata).
