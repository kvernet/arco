#!/usr/bin/env python3
"""Generate scatter plots from benchmark data."""

import matplotlib.pyplot as plt
import numpy as np
import os

DATA_DIR = "benchmark_data"
OUTPUT = os.path.join(DATA_DIR, "convergence.png")

ens_sizes = [10, 20, 50, 100, 256]
fig, axes = plt.subplots(2, 3, figsize=(14, 10))
axes = axes.flatten()

for i, n in enumerate(ens_sizes):
    path = os.path.join(DATA_DIR, f"n{n:03d}.csv")
    if not os.path.exists(path):
        continue
    
    data = np.loadtxt(path, delimiter=',', skiprows=1)
    exact = data[:, 1]
    mm = data[:, 3]  # MM estimator
    
    ax = axes[i]
    ax.scatter(exact, mm, alpha=0.3, s=8, color='#58a6ff')
    ax.plot([0, 1], [0, 1], '--', color='gray', alpha=0.4)
    ax.set_xlabel('Exact NMI')
    ax.set_ylabel('MM Estimate')
    ax.set_title(f'n = {n} ({(n*60)//256}× samples/state)')
    ax.set_xlim(-0.02, 1.02)
    ax.set_ylim(-0.02, 1.02)
    ax.set_aspect('equal')
    
    # Add correlation text
    r = np.corrcoef(exact, mm)[0, 1]
    mae = np.mean(np.abs(exact - mm))
    ax.text(0.05, 0.95, f'r = {r:.3f}\nMAE = {mae:.3f}', 
            transform=ax.transAxes, fontsize=9, verticalalignment='top',
            bbox=dict(boxstyle='round', facecolor='#161b22', alpha=0.8, edgecolor='#30363d'))

axes[5].set_visible(False)
plt.suptitle('Miller-Madow Estimator Convergence', fontweight='bold', fontsize=14)
plt.tight_layout()
plt.savefig(OUTPUT, dpi=150, bbox_inches='tight')
print(f"Saved {OUTPUT}")
plt.show()