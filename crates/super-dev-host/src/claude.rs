//! `ClaudeCodeDriver` — drives the `claude` CLI in non-interactive
//! print mode.
//!
//! The driver shells out to `claude --print "<prompt>"`. Because the
//! user has already authenticated `claude` (subscription / OAuth), no
//! API key is needed — the host CLI bills the user's existing session.
//!
//! The program name and print flag are overridable for forward
//! compatibility if the CLI's flags change:
//!
//! - `SUPER_DEV_CLAUDE_BIN`  — program name (default `claude`)
//! - `SUPER_DEV_CLAUDE_PRINT_FLAG` — print flag (default `--print`)

use std::time::Duration;

use async_trait::async_trait;
use super_dev_runtime::{
    CompletionRequest, CompletionResponse, Runtime, RuntimeError, RuntimeKind, Usage,
};

use crate::{
    default_workspace, merge_prompt, run_subprocess, HostDriver, ProbeResult, PromptChannel,
    SubprocessCall, DEFAULT_TIMEOUT,
};

/// Drives the `claude` CLI as a subprocess.
#[derive(Debug, Clone)]
pub struct ClaudeCodeDriver {
    program: String,
    print_flag: String,
    timeout: Duration,
}

impl Default for ClaudeCodeDriver {
    fn default() -> Self {
        Self {
            program: std::env::var("SUPER_DEV_CLAUDE_BIN").unwrap_or_else(|_| "claude".to_string()),
            print_flag: std::env::var("SUPER_DEV_CLAUDE_PRINT_FLAG")
                .unwrap_or_else(|_| "--print".to_string()),
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

impl ClaudeCodeDriver {
    /// Build a driver with explicit settings (mainly for tests).
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
    ///
    /// Flag rationale:
    /// - `--print` (or `-p`): non-interactive single-shot mode.
    /// - `--output-format text`: explicit text output — no JSON envelope
    ///   so the existing `clean_output` pipeline gets plain markdown.
    ///
    /// Deliberately **does NOT** pass `--bare`. Anthropic's headless
    /// docs recommend `--bare` for CI, but bare mode skips OAuth and
    /// keychain reads, requiring `ANTHROPIC_API_KEY`. Super Dev's whole
    /// pitch is "drive your already-logged-in subscription", so the
    /// keychain MUST be reachable — `--bare` would break the very
    /// users we exist to serve.
    #[must_use]
    pub fn base_args(&self) -> Vec<String> {
        vec![
            self.print_flag.clone(),
            "--output-format".to_string(),
            "text".to_string(),
        ]
    }
}

#[async_trait]
impl Runtime for ClaudeCodeDriver {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::Anthropic
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
            id: "claude-code-cli".to_string(),
            model: req.model,
            usage: Usage::default(),
        })
    }
}

#[async_trait]
impl HostDriver for ClaudeCodeDriver {
    fn backend_id(&self) -> &'static str {
        "claude-code"
    }

    fn display_name(&self) -> &'static str {
        "Claude Code CLI"
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
        let d = ClaudeCodeDriver::default();
        assert_eq!(d.backend_id(), "claude-code");
        assert_eq!(d.display_name(), "Claude Code CLI");
        assert_eq!(d.kind(), RuntimeKind::Anthropic);
        assert_eq!(
            d.base_args(),
            vec![
                "--print".to_string(),
                "--output-format".to_string(),
                "text".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn probe_reports_not_installed_for_missing_binary() {
        let d = ClaudeCodeDriver::with_program("super-dev-fake-claude-xyz");
        let probe = d.probe().await;
        assert!(matches!(probe, ProbeResult::NotInstalled { .. }));
        assert!(!probe.is_ready());
    }

    #[tokio::test]
    async fn complete_drives_a_fake_claude_binary() {
        // Use `echo` as a stand-in: it ignores --print and echoes the
        // remaining args, proving the driver passes the merged prompt
        // as a positional argument.
        let d = ClaudeCodeDriver::with_program("echo");
        let req = CompletionRequest {
            model: "claude-sonnet-4-6".into(),
            system: Some("be terse".into()),
            messages: vec![super_dev_runtime::Message {
                role: "user".into(),
                content: "ping".into(),
            }],
            max_tokens: None,
            temperature: None,
        };
        let resp = d.complete(req).await.unwrap();
        // echo prints "--print <prompt>"; the driver's clean_output trims it.
        assert!(resp.text.contains("be terse"));
        assert!(resp.text.contains("ping"));
        assert_eq!(resp.model, "claude-sonnet-4-6");
    }

    #[tokio::test]
    async fn complete_surfaces_host_process_error() {
        let d = ClaudeCodeDriver::with_program("super-dev-fake-claude-xyz");
        let req = CompletionRequest {
            model: "m".into(),
            system: None,
            messages: vec![super_dev_runtime::Message {
                role: "user".into(),
                content: "x".into(),
            }],
            max_tokens: None,
            temperature: None,
        };
        let err = d.complete(req).await.unwrap_err();
        assert!(matches!(err, RuntimeError::HostProcess(_)));
    }
}
