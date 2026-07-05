//! Trip Proposal Envelope — file validation (P-1).
//!
//! Separate from schema v8 Trip export validation (`trip validate-export`).

pub mod envelope;

pub use envelope::{run_proposal_inspect, run_proposal_show, run_proposal_validate};
