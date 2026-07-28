//! Standard hypotheses for the Cellular Automaton substrate.
//!
//! These hypotheses test whether measurable properties of CA rules
//! (reversibility, parity conservation, sensitivity, lambda, evenness
//! of rule number) predict storage. No human semantic labels are used —
//! all conditions are computed from the rule table.

use crate::hypotheses::Hypothesis;
use crate::substrates::ca::rules::CARule;

/// Generate the standard set of structural hypotheses for CA rules.
///
/// All conditions are based on measurable properties computed from
/// the rule table. No human labels or Wolfram class knowledge is used.
pub fn generate_ca_hypotheses<const N: usize, const R: usize>() -> Vec<Hypothesis<CARule<N, R>>> {
    let mut hypotheses = Vec::new();

    // H1: Reversible → Storage
    hypotheses.push(Hypothesis::new(
        "H1_REVERSIBLE",
        |rules: &[CARule<N, R>]| rules.iter().any(|r| r.is_reversible()),
        "storage",
        "Rule is reversible",
        1.0,
    ));

    // H2: Parity-conserving → Storage
    hypotheses.push(Hypothesis::new(
        "H2_PARITY",
        |rules: &[CARule<N, R>]| rules.iter().any(|r| r.conserves_parity()),
        "storage",
        "Rule conserves parity",
        1.0,
    ));

    // H3: Low sensitivity → Storage
    hypotheses.push(Hypothesis::new(
        "H3_LOW_SENSITIVITY",
        |rules: &[CARule<N, R>]| rules.iter().any(|r| r.sensitivity() < 2.0),
        "storage",
        "Rule has low sensitivity (< 2.0)",
        1.5,
    ));

    // H4: Even rule number → Storage (Wolfram's observation)
    hypotheses.push(Hypothesis::new(
        "H4_EVEN_RULE",
        |rules: &[CARule<N, R>]| {
            rules
                .iter()
                .any(|r| r.wolfram_number().is_some_and(|wn| wn % 2 == 0))
        },
        "storage",
        "Rule has even Wolfram number",
        1.0,
    ));

    // H5: Not Rule 0 → Storage (weak control)
    hypotheses.push(Hypothesis::new(
        "H5_NOT_RULE_0",
        |rules: &[CARule<N, R>]| rules.iter().any(|r| r.wolfram_number() != Some(0)),
        "storage",
        "Rule is not the zero rule",
        0.5,
    ));

    // H6: Mid-lambda → Storage (Langton's edge of chaos)
    hypotheses.push(Hypothesis::new(
        "H6_MID_LAMBDA",
        |rules: &[CARule<N, R>]| {
            rules.iter().any(|r| {
                let lambda = r.lambda();
                lambda > 0.2 && lambda < 0.8
            })
        },
        "storage",
        "Rule has mid-range lambda (edge of chaos)",
        1.5,
    ));

    hypotheses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypotheses_count() {
        let hyps = generate_ca_hypotheses::<8, 1>();
        assert_eq!(hyps.len(), 6);
    }

    #[test]
    fn test_h1_detects_reversible() {
        let rule15 = CARule::<4, 1>::from_wolfram_number(15);
        let hyps = generate_ca_hypotheses::<4, 1>();
        let h1 = hyps.iter().find(|h| h.name == "H1_REVERSIBLE").unwrap();
        assert!((h1.condition_fn)(&[rule15]));
    }

    #[test]
    fn test_h4_even_rule() {
        let even_rule = CARule::<8, 1>::from_wolfram_number(110);
        let odd_rule = CARule::<8, 1>::from_wolfram_number(111);

        let hyps = generate_ca_hypotheses::<8, 1>();
        let h4 = hyps.iter().find(|h| h.name == "H4_EVEN_RULE").unwrap();

        assert!((h4.condition_fn)(&[even_rule]));
        assert!(!(h4.condition_fn)(&[odd_rule]));
    }
}
