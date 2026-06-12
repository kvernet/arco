//! State trait and BinaryGraphState implementation.
//!
//! Per Constitution:
//!     A state space S is a set equipped with a canonical encoding function,
//!     a distance function, and a cardinality bound. States must be
//!     distinguishable via observation operators.

use bincode::Encode;
use rand::Rng;
use rand::RngExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt;
use std::fmt::Debug;
use std::fmt::Write;
use std::hash::{Hash, Hasher};

// ===================================================================
// State trait
// ===================================================================

/// Trait for states in an Information Universe.
///
/// A state must provide:
/// - A **canonical encoding** that uniquely and deterministically identifies it
/// - A **distance metric** satisfying identity, symmetry, and triangle inequality
/// - **Mutation methods** that return new states rather than modifying in place
///
/// # Design contract
///
/// States are immutable. All mutation methods return new state objects.
/// The canonical encoding must be independent of runtime state (e.g., RNG seed,
/// memory address) and identical across runs.
pub trait State: Clone + Eq + Hash + Send + Sync + Serialize {
    /// The type of the canonical encoding.
    /// Must be hashable, comparable, and serializable.
    type Encoding: Clone + Eq + Hash + Send + Sync + Serialize + Debug + Encode;

    /// Return a deterministic, unique encoding of the complete state.
    ///
    /// This is the identity observation — the maximally dynamically
    /// sufficient observation operator (Constitution Part 1.4.3).
    fn canonical_encoding(&self) -> Self::Encoding;

    /// Hamming distance between this state and another.
    ///
    /// Returns the number of bits that differ between the two states.
    /// Must satisfy the metric axioms: identity, symmetry, triangle inequality.
    fn distance(&self, other: &Self) -> u32;

    /// Return a stable digest of the state (SHA-256 of the canonical encoding).
    ///
    /// Useful for caching, distributed search, and cross-run reproducibility.
    /// Unlike Rust's default Hash, this is deterministic across runs.
    fn stable_digest(&self) -> String {
        let encoding = self.canonical_encoding();
        let bytes = bincode::encode_to_vec(&encoding, bincode::config::standard())
            .unwrap_or_else(|_| format!("{:?}", encoding).into_bytes());
        //let hash = Sha256::digest(&bytes);
        //format!("{:x}", hash)

        let hash = Sha256::digest(&bytes);
        let mut hex = String::new();
        for byte in hash.as_slice() {
            write!(&mut hex, "{:02x}", byte).unwrap();
        }
        hex
    }
}

// ===================================================================
// BinaryGraphState
// ===================================================================

/// A state in a graph-based Information Universe.
///
/// Represents a directed graph with `n` vertices, each vertex labeled
/// `{0, 1}`, each directed edge labeled `{0, 1}`.
///
/// # Encoding
///
/// The canonical encoding is `(adj_flat, labels)` where `adj_flat` is a
/// flattened adjacency matrix (length n²) and `labels` is the vertex label
/// vector (length n). Both are `Vec<u8>` with entries in `{0, 1}`.
///
/// # Vertex-order dependence
///
/// States are **vertex-order dependent**. Two isomorphic graphs with
/// permuted vertex labels are distinct states. Canonical graph isomorphism
/// reduction is deferred to the equivalence layer (Constitution Part 6).
///
/// # Examples
///
/// ```
/// use ndarray::{arr2, arr1};
/// use arco::state::BinaryGraphState;
///
/// let adj = arr2(&[[0, 1], [0, 0]]);
/// let labels = arr1(&[1, 0]);
/// let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
///
/// assert_eq!(state.n_vertices(), 2);
/// assert_eq!(state.label(0), 1);
/// assert_eq!(state.edge(0, 1), 1);
/// ```
#[derive(Clone, Serialize, Encode)]
pub struct BinaryGraphState {
    /// Number of vertices.
    n: usize,
    /// Flattened adjacency matrix, length n*n, entries in {0, 1}.
    adj_flat: Vec<u8>,
    /// Vertex labels, length n, entries in {0, 1}.
    labels: Vec<u8>,
}

impl BinaryGraphState {
    /// Create a new BinaryGraphState with full validation.
    ///
    /// # Arguments
    /// * `n_vertices` — Number of vertices.
    /// * `adj_matrix` — Adjacency matrix of shape (n, n), entries in {0, 1}.
    /// * `vertex_labels` — Vertex labels of shape (n,), entries in {0, 1}.
    ///
    /// # Errors
    /// Returns `Err` if shapes are incorrect or entries are not in {0, 1}.
    pub fn new(
        n_vertices: usize,
        adj_matrix: ndarray::ArrayView2<'_, i8>,
        vertex_labels: ndarray::ArrayView1<'_, i8>,
    ) -> Result<Self, StateError> {
        // Shape validation
        if adj_matrix.shape() != [n_vertices, n_vertices] {
            return Err(StateError::InvalidShape {
                expected: (n_vertices, n_vertices),
                got: (adj_matrix.shape()[0], adj_matrix.shape()[1]),
            });
        }
        if vertex_labels.len() != n_vertices {
            return Err(StateError::InvalidLength {
                expected: n_vertices,
                got: vertex_labels.len(),
            });
        }

        // Binary value validation
        for &val in adj_matrix.iter() {
            if val != 0 && val != 1 {
                return Err(StateError::InvalidValue {
                    context: "adjacency matrix",
                    value: val as i64,
                });
            }
        }
        for &val in vertex_labels.iter() {
            if val != 0 && val != 1 {
                return Err(StateError::InvalidValue {
                    context: "vertex labels",
                    value: val as i64,
                });
            }
        }

        Ok(Self {
            n: n_vertices,
            adj_flat: adj_matrix.iter().map(|&x| x as u8).collect(),
            labels: vertex_labels.iter().map(|&x| x as u8).collect(),
        })
    }

    /// Create a state from already-validated internal data.
    ///
    /// Bypasses validation for performance. Only call with data known to be
    /// valid (e.g., from mutation methods that preserve binary constraints).
    pub(crate) fn from_internal(n_vertices: usize, adj_flat: Vec<u8>, labels: Vec<u8>) -> Self {
        debug_assert_eq!(adj_flat.len(), n_vertices * n_vertices);
        debug_assert_eq!(labels.len(), n_vertices);
        debug_assert!(adj_flat.iter().all(|&x| x <= 1));
        debug_assert!(labels.iter().all(|&x| x <= 1));

        Self {
            n: n_vertices,
            adj_flat,
            labels,
        }
    }

    // --- Accessors ---

    /// Number of vertices.
    pub fn n_vertices(&self) -> usize {
        self.n
    }

    /// Label of a vertex.
    pub fn label(&self, vertex: usize) -> u8 {
        self.labels[vertex]
    }

    /// Edge value from `src` to `dst`.
    pub fn edge(&self, src: usize, dst: usize) -> u8 {
        self.adj_flat[src * self.n + dst]
    }

    /// Total number of edges (sum of adjacency matrix entries).
    pub fn edge_count(&self) -> usize {
        self.adj_flat.iter().filter(|&&x| x == 1).count()
    }

    /// Sum of vertex labels.
    pub fn label_sum(&self) -> usize {
        self.labels.iter().filter(|&&x| x == 1).count()
    }

    // --- Mutation methods ---

    /// Return a new state with one vertex label changed.
    pub fn mutate_label(&self, vertex: usize, value: u8) -> Result<Self, StateError> {
        if vertex >= self.n {
            return Err(StateError::IndexOutOfRange {
                index: vertex,
                max: self.n,
            });
        }
        if value > 1 {
            return Err(StateError::InvalidValue {
                context: "label",
                value: value as i64,
            });
        }

        let mut new_labels = self.labels.clone();
        new_labels[vertex] = value;
        Ok(Self::from_internal(
            self.n,
            self.adj_flat.clone(),
            new_labels,
        ))
    }

    /// Return a new state with all vertex labels replaced.
    pub fn mutate_labels(&self, new_labels: &[u8]) -> Result<Self, StateError> {
        if new_labels.len() != self.n {
            return Err(StateError::InvalidLength {
                expected: self.n,
                got: new_labels.len(),
            });
        }
        if !new_labels.iter().all(|&x| x <= 1) {
            return Err(StateError::InvalidValue {
                context: "labels",
                value: -1, // sentinel
            });
        }

        Ok(Self::from_internal(
            self.n,
            self.adj_flat.clone(),
            new_labels.to_vec(),
        ))
    }

    /// Return a new state with one edge changed.
    pub fn mutate_adj(&self, src: usize, dst: usize, value: u8) -> Result<Self, StateError> {
        if src >= self.n || dst >= self.n {
            return Err(StateError::IndexOutOfRange {
                index: src.max(dst),
                max: self.n,
            });
        }
        if value > 1 {
            return Err(StateError::InvalidValue {
                context: "edge",
                value: value as i64,
            });
        }

        let mut new_adj = self.adj_flat.clone();
        new_adj[src * self.n + dst] = value;
        Ok(Self::from_internal(self.n, new_adj, self.labels.clone()))
    }

    // --- Random generation ---

    /// Generate a random state with the given number of vertices.
    pub fn random(n_vertices: usize, rng: &mut impl Rng) -> Self {
        let n_edges = n_vertices * n_vertices;
        let adj_flat: Vec<u8> = (0..n_edges).map(|_| rng.random_range(0..=1)).collect();
        let labels: Vec<u8> = (0..n_vertices).map(|_| rng.random_range(0..=1)).collect();
        Self::from_internal(n_vertices, adj_flat, labels)
    }
}

// ===================================================================
// Trait implementations
// ===================================================================

impl State for BinaryGraphState {
    type Encoding = (Vec<u8>, Vec<u8>);

    fn canonical_encoding(&self) -> Self::Encoding {
        (self.adj_flat.clone(), self.labels.clone())
    }

    fn distance(&self, other: &Self) -> u32 {
        if self.n != other.n {
            panic!(
                "Cannot compute distance between states with different vertex counts: {} vs {}",
                self.n, other.n
            );
        }

        let mut diff: u32 = 0;
        for (a, b) in self.adj_flat.iter().zip(other.adj_flat.iter()) {
            if a != b {
                diff += 1;
            }
        }
        for (a, b) in self.labels.iter().zip(other.labels.iter()) {
            if a != b {
                diff += 1;
            }
        }
        diff
    }
}

impl PartialEq for BinaryGraphState {
    fn eq(&self, other: &Self) -> bool {
        self.n == other.n && self.adj_flat == other.adj_flat && self.labels == other.labels
    }
}

impl Eq for BinaryGraphState {}

impl Hash for BinaryGraphState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.n.hash(state);
        self.adj_flat.hash(state);
        self.labels.hash(state);
    }
}

impl fmt::Debug for BinaryGraphState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BinaryGraphState(n={}, labels={:?}, edges={})",
            self.n,
            self.labels,
            self.edge_count()
        )
    }
}

impl fmt::Display for BinaryGraphState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ===================================================================
// Error type
// ===================================================================

/// Errors that can occur when creating or mutating a state.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StateError {
    #[error("invalid shape: expected {expected:?}, got {got:?}")]
    InvalidShape {
        expected: (usize, usize),
        got: (usize, usize),
    },

    #[error("invalid length: expected {expected}, got {got}")]
    InvalidLength { expected: usize, got: usize },

    #[error("invalid value in {context}: {value}")]
    InvalidValue { context: &'static str, value: i64 },

    #[error("index {index} out of range [0, {max})")]
    IndexOutOfRange { index: usize, max: usize },
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{arr1, arr2};
    use rand::SeedableRng;

    #[test]
    fn test_new_valid_state() {
        let adj = arr2(&[[0, 1], [0, 0]]);
        let labels = arr1(&[1, 0]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
        assert_eq!(state.n_vertices(), 2);
        assert_eq!(state.label(0), 1);
        assert_eq!(state.label(1), 0);
        assert_eq!(state.edge(0, 1), 1);
        assert_eq!(state.edge(1, 0), 0);
    }

    #[test]
    fn test_new_invalid_shape() {
        let adj = arr2(&[[0, 1]]); // wrong shape
        let labels = arr1(&[1, 0]);
        let result = BinaryGraphState::new(2, adj.view(), labels.view());
        assert!(result.is_err());
    }

    #[test]
    fn test_new_invalid_values() {
        let adj = arr2(&[[0, 2], [0, 0]]); // 2 is invalid
        let labels = arr1(&[1, 0]);
        let result = BinaryGraphState::new(2, adj.view(), labels.view());
        assert!(result.is_err());
    }

    #[test]
    fn test_canonical_encoding_deterministic() {
        let adj = arr2(&[[1, 0], [0, 1]]);
        let labels = arr1(&[0, 1]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
        let enc1 = state.canonical_encoding();
        let enc2 = state.canonical_encoding();
        assert_eq!(enc1, enc2);
    }

    #[test]
    fn test_distance_same_state_is_zero() {
        let adj = arr2(&[[0, 0], [0, 0]]);
        let labels = arr1(&[0, 0]);
        let s1 = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
        let s2 = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
        assert_eq!(s1.distance(&s2), 0);
    }

    #[test]
    fn test_distance_different_labels() {
        let adj = arr2(&[[0, 0], [0, 0]]);
        let s1 = BinaryGraphState::new(2, adj.view(), arr1(&[0, 0]).view()).unwrap();
        let s2 = BinaryGraphState::new(2, adj.view(), arr1(&[1, 0]).view()).unwrap();
        assert_eq!(s1.distance(&s2), 1);
    }

    #[test]
    fn test_mutate_label() {
        let adj = arr2(&[[0, 0], [0, 0]]);
        let labels = arr1(&[0, 0]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
        let new_state = state.mutate_label(0, 1).unwrap();
        assert_eq!(new_state.label(0), 1);
        // Original unchanged
        assert_eq!(state.label(0), 0);
    }

    #[test]
    fn test_stable_digest_deterministic() {
        let adj = arr2(&[[1, 0], [0, 1]]);
        let labels = arr1(&[0, 1]);
        let state = BinaryGraphState::new(2, adj.view(), labels.view()).unwrap();
        let d1 = state.stable_digest();
        let d2 = state.stable_digest();
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_stable_digest_different_for_different_states() {
        let adj = arr2(&[[0, 0], [0, 0]]);
        let s1 = BinaryGraphState::new(2, adj.view(), arr1(&[0, 0]).view()).unwrap();
        let s2 = BinaryGraphState::new(2, adj.view(), arr1(&[1, 0]).view()).unwrap();
        assert_ne!(s1.stable_digest(), s2.stable_digest());
    }

    #[test]
    fn test_random_state_generation() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let state = BinaryGraphState::random(3, &mut rng);
        assert_eq!(state.n_vertices(), 3);
        // All labels should be 0 or 1
        for i in 0..3 {
            assert!(state.label(i) <= 1);
        }
        // All edges should be 0 or 1
        for i in 0..3 {
            for j in 0..3 {
                assert!(state.edge(i, j) <= 1);
            }
        }
    }
}
