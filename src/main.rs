//! ARCO — Automated Research into Computational Ontologies
//!
//! Command-line interface for running the scientific cycle on
//! any registered substrate.

use clap::{Args, Parser, Subcommand};
use rand::SeedableRng;
use rand::rngs::StdRng;

use arco::cycle::{CycleConfig, run_cycle};
use arco::record::ResearchRecord;

// ===================================================================
// CLI Structure
// ===================================================================

#[derive(Parser)]
#[command(
    name = "arco",
    version,
    about = "Automated Research into Computational Ontologies"
)]
struct Cli {
    #[command(subcommand)]
    substrate: Substrate,
}

#[derive(Subcommand)]
enum Substrate {
    /// Binary Graph Universe — directed graphs with rewrite rules
    Graph(GraphArgs),
    /// Cellular Automaton — 1D binary automata with lookup-table rules
    Ca(CaArgs),
}

// ===================================================================
// Shared options
// ===================================================================

#[derive(Args)]
struct SharedArgs {
    /// Number of training universes
    #[arg(long, default_value = "1000")]
    train: usize,

    /// Number of held-out test universes
    #[arg(long, default_value = "300")]
    test: usize,

    /// Random seed for reproducibility
    #[arg(long, default_value = "42")]
    seed: u64,

    /// Fast test run (overrides train/test)
    #[arg(long)]
    quick: bool,
}

// ===================================================================
// Graph-specific options
// ===================================================================

#[derive(Args)]
struct GraphArgs {
    #[command(flatten)]
    shared: SharedArgs,

    /// Number of vertices per state
    #[arg(long, default_value = "3")]
    vertices: usize,

    /// Observation operator: full_state, compound, label_vector,
    /// label_sum, root_label, edge_vector, edge_count
    #[arg(long, default_value = "compound")]
    obs: String,
}

// ===================================================================
// CA-specific options
// ===================================================================

#[derive(Args)]
struct CaArgs {
    #[command(flatten)]
    shared: SharedArgs,

    /// Number of cells
    #[arg(long, default_value = "8")]
    cells: usize,

    /// Neighborhood radius
    #[arg(long, default_value = "1")]
    radius: usize,

    /// Observation operator: full_state, density, parity
    #[arg(long, default_value = "full_state")]
    obs: String,
}

// ===================================================================
// Cycle configuration from shared args
// ===================================================================

fn cycle_config(shared: &SharedArgs) -> CycleConfig {
    if shared.quick {
        CycleConfig {
            n_train: 20,
            n_test: 5,
            seed: shared.seed,
            ..CycleConfig::default()
        }
    } else {
        CycleConfig {
            n_train: shared.train,
            n_test: shared.test,
            seed: shared.seed,
            ..CycleConfig::default()
        }
    }
}

// ===================================================================
// Substrate runners
// ===================================================================

fn run_graph(args: &GraphArgs) -> ResearchRecord<arco::substrates::graph::BinaryGraphUniverse> {
    use arco::substrates::graph::{
        BinaryGraphUniverse, generate_standard_hypotheses, verify_boolean_functions,
    };
    use std::collections::HashMap;

    let config = cycle_config(&args.shared);
    let mut rng = StdRng::seed_from_u64(args.shared.seed);
    let universe = BinaryGraphUniverse::new(
        args.vertices,
        &args.obs,
        &mut rng,
        config.n_train + config.n_test,
    );

    let mut hypotheses = generate_standard_hypotheses();

    let boolean_tester =
        |rules: &[arco::substrates::graph::RewriteRule]| -> HashMap<String, usize> {
            let verified = verify_boolean_functions(rules, 8, 5);
            verified.into_iter().map(|name| (name, 1)).collect()
        };

    run_cycle(&universe, &config, &mut hypotheses, Some(&boolean_tester))
}

fn run_ca(args: &CaArgs) -> ResearchRecord<arco::substrates::ca::CAUniverse<8, 1>> {
    // Note: N and R are fixed at compile time. For now, we support
    // the most common case (8 cells, radius 1). Adding runtime
    // configurability for const generics requires a macro or
    // dynamic dispatch — deferred to future work.
    assert_eq!(args.cells, 8, "Only N=8 is currently supported via CLI");
    assert_eq!(args.radius, 1, "Only R=1 is currently supported via CLI");

    use arco::substrates::ca::{CAUniverse, generate_ca_hypotheses};

    let config = cycle_config(&args.shared);
    let mut rng = StdRng::seed_from_u64(args.shared.seed);
    let universe = CAUniverse::<8, 1>::new(&args.obs, &mut rng, config.n_train + config.n_test);

    let mut hypotheses = generate_ca_hypotheses::<8, 1>();

    run_cycle(&universe, &config, &mut hypotheses, None)
}

// ===================================================================
// Main
// ===================================================================

fn main() {
    let cli = Cli::parse();

    match &cli.substrate {
        Substrate::Graph(args) => {
            let record = run_graph(args);
            println!("\n{}", record.summary());
            print_spectrum_graph(&record);
        }
        Substrate::Ca(args) => {
            let record = run_ca(args);
            println!("\n{}", record.summary());
            print_spectrum_ca(&record);
        }
    }
}

// ===================================================================
// Spectrum printing
// ===================================================================

fn print_spectrum_graph(record: &ResearchRecord<arco::substrates::graph::BinaryGraphUniverse>) {
    let threshold = record.thresholds.get("storage").copied().unwrap_or(0.0);
    let brackets: &[(&str, f64, f64)] = &[
        ("Noise", 0.00, 0.15),
        ("Noise-dominated", 0.15, 0.40),
        ("Balanced", 0.40, 0.60),
        ("Structure-dominated", 0.60, 0.85),
        ("Structured", 0.85, 1.01),
    ];
    println!("\nStorage Spectrum:");
    println!(
        "  {:<20} {:<6} {:<8} {:<8}",
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
        let stor_pct =
            100.0 * group.iter().filter(|r| r.storage > threshold).count() as f64 / n as f64;
        let mean_stor = group.iter().map(|r| r.storage).sum::<f64>() / n as f64;
        println!(
            "  {:<20} {:<6} {:<8.1} {:<8.4}",
            label, n, stor_pct, mean_stor
        );
    }
}

fn print_spectrum_ca(record: &ResearchRecord<arco::substrates::ca::CAUniverse<8, 1>>) {
    let threshold = record.thresholds.get("storage").copied().unwrap_or(0.0);
    let brackets: &[(&str, f64, f64)] = &[
        ("Low structure (0.0--0.3)", 0.0, 0.3),
        ("High structure (0.7--1.0)", 0.7, 1.0),
    ];
    println!("\nStorage Spectrum:");
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
        let stor_pct =
            100.0 * group.iter().filter(|r| r.storage > threshold).count() as f64 / n as f64;
        let mean_stor = group.iter().map(|r| r.storage).sum::<f64>() / n as f64;
        println!(
            "  {:<30} {:<6} {:<8.1} {:<8.4}",
            label, n, stor_pct, mean_stor
        );
    }
}
