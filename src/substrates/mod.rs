//! Substrate implementations for ARCO.
//!
//! Substrates are concrete implementations of the core traits
//! ([`State`], [`Rule`], [`Observation`], [`Schedule`],
//! [`InformationUniverse`]) for specific kinds of systems.
//!
//! # Available substrates
//!
//! - [`graph`]: Binary Graph Universe — directed graphs with binary
//!   vertex labels and binary edge labels. The validation substrate
//!   used to calibrate ARCO's measurement apparatus. Includes
//!   structured rules (logic gates, transport), destructive rules
//!   (scramblers), the all-vertices asynchronous schedule, and
//!   standard hypotheses including the Transport Law.
//!
//! # Writing your own substrate
//!
//! To add a new substrate, create a module under `substrates/`,
//! implement the core traits for your state, rule, observation,
//! and schedule types, then implement [`InformationUniverse`]
//! to bundle them together. Your substrate works with the full
//! ARCO pipeline — metrics, calibration, hypotheses, and the
//! scientific cycle — without modifying ARCO's core.

pub mod ca;
pub mod graph;
