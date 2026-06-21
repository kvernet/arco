//! Cross-validate storage metric against the reference implementation.
//!
//! ```bash
//! cargo run --example cross_validate --release
//! ```

use arco::dynamics::{DEFAULT_SCHEDULE, generate_ensemble};
use arco::metrics::compute_storage;
use arco::observation::observe_windowed;
use arco::rules::{Rule, create_destructive_rules, create_structured_rules};
use arco::state::BinaryGraphState;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    let n_vertices = 3;
    let n_ensemble = 10;
    let steps = 60;
    let max_delta = 15;
    let n_shuffles = 10;
    let seed = 42;

    let mut rng = StdRng::seed_from_u64(seed);
    let state_pool: Vec<BinaryGraphState> = (0..500)
        .map(|_| BinaryGraphState::random(n_vertices, &mut rng))
        .collect();

    let initial_states: Vec<BinaryGraphState> =
        state_pool.iter().take(n_ensemble).cloned().collect();

    let structured = create_structured_rules();
    let destructive = create_destructive_rules();
    let rules = vec![
        structured[0].clone(),  // IDENTITY
        structured[1].clone(),  // TOGGLE
        destructive[0].clone(), // DESTROY_SCRAMBLE_ALL_0
    ];

    println!(
        "Rule set: {}, {}, {}",
        rules[0].name(),
        rules[1].name(),
        rules[2].name()
    );

    let ensemble = generate_ensemble(
        &initial_states,
        &rules,
        steps,
        n_ensemble,
        1,
        &DEFAULT_SCHEDULE,
        &observe_windowed,
        seed,
    );

    let storage = compute_storage(&ensemble, max_delta, n_shuffles, seed);
    println!("Storage: {:.6}", storage);
    println!("Python reference value: 0.109705");
    println!("Difference: {:.6}", (storage - 0.109705).abs());
}
