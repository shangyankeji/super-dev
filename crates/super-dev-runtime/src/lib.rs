//! `super-dev-runtime` — the `Runtime` trait that every "brain" of the
//! pipeline implements.
//!
//! Super Dev is the **project manager**, not an LLM client. It does not
//! own a model endpoint. The actual coding work happens in the user's
//! already-logged-in host CLI (`claude` / `codex`), driven from
//! [`super-dev-host`]. This crate just defines the trait those drivers
//! implement, plus an [`OfflineRuntime`] that returns empty bodies so
//! the pipeline falls back to deterministic templates when no host CLI
//! is selected.
//!
//! [`super-dev-host`]: super-dev-host

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::must_use_candidate
)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use super_dev_spec::RuntimeKind;

/// A single message in a runtime conversation. Wire format is normalised
/// to plain text + role; this struct does not model tool calls or
/// multi-modal parts — those live in the host CLI on the other side of
/// `super-dev-host` and never touch this crate.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// `system` | `user` | `assistant`.
    pub role: String,
    /// Plain text body.
    pub content: String,
}

/// Request body the agent hands to a runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model identifier; format is provider-specific (drivers may
    /// ignore this — `claude --print` decides its own model).
    pub model: String,
    /// Conversation so far, oldest → newest.
    pub messages: Vec<Message>,
    /// Optional max-tokens cap; drivers may ignore.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Optional sampling temperature; drivers may ignore.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Optional system prompt; host drivers merge it into the user
    /// prompt because the host CLIs only take one prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

/// Response a runtime returns.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Plain-text completion (host CLI's stdout, cleaned).
    pub text: String,
    /// Identifier — driver-specific ("claude-code-cli", "codex-cli", "offline").
    pub id: String,
    /// Effective model name reported by the backend (or whatever the
    /// caller asked for).
    pub model: String,
    /// Approximate token usage; defaults to zero when the backend does
    /// not report it (host CLIs typically do not).
    #[serde(default)]
    pub usage: Usage,
}

/// Approximate token usage. Fields default to 0 when the backend does
/// not report them.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens consumed from the request.
    pub input_tokens: u32,
    /// Tokens produced in the response.
    pub output_tokens: u32,
}

/// Errors any runtime may return.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Response body could not be parsed as JSON (rarely surfaces — host
    /// drivers return plain text and never decode JSON).
    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
    /// Required configuration missing.
    #[error("config: {0}")]
    Config(String),
    /// A host-CLI subprocess driver failed (spawn, timeout, non-zero exit).
    /// Emitted by `super-dev-host` drivers wrapping `claude` / `codex`.
    #[error("host process: {0}")]
    HostProcess(String),
}

/// The contract every backend implements.
///
/// In practice the only implementations are:
/// - [`OfflineRuntime`] in this crate — returns empty bodies.
/// - `ClaudeCodeDriver` / `CodexDriver` in `super-dev-host` — drive a
///   logged-in host CLI as a subprocess.
#[async_trait]
pub trait Runtime: Send + Sync {
    /// Stable runtime kind (drives audit identifiers).
    fn kind(&self) -> RuntimeKind;

    /// One completion turn.
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, RuntimeError>;
}

/// Lets a boxed runtime be used wherever a concrete `Runtime` is
/// expected — e.g. `AgentRunner<Box<dyn Runtime>>`, which the TUI uses
/// to pick its brain (offline / host CLI) at runtime.
#[async_trait]
impl Runtime for Box<dyn Runtime> {
    fn kind(&self) -> RuntimeKind {
        (**self).kind()
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, RuntimeError> {
        (**self).complete(req).await
    }
}

/// A runtime that never touches the network: every completion returns
/// an empty body, so the pipeline falls back to deterministic
/// templates. This is what gets used when no host CLI is selected.
#[derive(Debug, Clone, Copy)]
pub struct OfflineRuntime {
    kind: RuntimeKind,
}

impl OfflineRuntime {
    /// Build an offline runtime that reports `kind` (for audit labels).
    #[must_use]
    pub fn new(kind: RuntimeKind) -> Self {
        Self { kind }
    }
}

impl Default for OfflineRuntime {
    fn default() -> Self {
        Self {
            kind: RuntimeKind::Anthropic,
        }
    }
}

#[async_trait]
impl Runtime for OfflineRuntime {
    fn kind(&self) -> RuntimeKind {
        self.kind
    }

    async fn complete(&self, _req: CompletionRequest) -> Result<CompletionResponse, RuntimeError> {
        Ok(CompletionResponse {
            text: String::new(),
            id: "offline".to_string(),
            model: "offline".to_string(),
            usage: Usage::default(),
        })
    }
}
