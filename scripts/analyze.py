#!/usr/bin/env python3
"""Analyze ARCO sweep data and generate ready-to-copy README tables."""

import json, os, sys
from collections import defaultdict

DATA_DIR = sys.argv[1] if len(sys.argv) > 1 else "sweep_data"

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

def hypothesis_summary(records, hyp_name):
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
    return (survivals, min(accs), max(accs), sum(accs)/len(accs))

def all_hypotheses_summary(records):
    """Return all hypotheses with survival, acc range, mean."""
    data = defaultdict(lambda: {'survivals': 0, 'accs': []})
    for record in records.values():
        for h in record['hypotheses']:
            data[h['name']]['accs'].append(h['accuracy'] * 100)
            data[h['name']]['desc'] = h['condition_desc']
            if h['survives']:
                data[h['name']]['survivals'] += 1
    
    result = []
    for name, d in data.items():
        accs = d['accs']
        result.append({
            'name': name,
            'desc': d['desc'],
            'survivals': d['survivals'],
            'acc_min': min(accs),
            'acc_max': max(accs),
            'acc_mean': sum(accs)/len(accs),
        })
    result.sort(key=lambda x: -x['survivals'])
    return result

def spectrum_summary(records, brackets):
    """Return storage spectrum across structured ratio brackets."""
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
                'rate_mean': sum(rates)/len(rates),
                'storage_mean': sum(means)/len(means),
            })
    return result

def estimator_comparison(records_map, substrate, key_hypothesis):
    """Generate estimator comparison table."""
    estimators = ["plugin", "mm", "qe"]
    lines = []
    lines.append(f"| Substrate | Estimator | Storage Rate | Structured Storage | {key_hypothesis} Acc | Survival |")
    lines.append(f"|-----------|-----------|-------------|-------------------|----------|----------|")
    
    for est in estimators:
        records = records_map.get((substrate, est), {})
        if not records:
            continue
        
        if substrate == "graph":
            s = spectrum_summary(records, [("Structured", 0.85, 1.01)])
            structured = f"{s[0]['rate_min']:.1f}–{s[0]['rate_max']:.1f}% ({s[0]['rate_mean']:.1f})" if s else "—"
        else:
            s = spectrum_summary(records, [("Structured", 0.7, 1.01)])
            structured = f"{s[0]['rate_min']:.1f}–{s[0]['rate_max']:.1f}% ({s[0]['rate_mean']:.1f})" if s else "—"
        
        surv, acc_min, acc_max, acc_mean = hypothesis_summary(records, key_hypothesis)
        n_seeds = len(records)
        storage_min = min(100.0 * sum(1 for r in rec['results'] if r['storage'] > rec['thresholds'].get('storage', 0.0)) / len(rec['results']) for rec in records.values())
        storage_max = max(100.0 * sum(1 for r in rec['results'] if r['storage'] > rec['thresholds'].get('storage', 0.0)) / len(rec['results']) for rec in records.values())
        storage_mean = sum(100.0 * sum(1 for r in rec['results'] if r['storage'] > rec['thresholds'].get('storage', 0.0)) / len(rec['results']) for rec in records.values()) / n_seeds
        
        lines.append(f"| {substrate.capitalize():<9} | {est:<9} | {storage_min:.1f}–{storage_max:.1f}% ({storage_mean:.1f}) | {structured} | {acc_min:.1f}–{acc_max:.1f}% ({acc_mean:.1f}) | {surv}/{n_seeds} |")
    
    return "\n".join(lines)

def hypothesis_table(hypotheses, n_seeds):
    """Generate hypothesis survival table."""
    lines = []
    lines.append(f"| ID | Condition | Survival | Acc. Range | Mean |")
    lines.append(f"|----|-----------|----------|-----------|------|")
    for h in hypotheses:
        lines.append(f"| {h['name']} | {h['desc']} | {h['survivals']}/{n_seeds} | {h['acc_min']:.1f}–{h['acc_max']:.1f}% | {h['acc_mean']:.1f}% |")
    return "\n".join(lines)

def spectrum_table(spectrum):
    """Generate spectrum table."""
    lines = []
    lines.append(f"| Bracket | Storage Rate Range | Mean |")
    lines.append(f"|---------|-------------------|------|")
    for s in spectrum:
        lines.append(f"| {s['label']} | {s['rate_min']:.1f}–{s['rate_max']:.1f}% | {s['rate_mean']:.1f}% |")
    return "\n".join(lines)

# ================================================================
# Main
# ================================================================

N_SEEDS = 10

# Load all records
records_map = {}
for substrate in ["graph", "ca"]:
    for est in ["plugin", "mm", "qe"]:
        records = load_records(substrate, est)
        if records:
            records_map[(substrate, est)] = records

# Use plugin estimator for detailed results
graph_records = records_map.get(("graph", "plugin"), {})
ca_records = records_map.get(("ca", "plugin"), {})

# ================================================================
# SECTION 1: Estimator Validation
# ================================================================
print("=" * 80)
print("SECTION 1: ESTIMATOR VALIDATION (copy to README)")
print("=" * 80)
print()
print(estimator_comparison(records_map, "graph", "H5_TRANSPORT"))
print()
print(estimator_comparison(records_map, "ca", "H3_LOW_SENSITIVITY"))
print()

# ================================================================
# SECTION 2: Graph Substrate Details
# ================================================================
print("=" * 80)
print("SECTION 2: GRAPH SUBSTRATE (copy to README)")
print("=" * 80)
print()

# Spectrum
graph_brackets = [
    ("Noise (0.00–0.15)", 0.00, 0.15),
    ("Balanced (0.40–0.60)", 0.40, 0.60),
    ("Structured (0.85–1.00)", 0.85, 1.01),
]
graph_spectrum = spectrum_summary(graph_records, graph_brackets)
print("#### Structure-Storage Gradient")
print()
print(spectrum_table(graph_spectrum))
print()

# Hypotheses
graph_hyps = all_hypotheses_summary(graph_records)
print("#### Hypothesis Survival")
print()
print(hypothesis_table(graph_hyps, N_SEEDS))
print()

# ================================================================
# SECTION 3: CA Substrate Details
# ================================================================
print("=" * 80)
print("SECTION 3: CA SUBSTRATE (copy to README)")
print("=" * 80)
print()

ca_hyps = all_hypotheses_summary(ca_records)
print("#### Hypothesis Survival")
print()
print(hypothesis_table(ca_hyps, N_SEEDS))
print()