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

    // Create the universe
    let mut rng = StdRng::seed_from_u64(42);
    let universe = BinaryGraphUniverse::new(3, "compound", &mut rng);

    // Generate rule sets across the structured/destructive spectrum
    let n_train = 300;
    let n_test = 100;
    let mut rule_generator = spectrum_rule_generator(n_train + n_test, 42);

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

    println!("{}", record.summary());
}
