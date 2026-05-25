//! `super-dev-host` — drive an already-logged-in host CLI as a subprocess.
//!
//! Super Dev 4.x's primary execution mode does not call any LLM API and
//! does not need an API key. Instead it spawns the host CLI the user has
//! already installed and authenticated (`claude`, `codex`, `gemini`,
//! `droid`, `opencode`, `cursor-agent`, `qwen`, `cn`, `copilot`, `aider`)
//! in non-interactive mode and captures the response.
//!
//! Each driver implements [`super_dev_runtime::Runtime`] so the existing
//! `AgentRunner` machinery drives it unchanged — a host CLI is just
//! another "prompt in, text out" backend.
//!
//! Drivers additionally expose [`HostDriver::probe`] to report whether
//! the underlying CLI is installed + reachable before a run starts.
//!
//! ## Backend matrix (13 hosts)
//!
//! | id              | binary           | non-interactive form                              |
//! |-----------------|------------------|---------------------------------------------------|
//! | `claude-code`   | `claude`         | `claude --print --output-format text "<p>"`       |
//! | `codex`         | `codex`          | `codex exec --skip-git-repo-check --sandbox …`    |
//! | `gemini`        | `gemini`         | `gemini -p "<prompt>"`                            |
//! | `droid`         | `droid`          | `droid exec --auto low -o text "<p>"`             |
//! | `opencode`      | `opencode`       | `opencode run "<prompt>"`                         |
//! | `cursor-agent`  | `cursor-agent`   | `cursor-agent -p --output-format text "<p>"`      |
//! | `qwen`          | `qwen`           | `qwen -p "<prompt>"`                              |
//! | `continue`      | `cn`             | `cn -p "<prompt>"`                                |
//! | `copilot`       | `copilot`        | `copilot -p --allow-all-tools "<p>"`              |
//! | `aider`         | `aider`          | `aider --yes --no-stream --message "<p>"`         |
//! | `trae`          | `trae-cli`       | `trae-cli run "<prompt>"`                         |
//! | `plandex`       | `plandex`        | `plandex tell --skip-menu --stop "<p>"`           |
//! | `cody`          | `cody`           | `cody chat --message "<prompt>"`                  |
//!
//! See [`SimpleHostDriver`] for the generic driver shared by every
//! backend below the top two — and [`ClaudeCodeDriver`] /
//! [`CodexDriver`] for the bespoke ones.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]

pub mod claude;
pub mod codex;
pub mod simple;

use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub use claude::ClaudeCodeDriver;
pub use codex::CodexDriver;
pub use simple::SimpleHostDriver;

/// Outcome of probing a host CLI for availability.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProbeResult {
    /// The CLI is installed and responded to `--version`.
    Ready {
        /// Raw version string the CLI reported.
        version: String,
    },
    /// The CLI binary was not found on `PATH`.
    NotInstalled {
        /// The program name that was looked up.
        program: String,
    },
    /// The CLI was found but behaved unexpectedly (non-zero `--version`).
    Unhealthy {
        /// Human-readable detail.
        detail: String,
    },
}

impl ProbeResult {
    /// `true` when the host CLI is ready to drive.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}

/// Extension trait every host driver implements on top of [`Runtime`].
///
/// [`Runtime`]: super_dev_runtime::Runtime
#[async_trait]
pub trait HostDriver: super_dev_runtime::Runtime {
    /// Stable identifier used as the `--backend` flag value.
    fn backend_id(&self) -> &'static str;

    /// Human-facing name.
    fn display_name(&self) -> &'static str;

    /// Check whether the underlying CLI is installed + reachable.
    async fn probe(&self) -> ProbeResult;
}

/// Let a boxed driver be used wherever a [`Runtime`] is expected — e.g.
/// `AgentRunner::new(driver_for("claude-code").unwrap(), opts)`.
///
/// [`Runtime`]: super_dev_runtime::Runtime
#[async_trait]
impl super_dev_runtime::Runtime for Box<dyn HostDriver> {
    fn kind(&self) -> super_dev_runtime::RuntimeKind {
        (**self).kind()
    }

    async fn complete(
        &self,
        req: super_dev_runtime::CompletionRequest,
    ) -> Result<super_dev_runtime::CompletionResponse, super_dev_runtime::RuntimeError> {
        (**self).complete(req).await
    }
}

/// How a host CLI consumes the prompt.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum PromptChannel {
    /// Prompt is passed as the last positional argument.
    Arg,
    /// Prompt is written to the child's stdin.
    Stdin,
}

/// Shared subprocess plumbing used by every driver.
///
/// Spawns `program` with `args`, optionally feeds `prompt` via stdin or
/// as a trailing argument, enforces `timeout`, and returns the captured
/// stdout (trimmed). Stderr is folded into the error on failure.
pub(crate) struct SubprocessCall<'a> {
    pub program: &'a str,
    pub args: &'a [String],
    pub prompt: &'a str,
    pub channel: PromptChannel,
    pub workspace: &'a std::path::Path,
    pub timeout: Duration,
}

/// What a successful subprocess call produced.
///
/// Wall-clock duration is logged via `tracing` rather than carried on
/// this struct; the TUI (M3) will add a structured timing field when it
/// needs to render a per-host latency panel.
#[derive(Debug, Clone)]
pub(crate) struct SubprocessOutput {
    pub stdout: String,
}

/// Run a host CLI subprocess. Errors carry a human-readable string
/// suitable for `RuntimeError::HostProcess`.
pub(crate) async fn run_subprocess(call: SubprocessCall<'_>) -> Result<SubprocessOutput, String> {
    let started = Instant::now();
    let mut cmd = Command::new(call.program);
    cmd.args(call.args);
    if matches!(call.channel, PromptChannel::Arg) {
        cmd.arg(call.prompt);
    }
    cmd.current_dir(call.workspace);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("`{}` not found on PATH", call.program)
        } else {
            format!("failed to spawn `{}`: {e}", call.program)
        }
    })?;

    if matches!(call.channel, PromptChannel::Stdin) {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(call.prompt.as_bytes())
                .await
                .map_err(|e| format!("failed to write prompt to stdin: {e}"))?;
            // Drop closes the pipe so the child sees EOF.
        }
    }

    let output = match tokio::time::timeout(call.timeout, child.wait_with_output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => return Err(format!("`{}` failed: {e}", call.program)),
        Err(_) => {
            return Err(format!(
                "`{}` timed out after {}s",
                call.program,
                call.timeout.as_secs()
            ))
        }
    };

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let mut stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        stderr.truncate(2048);
        return Err(format!(
            "`{}` exited with code {code}: {}",
            call.program,
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let cleaned = clean_output(&stdout);
    tracing::debug!(
        program = call.program,
        millis = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        bytes = cleaned.len(),
        "host subprocess completed"
    );
    Ok(SubprocessOutput { stdout: cleaned })
}

/// Strip common host-CLI noise: ANSI escape codes and a leading
/// `assistant:` style prefix some CLIs emit.
pub(crate) fn clean_output(raw: &str) -> String {
    let no_ansi = strip_ansi(raw);
    no_ansi.trim().to_string()
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip CSI sequence: ESC [ ... <final byte 0x40-0x7E>
            if chars.peek() == Some(&'[') {
                chars.next();
                for inner in chars.by_ref() {
                    if ('\x40'..='\x7e').contains(&inner) {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// Merge a [`CompletionRequest`]'s system + user messages into a single
/// prompt string for host CLIs that take only one prompt.
///
/// [`CompletionRequest`]: super_dev_runtime::CompletionRequest
#[must_use]
pub(crate) fn merge_prompt(req: &super_dev_runtime::CompletionRequest) -> String {
    let mut buf = String::new();
    if let Some(system) = &req.system {
        buf.push_str(system);
        buf.push_str("\n\n---\n\n");
    }
    for (i, msg) in req.messages.iter().enumerate() {
        if i > 0 {
            buf.push_str("\n\n");
        }
        buf.push_str(&msg.content);
    }
    buf
}

/// Build a driver for the given backend id, or `None` for an unknown id.
#[must_use]
pub fn driver_for(backend_id: &str) -> Option<Box<dyn HostDriver>> {
    match backend_id {
        "claude-code" => Some(Box::new(ClaudeCodeDriver::default())),
        "codex" => Some(Box::new(CodexDriver::default())),
        "droid" => Some(Box::new(SimpleHostDriver::droid())),
        "opencode" => Some(Box::new(SimpleHostDriver::opencode())),
        "gemini" | "antigravity" => Some(Box::new(SimpleHostDriver::gemini())),
        "cursor-agent" => Some(Box::new(SimpleHostDriver::cursor_agent())),
        "qwen" => Some(Box::new(SimpleHostDriver::qwen())),
        "continue" => Some(Box::new(SimpleHostDriver::continue_cli())),
        "copilot" => Some(Box::new(SimpleHostDriver::copilot())),
        "aider" => Some(Box::new(SimpleHostDriver::aider())),
        "trae" => Some(Box::new(SimpleHostDriver::trae())),
        "plandex" => Some(Box::new(SimpleHostDriver::plandex())),
        "cody" => Some(Box::new(SimpleHostDriver::cody())),
        "goose" => Some(Box::new(SimpleHostDriver::goose())),
        "kimi" => Some(Box::new(SimpleHostDriver::kimi())),
        "amp" => Some(Box::new(SimpleHostDriver::amp())),
        "junie" => Some(Box::new(SimpleHostDriver::junie())),
        "grok-build" => Some(Box::new(SimpleHostDriver::grok_build())),
        "amazon-q" => Some(Box::new(SimpleHostDriver::amazon_q())),
        "crush" => Some(Box::new(SimpleHostDriver::crush())),
        "gptme" => Some(Box::new(SimpleHostDriver::gptme())),
        "codebuddy" => Some(Box::new(SimpleHostDriver::codebuddy())),
        "qoder" => Some(Box::new(SimpleHostDriver::qoder())),
        _ => None,
    }
}

/// All backend ids `driver_for` accepts. Order is the canonical display
/// order for the TUI picker — flagship hosts first.
pub const BACKEND_IDS: &[&str] = &[
    "claude-code",
    "codex",
    "gemini",
    "droid",
    "opencode",
    "qwen",
    "copilot",
    "trae",
    "codebuddy",
    "qoder",
    "kimi",
];

/// Default per-call timeout for a host CLI invocation.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// Availability of one host backend, as reported by [`probe_all`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BackendStatus {
    /// Stable backend id (`claude-code` / `codex`).
    pub id: &'static str,
    /// Human-facing name.
    pub display_name: &'static str,
    /// The probe result.
    pub probe: ProbeResult,
}

/// Concurrently probe every known backend. The TUI uses this to render
/// its "backends detected" panel — one `--version` check per host, run
/// in parallel so a slow host never serialises startup.
pub async fn probe_all() -> Vec<BackendStatus> {
    // Probe backends in batches of 5 to avoid spawning too many
    // subprocesses at once (each probe runs `<binary> --version`).
    // 21 concurrent spawns can overwhelm the system on some machines.
    let drivers: Vec<Box<dyn HostDriver>> =
        BACKEND_IDS.iter().filter_map(|id| driver_for(id)).collect();

    let mut results = Vec::with_capacity(drivers.len());
    for chunk in drivers.chunks(5) {
        let mut batch = Vec::with_capacity(chunk.len());
        for d in chunk {
            batch.push(async {
                let probe = d.probe().await;
                BackendStatus {
                    id: d.backend_id(),
                    display_name: d.display_name(),
                    probe,
                }
            });
        }
        let batch_results = futures::future::join_all(batch).await;
        results.extend(batch_results);
    }
    results
}

/// Resolve the workspace a driver should run in. Drivers default to the
/// current directory when a caller does not pin one.
#[must_use]
pub(crate) fn default_workspace() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super_dev_runtime::{CompletionRequest, Message};

    #[test]
    fn strip_ansi_removes_color_codes() {
        let painted = "\x1b[1;32mhello\x1b[0m world";
        assert_eq!(strip_ansi(painted), "hello world");
    }

    #[test]
    fn clean_output_trims_and_strips() {
        let raw = "  \x1b[33m# PRD\x1b[0m\n\nbody  \n";
        assert_eq!(clean_output(raw), "# PRD\n\nbody");
    }

    #[test]
    fn merge_prompt_joins_system_and_user() {
        let req = CompletionRequest {
            model: "m".into(),
            system: Some("SYSTEM".into()),
            messages: vec![Message {
                role: "user".into(),
                content: "USER".into(),
            }],
            max_tokens: None,
            temperature: None,
        };
        let merged = merge_prompt(&req);
        assert!(merged.starts_with("SYSTEM"));
        assert!(merged.contains("---"));
        assert!(merged.ends_with("USER"));
    }

    #[test]
    fn merge_prompt_without_system() {
        let req = CompletionRequest {
            model: "m".into(),
            system: None,
            messages: vec![Message {
                role: "user".into(),
                content: "just user".into(),
            }],
            max_tokens: None,
            temperature: None,
        };
        assert_eq!(merge_prompt(&req), "just user");
    }

    #[test]
    fn driver_for_known_and_unknown() {
        for id in BACKEND_IDS {
            assert!(
                driver_for(id).is_some(),
                "BACKEND_IDS contains `{id}` but driver_for can't build it"
            );
        }
        assert!(driver_for("nope").is_none());
        assert!(driver_for("").is_none());
    }

    #[test]
    fn backend_ids_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for id in BACKEND_IDS {
            assert!(seen.insert(*id), "duplicate id in BACKEND_IDS: {id}");
        }
    }

    #[test]
    fn backend_count_is_eleven() {
        assert_eq!(BACKEND_IDS.len(), 11);
    }

    #[test]
    fn backend_ids_match_driver_for() {
        for id in BACKEND_IDS {
            assert!(
                driver_for(id).is_some(),
                "BACKEND_IDS has unbuildable id {id}"
            );
        }
    }

    #[tokio::test]
    async fn probe_all_reports_every_backend() {
        let statuses = probe_all().await;
        assert_eq!(statuses.len(), BACKEND_IDS.len());
        // Every BACKEND_IDS entry is represented exactly once.
        for id in BACKEND_IDS {
            assert_eq!(
                statuses.iter().filter(|s| s.id == *id).count(),
                1,
                "probe_all missing backend {id}"
            );
        }
        // Each status carries a non-empty display name.
        assert!(statuses.iter().all(|s| !s.display_name.is_empty()));
    }

    #[tokio::test]
    async fn run_subprocess_captures_stdout() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = run_subprocess(SubprocessCall {
            program: "echo",
            args: &[],
            prompt: "hello-from-test",
            channel: PromptChannel::Arg,
            workspace: tmp.path(),
            timeout: Duration::from_secs(5),
        })
        .await
        .unwrap();
        assert_eq!(out.stdout, "hello-from-test");
    }

    #[tokio::test]
    async fn run_subprocess_reports_missing_program() {
        let tmp = tempfile::TempDir::new().unwrap();
        let err = run_subprocess(SubprocessCall {
            program: "super-dev-definitely-not-a-real-binary",
            args: &[],
            prompt: "x",
            channel: PromptChannel::Arg,
            workspace: tmp.path(),
            timeout: Duration::from_secs(5),
        })
        .await
        .unwrap_err();
        assert!(err.contains("not found on PATH"));
    }

    #[tokio::test]
    async fn run_subprocess_feeds_stdin() {
        let tmp = tempfile::TempDir::new().unwrap();
        // `cat` echoes stdin back to stdout.
        let out = run_subprocess(SubprocessCall {
            program: "cat",
            args: &[],
            prompt: "piped-prompt-body",
            channel: PromptChannel::Stdin,
            workspace: tmp.path(),
            timeout: Duration::from_secs(5),
        })
        .await
        .unwrap();
        assert_eq!(out.stdout, "piped-prompt-body");
    }

    #[tokio::test]
    async fn run_subprocess_reports_nonzero_exit() {
        let tmp = tempfile::TempDir::new().unwrap();
        let err = run_subprocess(SubprocessCall {
            program: "sh",
            args: &["-c".into(), "echo boom >&2; exit 3".into()],
            prompt: "",
            channel: PromptChannel::Stdin,
            workspace: tmp.path(),
            timeout: Duration::from_secs(5),
        })
        .await
        .unwrap_err();
        assert!(err.contains("code 3"));
        assert!(err.contains("boom"));
    }
}
