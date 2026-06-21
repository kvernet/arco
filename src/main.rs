use arco::cycle::{CycleConfig, run_cycle};
use clap::Parser;

/// ARCO — Automated Research into Computational Ontologies
///
/// A computational science platform for discovering the conditions
/// under which computation, memory, and learning emerge in arbitrary
/// information systems.
#[derive(Parser)]
#[command(name = "arco", version, about)]
struct Cli {
    /// Number of training universes
    #[arg(long, default_value = "300")]
    train: usize,

    /// Number of test universes
    #[arg(long, default_value = "100")]
    test: usize,

    /// Number of vertices per state
    #[arg(long, default_value = "3")]
    vertices: usize,

    /// Ensemble size per universe
    #[arg(long, default_value = "10")]
    ensemble: usize,

    /// Timesteps per trajectory
    #[arg(long, default_value = "60")]
    steps: usize,

    /// Observation window size
    #[arg(long, default_value = "1")]
    window: usize,

    /// Observation operator name
    #[arg(long, default_value = "compound")]
    obs: String,

    /// Maximum timescale for storage/memory
    #[arg(long, default_value = "15")]
    max_delta: usize,

    /// Number of shuffles for bias correction
    #[arg(long, default_value = "10")]
    shuffles: usize,

    /// Number of null universes for calibration
    #[arg(long, default_value = "30")]
    null: usize,

    /// Random seed for reproducibility
    #[arg(long, default_value = "42")]
    seed: u64,

    /// Quick test run (overrides train/test/ensemble/steps)
    #[arg(long)]
    quick: bool,
}

fn main() {
    let cli = Cli::parse();

    let config = if cli.quick {
        CycleConfig {
            n_train: 20,
            n_test: 5,
            n_vertices: cli.vertices,
            n_ensemble: 4,
            steps: 10,
            window_size: cli.window,
            obs_name: cli.obs,
            max_delta: cli.max_delta,
            n_shuffles: cli.shuffles,
            n_null_universes: 5,
            seed: cli.seed,
        }
    } else {
        CycleConfig {
            n_train: cli.train,
            n_test: cli.test,
            n_vertices: cli.vertices,
            n_ensemble: cli.ensemble,
            steps: cli.steps,
            window_size: cli.window,
            obs_name: cli.obs,
            max_delta: cli.max_delta,
            n_shuffles: cli.shuffles,
            n_null_universes: cli.null,
            seed: cli.seed,
        }
    };

    let record = run_cycle(&config);
    println!("{}", record.summary());
}
