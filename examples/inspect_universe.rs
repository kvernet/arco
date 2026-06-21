//! Generate a single universe and inspect its structure and metrics.
//!
//! ```bash
//! cargo run --example inspect_universe --release
//! ```

use arco::dynamics::{DEFAULT_SCHEDULE, generate_ensemble};
use arco::metrics::{compute_memory, compute_storage};
use arco::observation::observe_windowed;
use arco::rules::{Rule, create_destructive_rules, create_structured_rules};
use arco::state::BinaryGraphState;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    let n_vertices = 3;
    let n_ensemble = 10;
    let steps = 60;
    let seed = 42;

    let mut rng = StdRng::seed_from_u64(seed);

    // Create a mixed rule set: 3 structured + 2 destructive
    let structured = create_structured_rules();
    let destructive = create_destructive_rules();
    let rules = vec![
        structured[0].clone(),  // IDENTITY
        structured[5].clone(),  // NAND
        structured[11].clone(), // SWAP
        destructive[0].clone(), // DESTROY_SCRAMBLE_ALL_0
        destructive[2].clone(), // DESTROY_ZERO
    ];

    println!("Rule set:");
    for r in &rules {
        println!(
            "  {} ({}, deterministic={}, locality={})",
            r.name(),
            r.rule_type(),
            r.is_deterministic(),
            r.locality_radius(),
        );
    }

    // Generate initial states
    let initial_states: Vec<BinaryGraphState> = (0..n_ensemble)
        .map(|_| BinaryGraphState::random(n_vertices, &mut rng))
        .collect();

    // Generate ensemble
    let ensemble = generate_ensemble(
        &initial_states,
        &rules,
        steps,
        n_ensemble,
        1, // window_size
        &DEFAULT_SCHEDULE,
        &observe_windowed,
        seed,
    );

    println!(
        "\nEnsemble: {} trajectories x {} steps",
        ensemble.len(),
        ensemble[0].len()
    );
    println!("First 3 observations of trajectory 0:");
    for (t, obs) in ensemble[0].iter().take(3).enumerate() {
        println!("  t={}: {:?}", t, obs);
    }

    // Compute metrics
    let storage = compute_storage(&ensemble, 15, 10, seed);
    let memory = compute_memory(&ensemble, 15, 10, seed);

    println!("\nEmergence metrics:");
    println!("  Storage: {:.6}", storage);
    println!("  Memory:  {:.6}", memory);

    let threshold = 0.12; // typical calibrated threshold
    if storage > threshold {
        println!("\n✓ Storage detected (above threshold {:.2})", threshold);
    } else {
        println!("\n✗ No storage detected (below threshold {:.2})", threshold);
    }
}
