//! Standard hypotheses for the Binary Graph Universe.
//!
//! These hypotheses test whether structural properties of graph
//! rewrite rule sets predict emergent properties (storage, memory).
//!
//! # Key hypotheses
//!
//! - **H5 (Transport Law)**: Rule sets containing transport operations
//!   (PROPAGATE, SWAP, COPY_TO_OUT, COPY_FROM_IN) exhibit storage
//!   at significantly higher rates.
//! - **H6 (Negative control)**: All-destructive rule sets should not
//!   exhibit storage. Its failure validates calibration.

use crate::hypotheses::Hypothesis;
use crate::rules::Rule;
use crate::substrates::graph::rules::RewriteRule;

/// Generate the standard set of 8 hypotheses for the Binary Graph Universe.
pub fn generate_standard_hypotheses() -> Vec<Hypothesis<RewriteRule>> {
    let mut hypotheses = Vec::new();

    // H1: Has structured rule → persistence
    hypotheses.push(Hypothesis::new(
        "H1_HAS_STRUCTURED",
        |rules: &[RewriteRule]| rules.iter().any(|r| r.rule_type() == "structured"),
        "persistence",
        "Rule set contains at least one structured rule",
        1.0,
    ));

    // H2: Majority structured → memory
    hypotheses.push(Hypothesis::new(
        "H2_MAJORITY_STRUCTURED",
        |rules: &[RewriteRule]| {
            if rules.is_empty() {
                return false;
            }
            let n = rules
                .iter()
                .filter(|r| r.rule_type() == "structured")
                .count();
            n as f64 / rules.len() as f64 >= 0.5
        },
        "memory",
        "Majority of rules are structured",
        1.5,
    ));

    // H3: Has logic gate → memory
    hypotheses.push(Hypothesis::new(
        "H3_LOGIC_GATE",
        |rules: &[RewriteRule]| {
            let logic = ["NAND", "NOR", "AND", "OR", "XOR", "NOT"];
            rules.iter().any(|r| logic.contains(&r.name()))
        },
        "memory",
        "Rule set contains a logic gate",
        1.5,
    ));

    // H4: All structured → persistence
    hypotheses.push(Hypothesis::new(
        "H4_ALL_STRUCTURED",
        |rules: &[RewriteRule]| {
            !rules.is_empty() && rules.iter().all(|r| r.rule_type() == "structured")
        },
        "persistence",
        "All rules are structured",
        1.0,
    ));

    // H5: Has transport rule → storage (THE TRANSPORT LAW)
    hypotheses.push(Hypothesis::new(
        "H5_TRANSPORT",
        |rules: &[RewriteRule]| {
            let transport = ["PROPAGATE", "SWAP", "COPY_TO_OUT", "COPY_FROM_IN"];
            rules.iter().any(|r| transport.contains(&r.name()))
        },
        "storage",
        "Rule set contains an information transport rule",
        1.0,
    ));

    // H6: All destructive → persistence (negative control)
    hypotheses.push(Hypothesis::new(
        "H6_ALL_DESTRUCTIVE",
        |rules: &[RewriteRule]| {
            !rules.is_empty() && rules.iter().all(|r| r.rule_type() == "destructive")
        },
        "persistence",
        "All rules are destructive (negative control)",
        0.5,
    ));

    // H7: Multiple logic gates → memory
    hypotheses.push(Hypothesis::new(
        "H7_MULTIPLE_LOGIC",
        |rules: &[RewriteRule]| {
            let logic = ["NAND", "NOR", "AND", "OR", "XOR", "NOT"];
            rules.iter().filter(|r| logic.contains(&r.name())).count() >= 2
        },
        "memory",
        "Rule set contains at least 2 logic gates",
        2.0,
    ));

    // H8: Mixed rules → persistence
    hypotheses.push(Hypothesis::new(
        "H8_MIXED",
        |rules: &[RewriteRule]| {
            if rules.is_empty() {
                return false;
            }
            let ratio = rules
                .iter()
                .filter(|r| r.rule_type() == "structured")
                .count() as f64
                / rules.len() as f64;
            ratio > 0.3 && ratio < 0.7
        },
        "persistence",
        "Mixed structured and destructive rules",
        2.0,
    ));

    hypotheses
}
