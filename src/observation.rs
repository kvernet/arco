//! Observation trait — epistemic access to Information Universes.
//!
//! Per the Mathematical Constitution:
//!     O is a set of functions from S to Y, where Y is an observation
//!     space. Observation operators must be dynamically sufficient
//!     for the transformation dynamics.
//!
//! # The Observation trait
//!
//! An observation operator defines what an external observer can
//! "see" when looking at a state. The same universe can appear to
//! have high or low emergence depending on how it is observed.
//! This is not a bug — it is the observer-relative nature of
//! emergence, made explicit.
//!
//! # Granularity
//!
//! Observation operators range from the identity observation
//! (distinguishes every state) to coarse aggregates (e.g., counting
//! the number of active elements). Coarser observations have smaller
//! output alphabets, reducing estimator bias but potentially missing
//! information preserved in the details.
//!
//! # Dynamic Sufficiency
//!
//! An observation operator is dynamically sufficient if switching
//! from the identity observation to this operator does not
//! qualitatively change emergence metrics. If a coarser observation
//! reports zero storage while the identity observation reports
//! nonzero storage, the coarser observation is insufficient — it
//! misses the information being preserved.
//!
//! # Implementing Observation
//!
//! ```rust
//! use arco::state::State;
//! use arco::observation::Observation;
//!
//! #[derive(Clone, PartialEq, Eq, Debug, Hash)]
//! struct MyState { data: Vec<u8> }
//! impl State for MyState {
//!     type Encoding = Vec<u8>;
//!     fn canonical_encoding(&self) -> Self::Encoding { self.data.clone() }
//!     fn distance(&self, other: &Self) -> u32 {
//!         self.data.iter().zip(other.data.iter())
//!             .map(|(a, b)| if a != b { 1 } else { 0 }).sum()
//!     }
//! }
//!
//! /// Observe only the sum of all elements.
//! struct SumObserver;
//!
//! impl Observation<MyState> for SumObserver {
//!     type Output = u8;
//!
//!     fn observe(&self, state: &MyState) -> Self::Output {
//!         state.data.iter().sum()
//!     }
//! }
//! ```
//!
//! # Built-in observers
//!
//! Substrate-specific observation operators live in their substrate
//! modules. See [`arco::substrates::graph::observation`] for Binary
//! Graph observers (full state, label vector, label sum, compound).

use crate::state::State;
use std::fmt::Debug;
use std::hash::Hash;

/// An observation operator for an Information Universe.
///
/// Defines how an external observer perceives states. The observation
/// output type must be hashable and comparable — this is required
/// for mutual information estimation, which groups observations
/// by equality.
///
/// # Type parameters
///
/// - `S: State` — The state type being observed.
/// - `Output` — The type of observation values. Must be hashable,
///   comparable, cloneable, and thread-safe.
///
/// # Determinism
///
/// Observation operators must be deterministic: the same state
/// always produces the same observation. No RNG, no timestamps,
/// no memory addresses.
///
/// # Statelessness
///
/// Observation operators should be stateless. If stateful observation
/// is needed (e.g., windowed observation over multiple timesteps),
/// the state should be managed externally and passed as context.
pub trait Observation<S: State> {
    /// The type of observation values produced by this operator.
    ///
    /// Must be hashable, comparable, cloneable, and thread-safe.
    /// Common choices: `Vec<u8>`, `u8`, `(Vec<u8>, Vec<u8>)`.
    type Output: Clone + Eq + Hash + Send + Sync + Debug;

    /// Observe a state and return an observation value.
    ///
    /// The observation captures what an external observer can
    /// perceive about the state. Two states that are distinct
    /// may produce the same observation — this is partial
    /// observability, not a bug.
    fn observe(&self, state: &S) -> Self::Output;
}

/// The identity observation operator for any state type.
///
/// Returns the canonical encoding — the maximally dynamically
/// sufficient observation. Distinguishes every distinct state.
/// Use this as the baseline when testing whether coarser
/// observations are sufficient.
///
/// # Example
///
/// ```rust
/// use arco::state::State;
/// use arco::observation::{IdentityObserver, Observation};
///
/// #[derive(Clone, PartialEq, Eq, Debug, Hash)]
/// struct MyState { data: Vec<u8> }
/// impl State for MyState {
///     type Encoding = Vec<u8>;
///     fn canonical_encoding(&self) -> Self::Encoding { self.data.clone() }
///     fn distance(&self, other: &Self) -> u32 {
///         self.data.iter().zip(other.data.iter())
///             .map(|(a, b)| if a != b { 1 } else { 0 }).sum()
///     }
/// }
///
/// let state = MyState { data: vec![0, 1, 0, 1] };
/// // For any State type, IdentityObserver returns the canonical encoding.
/// let observer = IdentityObserver;
/// let encoding = observer.observe(&state);
/// ```
pub struct IdentityObserver;

impl<S: State> Observation<S> for IdentityObserver {
    type Output = S::Encoding;

    fn observe(&self, state: &S) -> Self::Output {
        state.canonical_encoding()
    }
}

/// A windowed observation wraps a single-state observer to observe
/// multiple consecutive states.
///
/// Windowed observation enables detection of temporal patterns
/// (oscillations, propagation delays) that are invisible at the
/// single-step level.
///
/// # Type parameters
///
/// - `S: State` — The state type.
/// - `O: Observation<S>` — The inner single-state observer.
///
/// # Example
///
/// ```rust
/// use arco::state::State;
/// use arco::observation::{WindowedObserver, IdentityObserver};
///
/// #[derive(Clone, PartialEq, Eq, Debug, Hash)]
/// struct MyState { data: Vec<u8> }
/// impl State for MyState {
///     type Encoding = Vec<u8>;
///     fn canonical_encoding(&self) -> Self::Encoding { self.data.clone() }
///     fn distance(&self, other: &Self) -> u32 {
///         self.data.iter().zip(other.data.iter())
///             .map(|(a, b)| if a != b { 1 } else { 0 }).sum()
///     }
/// }
///
/// let state = MyState { data: vec![0, 1, 0, 1] };
/// let observer = WindowedObserver::new(IdentityObserver, 3);
/// // observes all three states
/// observer.observe_window(&[state.clone(), state.clone(), state.clone()]);
/// ```
pub struct WindowedObserver<S: State, O: Observation<S>> {
    inner: O,
    window_size: usize,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: State, O: Observation<S>> WindowedObserver<S, O> {
    /// Create a windowed observer with the given inner observer and
    /// window size.
    ///
    /// # Arguments
    /// * `inner` — The single-state observer applied to each state.
    /// * `window_size` — Number of consecutive states to observe.
    ///   Must be ≥ 1.
    pub fn new(inner: O, window_size: usize) -> Self {
        assert!(window_size >= 1, "Window size must be at least 1");
        Self {
            inner,
            window_size,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Observe a window of consecutive states.
    ///
    /// Returns a concatenation of the inner observer's output for
    /// each state in the window. The window must have exactly
    /// `window_size` elements.
    ///
    /// # Panics
    /// Panics if the window length does not match `window_size`.
    pub fn observe_window(&self, window: &[S]) -> Vec<O::Output> {
        assert_eq!(
            window.len(),
            self.window_size,
            "Window size mismatch: expected {}, got {}",
            self.window_size,
            window.len()
        );
        window.iter().map(|s| self.inner.observe(s)).collect()
    }
}
