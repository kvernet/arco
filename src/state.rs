//! State trait — the fundamental abstraction for ARCO.
//!
//! Per the Mathematical Constitution:
//!     A state space S is a set equipped with a canonical encoding
//!     function, a distance function, and a cardinality bound.
//!     States must be distinguishable via observation operators.
//!
//! # The State trait
//!
//! A type implementing `State` represents a single configuration of
//! an Information Universe at a point in time. States are immutable:
//! mutation methods return new states rather than modifying in place.
//!
//! # Requirements
//!
//! Implementors must provide:
//!
//! - **Canonical encoding**: A deterministic, unique representation
//!   of the state as a hashable, comparable value. This is the
//!   identity observation — the maximally dynamically sufficient
//!   observation operator. Two states are equal if and only if
//!   their canonical encodings are equal.
//!
//! - **Distance metric**: A function satisfying identity
//!   (d(s, s) = 0), symmetry (d(s1, s2) = d(s2, s1)), and the
//!   triangle inequality (d(s1, s3) ≤ d(s1, s2) + d(s2, s3)).
//!   Used for perturbation analysis and error resilience metrics.
//!
//! # Design contract
//!
//! - States are immutable. Mutation operations return new state
//!   objects.
//! - The canonical encoding must be independent of runtime state
//!   (memory addresses, RNG seeds, platform-specific layout) and
//!   must be identical across runs.
//! - Equality and hashing must be consistent with the canonical
//!   encoding: `s1 == s2` if and only if
//!   `s1.canonical_encoding() == s2.canonical_encoding()`.
//!
//! # Quick start
//!
//! Implement `State` for your type to use it with ARCO's metrics,
//! calibration, and scientific cycle:
//!
//! ```rust
//! use arco::state::State;
//!
//! #[derive(Clone, PartialEq, Eq, Hash, Debug)]
//! struct MyState {
//!     data: Vec<u8>,
//! }
//!
//! impl State for MyState {
//!     type Encoding = Vec<u8>;
//!
//!     fn canonical_encoding(&self) -> Self::Encoding {
//!         self.data.clone()
//!     }
//!
//!     fn distance(&self, other: &Self) -> u32 {
//!         self.data.iter()
//!             .zip(other.data.iter())
//!             .map(|(a, b)| if a != b { 1 } else { 0 })
//!             .sum()
//!     }
//! }
//! ```
//!
//! # Substrate-specific state types
//!
//! Concrete state implementations live in substrate modules:
//!
//! - [`arco::substrates::graph::BinaryGraphState`] — directed graphs
//!   with binary vertex labels and binary edge labels.
//! - [`arco::substrates::ca::CAState`] — 1D binary cellular automaton
//!   with periodic boundary conditions. Each state is a fixed-length
//!   binary tuple representing the cells at a single timestep.
//!
//! Users define their own state types by implementing this trait.

use std::fmt::Debug;
use std::hash::Hash;

/// The fundamental abstraction for a state in an Information Universe.
///
/// A state represents a single configuration at a point in time.
/// States are immutable and must provide a canonical encoding for
/// identity, a distance metric for perturbation analysis, and
/// support cloning, equality, hashing, and thread-safe sharing.
///
/// # Type parameter
///
/// - `Encoding`: The type of the canonical encoding. Must be
///   hashable, comparable, cloneable, and thread-safe. Common
///   choices are `Vec<u8>`, `(Vec<u8>, Vec<u8>)`, or a custom
///   deterministic representation.
///
/// # Example
///
/// ```rust
/// use arco::state::State;
///
/// #[derive(Clone, PartialEq, Eq, Hash, Debug)]
/// struct CounterState {
///     value: u8,
/// }
///
/// impl State for CounterState {
///     type Encoding = Vec<u8>;
///
///     fn canonical_encoding(&self) -> Self::Encoding {
///         vec![self.value]
///     }
///
///     fn distance(&self, other: &Self) -> u32 {
///         if self.value == other.value { 0 } else { 1 }
///     }
/// }
///
/// let s1 = CounterState { value: 0 };
/// let s2 = CounterState { value: 1 };
/// assert_eq!(s1.distance(&s2), 1);
/// assert_eq!(s1.distance(&s1), 0);
/// ```
pub trait State: Clone + Eq + Hash + Send + Sync + Debug {
    /// The type of the canonical encoding.
    ///
    /// Must uniquely identify the state. Two states with the same
    /// encoding are considered identical.
    type Encoding: Clone + Eq + Hash + Send + Sync + Debug;

    /// Return a deterministic, unique encoding of the complete state.
    ///
    /// This is the identity observation — the maximally dynamically
    /// sufficient observation operator per the Constitution. It
    /// distinguishes every distinct state.
    ///
    /// # Determinism
    ///
    /// The encoding must be identical across runs, platforms, and
    /// runtime environments. It must not depend on memory addresses,
    /// RNG state, or platform-specific layout.
    fn canonical_encoding(&self) -> Self::Encoding;

    /// Hamming distance between this state and another.
    ///
    /// Returns the number of atomic differences between the two
    /// states. Must satisfy the metric axioms:
    ///
    /// - **Identity**: `distance(s, s) == 0`
    /// - **Symmetry**: `distance(a, b) == distance(b, a)`
    /// - **Triangle inequality**: `distance(a, c) ≤ distance(a, b) + distance(b, c)`
    ///
    /// Used for perturbation analysis, error resilience metrics,
    /// and sensitivity measurements.
    fn distance(&self, other: &Self) -> u32;
}
