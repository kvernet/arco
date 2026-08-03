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

use arco::calibration::generate_trajectories;
use arco::cycle::{CycleConfig, run_cycle};
use arco::metrics::{MetricConfig, storage};
use arco::substrates::ca::{CARule, CAState, CAUniverse, generate_ca_hypotheses};
use arco::universe::InformationUniverse;
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

    // Classification
    classify(&universe);

    println!("\n{}", record.summary());
}

fn classify(universe: &CAUniverse<8, 1>) {
    let seeds = [42, 99, 137, 256, 512];

    println!("\nWolfram Class Recovery ({} seeds):", seeds.len());
    println!(
        "  {:<10} {:<18} {:<15} {}",
        "Rule", "Storage", "Wolfram Class", "Description"
    );
    println!(
        "  {:<10} {:<18} {:<15} {}",
        "----", "-------", "-------------", "-----------"
    );

    let famous: &[(u64, &str, &str)] = &[
        (0, "Class 1", "Fixed point (all 0)"),
        (30, "Class 3", "Chaotic"),
        (54, "Class 4", "Complex, Turing-complete"),
        (90, "Class 2", "Sierpinski triangle"),
        (110, "Class 4", "Turing-complete"),
        (184, "Class 2", "Traffic flow model"),
        (255, "Class 1", "Fixed point (all 1)"),
    ];

    let observer = universe.observation();
    let schedule = universe.schedule();

    for &(rule_num, class, desc) in famous {
        let mut storages = Vec::new();

        for &seed in &seeds {
            let mut diag_rng = StdRng::seed_from_u64(seed);
            let rule = CARule::<8, 1>::from_wolfram_number(rule_num);
            let rules = vec![rule];

            let initial_states: Vec<_> = (0..10)
                .map(|_| CAState::<8, 1>::random(&mut diag_rng))
                .collect();

            let trajectories =
                generate_trajectories(&initial_states, &rules, observer, schedule, 60, seed);

            let met_config = MetricConfig::default();
            storages.push(storage(&trajectories, &met_config));
        }

        let min_s = storages.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_s = storages.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean_s = storages.iter().sum::<f64>() / storages.len() as f64;

        println!(
            "  Rule {:<3}   {:.2}–{:.2} ({:.2})    {:<15} {}",
            rule_num, min_s, max_s, mean_s, class, desc
        );
    }
}
