#!/usr/bin/env python3
"""scripts/analyze.py — Analyze ARCO sweep data and generate plots.

Usage:
    ./scripts/sweep.sh                    # Run experiments and save JSON
    python3 scripts/analyze.py sweep_data # Analyze and plot

Requires: matplotlib, numpy
"""

import json
import os
import sys
from collections import defaultdict
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker

DATA_DIR = sys.argv[1] if len(sys.argv) > 1 else "sweep_data"

# ===================================================================
# Load data
# ===================================================================

def load_records(substrate):
    """Load all JSON records for a substrate."""
    records = {}
    for filename in os.listdir(DATA_DIR):
        if not filename.endswith('.json'):
            continue
        parts = filename.replace('.json', '').split('_')
        if parts[0] != substrate:
            continue
        seed = parts[1]
        with open(os.path.join(DATA_DIR, filename)) as f:
            records[seed] = json.load(f)
    return records

graph = load_records("graph")
ca = load_records("ca")

# ===================================================================
# Extract data
# ===================================================================

def extract_hypotheses(records):
    """Extract hypothesis accuracies across seeds."""
    data = defaultdict(list)
    survival = defaultdict(int)
    for seed, record in records.items():
        for h in record['hypotheses']:
            data[h['name']].append(h['accuracy'] * 100)
            if h['survives']:
                survival[h['name']] += 1
    return data, survival

def extract_spectrum(records):
    """Extract storage rates by structured ratio bracket."""
    brackets = {
        'Noise': (0.00, 0.15),
        'Noise-dominated': (0.15, 0.40),
        'Balanced': (0.40, 0.60),
        'Structure-dominated': (0.60, 0.85),
        'Structured': (0.85, 1.01),
    }
    data = defaultdict(list)
    for seed, record in records.items():
        threshold = record['thresholds'].get('storage', 0.0)
        for bracket, (low, high) in brackets.items():
            group = [r for r in record['results']
                     if low <= r['structured_ratio'] < high]
            if group:
                rate = 100.0 * sum(1 for r in group if r['storage'] > threshold) / len(group)
                data[bracket].append(rate)
    return data

def extract_storage_distribution(records):
    """Extract raw storage values by bracket for distribution plots."""
    brackets = {
        'Noise': (0.00, 0.15),
        'Noise-dominated': (0.15, 0.40),
        'Balanced': (0.40, 0.60),
        'Structure-dominated': (0.60, 0.85),
        'Structured': (0.85, 1.01),
    }
    data = defaultdict(list)
    for seed, record in records.items():
        for bracket, (low, high) in brackets.items():
            for r in record['results']:
                if low <= r['structured_ratio'] < high:
                    data[bracket].append(r['storage'])
    return data

# ===================================================================
# Plot 1: Structure-Storage Gradient
# ===================================================================

def plot_spectrum(graph_spectrum, ca_spectrum, title, filename):
    fig, ax = plt.subplots(figsize=(10, 5))
    
    brackets = ['Noise', 'Noise-dominated', 'Balanced', 'Structure-dominated', 'Structured']
    x = np.arange(len(brackets))
    width = 0.35
    
    # Graph data
    means = [np.mean(graph_spectrum.get(b, [0])) for b in brackets]
    mins = [np.min(graph_spectrum.get(b, [0])) for b in brackets]
    maxs = [np.max(graph_spectrum.get(b, [0])) for b in brackets]
    errors_low = [m - lo for m, lo in zip(means, mins)]
    errors_high = [hi - m for m, hi in zip(means, maxs)]
    
    bars1 = ax.bar(x - width/2, means, width, 
                   yerr=[errors_low, errors_high],
                   label='Binary Graph', color='#58a6ff', capsize=3)
    
    # CA doesn't have the same brackets — skip for now or use simplified brackets
    # For CA, we have "Low structure" and "High structure"
    
    ax.set_ylabel('Storage Rate (%)')
    ax.set_title(title, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels(brackets, rotation=15)
    ax.legend()
    ax.set_ylim(0, 110)
    ax.yaxis.set_major_formatter(mticker.FormatStrFormatter('%.0f%%'))
    
    plt.tight_layout()
    plt.savefig(filename, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"Saved {filename}")

# ===================================================================
# Plot 2: Hypothesis Survival
# ===================================================================

def plot_hypotheses(graph_data, graph_survival, ca_data, ca_survival, filename):
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))
    
    for ax, (data, survival), title in [
        (ax1, (graph_data, graph_survival), 'Binary Graph'),
        (ax2, (ca_data, ca_survival), 'Cellular Automata')
    ]:
        names = sorted(data.keys())
        means = [np.mean(data[n]) for n in names]
        mins = [np.min(data[n]) for n in names]
        maxs = [np.max(data[n]) for n in names]
        errors_low = [m - lo for m, lo in zip(means, mins)]
        errors_high = [hi - m for m, hi in zip(means, maxs)]
        
        colors = []
        for n in names:
            s = survival.get(n, 0)
            if s >= 8:
                colors.append('#3fb950')  # green
            elif s >= 5:
                colors.append('#d29922')  # yellow
            else:
                colors.append('#f85149')  # red
        
        x = np.arange(len(names))
        bars = ax.bar(x, means, color=colors, capsize=3)
        ax.errorbar(x, means, yerr=[errors_low, errors_high], fmt='none', 
                    ecolor='#8b949e', capsize=3)
        
        # Survival count labels
        for i, n in enumerate(names):
            ax.text(i, means[i] + errors_high[i] + 1, 
                   f'{survival.get(n, 0)}/10',
                   ha='center', fontsize=7, color='#8b949e')
        
        ax.set_title(title, fontweight='bold')
        ax.set_ylabel('Accuracy (%)')
        ax.set_xticks(x)
        ax.set_xticklabels([n.replace('_', '\n') for n in names], 
                          rotation=0, fontsize=8)
        ax.set_ylim(0, 110)
        ax.axhline(y=50, color='#8b949e', linestyle='--', alpha=0.5)
        ax.yaxis.set_major_formatter(mticker.FormatStrFormatter('%.0f%%'))
    
    plt.suptitle('Hypothesis Survival Across 10 Seeds', fontweight='bold')
    plt.tight_layout()
    plt.savefig(filename, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"Saved {filename}")

# ===================================================================
# Plot 3: Storage Distribution
# ===================================================================

def plot_distribution(graph_dist, filename):
    fig, ax = plt.subplots(figsize=(10, 5))
    
    brackets = ['Noise', 'Noise-dominated', 'Balanced', 'Structure-dominated', 'Structured']
    data = [graph_dist.get(b, []) for b in brackets]
    
    parts = ax.violinplot(data, positions=np.arange(len(brackets)), showmeans=True, showmedians=True)
    
    for pc in parts['bodies']:
        pc.set_facecolor('#58a6ff')
        pc.set_alpha(0.7)
    
    ax.set_ylabel('Storage Value')
    ax.set_title('Storage Distribution by Structured Ratio (Binary Graph)', fontweight='bold')
    ax.set_xticks(np.arange(len(brackets)))
    ax.set_xticklabels(brackets, rotation=15)
    ax.set_ylim(0, 1.1)
    
    plt.tight_layout()
    plt.savefig(filename, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"Saved {filename}")


# ===================================================================
# Print spectrum
# ===================================================================

def print_spectrum_table(spectrum, substrate_name):
    """Print the full Structure-Storage Gradient table."""
    brackets = ['Noise', 'Noise-dominated', 'Balanced', 'Structure-dominated', 'Structured']
    
    print(f"\n{substrate_name} — Structure-Storage Gradient:")
    print(f"{'Bracket':<22} {'Storage Rate Range':<22} {'Mean':<8}")
    print("-" * 52)
    for b in brackets:
        if b in spectrum and spectrum[b]:
            vals = spectrum[b]
            print(f"{b:<22} {min(vals):.1f}--{max(vals):.1f}%{'':<8} {np.mean(vals):.1f}%")


def print_hypothesis(graph_hyp, graph_surv, ca_hyp, ca_surv):
    """Print hypothesis summary table"""

    print("\nHypothesis Summary:")
    print(f"{'Hypothesis':<35} {'Survival':<10} {'Acc Range':<18} {'Mean':<8}")
    print("-" * 71)

    all_hyp = {}
    for name, accs in graph_hyp.items():
        all_hyp[f"graph/{name}"] = (accs, graph_surv.get(name, 0))
    for name, accs in ca_hyp.items():
        all_hyp[f"ca/{name}"] = (accs, ca_surv.get(name, 0))

    for name in sorted(all_hyp.keys()):
        accs, surv = all_hyp[name]
        print(f"{name:<35} {surv}/10{'':<4} {min(accs):.1f}--{max(accs):.1f}%{'':<8} {np.mean(accs):.1f}%")


# ===================================================================
# Main
# ===================================================================

# Extract
graph_hyp, graph_surv = extract_hypotheses(graph)
ca_hyp, ca_surv = extract_hypotheses(ca)
graph_spectrum = extract_spectrum(graph)
graph_dist = extract_storage_distribution(graph)

# Plot
plot_spectrum(graph_spectrum, {}, 'Structure-Storage Gradient (Binary Graph)', f'{DATA_DIR}/plot_spectrum.png')
plot_hypotheses(graph_hyp, graph_surv, ca_hyp, ca_surv, f'{DATA_DIR}/plot_hypotheses.png')
plot_distribution(graph_dist, f'{DATA_DIR}/plot_distribution.png')

# Print spectrum
print_spectrum_table(graph_spectrum, "Binary Graph")

# Print hypothesis
print_hypothesis(graph_hyp, graph_surv, ca_hyp, ca_surv)