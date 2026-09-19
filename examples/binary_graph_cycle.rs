use std::collections::HashMap;

use arco::cycle::{CycleConfig, run_cycle};
use arco::substrates::graph::{
    BinaryGraphUniverse, generate_standard_hypotheses, verify_boolean_functions,
};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    println!(
        "=== ARCO Binary Graph Universe — Full Cycle (seed={}) ===\n",
        seed
    );

    let mut rng = StdRng::seed_from_u64(seed);
    let n_train = 1000;
    let n_test = 300;
    let universe = BinaryGraphUniverse::new(3, "compound", &mut rng, n_train + n_test);

    let mut hypotheses = generate_standard_hypotheses();

    let boolean_tester =
        |rules: &[arco::substrates::graph::RewriteRule]| -> HashMap<String, usize> {
            let verified = verify_boolean_functions(rules, 8, 5);
            verified.into_iter().map(|name| (name, 1)).collect()
        };

    let config = CycleConfig {
        n_train,
        n_test,
        seed,
        ..CycleConfig::default()
    };

    let record = run_cycle(&universe, &config, &mut hypotheses, Some(&boolean_tester));

    // Print storage & memory spectrum
    println!("\nStorage & Memory Spectrum:");
    let storage_threshold = record.thresholds.get("storage").copied().unwrap_or(0.0);
    let memory_threshold = record.thresholds.get("memory").copied().unwrap_or(0.0);
    let brackets: &[(&str, f64, f64)] = &[
        ("Noise", 0.00, 0.15),
        ("Noise-dominated", 0.15, 0.40),
        ("Balanced", 0.40, 0.60),
        ("Structure-dominated", 0.60, 0.85),
        ("Structured", 0.85, 1.01),
    ];
    println!(
        "  {:<20} {:<6} {:<8} {:<10} {:<8} {:<8}",
        "Class", "n", "Stor%", "MeanStor", "Mem%", "MeanMem"
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
        let mem_pct =
            100.0 * group.iter().filter(|r| r.memory > memory_threshold).count() as f64 / n as f64;
        let mean_mem = group.iter().map(|r| r.memory).sum::<f64>() / n as f64;
        println!(
            "  {:<20} {:<6} {:<8.1} {:<10.4} {:<8.1} {:<8.4}",
            label, n, stor_pct, mean_stor, mem_pct, mean_mem
        );
    }

    println!("\n{}", record.summary());
}
