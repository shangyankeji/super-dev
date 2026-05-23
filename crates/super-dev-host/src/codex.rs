//! `CodexDriver` — drives the `codex` CLI in non-interactive exec mode.
//!
//! Shells out to `codex exec "<prompt>"`. Like the Claude Code driver,
//! it uses the user's already-authenticated `codex` session — no API
//! key required.
//!
//! Overridable for forward compatibility:
//!
//! - `SUPER_DEV_CODEX_BIN`       — program name (default `codex`)
//! - `SUPER_DEV_CODEX_EXEC_SUBCMD` — exec subcommand (default `exec`)

use std::time::Duration;

use async_trait::async_trait;
use super_dev_runtime::{
    CompletionRequest, CompletionResponse, Runtime, RuntimeError, RuntimeKind, Usage,
};

use crate::{
    default_workspace, merge_prompt, run_subprocess, HostDriver, ProbeResult, PromptChannel,
    SubprocessCall, DEFAULT_TIMEOUT,
};

/// Drives the `codex` CLI as a subprocess.
#[derive(Debug, Clone)]
pub struct CodexDriver {
    program: String,
    exec_subcmd: String,
    timeout: Duration,
}

impl Default for CodexDriver {
    fn default() -> Self {
        Self {
            program: std::env::var("SUPER_DEV_CODEX_BIN").unwrap_or_else(|_| "codex".to_string()),
            exec_subcmd: std::env::var("SUPER_DEV_CODEX_EXEC_SUBCMD")
                .unwrap_or_else(|_| "exec".to_string()),
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

impl CodexDriver {
    /// Build a driver with an explicit program name (mainly for tests).
    #[must_use]
    pub fn with_program(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            ..Self::default()
        }
    }

    /// Override the per-call timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// The argument vector preceding the prompt. Exposed for tests.
    #[must_use]
    pub fn base_args(&self) -> Vec<String> {
        vec![self.exec_subcmd.clone()]
    }
}

#[async_trait]
impl Runtime for CodexDriver {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::Openai
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, RuntimeError> {
        let prompt = merge_prompt(&req);
        let args = self.base_args();
        let out = run_subprocess(SubprocessCall {
            program: &self.program,
            args: &args,
            prompt: &prompt,
            channel: PromptChannel::Arg,
            workspace: &default_workspace(),
            timeout: self.timeout,
        })
        .await
        .map_err(RuntimeError::HostProcess)?;

        Ok(CompletionResponse {
            text: out.stdout,
            id: "codex-cli".to_string(),
            model: req.model,
            usage: Usage::default(),
        })
    }
}

#[async_trait]
impl HostDriver for CodexDriver {
    fn backend_id(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "Codex"
    }

    async fn probe(&self) -> ProbeResult {
        let tmp = default_workspace();
        match run_subprocess(SubprocessCall {
            program: &self.program,
            args: &["--version".to_string()],
            prompt: "",
            channel: PromptChannel::Stdin,
            workspace: &tmp,
            timeout: Duration::from_secs(10),
        })
        .await
        {
            Ok(out) => ProbeResult::Ready {
                version: out.stdout.lines().next().unwrap_or("unknown").to_string(),
            },
            Err(e) if e.contains("not found on PATH") => ProbeResult::NotInstalled {
                program: self.program.clone(),
            },
            Err(e) => ProbeResult::Unhealthy { detail: e },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let d = CodexDriver::default();
        assert_eq!(d.backend_id(), "codex");
        assert_eq!(d.display_name(), "Codex");
        assert_eq!(d.kind(), RuntimeKind::Openai);
        assert_eq!(d.base_args(), vec!["exec".to_string()]);
    }

    #[tokio::test]
    async fn probe_reports_not_installed_for_missing_binary() {
        let d = CodexDriver::with_program("super-dev-fake-codex-xyz");
        assert!(matches!(d.probe().await, ProbeResult::NotInstalled { .. }));
    }

    #[tokio::test]
    async fn complete_drives_a_fake_codex_binary() {
        // `echo` stands in for `codex`: it echoes `exec <prompt>`.
        let d = CodexDriver::with_program("echo");
        let req = CompletionRequest {
            model: "gpt-5-codex".into(),
            system: None,
            messages: vec![super_dev_runtime::Message {
                role: "user".into(),
                content: "generate a migration".into(),
            }],
            max_tokens: None,
            temperature: None,
        };
        let resp = d.complete(req).await.unwrap();
        assert!(resp.text.contains("generate a migration"));
        assert_eq!(resp.model, "gpt-5-codex");
    }
}
