# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and version numbers follow [Semantic Versioning](https://semver.org/)
(pre-1.0: a minor version bump may include breaking changes).

## Yanked versions

**All versions prior to 0.5.0 have been yanked from crates.io.** They
exposed non-default MI estimators (`--estimator mm`, `--estimator nsb`)
with correctness bugs described under 0.5.0 below. If you depended on
one of these versions:

- If you only used the default estimator (plugin with shuffle
  correction — i.e. you never passed `--estimator mm` or
  `--estimator nsb`, and never constructed a `MetricConfig` with a
  non-default `estimator` field), your results are **not** affected
  by either bug.
- If you used `mm` or `nsb`, treat those results as unreliable and
  re-run under 0.5.0 or later.

## [0.5.0] - 2026-08-03

### Added

- `Estimator` enum (`Plugin`, `MillerMadow`, `QE`) and `MetricConfig`,
  threaded consistently through `calibrate_thresholds` and the main
  scoring loop in `run_cycle`, so calibration and observed-universe
  scoring always use the same estimator.
- `--estimator <plugin|mm|qe>` CLI flag. Default remains `plugin`.
- Miller-Madow bias-corrected entropy/MI estimator (`src/metrics/mm.rs`).
- Quadratic extrapolation (QE) bias-corrected MI estimator
  (`src/metrics/qe.rs`), following Strong, Koberle, de Ruyter van
  Steveninck, Bialek (1998).
- `src/metrics/` split into focused submodules: `entropy`, `mm`, `qe`,
  `persistence`, `separation`, `shuffle`, `storage`.

### Fixed

- **Miller-Madow formula.** The MI correction term used
  `(m_x - 1)(m_y - 1) / (2N ln 2)`, where `m_x`, `m_y` are the number
  of *observed* distinct values — the degrees-of-freedom term for a
  fully dense contingency table. The standard correction for mutual
  information is `(K_xy - K_x - K_y + 1) / (2N ln 2)`, where `K_xy` is
  the number of *observed* joint pairs. The two are only equal when
  every possible (x, y) combination has been observed. In the
  undersampled, large-alphabet regime this crate operates in, the old
  formula massively over-corrected — on the graph substrate
  (4096-symbol alphabet) it drove hypothesis survival to 0/10 across
  all seeds. Fixed to use the standard formula; graph-substrate MM
  survival is now 10/10, in line with plugin and QE.
- **QE subsampling independence.** The subsampling RNG seed was
  derived from the caller-supplied `seed` alone. `storage()` passes
  `config.seed + delta`, identical for every universe scored at a
  given timescale — so every distinct dataset sharing a sequence
  length received the *same* "random" subsample rather than an
  independent draw, undermining the extrapolation's statistical
  basis. Now derives the subsampling seed from a hash of the actual
  `(x_seq, y_seq)` content combined with the caller's seed, so
  distinct datasets get independent draws regardless of what seed the
  caller passes; identical inputs remain fully reproducible. Also now
  averages `N_RESAMPLES = 3` independent subsamples at each fraction
  below 1.0, rather than a single arbitrary draw. Graph-substrate
  survival under this estimator went from 9/10 to 10/10; cross-
  estimator agreement (plugin / MM / QE) tightened from within 3
  points to within 2 points across both substrates and all 10 seeds.

### Changed

- **`Estimator::Nsb` renamed to `Estimator::QE`** (`--estimator nsb`
  is now `--estimator qe`). The implementation is Quadratic
  Extrapolation (Strong et al., 1998), not the Nemenman-Shafee-Bialek
  estimator (Nemenman, Shafee, Bialek, 2002) it was previously
  labeled as and cited against. This crate does not implement NSB.
  Do not cite results computed with this estimator as NSB.

## Prior to 0.5.0

Not individually documented — see "Yanked versions" above.
