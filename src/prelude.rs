//! ARCO — Commonly used items for convenient importing.
//!
//! This module re-exports the types and traits that are commonly needed when
//! working with this crate.
//!
//! Most applications can start with:
//!
//! ```rust
//! use arco::prelude::*;
//! ```
//!
//! Prefer explicit imports when you only need a small subset of the API.

pub use crate::{
    cycle::{CycleConfig, run_cycle},
    hypotheses::Hypothesis,
    invariants::Invariant,
    observation::Observation,
    resources::{ResourceUsage, Resources, UnitResources},
    rules::{NoContext, Rule},
    schedule::Schedule,
    state::State,
    universe::InformationUniverse,
};
