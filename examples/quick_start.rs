//! Run the scientific cycle with default parameters.
//!
//! ```bash
//! cargo run --example quick_start --release
//! ```

use arco::cycle::{CycleConfig, run_cycle};

fn main() {
    let config = CycleConfig::default();
    let record = run_cycle(&config);
    println!("{}", record.summary());
}
