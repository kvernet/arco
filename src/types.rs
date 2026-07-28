//! Central type aliases for ARCO.

use rand::Rng;
use std::sync::Arc;

use crate::observation::Observation;
use crate::substrates::graph::{BinaryGraphState, MatchInfo};
use crate::universe::InformationUniverse;
use std::collections::HashMap;

// === Calibration types ===

/// Observation output type for a universe.
pub type ObsOutput<U> = <<U as InformationUniverse>::Observation as Observation<
    <U as InformationUniverse>::State,
>>::Output;

/// Null trajectory ensembles.
pub type NullEnsembles<U> = Vec<Vec<Vec<ObsOutput<U>>>>;

// === Cycle types ===

/// Boolean function tester: maps rule sets to gate validation counts.
pub type BooleanTester<U> = dyn Fn(&[<U as InformationUniverse>::Rule]) -> HashMap<String, usize>;

/// Test trajectory ensembles.
pub type TestEnsembles<U> = Vec<Vec<Vec<ObsOutput<U>>>>;

// === Hypothesis types ===

/// Structural condition predicate on rule sets.
pub type ConditionPredicate<R> = dyn Fn(&[R]) -> bool + Send + Sync;

// Functions for RewriteRule
pub type ConditionFn = Arc<dyn Fn(&BinaryGraphState, usize) -> Option<MatchInfo> + Send + Sync>;

pub type ActionFn =
    Arc<dyn Fn(&BinaryGraphState, &MatchInfo, &mut dyn Rng) -> BinaryGraphState + Send + Sync>;

/// A truth table for a 2-input Boolean function.
pub type TruthTable = [((u8, u8), u8)];
