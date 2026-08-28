//! Fast, standalone argmax-delta diagnostic.
//!
//! Does NOT require running the full exact_mi.rs sweep first — this
//! recomputes only what it needs (7 canonical rules, exhaustive
//! coverage, single pass) and should take seconds, not the 20+ minutes
//! the full estimator sweep takes. Pulled out of exact_mi.rs specifically
//! because burying this behind the full sweep was a real design mistake:
//! the actual new question (does MM's max come from the same delta as
//! the true max?) shouldn't require re-paying for a full n_ens sweep and
//! a shuffle-count sweep whose flatness was already established and
//! explained.
//!
//! Answers one question: when MM's estimate diverges from exact ground
//! truth (rules 30, 54, 110, 184), is it because MM picks a DIFFERENT
//! delta than the true optimum (a max-selection artifact — fixable,
//! specific), or does MM pick the SAME delta and just estimate it
//! slightly wrong (an intrinsic bias in MM's first-order correction
//! formula — a different, less fixable kind of problem)?

use arco::calibration::generate_trajectories;
use arco::metrics::mm::{MMShuffleConfig, shuffle_corrected_mm};
use arco::substrates::ca::{CARule, CAState, CAUniverse};
use arco::universe::InformationUniverse;
use rand::SeedableRng;
use rand::rngs::StdRng;

const N_STATES: usize = 256;
const STEPS: usize = 60;
const MAX_DELTA: usize = 15;

fn bits_to_cells(bits: usize) -> [u8; 8] {
    let mut cells = [0u8; 8];
    for i in 0..8 {
        cells[i] = ((bits >> i) & 1) as u8;
    }
    cells
}

fn pack_state(state: &CAState<8, 1>) -> usize {
    let mut bits = 0usize;
    for (i, &c) in state.cells().iter().enumerate() {
        bits |= (c as usize) << i;
    }
    bits
}

fn all_tokens(rule: &CARule<8, 1>) -> Vec<Vec<usize>> {
    let mut tokens: Vec<Vec<usize>> = (0..N_STATES)
        .map(|ic| {
            let state = CAState::<8, 1>::new(bits_to_cells(ic));
            vec![pack_state(&state)]
        })
        .collect();

    for step in 0..STEPS {
        for ic in 0..N_STATES {
            let current = CAState::<8, 1>::new(bits_to_cells(tokens[ic][step]));
            let next = rule.apply_sync(&current);
            tokens[ic].push(pack_state(&next));
        }
    }

    tokens
}

fn pool_entropy(pool: &[usize]) -> f64 {
    let n = pool.len() as f64;
    let mut counts = vec![0usize; N_STATES];
    for &token in pool {
        counts[token] += 1;
    }
    let mut h = 0.0f64;
    for &c in &counts {
        if c > 0 {
            let p = c as f64 / n;
            h -= p * p.log2();
        }
    }
    h
}

fn exact_storage_with_argmax(tokens: &[Vec<usize>]) -> (f64, usize) {
    let mut best = 0.0f64;
    let mut best_delta = 0usize;
    for delta in 1..=MAX_DELTA {
        let pool_size = N_STATES * (STEPS + 1 - delta);
        let mut x_pool = Vec::with_capacity(pool_size);
        let mut y_pool = Vec::with_capacity(pool_size);

        for ic in 0..N_STATES {
            for t in 0..(STEPS + 1 - delta) {
                x_pool.push(tokens[ic][t]);
                y_pool.push(tokens[ic][t + delta]);
            }
        }

        let h_x = pool_entropy(&x_pool);
        let h_y = pool_entropy(&y_pool);
        if h_x > 0.0 {
            let val = (h_y / h_x).sqrt();
            if val > best {
                best = val;
                best_delta = delta;
            }
        }
    }
    (best, best_delta)
}

fn mm_storage_with_argmax<T: Eq + std::hash::Hash + Clone>(
    trajectories: &[Vec<T>],
    n_shuffles: usize,
    seed: u64,
) -> (f64, usize) {
    let traj_len = trajectories.iter().map(|t| t.len()).min().unwrap_or(0);
    let max_delta = MAX_DELTA.min(traj_len.saturating_sub(1));
    let mut best = 0.0f64;
    let mut best_delta = 0usize;

    for delta in 1..=max_delta {
        let mut all_x = Vec::new();
        let mut all_y = Vec::new();
        for traj in trajectories {
            for t in 0..(traj.len().saturating_sub(delta)) {
                all_x.push(&traj[t]);
                all_y.push(&traj[t + delta]);
            }
        }
        if all_x.len() > 10 {
            let config = MMShuffleConfig::new(n_shuffles, seed + delta as u64);
            let score = shuffle_corrected_mm(&all_x, &all_y, &config);
            if score > best {
                best = score;
                best_delta = delta;
            }
        }
    }
    (best, best_delta)
}

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    let universe = CAUniverse::<8, 1>::new("full_state", &mut rng, 400);
    let observer = universe.observation();
    let schedule = universe.schedule();

    let canon: &[(u64, &str)] = &[
        (0, "fixed point (all-0)"),
        (255, "fixed point (all-1)"),
        (30, "chaotic"),
        (54, "particle/glider structure"),
        (90, "additive/XOR, Sierpinski"),
        (110, "Turing-complete"),
        (184, "traffic/particle-hopping"),
    ];

    println!("ARGMAX-DELTA DIAGNOSTIC (canonical rules, exhaustive coverage, MM)");
    println!("{}", "-".repeat(88));
    println!(
        "{:<6}{:>10}{:>8}{:>10}{:>10}  {}",
        "Rule", "Exact Δ*", "MM Δ*", "Exact", "MM", "Description"
    );

    for &(r, desc) in canon {
        let rule = CARule::<8, 1>::from_wolfram_number(r);
        let tokens = all_tokens(&rule);
        let (exact_val, exact_delta) = exact_storage_with_argmax(&tokens);

        let initial_states: Vec<_> = (0..N_STATES)
            .map(|ic| CAState::<8, 1>::new(bits_to_cells(ic)))
            .collect();
        let rules = vec![rule.clone()];
        let trajectories =
            generate_trajectories(&initial_states, &rules, observer, schedule, STEPS, 0);
        let (mm_val, mm_delta) = mm_storage_with_argmax(&trajectories, 10, 42);

        let mismatch = if exact_delta == mm_delta {
            ""
        } else {
            " <-- MISMATCH"
        };
        println!(
            "{:<6}{:>10}{:>8}{:>10.3}{:>10.3}  {}{}",
            r, exact_delta, mm_delta, exact_val, mm_val, desc, mismatch
        );
    }

    println!();
    println!(
        "If Δ* differs for the biased rules (30, 54, 110, 184): MM's max-selection is landing \
         on the wrong delta — a specific, fixable mechanism."
    );
    println!(
        "If Δ* matches despite the persistent error: the bias is intrinsic to MM's first-order \
         correction formula itself, not a delta-selection artifact — a different, harder-to-fix \
         finding."
    );
}
