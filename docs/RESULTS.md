## Key Findings

All results from 10 independent seeds. Reproduce with:

```bash
./scripts/sweep.sh
python3 scripts/analyze.py sweep_data
```

### Estimator Validation

Three mutual information estimators were compared across 10 seeds on both substrates:

| Substrate | Estimator | Storage Rate | Structured Storage | H5_TRANSPORT Acc | Survival |
|-----------|-----------|-------------|-------------------|----------|----------|
| Graph     | plugin    | 37.1–70.7% (48.2) | 90.5–99.6% (93.9) | 50.5–85.2% (65.4) | 10/10 |
| Graph     | mm        | 37.4–70.6% (48.2) | 90.5–99.6% (94.0) | 52.6–85.2% (65.8) | 10/10 |
| Graph     | qe        | 37.8–71.3% (48.2) | 90.5–99.6% (93.7) | 53.6–85.2% (65.6) | 10/10 |

| Substrate | Estimator | Storage Rate | Structured Storage | H3_LOW_SENSITIVITY Acc | Survival |
|-----------|-----------|-------------|-------------------|----------|----------|
| Ca        | plugin    | 73.7–90.4% (82.7) | 75.5–90.7% (83.7) | 79.7–92.6% (85.4) | 10/10 |
| Ca        | mm        | 78.1–90.0% (84.4) | 80.5–90.9% (86.1) | 82.7–91.8% (87.0) | 10/10 |
| Ca        | qe        | 77.6–89.3% (83.8) | 78.6–90.9% (85.0) | 82.3–91.8% (86.5) | 10/10 |

Three independent mutual information estimators (plugin with shuffle
correction, Miller-Madow, and quadratic extrapolation) agree within
2 points across both substrates and all 10 seeds. All results below
use the plugin estimator with shuffle correction (the default).

#### Structure-Storage Gradient

| Bracket | Storage Rate Range | Mean |
|---------|-------------------|------|
| Noise (0.00–0.15) | 11.0–41.6% | 21.1% |
| Balanced (0.40–0.60) | 20.8–71.3% | 35.2% |
| Structured (0.85–1.00) | 90.5–99.6% | **93.9%** |

A 4.5× difference. This is ARCO's most robust finding.

#### Hypothesis Survival

| ID | Condition | Survival | Acc. Range | Mean |
|----|-----------|----------|-----------|------|
| H2_MAJORITY_STRUCTURED | Majority of rules are structured | 10/10 | 54.8–86.6% | 68.2% |
| H5_TRANSPORT | Rule set contains an information transport rule | 10/10 | 50.5–85.2% | 65.4% |
| H7_MULTIPLE_LOGIC | Rule set contains at least 2 logic gates | 8/10 | 47.2–79.5% | 63.3% |
| H3_LOGIC_GATE | Rule set contains a logic gate | 7/10 | 38.6–80.5% | 57.8% |
| H1_HAS_STRUCTURED | Rule set contains at least one structured rule | 0/10 | 0.0–2.3% | 1.0% |
| H4_ALL_STRUCTURED | All rules are structured | 0/10 | 0.0–6.2% | 2.3% |
| H6_ALL_DESTRUCTIVE | All rules are destructive (negative control) | 0/10 | 0.0–4.6% | 1.1% |
| H8_MIXED | Mixed structured and destructive rules | 0/10 | 0.0–2.1% | 0.5% |

H2 and H5 (Transport Law) are the most reliable. H1, H4, H6, H8 did not survive (0/10). H6 is a negative control (all-destructive → storage) — its consistent failure validates calibration.

### CA Substrate — n=1,000

Null distribution sampled from a pool of known chaotic rules (30, 45, 86, 106, 135, 149). All hypotheses use measurable properties only. Rule sets are sampled independently for train and test from the full Wolfram pool.

#### Hypothesis Survival

| ID | Condition | Survival | Acc. Range | Mean |
|----|-----------|----------|-----------|------|
| H2_PARITY | Rule conserves parity | 10/10 | 57.1–89.5% | 74.9% |
| H3_LOW_SENSITIVITY | Rule has low sensitivity (< 2.0) | 10/10 | 79.7–92.6% | 85.4% |
| H4_EVEN_RULE | Rule has even Wolfram number | 10/10 | 77.7–89.0% | 83.8% |
| H5_NOT_RULE_0 | Rule is not the zero rule | 10/10 | 76.8–90.3% | 82.5% |
| H6_MID_LAMBDA | Rule has mid-range lambda (edge of chaos) | 10/10 | 77.9–91.1% | 83.2% |
| H1_REVERSIBLE | Rule is reversible | 7/10 | 33.3–81.8% | 55.6% |

H4 (even rule number) corresponds to quiescent rules in Wolfram's classification — ARCO recovered this connection without being told about quiescence.

### Estimators

ARCO supports three mutual information estimators, selectable via `--estimator`:

| Estimator | Flag | Best for |
|-----------|------|----------|
| Plugin + shuffle | `plugin` (default) | General use |
| QE | `qe` | Large alphabets, publication-quality |
| Miller-Madow | `mm` | Small alphabets, fast bias correction |

The plugin estimator with shuffle correction is the default and has been validated against QE and MM on both substrates (see Estimator Validation above).

### Limitations

- Quadratic extrapolation (QE) is an approximation to the full NSB
  estimator.
- The Binary Graph Universe is a **validation substrate** — rules are
  hand-coded to calibrate the instrument. Discovery substrates are
  the next milestone.
- All findings are from small state spaces (3-vertex graphs, 8-cell
  automata).

## Reproducibility

```bash
./scripts/sweep.sh                    # Run 10-seed sweep, save JSON
python3 scripts/analyze.py sweep_data # Analyze and generate plots
```

Every number in this README is traceable to a specific seed produced by `scripts/sweep.sh`.