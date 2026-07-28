//! Cellular Automaton — Full Scientific Cycle
//!
//! This example runs the complete ARCO pipeline on the Cellular
//! Automaton substrate, demonstrating paradigm-neutrality: the
//! same `run_cycle` function used for the Binary Graph Universe
//! operates on CA without modification.
//!
//! # Running
//!
//! ```bash
//! cargo run --example ca_cycle --release
//! cargo run --example ca_cycle --release -- 42
//! ```

use arco::cycle::{CycleConfig, run_cycle};
use arco::substrates::ca::{CAUniverse, generate_ca_hypotheses};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    println!(
        "=== ARCO Cellular Automaton — Full Cycle (seed={}) ===\n",
        seed
    );

    let mut rng = StdRng::seed_from_u64(seed);
    let universe = CAUniverse::<8, 1>::new("full_state", &mut rng, 400);

    let mut hypotheses = generate_ca_hypotheses::<8, 1>();

    let config = CycleConfig {
        n_train: 256,
        n_test: 50,
        seed,
        ..CycleConfig::default()
    };

    let record = run_cycle(&universe, &config, &mut hypotheses, None);

    // Print storage spectrum
    println!("\nStorage Spectrum:");
    let storage_threshold = record.thresholds.get("storage").copied().unwrap_or(0.0);
    let brackets: &[(&str, f64, f64)] = &[
        ("Low structure (0.0--0.3)", 0.0, 0.3),
        ("High structure (0.7--1.0)", 0.7, 1.0),
    ];
    println!(
        "  {:<30} {:<6} {:<8} {:<8}",
        "Class", "n", "Stor%", "MeanStor"
    );
    for (label, low, high) in brackets {
        let group: Vec<_> = record
            .results
            .iter()
            .filter(|r| r.structured_ratio >= *low && r.structured_ratio < *high)
            .collect();
        if group.is_empty() {
            continue;
        }
        let n = group.len();
        let stor_pct = 100.0
            * group
                .iter()
                .filter(|r| r.storage > storage_threshold)
                .count() as f64
            / n as f64;
        let mean_stor = group.iter().map(|r| r.storage).sum::<f64>() / n as f64;
        println!(
            "  {:<30} {:<6} {:<8.1} {:<8.4}",
            label, n, stor_pct, mean_stor
        );
    }

    println!("\n{}", record.summary());
}
