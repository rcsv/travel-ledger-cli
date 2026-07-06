//! Trip Proposal Envelope / Proposal Fragment — file validation.
//!
//! Separate from schema v8 Trip export validation (`trip validate-export`).

pub mod apply;
pub mod envelope;
pub mod fragment;
pub mod materialize;

pub use apply::{run_fragment_apply, FragmentApplyOptions};
pub use envelope::{run_proposal_inspect, run_proposal_show, run_proposal_validate};
pub use fragment::run_fragment_validate;
pub use materialize::{
    run_proposal_materialize, ProposalMaterializeOptions, ProposalMaterializeParams,
};
