//! Research record — the output of a scientific cycle.
//!
//! Per the Mathematical Constitution:
//!     Each cycle produces a Research Record containing universes
//!     generated, metrics computed, hypotheses proposed and tested,
//!     and surviving rules.
//!
//! # Design
//!
//! The record is generic over any [`InformationUniverse`] type.
//! It stores per-universe results, hypothesis outcomes, boolean
//! validation, calibration data, and failure conditions.
//!
//! The record is designed for:
//! - **Reproducibility**: All parameters and seeds are stored.
//! - **Auditability**: Every number is traceable to a specific
//!   universe and hypothesis.
//! - **Serialization**: The record can be serialized to JSON for
//!   external analysis and archival.
//!
//! # Serialization
//!
//! Enable the `serialize` feature to serialize research records
//! to JSON for external analysis and archival:
//!
//! ```toml
//! [dependencies]
//! arco = { version = "0.3", features = ["serialize"] }
//! ```

use crate::hypotheses::Hypothesis;
use crate::universe::InformationUniverse;
use std::collections::HashMap;

// ===================================================================
// Research Record
// ===================================================================

/// The output of one complete scientific cycle.
///
/// Contains all data needed to reproduce, audit, or extend the
/// experimental results.
///
/// # Type parameters
///
/// - `U: InformationUniverse` — The universe type this record
///   describes.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct ResearchRecord<U: InformationUniverse> {
    /// ARCO version string.
    pub version: String,
    /// Wall-clock duration in seconds.
    pub elapsed_seconds: f64,
    /// Configuration parameters.
    pub config: HashMap<String, String>,
    /// Calibrated emergence thresholds.
    pub thresholds: HashMap<String, f64>,
    /// Per-universe results.
    pub results: Vec<UniverseResult>,
    /// Hypothesis records.
    pub hypotheses: Vec<HypothesisRecord>,
    /// Boolean function verifications (gate name → count).
    pub boolean_verifications: HashMap<String, usize>,
    /// Triggered failure conditions.
    pub failure_conditions: Vec<String>,
    /// Phantom to bind the universe type parameter.
    _phantom: std::marker::PhantomData<U>,
}

/// Per-universe experimental result.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct UniverseResult {
    /// Universe index in the experiment.
    pub universe_id: usize,
    /// Fraction of rules that are structured (0.0 to 1.0).
    pub structured_ratio: f64,
    /// Total number of rules in this universe.
    pub n_rules: usize,
    /// Names of rules in this universe.
    pub rule_names: Vec<String>,
    /// Persistence score.
    pub persistence: f64,
    /// Storage score.
    pub storage: f64,
    /// Memory score.
    pub memory: f64,
}

/// Serialized hypothesis record for the research output.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct HypothesisRecord {
    /// Hypothesis identifier.
    pub name: String,
    /// Human-readable condition description.
    pub condition_desc: String,
    /// Which emergence property was predicted.
    pub property_name: String,
    /// MDL complexity penalty weight.
    pub complexity: f64,
    /// Fraction of test universes where the prediction held.
    pub accuracy: f64,
    /// Accuracy minus complexity penalty.
    pub score: f64,
    /// Whether the hypothesis survived.
    pub survives: bool,
}

impl<R> From<&Hypothesis<R>> for HypothesisRecord {
    fn from(h: &Hypothesis<R>) -> Self {
        Self {
            name: h.name.clone(),
            condition_desc: h.condition_desc.clone(),
            property_name: h.property_name.clone(),
            complexity: h.complexity,
            accuracy: h.accuracy,
            score: h.score,
            survives: h.survives(),
        }
    }
}

impl<U: InformationUniverse> ResearchRecord<U> {
    /// Create a new empty research record.
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            elapsed_seconds: 0.0,
            config: HashMap::new(),
            thresholds: HashMap::new(),
            results: Vec::new(),
            hypotheses: Vec::new(),
            boolean_verifications: HashMap::new(),
            failure_conditions: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Number of universes above the persistence threshold.
    pub fn n_persistent(&self) -> usize {
        let threshold = self
            .thresholds
            .get("persistence")
            .copied()
            .unwrap_or(f64::MAX);
        self.results
            .iter()
            .filter(|r| r.persistence > threshold)
            .count()
    }

    /// Number of universes above the storage threshold.
    pub fn n_storage(&self) -> usize {
        let threshold = self.thresholds.get("storage").copied().unwrap_or(f64::MAX);
        self.results
            .iter()
            .filter(|r| r.storage > threshold)
            .count()
    }

    /// Number of universes above the memory threshold.
    pub fn n_memory(&self) -> usize {
        let threshold = self.thresholds.get("memory").copied().unwrap_or(f64::MAX);
        self.results.iter().filter(|r| r.memory > threshold).count()
    }

    /// Human-readable summary of the cycle results.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("ARCO v{} — Scientific Cycle Report", self.version));
        lines.push("=".repeat(60));
        lines.push(format!("Universes: {}", self.results.len()));
        lines.push(format!("Duration:  {:.1}s", self.elapsed_seconds));
        lines.push(String::new());
        lines.push("Emergence (above calibrated thresholds):".to_string());
        lines.push(format!(
            "  Storage: {}/{} ({:.1}%)",
            self.n_storage(),
            self.results.len(),
            100.0 * self.n_storage() as f64 / self.results.len() as f64,
        ));
        lines.push(format!(
            "  Memory:  {}/{} ({:.1}%)",
            self.n_memory(),
            self.results.len(),
            100.0 * self.n_memory() as f64 / self.results.len() as f64,
        ));
        lines.push(String::new());
        lines.push(format!("Hypotheses tested: {}", self.hypotheses.len()));
        let surviving: Vec<&HypothesisRecord> =
            self.hypotheses.iter().filter(|h| h.survives).collect();
        lines.push(format!("Hypotheses survived: {}", surviving.len()));

        if !surviving.is_empty() {
            lines.push(String::new());
            lines.push("Surviving hypotheses:".to_string());
            for h in surviving {
                lines.push(format!(
                    "  {}: {} (acc={:.3}, score={:.3})",
                    h.name, h.condition_desc, h.accuracy, h.score,
                ));
            }
        }

        if !self.boolean_verifications.is_empty() {
            lines.push(String::new());
            lines.push("Boolean functions verified:".to_string());
            let mut gates: Vec<(&String, &usize)> = self.boolean_verifications.iter().collect();
            gates.sort_by(|a, b| a.0.cmp(b.0));
            for (gate, count) in gates {
                lines.push(format!("  {}: {}", gate, count));
            }
        }

        if !self.failure_conditions.is_empty() {
            lines.push(String::new());
            lines.push("FAILURE CONDITIONS TRIGGERED:".to_string());
            for fc in &self.failure_conditions {
                lines.push(format!("  ! {}", fc));
            }
        }

        lines.join("\n")
    }
}

impl<U: InformationUniverse> Default for ResearchRecord<U> {
    fn default() -> Self {
        Self::new(env!("CARGO_PKG_VERSION"))
    }
}
