//! Resource accounting for the Cellular Automaton substrate.

use crate::resources::Resources;
use crate::substrates::ca::rules::CARule;
use crate::substrates::ca::state::CAState;

/// Resource accounting for [`crate::substrates::ca::CAUniverse`].
///
/// A single struct works across every `CAUniverse<N, R>`
/// instantiation, the same way [`crate::substrates::ca::SynchronousCASchedule`]
/// and [`crate::substrates::ca::CAObserver`] are not themselves
/// parameterized by `N`/`R`.
///
/// - **Space** (`R_space`): `N` cells — the CA's state size never
///   changes.
/// - **Time** (`R_time`): `1.0` per synchronous update.
/// - **Locality** (`R_local`): `R`, the neighborhood radius. Every
///   cell's next state depends on exactly `2R+1` neighbors, so `R` is
///   the substrate's fixed interaction radius regardless of which
///   rule (lookup table) is used.
#[derive(Debug, Clone, Copy, Default)]
pub struct CAResources;

impl<const N: usize, const R: usize> Resources<CAState<N, R>, CARule<N, R>> for CAResources {
    fn name(&self) -> &str {
        "ca_resources"
    }

    fn space(&self, _state: &CAState<N, R>) -> f64 {
        N as f64
    }

    fn time(&self, _rule: &CARule<N, R>) -> f64 {
        1.0
    }

    fn locality(&self, _rule: &CARule<N, R>) -> f64 {
        R as f64
    }
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_equals_cell_count() {
        let model = CAResources;
        let state = CAState::<8, 1>::new([0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            <CAResources as Resources<CAState<8, 1>, CARule<8, 1>>>::space(&model, &state),
            8.0
        );
    }

    #[test]
    fn locality_equals_neighborhood_radius() {
        let model = CAResources;
        let rule = CARule::<8, 1>::from_wolfram_number(110);
        assert_eq!(
            <CAResources as Resources<CAState<8, 1>, CARule<8, 1>>>::locality(&model, &rule),
            1.0
        );

        let rule2 = CARule::<8, 2>::from_wolfram_number(0);
        assert_eq!(
            <CAResources as Resources<CAState<8, 2>, CARule<8, 2>>>::locality(&model, &rule2),
            2.0
        );
    }
}
