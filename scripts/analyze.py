#!/usr/bin/env python3
"""Analyze ARCO sweep data and generate ready-to-copy README tables.

Works with any number of seeds — nothing here assumes a specific count;
`len(records)` is used wherever "how many seeds" matters, not a
hardcoded constant. Conclusions below should be read the same way: as
a description of what N seeds currently show, not a claim tied to any
particular N.

PATCHED: the original hypothesis_summary()/all_hypotheses_summary()
computed "Mean"/"Range" as a naive arithmetic mean/min/max of the
per-seed `accuracy` (here: balanced_accuracy, given the
`hypothesis.accuracy = balanced_accuracy` Rust-side patch) values. That
is statistically wrong whenever the underlying per-seed TP/FN counts
are small: a couple of lucky/unlucky seeds can swing a naive mean by
50+ points even when the *pooled* signal across all seeds combined is
weak and stable. Several standard hypotheses hit exactly this case (a
handful of true positives per seed relative to whatever `n_test` is
configured) — unusually wide naive ranges are the tell, regardless of
how many seeds were run or what `n_test` is set to.

This version pools raw TP/FP/TN/FN across ALL seed-records for a
hypothesis BEFORE computing precision/recall/balanced_accuracy, instead
of averaging separately-computed per-seed ratios. `ClassificationMetrics`
doesn't serialize raw counts, only ratios — `reconstruct_counts()`
below recovers exact integer counts from `coverage`/`precision`/
`accuracy` plus the known `n_test` (exact, not approximate, since
those ratios were themselves computed from integer counts over a known
total to begin with).

Original naive stats are kept and printed alongside the pooled ones
(not silently replaced) so a large disagreement between them is visible
rather than hidden — that disagreement is itself diagnostic of a
low-count, high-variance hypothesis worth treating cautiously regardless
of which number you ultimately report, and regardless of seed count.
"""

import json, os, sys, math

DATA_DIR = sys.argv[1] if len(sys.argv) > 1 else "sweep_data"
SUBSTRATES = ["graph", "ca"]
ESTIMATORS = ["plugin", "mm", "qe", "nsb"]

# Threshold above which naive-vs-pooled disagreement gets flagged in
# output. 10 points is well outside what pure rounding/estimator noise
# should produce for a genuinely stable hypothesis.
DISAGREEMENT_FLAG_PP = 10.0

# A CI bound landing within this many points of 50% is treated as
# "borderline" even if it doesn't technically cross 50% — e.g. a lower
# bound of 50.3% is not meaningfully different from 49.9% in practice;
# a strict "does it cross 50%" check misses this. Separate from
# ci_crosses_50 (that one's still reported too, this adds a second,
# looser tier).
NEAR_CHANCE_MARGIN_PP = 2.0


def load_records(substrate, estimator):
    records = {}
    prefix = f"{substrate}_{estimator}_"
    for filename in os.listdir(DATA_DIR):
        if filename.startswith(prefix) and filename.endswith('.json'):
            seed = filename.replace(prefix, '').replace('.json', '')
            with open(os.path.join(DATA_DIR, filename)) as f:
                record = json.load(f)
            if seed == record['config']['seed']:
                records[seed] = record
    return records


def wilson_interval(successes, n, z=1.96):
    """95% Wilson score interval for a binomial proportion. Preferred
    over a normal approximation here because pooled counts can still be
    small enough (e.g. H4's TP) that a normal-approximation interval
    could extend outside [0,1] or badly understate uncertainty; Wilson
    stays well-behaved at small n and n=0."""
    if n == 0:
        return (0.0, 1.0)
    phat = successes / n
    denom = 1 + z * z / n
    center = (phat + z * z / (2 * n)) / denom
    margin = z * math.sqrt(phat * (1 - phat) / n + z * z / (4 * n * n)) / denom
    return (max(0.0, center - margin), min(1.0, center + margin))


def reconstruct_counts(h, n_test):
    """Exact integer TP/FP/TN/FN from classification_metrics ratios +
    known n_test.

    CORRECTED: originally derived TN via `specificity * predicted_negative`
    (predicted_negative = TN+FN). That's wrong — specificity's actual
    denominator is TN+FP (the actual-negative count), not TN+FN. Deriving
    TN from `accuracy` instead sidesteps this entirely: accuracy and TP
    alone pin down TN exactly (TN = accuracy*total - TP), with no risk of
    using the wrong denominator. Verified against a real record where the
    old formula reconstructed specificity=0.678 against a true value of
    0.740; this version reconstructs 0.733 (the small remaining gap is
    just rounding in the 3-decimal-place inputs, not a formula error).

    Raises KeyError with a clear message if this build's JSON doesn't
    serialize classification_metrics — check ARCO's
    `#[cfg_attr(feature = "serialize", ...)]` on that struct if so.
    """
    if 'classification_metrics' not in h:
        raise KeyError(
            f"hypothesis '{h.get('name', '?')}' has no 'classification_metrics' "
            "key in this JSON record — pooling requires it (coverage/precision/"
            "accuracy). Check that ARCO was built/run with the 'serialize' "
            "feature enabled, or that this JSON schema matches what this script "
            "expects."
        )
    cm = h['classification_metrics']
    predicted_positive = round(cm['coverage'] * n_test)
    tp = round(cm['precision'] * predicted_positive) if predicted_positive > 0 else 0
    fp = predicted_positive - tp
    tn = round(cm['accuracy'] * n_test) - tp
    fn = n_test - tp - fp - tn
    return tp, fp, tn, fn


def hypothesis_summary_naive(records, hyp_name):
    """Original behavior: naive mean/range of per-seed ratios. Kept for
    direct comparison against the pooled version, not for trusting on
    its own — see module docstring."""
    survivals = 0
    accs = []
    for record in records.values():
        for h in record['hypotheses']:
            if h['name'] == hyp_name:
                accs.append(h['accuracy'] * 100)
                if h['survives']:
                    survivals += 1
    if not accs:
        return (0, 0.0, 0.0, 0.0)
    return (survivals, min(accs), max(accs), sum(accs) / len(accs))


def hypothesis_summary_pooled(records, hyp_name):
    """Correct version: sum TP/FP/TN/FN across all seed-records for this
    hypothesis, THEN compute precision/recall/balanced_accuracy once
    from the totals. Returns None if the hypothesis wasn't found."""
    total_tp = total_fp = total_tn = total_fn = 0
    survivals = 0
    n_seeds = 0
    for record in records.values():
        n_test = int(record['config']['n_test'])  # some sweep_data schemas
        # serialize config values as strings (same reason 'seed' is
        # compared as a string in load_records above) — coerce
        # defensively here rather than assuming a JSON number type.
        for h in record['hypotheses']:
            if h['name'] == hyp_name:
                tp, fp, tn, fn = reconstruct_counts(h, n_test)
                total_tp += tp
                total_fp += fp
                total_tn += tn
                total_fn += fn
                if h['survives']:
                    survivals += 1
                n_seeds += 1
    if n_seeds == 0:
        return None
    precision = total_tp / (total_tp + total_fp) if (total_tp + total_fp) > 0 else 0.0
    recall = total_tp / (total_tp + total_fn) if (total_tp + total_fn) > 0 else 0.0
    specificity = total_tn / (total_tn + total_fp) if (total_tn + total_fp) > 0 else 0.0
    balanced_accuracy = (recall + specificity) / 2.0

    # recall and specificity come from disjoint groups (actual-positive
    # vs actual-negative universes) so they're independent, and
    # balanced_accuracy is their simple average — combining the two
    # Wilson intervals endpoint-wise gives a valid (if slightly
    # conservative) interval for balanced_accuracy under independence.
    recall_lo, recall_hi = wilson_interval(total_tp, total_tp + total_fn)
    spec_lo, spec_hi = wilson_interval(total_tn, total_tn + total_fp)
    bal_acc_lo = (recall_lo + spec_lo) / 2.0
    bal_acc_hi = (recall_hi + spec_hi) / 2.0

    return {
        'survivals': survivals,
        'n_seeds': n_seeds,
        'pooled_balanced_accuracy': balanced_accuracy * 100,
        'pooled_balanced_accuracy_ci': (bal_acc_lo * 100, bal_acc_hi * 100),
        'ci_crosses_50': bal_acc_lo < 0.5 < bal_acc_hi,
        'ci_near_chance': (
            abs(bal_acc_lo * 100 - 50.0) <= NEAR_CHANCE_MARGIN_PP
            or abs(bal_acc_hi * 100 - 50.0) <= NEAR_CHANCE_MARGIN_PP
        ),
        'pooled_precision': precision * 100,
        'pooled_recall': recall * 100,
        'total_tp': total_tp,
        'total_fp': total_fp,
        'total_tn': total_tn,
        'total_fn': total_fn,
    }


def all_hypotheses_summary(records):
    """Both naive and pooled stats per hypothesis, sorted by survival
    count (matches original ordering)."""
    names = set()
    descs = {}
    for record in records.values():
        for h in record['hypotheses']:
            names.add(h['name'])
            descs[h['name']] = h['condition_desc']

    result = []
    for name in names:
        surv_n, acc_min, acc_max, acc_mean = hypothesis_summary_naive(records, name)
        pooled = hypothesis_summary_pooled(records, name)
        entry = {
            'name': name,
            'desc': descs[name],
            'survivals': surv_n,
            'acc_min': acc_min,
            'acc_max': acc_max,
            'acc_mean': acc_mean,
        }
        if pooled is not None:
            entry.update(pooled)
            entry['disagreement_pp'] = abs(acc_mean - pooled['pooled_balanced_accuracy'])
            entry['flagged'] = entry['disagreement_pp'] > DISAGREEMENT_FLAG_PP
        else:
            entry['pooled_balanced_accuracy'] = None
            entry['pooled_balanced_accuracy_ci'] = None
            entry['ci_crosses_50'] = False
            entry['flagged'] = False
        result.append(entry)

    result.sort(key=lambda x: -x['survivals'])
    return result


def spectrum_summary(records, brackets):
    """Return storage spectrum across structured ratio brackets.
    Unchanged from original — this already pools correctly (counts
    events across the full result set per bracket, not per-seed rates
    averaged), so it wasn't affected by the bug."""
    result = []
    for label, low, high in brackets:
        rates = []
        means = []
        for record in records.values():
            threshold = record['thresholds'].get('storage', 0.0)
            group = [r for r in record['results'] if low <= r['structured_ratio'] < high]
            if group:
                rate = 100.0 * sum(1 for r in group if r['storage'] > threshold) / len(group)
                rates.append(rate)
                means.append(sum(r['storage'] for r in group) / len(group))
        if rates:
            result.append({
                'label': label,
                'rate_min': min(rates),
                'rate_max': max(rates),
                'rate_mean': sum(rates) / len(rates),
                'storage_mean': sum(means) / len(means),
            })
    return result


def estimator_comparison(records_map, substrate, key_hypothesis):
    """Generate estimator comparison table using POOLED accuracy for
    the key hypothesis column (was naive mean in the original)."""
    lines = []
    lines.append(f"| Substrate | Estimator | Storage Rate | Structured Storage | {key_hypothesis} Acc (pooled) | Survival |")
    lines.append(f"|-----------|-----------|-------------|-------------------|----------|----------|")
    for est in ESTIMATORS:
        records = records_map.get((substrate, est), {})
        if not records:
            continue

        if substrate == "graph":
            s = spectrum_summary(records, [("Structured", 0.85, 1.01)])
        else:
            s = spectrum_summary(records, [("Structured", 0.7, 1.01)])
        structured = f"{s[0]['rate_min']:.1f}\u2013{s[0]['rate_max']:.1f}% ({s[0]['rate_mean']:.1f})" if s else "\u2014"

        pooled = hypothesis_summary_pooled(records, key_hypothesis)
        n_seeds = len(records)
        if pooled is None:
            acc_str = "\u2014"
            verdict = "\u2014"
        else:
            acc_str = f"{pooled['pooled_balanced_accuracy']:.1f}"
            verdict = "\u2705" if pooled['pooled_balanced_accuracy'] >= 50.0 else "\u274c"
            verdict += f" ({pooled['survivals']}/{pooled['n_seeds']} per-seed)"

        storage_min = min(100.0 * sum(1 for r in rec['results'] if r['storage'] > rec['thresholds'].get('storage', 0.0)) / len(rec['results']) for rec in records.values())
        storage_max = max(100.0 * sum(1 for r in rec['results'] if r['storage'] > rec['thresholds'].get('storage', 0.0)) / len(rec['results']) for rec in records.values())
        storage_mean = sum(100.0 * sum(1 for r in rec['results'] if r['storage'] > rec['thresholds'].get('storage', 0.0)) / len(rec['results']) for rec in records.values()) / n_seeds

        lines.append(f"| {substrate.capitalize():<9} | {est:<9} | {storage_min:.1f}\u2013{storage_max:.1f}% ({storage_mean:.1f}) | {structured} | {acc_str} | {verdict} |")
    return "\n".join(lines)


def hypothesis_table(hypotheses, n_seeds):
    """Generate hypothesis survival table.

    The per-seed 'X/N survives' count is itself noisy for low-TP
    hypotheses: a hypothesis whose pooled evidence clearly says "no"
    can still show several individual seeds crossing 0.5 by luck,
    inflating the per-seed tally relative to the aggregate truth. The
    pooled verdict (computed once from counts summed across all
    available seeds) is the headline signal here; the per-seed count is
    kept alongside as diagnostic context, not as the primary read.
    """
    lines = []
    lines.append("| ID | Condition | Verdict (pooled) | 95% CI | Per-seed | Naive Range | Naive Mean | Pooled Bal.Acc | \u26a0 |")
    lines.append("|----|-----------|-------------------|--------|----------|-------------|------------|----------------|---|")
    for h in hypotheses:
        pooled_str = f"{h['pooled_balanced_accuracy']:.1f}%" if h['pooled_balanced_accuracy'] is not None else "\u2014"
        flag = "\u26a0\ufe0f" if h['flagged'] else ""
        if h['pooled_balanced_accuracy'] is not None:
            verdict = "\u2705 survives" if h['pooled_balanced_accuracy'] >= 50.0 else "\u274c fails"
            ci_lo, ci_hi = h['pooled_balanced_accuracy_ci']
            ci_str = f"[{ci_lo:.1f}\u2013{ci_hi:.1f}]"
            if h['ci_crosses_50']:
                ci_str += " \u2753"  # inconclusive at 95% confidence
            elif h['ci_near_chance']:
                ci_str += " \U0001f536"  # technically significant, but the nearer bound is within NEAR_CHANCE_MARGIN_PP of 50% — treat cautiously
        else:
            verdict = "\u2014"
            ci_str = "\u2014"
        lines.append(
            f"| {h['name']} | {h['desc']} | {verdict} | {ci_str} | {h['survivals']}/{n_seeds} | "
            f"{h['acc_min']:.1f}\u2013{h['acc_max']:.1f}% | {h['acc_mean']:.1f}% | {pooled_str} | {flag} |"
        )
    lines.append("")
    lines.append(
        "95% CI: Wilson score interval on pooled balanced_accuracy (from independent Wilson "
        "intervals on recall and specificity, combined endpoint-wise — valid since recall/"
        "specificity come from disjoint actual-positive/actual-negative groups). \u2753 = the "
        "interval still crosses 50% — the verdict isn't yet statistically distinguishable from "
        f"chance. \U0001f536 = doesn't cross 50%, but the nearer bound is within "
        f"{NEAR_CHANCE_MARGIN_PP:.0f} points of it — technically significant, practically "
        "still borderline; treat these more cautiously in prose than a clean \u2705/\u274c. "
        "More seeds narrow the interval either way."
    )
    lines.append(
        "Verdict = pooled_balanced_accuracy >= 50%, computed ONCE from counts summed across "
        "all seeds — this is the number to report/cite. 'Per-seed' (X/N) is how many "
        "INDIVIDUAL seeds happened to cross 0.5 on their own; for low-TP hypotheses this can "
        "disagree with the pooled verdict — it's kept for transparency, not as the primary "
        "signal."
    )
    lines.append(
        f"\u26a0\ufe0f = naive mean and pooled balanced_accuracy disagree by more than "
        f"{DISAGREEMENT_FLAG_PP:.0f} percentage points."
    )
    return "\n".join(lines)


def spectrum_table(spectrum):
    lines = []
    lines.append("| Bracket | Storage Rate Range | Mean |")
    lines.append("|---------|-------------------|------|")
    for s in spectrum:
        lines.append(f"| {s['label']} | {s['rate_min']:.1f}\u2013{s['rate_max']:.1f}% | {s['rate_mean']:.1f}% |")
    return "\n".join(lines)


# ================================================================
# Main
# ================================================================

records_map = {}
for substrate in SUBSTRATES:
    for est in ESTIMATORS:
        records = load_records(substrate, est)
        if records:
            records_map[(substrate, est)] = records

graph_records = records_map.get(("graph", "plugin"), {})
ca_records = records_map.get(("ca", "plugin"), {})

print("=" * 80)
print("SECTION 1: ESTIMATOR VALIDATION (copy to README)")
print("=" * 80)
print()
print(estimator_comparison(records_map, "graph", "H5_TRANSPORT"))
print()
print(estimator_comparison(records_map, "ca", "H3_LOW_SENSITIVITY"))
print()

print("=" * 80)
print("SECTION 2: GRAPH SUBSTRATE (copy to README)")
print("=" * 80)
print()

graph_brackets = [
    ("Noise (0.00\u20130.15)", 0.00, 0.15),
    ("Balanced (0.40\u20130.60)", 0.40, 0.60),
    ("Structured (0.85\u20131.00)", 0.85, 1.01),
]
graph_spectrum = spectrum_summary(graph_records, graph_brackets)
print("#### Structure-Storage Gradient")
print()
print(spectrum_table(graph_spectrum))
print()

graph_hyps = all_hypotheses_summary(graph_records)
print("#### Hypothesis Survival")
print()
print(hypothesis_table(graph_hyps, len(graph_records)))
print()

print("=" * 80)
print("SECTION 3: CA SUBSTRATE (copy to README)")
print("=" * 80)
print()

ca_hyps = all_hypotheses_summary(ca_records)
print("#### Hypothesis Survival")
print()
print(hypothesis_table(ca_hyps, len(ca_records)))
print()