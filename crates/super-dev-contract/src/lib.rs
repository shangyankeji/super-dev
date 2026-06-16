//! `super-dev-contract` — the machine-verifiable API contract layer.
//!
//! Replaces the free-form Markdown API table
//! (`| Method | Path | Request | Response | Auth | Description |`) in
//! `output/<slug>-architecture.md` with a structured OpenAPI 3.1 document
//! that is the single source of truth for SD-CODE-003 (frontend↔backend API
//! path alignment).
//!
//! ## What this crate does
//! - [`api_spec`] — the in-memory contract: a set of endpoints, each with
//!   method, path template, operationId, request/response shapes, security.
//! - [`parse`] — extract an [`ApiSpec`] from the architecture markdown doc
//!   (upgrading the existing `|`-splitting parser to typed validation).
//! - [`render_json`] / [`render_yaml`] — emit `openapi.json` / `openapi.yaml`
//!   to `.super-dev/contracts/` (SD-CODE-003 references this location).
//! - [`extract_frontend_calls`] — scan the worker's produced frontend source
//!   for `fetch()` / `axios` calls, returning typed `(method, path)` tuples
//!   (upgrading `extract_api_urls` which returned bare path strings).
//! - [`validate`] — cross-check frontend calls + PRD routes against the
//!   contract, returning structured violations instead of substring matches.
//!
//! ## Why not depend on `oas3`?
//! `oas3` 0.22 pulls `yaml_serde` + a large ICU normaliser tree and gates
//! YAML parsing behind a feature flag. Super Dev's contract layer needs only
//! a typed subset (paths/methods/operationId/response codes) for validation —
//! not full OpenAPI deserialisation. A self-contained model keeps this crate
//! dependency-light (matching `super-dev-spec` / `super-dev-governance`) and
//! the emitted `openapi.yaml` is a stable, reviewable artifact.
//!
//! ## Safety contract
//! Fail-open: a malformed architecture doc yields an empty `ApiSpec`, never
//! an error — the quality gate then reports "no contract found", which is
//! the same outcome as today.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::format_push_string,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::unnecessary_map_or,
    clippy::case_sensitive_file_extension_comparisons
)]

pub mod derive;
pub mod extract;
pub mod parse;
pub mod render;
pub mod validate;

pub use derive::{
    derive_endpoints_from_requirement, extract_entities, fields_for_entity, merge_specs,
};
pub use extract::{extract_frontend_calls, FrontendCall};
pub use parse::{parse_architecture, ApiSpec, Endpoint, HttpVerb, SecurityKind};
pub use render::{render_json, render_yaml, write_contract};
pub use validate::{
    extract_prd_routes, validate_frontend_vs_contract, validate_prd_vs_contract, ContractViolation,
};

/// On-disk directory for contract artifacts, relative to the project root.
/// `openapi.yaml` + `openapi.json` live here. SD-CODE-003 references this.
pub const CONTRACT_DIR: &str = ".super-dev/contracts";
