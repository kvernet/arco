use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

/// Initial condition separation: how distinguishable are futures
/// given different initial states?
///
/// For each pair of distinct initial observations (at t=0), computes
/// the total variation distance between their conditional output
/// distributions after Δ steps. High values mean different initial
/// states produce distinguishable futures.
///
/// This measures sensitivity to initial conditions — NOT memory.
/// A chaotic deterministic system can score high even if it
/// preserves no recoverable information about the past.
///
/// # Limitation
///
/// This metric only conditions on the first observation (`traj[0]`),
/// not on every intermediate state. For non-stationary dynamics,
/// consider conditioning on multiple timepoints.
///
/// Preserved for diagnostic use alongside [`memory`].
pub fn init_separation<T: Eq + Hash + Clone>(trajectories: &[Vec<T>], max_delta: usize) -> f64 {
    let n_traj = trajectories.len();
    if n_traj < 2 {
        return 0.0;
    }

    let traj_len = trajectories.iter().map(|t| t.len()).min().unwrap_or(0);
    let max_delta = max_delta.min(traj_len.saturating_sub(1));
    let mut scores = Vec::new();

    for delta in 1..=max_delta {
        let mut initial_to_later: HashMap<&T, Vec<&T>> = HashMap::new();

        for traj in trajectories {
            initial_to_later
                .entry(&traj[0])
                .or_default()
                .push(&traj[delta]);
        }

        let initial_vals: Vec<&T> = initial_to_later.keys().copied().collect();

        for i in 0..initial_vals.len() {
            for j in (i + 1)..initial_vals.len() {
                let later_i = &initial_to_later[initial_vals[i]];
                let later_j = &initial_to_later[initial_vals[j]];

                let all_keys: HashSet<&&T> = later_i.iter().chain(later_j.iter()).collect();
                let total_i = later_i.len() as f64;
                let total_j = later_j.len() as f64;

                if total_i > 0.0 && total_j > 0.0 {
                    let mut tv = 0.0;
                    for key in &all_keys {
                        let count_i = later_i.iter().filter(|&x| x == *key).count() as f64;
                        let count_j = later_j.iter().filter(|&x| x == *key).count() as f64;
                        tv += (count_i / total_i - count_j / total_j).abs();
                    }
                    scores.push(0.5 * tv);
                }
            }
        }
    }

    if scores.is_empty() {
        0.0
    } else {
        scores.iter().sum::<f64>() / scores.len() as f64
    }
}
