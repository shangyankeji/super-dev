//! `super-dev-agent` — the spec-aware orchestrator.
//!
//! Drives the `SUPER_DEV_HOST_SPEC_V1` 9-phase pipeline (research → docs
//! → `docs_confirm` → spec → frontend → `preview_confirm` → backend →
//! quality → delivery), honours both confirmation gates, and emits the
//! Layer-4 evidence chain along the way.
//!
//! V1 skeleton — `runner.rs` will be fleshed out as the runtime
//! integration lands. The shape stabilises now so downstream crates
//! (CLI, CI plugins) can already import it.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::format_push_string,
    clippy::unused_async,
    clippy::ref_option,
    clippy::single_char_pattern
)]

pub mod coach;
pub mod config;
pub mod events;
pub mod experts;
pub mod gates;
pub mod manifest;
pub mod phases;
pub mod runner;
pub mod state;
pub mod verify;

pub use events::{ChannelSink, EngineEvent, EventSink, NullSink, RecordingSink};
pub use gates::{classify_reply, Gate, GateOutcome};
pub use manifest::{ConformanceLevel, Profile, SpecManifest};
pub use phases::{knowledge_top_files, phase_knowledge_digest, PhaseOutput};
pub use runner::{AgentRunner, RunOptions, RunReport};
pub use state::{read_workflow_state, write_workflow_state, WorkflowState};
pub use verify::{detect_project, run_verify, ProjectKind, VerifyOutcome};
