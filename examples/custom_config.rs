//! Run the scientific cycle with custom parameters.
//!
//! ```bash
//! cargo run --example custom_config --release
//! ```

use arco::cycle::{CycleConfig, run_cycle};

fn main() {
    let config = CycleConfig {
        n_train: 100,
        n_test: 30,
        n_vertices: 3,
        n_ensemble: 10,
        steps: 60,
        window_size: 1,
        obs_name: "compound".to_string(),
        max_delta: 15,
        n_shuffles: 10,
        n_null_universes: 30,
        seed: 99,
    };

    println!("Running with custom configuration...\n");
    let record = run_cycle(&config);
    println!("{}", record.summary());
}
