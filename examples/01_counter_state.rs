//! Example 01 — Implement State for a custom type.
//!
//! This is the first step to using ARCO with your own system.
//! The `State` trait requires:
//! - A canonical encoding (deterministic, unique, hashable)
//! - A distance metric (identity, symmetry, triangle inequality)
//!
//! We implement it for a simple counter that wraps at a maximum value.
//!
//! # Design principle
//!
//! The state is **data**, not behavior. It provides reusable operations
//! (like `increment`) that rules can call. Rules own the decision logic
//! — they decide *when* and *whether* to apply an operation. The state
//! provides the *mechanism*. This keeps rules composable and states
//! reusable across different rule sets.

use arco::state::State;
use std::fmt;
use std::hash::{Hash, Hasher};

// ===================================================================
// Our custom state: a modular counter
// ===================================================================

/// A counter that increments from 0 to `max_value` and wraps around.
///
/// This is a minimal state type — just one byte of data. Real states
/// can be graphs, tensors, strings, or any mathematical structure.
#[derive(Clone)]
struct Counter {
    /// Current value (0..=max_value).
    value: u8,
    /// Maximum value before wrapping to 0.
    max_value: u8,
}

impl Counter {
    fn new(value: u8, max_value: u8) -> Self {
        assert!(value <= max_value);
        Self { value, max_value }
    }

    /// Reusable operation: increment with wrap-around.
    /// Rules call this; the rule decides *when* to increment.
    fn increment(&self) -> Self {
        if self.value >= self.max_value {
            Self { value: 0, ..*self }
        } else {
            Self {
                value: self.value + 1,
                ..*self
            }
        }
    }

    /// Reusable operation: reset to zero.
    fn reset(&self) -> Self {
        Self { value: 0, ..*self }
    }
}

// ===================================================================
// Implement the State trait
// ===================================================================

impl State for Counter {
    /// The canonical encoding type. Must be hashable and comparable.
    /// For simple types, `Vec<u8>` is a good default.
    type Encoding = Vec<u8>;

    /// Return a deterministic, unique representation of the state.
    /// Two states with the same encoding are considered equal.
    ///
    /// Must be deterministic — the same state always produces the
    /// same encoding, across runs and platforms. No timestamps,
    /// no random nonces. This is required for mutual information
    /// estimation, which groups observations by equality.
    fn canonical_encoding(&self) -> Self::Encoding {
        vec![self.value, self.max_value]
    }

    /// Hamming distance: number of bytes that differ between two states.
    ///
    /// Must satisfy the metric axioms:
    /// - d(s, s) == 0 (identity)
    /// - d(a, b) == d(b, a) (symmetry)
    /// - d(a, c) ≤ d(a, b) + d(b, c) (triangle inequality)
    ///
    /// Used for sensitivity analysis and perturbation metrics.
    /// If you don't need it, return 1 for different states and
    /// 0 for identical — that's a valid metric.
    fn distance(&self, other: &Self) -> u32 {
        let mut diff = 0u32;
        if self.value != other.value {
            diff += 1;
        }
        if self.max_value != other.max_value {
            diff += 1;
        }
        diff
    }
}

// Manual implementations required by the State trait bounds.
// These must be consistent with canonical_encoding:
// s1 == s2  iff  s1.canonical_encoding() == s2.canonical_encoding()

impl PartialEq for Counter {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.max_value == other.max_value
    }
}

impl Eq for Counter {}

impl Hash for Counter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.max_value.hash(state);
    }
}

impl fmt::Debug for Counter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Counter({}/{})", self.value, self.max_value)
    }
}

impl fmt::Display for Counter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// ===================================================================
// Test our implementation
// ===================================================================

fn main() {
    println!("=== Example 01: Custom State ===\n");

    // Create two counters
    let s1 = Counter::new(0, 5);
    let s2 = Counter::new(3, 5);
    let s3 = Counter::new(0, 5); // same as s1

    // Canonical encoding
    println!("s1 encoding: {:?}", s1.canonical_encoding());
    println!("s2 encoding: {:?}", s2.canonical_encoding());
    println!("s3 encoding: {:?}", s3.canonical_encoding());

    // Equality — consistent with canonical encoding
    println!("\ns1 == s2: {}", s1 == s2);
    println!("s1 == s3: {}", s1 == s3); // true — same value and max

    // Distance
    println!("\nDistance s1 → s2: {}", s1.distance(&s2)); // 1 (value differs)
    println!("Distance s1 → s3: {}", s1.distance(&s3)); // 0 (identical)
    println!("Distance s1 → s1: {}", s1.distance(&s1)); // 0 (identity axiom)

    // Immutability — operations return new states
    let s4 = s1.increment();
    println!("\ns1 after increment: {} (unchanged)", s1.value);
    println!("s4 after increment: {} (new state)", s4.value);

    // Reusable operations
    let s5 = s1.reset();
    println!("s1 after reset:     {} (unchanged)", s1.value);
    println!("s5 after reset:     {} (new state)", s5.value);

    println!("\n✓ Counter implements State correctly.");
    println!("  The state provides reusable operations (increment, reset).");
    println!("  Rules will decide *when* to call them.");
}
