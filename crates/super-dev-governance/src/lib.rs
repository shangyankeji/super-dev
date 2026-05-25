//! `super-dev-governance` — the kernel that enforces `SUPER_DEV_HOST_SPEC_V1`.
//!
//! Every Super Dev binding (CLI hook entry, agent runtime, MCP shim,
//! CI evaluator) calls into this crate. Functions in here are the
//! single source of truth for "what counts as commercial-grade output";
//! anything that wants to enforce or audit Super Dev rules MUST use
//! these functions rather than re-implementing the regex.
//!
//! Safety contract:
//! - Every public function fails open: an exceptional input returns
//!   [`Decision::pass`] or an empty audit record. The host MUST NEVER
//!   be blocked by a bug in the governor.
//! - No global mutable state. Every function takes inputs and returns
//!   pure data; callers handle persistence.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::too_many_arguments,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::match_same_arms
)]

pub mod audit;
pub mod compliance;
pub mod context;
pub mod rules;

pub use audit::{
    extract_api_urls, record_api_calls, record_tool_call, ApiCallRecord, ToolCallRecord,
};
pub use compliance::{
    build_compliance_mapping, ClauseEvidence, ComplianceFrameworks, CLAUSE_COMPLIANCE,
};
pub use context::{compose_session_context, SessionContext};
pub use rules::{check_ai_slop, check_color_tokens, check_emoji, Decision};

/// Re-export the spec marker so downstream crates can pin against it.
pub use super_dev_spec::SPEC_VERSION;
