//! Example 03 — Implement Observation for a custom type.
//!
//! The `Observation` trait requires:
//! - An `Output` type (hashable and comparable)
//! - An `observe()` method (state → output)
//!
//! We implement three observers for our Counter at different
//! granularities, showing how the same universe can appear to
//! have different levels of emergence depending on the observer.
//!
//! # Design principle
//!
//! The observer is the **epistemic lens** — it determines what
//! information about the state is visible. A coarse observer
//! (e.g., only seeing whether the value is zero or not) may miss
//! information that a fine-grained observer (e.g., seeing the
//! exact value) captures. This is the observer-relative nature
//! of emergence, made explicit.

use arco::observation::Observation;
use arco::state::State;
use std::fmt;
use std::hash::{Hash, Hasher};

// ===================================================================
// Our state (same as examples 01–02)
// ===================================================================

#[derive(Clone)]
struct Counter {
    value: u8,
    max_value: u8,
}

impl Counter {
    fn new(value: u8, max_value: u8) -> Self {
        assert!(value <= max_value);
        Self { value, max_value }
    }

    #[allow(dead_code)]
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
}

impl State for Counter {
    type Encoding = Vec<u8>;
    fn canonical_encoding(&self) -> Self::Encoding {
        vec![self.value, self.max_value]
    }
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
// Observer 1: Full state (identity observation)
// ===================================================================

/// Observes the complete state: both value and max_value.
///
/// This is the maximally dynamically sufficient observer —
/// it distinguishes every distinct state. Use this as the
/// baseline when testing whether coarser observers are sufficient.
#[derive(Debug, Clone, Default)]
struct FullObserver;

impl Observation<Counter> for FullObserver {
    /// Output is the full canonical encoding.
    type Output = Vec<u8>;

    fn observe(&self, state: &Counter) -> Self::Output {
        state.canonical_encoding()
    }
}

// ===================================================================
// Observer 2: Value only (coarse)
// ===================================================================

/// Observes only the current value, ignoring max_value.
///
/// Two counters with the same value but different max_value
/// will look identical to this observer. If the dynamics
/// depend on max_value, this observer may be dynamically
/// insufficient — it misses information that matters.
#[derive(Debug, Clone, Default)]
struct ValueObserver;

impl Observation<Counter> for ValueObserver {
    type Output = Vec<u8>;

    fn observe(&self, state: &Counter) -> Self::Output {
        vec![state.value]
    }
}

// ===================================================================
// Observer 3: Non-zero check (very coarse)
// ===================================================================

/// Observes only whether the value is zero or non-zero.
///
/// This is extremely coarse — it collapses all non-zero states
/// into a single observation. Useful as a negative control:
/// if storage is still detected with this observer, the
/// information preservation is extremely robust.
#[derive(Debug, Clone, Default)]
struct NonZeroObserver;

impl Observation<Counter> for NonZeroObserver {
    type Output = Vec<u8>;

    fn observe(&self, state: &Counter) -> Self::Output {
        vec![if state.value == 0 { 0 } else { 1 }]
    }
}

// ===================================================================
// Use our observers
// ===================================================================

fn main() {
    println!("=== Example 03: Custom Observers ===\n");

    let s1 = Counter::new(0, 5);
    let s2 = Counter::new(3, 5);
    let s3 = Counter::new(3, 10); // same value, different max
    let s4 = Counter::new(5, 5); // non-zero value

    let full = FullObserver;
    let value = ValueObserver;
    let zero = NonZeroObserver;

    // Full observer: distinguishes everything
    println!("Full observer:");
    println!("  s1 (0/5): {:?}", full.observe(&s1));
    println!("  s2 (3/5): {:?}", full.observe(&s2));
    println!("  s3 (3/10): {:?}", full.observe(&s3));
    println!("  s1 == s2? {}", full.observe(&s1) == full.observe(&s2)); // false

    // Value observer: ignores max_value
    println!("\nValue observer:");
    println!("  s1 (0/5): {:?}", value.observe(&s1));
    println!("  s2 (3/5): {:?}", value.observe(&s2));
    println!("  s3 (3/10): {:?}", value.observe(&s3));
    println!("  s2 == s3? {}", value.observe(&s2) == value.observe(&s3)); // true — same value

    // Zero observer: only cares about zero vs non-zero
    println!("\nZero observer:");
    println!("  s1 (0/5): {:?}", zero.observe(&s1));
    println!("  s2 (3/5): {:?}", zero.observe(&s2));
    println!("  s4 (5/5): {:?}", zero.observe(&s4));
    println!("  s2 == s4? {}", zero.observe(&s2) == zero.observe(&s4)); // true — both non-zero

    // Dynamic sufficiency check
    println!("\nDynamic sufficiency:");
    println!("  Full observer: {} distinct values", 4);
    println!("  Value observer: {} distinct values", 3); // s2==s3 collapsed
    println!("  Zero observer:  {} distinct values", 2); // s2==s3==s4 collapsed
    println!("  Coarser observers lose information.");
    println!("  Whether this matters depends on the dynamics.");

    println!("\n✓ Three observers at different granularities.");
}
