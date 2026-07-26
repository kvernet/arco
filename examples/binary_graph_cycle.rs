//! Binary Graph Universe — full scientific cycle.
//!
//! This example runs the complete ARCO pipeline on the Binary Graph
//! Universe, reproducing the Structure-Storage Gradient and the
//! Transport Law.
//!
//! ```bash
//! cargo run --example binary_graph_cycle --release
//! ```

use std::collections::HashMap;

use arco::cycle::{CycleConfig, run_cycle};
use arco::substrates::graph::{
    BinaryGraphUniverse, generate_standard_hypotheses, spectrum_rule_generator,
    verify_boolean_functions,
};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    println!("=== ARCO Binary Graph Universe — Full Cycle ===\n");

    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    // Create the universe
    let mut rng = StdRng::seed_from_u64(seed);
    let universe = BinaryGraphUniverse::new(3, "compound", &mut rng);

    // Generate rule sets across the structured/destructive spectrum
    let n_train = 1000;
    let n_test = 300;
    let mut rule_generator = spectrum_rule_generator(n_train + n_test, seed);

    // Standard hypotheses
    let mut hypotheses = generate_standard_hypotheses();

    // Boolean verification closure
    let boolean_tester =
        |rules: &[arco::substrates::graph::RewriteRule]| -> HashMap<String, usize> {
            let verified = verify_boolean_functions(rules, 8, 5);
            verified.into_iter().map(|name| (name, 1)).collect()
        };

    // Run the cycle
    let config = CycleConfig::default();
    let record = run_cycle(
        &universe,
        &config,
        &mut hypotheses,
        &mut rule_generator,
        Some(&boolean_tester),
    );

    // Print storage spectrum
    println!("\nStorage Spectrum:");
    let thresholds = &record.thresholds;
    let storage_threshold = thresholds.get("storage").copied().unwrap_or(0.0);

    let brackets: &[(&str, f64, f64)] = &[
        ("Noise", 0.00, 0.15),
        ("Noise-dominated", 0.15, 0.40),
        ("Balanced", 0.40, 0.60),
        ("Structure-dominated", 0.60, 0.85),
        ("Structured", 0.85, 1.01),
    ];

    println!(
        "  {:<20} {:<6} {:<12} {:<18}",
        "Class", "n", "Storage %", "Mean Storage"
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
            "  {:<20} {:<6} {:<12.1} {:<18.4}",
            label, n, stor_pct, mean_stor
        );
    }

    println!("{}", record.summary());
}
