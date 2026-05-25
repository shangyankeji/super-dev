//! `SimpleHostDriver` — a generic driver for any host CLI whose
//! non-interactive form fits the shape
//!
//! ```text
//! <program> [base-args…] "<prompt>"     # prompt as the trailing arg
//! <program> [base-args…] < "<prompt>"   # prompt piped to stdin
//! ```
//!
//! Most modern coding CLIs accept exactly this shape — only the leading
//! flag set and the program name differ. Rather than copy the
//! claude / codex driver pattern eight more times, we describe each
//! backend as a small piece of configuration and let one struct drive
//! them all.
//!
//! The blessed factories live below ([`droid`](SimpleHostDriver::droid),
//! [`opencode`](SimpleHostDriver::opencode), [`gemini`](SimpleHostDriver::gemini),
//! [`cursor_agent`](SimpleHostDriver::cursor_agent),
//! [`qwen`](SimpleHostDriver::qwen),
//! [`continue_cli`](SimpleHostDriver::continue_cli),
//! [`copilot`](SimpleHostDriver::copilot),
//! [`aider`](SimpleHostDriver::aider)). The dedicated `ClaudeCodeDriver`
//! and `CodexDriver` modules are intentionally kept separate because
//! they shipped first and have callers in the public surface.

use std::time::Duration;

use async_trait::async_trait;
use super_dev_runtime::{
    CompletionRequest, CompletionResponse, Runtime, RuntimeError, RuntimeKind, Usage,
};

use crate::{
    default_workspace, merge_prompt, run_subprocess, HostDriver, ProbeResult, PromptChannel,
    SubprocessCall, DEFAULT_TIMEOUT,
};

/// Generic single-shot host CLI driver.
///
/// Every field except `program` is `'static` because each backend is
/// known at compile time. `program` is `String` so callers can override
/// it via env var or `with_program(...)` in tests.
#[derive(Debug, Clone)]
pub struct SimpleHostDriver {
    backend_id: &'static str,
    display_name: &'static str,
    kind: RuntimeKind,
    program: String,
    /// CLI arguments that come BEFORE the prompt (e.g. `["exec", "-p"]`).
    base_args: Vec<String>,
    channel: PromptChannel,
    /// Subcommand that prints the version (e.g. `--version`). Used for
    /// the probe.
    version_args: Vec<String>,
    timeout: Duration,
    /// Optional text appended to every prompt for this backend. Used to
    /// instruct agentic CLIs (Droid) that try to call tools / write
    /// files by default to instead return everything to stdout. Empty
    /// string means "no override".
    prompt_suffix: &'static str,
}

const STDOUT_ONLY_SUFFIX: &str = "\n\n---\n\n\
    OUTPUT REQUIREMENTS (override any default agent behavior):\n\
    - Do NOT call any tools. Do NOT execute shell commands.\n\
    - Do NOT write, create, edit, or modify any file on disk.\n\
    - Do NOT use Edit / Create / Execute / WebFetch / Skill tools.\n\
    - Your ONLY output is the markdown body printed directly to stdout.\n\
    - The full document must be in your textual reply, not in a file.";

impl SimpleHostDriver {
    /// Override the program name (mainly for tests).
    #[must_use]
    pub fn with_program(mut self, program: impl Into<String>) -> Self {
        self.program = program.into();
        self
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
        self.base_args.clone()
    }

    /// Stable backend id. Same as [`HostDriver::backend_id`].
    #[must_use]
    pub fn backend_id(&self) -> &'static str {
        self.backend_id
    }

    fn env(name: &str, default: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| default.to_string())
    }

    /// Factory: Factory.ai's [Droid CLI](https://docs.factory.ai/cli/).
    ///
    /// `droid exec --auto low -o text "<prompt>"`. Flag rationale:
    /// - `--auto low`: read-only mode. **Critical for Super Dev** —
    ///   without this, Droid happily writes files into the workspace
    ///   based on the prompt (e.g. creates `output/prd.md` directly),
    ///   colliding with the slug-prefixed artifacts Super Dev itself
    ///   writes. We want text generation only; the pipeline owns disk
    ///   writes. Override with `SUPER_DEV_DROID_AUTO` if you actually
    ///   want Droid to mutate the workspace.
    /// - `-o text`: plain text output (default anyway, but explicit so a
    ///   future Droid default change doesn't surprise us).
    ///
    /// Verified with `droid 0.130.0` against `say "hello from droid"` —
    /// returns clean single-line stdout, no preamble or trailing noise.
    #[must_use]
    pub fn droid() -> Self {
        let auto = Self::env("SUPER_DEV_DROID_AUTO", "low");
        Self {
            backend_id: "droid",
            display_name: "Droid CLI",
            kind: RuntimeKind::Anthropic, // Droid defaults to claude-opus-4-7 under the hood
            program: Self::env("SUPER_DEV_DROID_BIN", "droid"),
            base_args: vec![
                "exec".to_string(),
                "--auto".to_string(),
                auto,
                "--disabled-tools".to_string(),
                "Execute".to_string(),
                "-o".to_string(),
                "text".to_string(),
            ],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            // Critical: Droid is an "agent" CLI that defaults to using
            // Execute/Edit/Create tools (file writes, shell commands).
            // Even with --auto low, it'll happily `cat > research.md`
            // via the Execute tool and return only a brief "I wrote the
            // file" summary to stdout — losing the actual LLM body
            // Super Dev needs. We explicitly tell it: "no tools, stdout
            // only" so it returns the full markdown directly.
            prompt_suffix: STDOUT_ONLY_SUFFIX,
        }
    }

    /// Factory: [OpenCode](https://opencode.ai) —
    /// `opencode run --log-level ERROR "<prompt>"`.
    /// `--log-level ERROR` suppresses INFO/WARN logs that would
    /// otherwise leak into stdout and pollute the LLM response body.
    #[must_use]
    pub fn opencode() -> Self {
        Self {
            backend_id: "opencode",
            display_name: "OpenCode CLI",
            kind: RuntimeKind::Anthropic, // OpenCode is multi-provider; default to Anthropic
            program: Self::env("SUPER_DEV_OPENCODE_BIN", "opencode"),
            base_args: vec![
                "run".to_string(),
                "--log-level".to_string(),
                "ERROR".to_string(),
            ],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: [Google Gemini CLI](https://github.com/google-gemini/gemini-cli)
    /// — `gemini -p "<prompt>"`. Maps to `RuntimeKind::Antigravity` (Google).
    #[must_use]
    pub fn gemini() -> Self {
        Self {
            backend_id: "gemini",
            display_name: "Gemini CLI",
            kind: RuntimeKind::Antigravity,
            program: Self::env("SUPER_DEV_GEMINI_BIN", "gemini"),
            base_args: vec!["-p".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: [Cursor Agent CLI](https://cursor.com/docs/cli/headless)
    /// — `cursor-agent -p --output-format text "<prompt>"`. Print-mode
    /// is non-interactive; `--output-format text` strips the NDJSON
    /// stream formatting.
    #[must_use]
    pub fn cursor_agent() -> Self {
        Self {
            backend_id: "cursor-agent",
            display_name: "Cursor CLI",
            kind: RuntimeKind::Anthropic, // Cursor defaults to Claude
            program: Self::env("SUPER_DEV_CURSOR_BIN", "cursor-agent"),
            base_args: vec![
                "-p".to_string(),
                "--output-format".to_string(),
                "text".to_string(),
            ],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: [Qwen Code](https://github.com/QwenLM/qwen-code) —
    /// `qwen -y -p "<prompt>"`. A Gemini CLI fork running Qwen3-Coder.
    /// `-y` = yolo mode (auto-approve all tool calls, required for headless).
    #[must_use]
    pub fn qwen() -> Self {
        Self {
            backend_id: "qwen",
            display_name: "Qwen Code CLI",
            kind: RuntimeKind::Openai, // Qwen exposes an OpenAI-compatible API
            program: Self::env("SUPER_DEV_QWEN_BIN", "qwen"),
            base_args: vec!["-y".to_string(), "-p".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: [Continue CLI](https://docs.continue.dev/guides/cli) —
    /// `cn -p "<prompt>"`. Headless mode reads `CONTINUE_API_KEY`.
    #[must_use]
    pub fn continue_cli() -> Self {
        Self {
            backend_id: "continue",
            display_name: "Continue CLI",
            kind: RuntimeKind::Anthropic, // Continue is multi-provider; default Anthropic
            program: Self::env("SUPER_DEV_CONTINUE_BIN", "cn"),
            base_args: vec!["-p".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: [GitHub Copilot CLI](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-command-reference)
    /// — `copilot -p "<prompt>" --allow-all-tools`. The new official
    /// `copilot` binary (not the deprecated `gh copilot` extension).
    #[must_use]
    pub fn copilot() -> Self {
        Self {
            backend_id: "copilot",
            display_name: "Copilot CLI",
            kind: RuntimeKind::Openai, // Copilot routes through OpenAI
            program: Self::env("SUPER_DEV_COPILOT_BIN", "copilot"),
            base_args: vec!["-p".to_string(), "--allow-all-tools".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: `ByteDance`'s [Trae Agent](https://github.com/bytedance/trae-agent)
    /// — `trae-cli run "<prompt>"`. Note: the binary is **`trae-cli`**
    /// (the open-source agent), NOT `trae` (which is the Trae IDE GUI
    /// launcher, like `code` for VS Code). Install via the trae-agent
    /// pip package or its release artifacts.
    #[must_use]
    pub fn trae() -> Self {
        Self {
            backend_id: "trae",
            display_name: "Trae CLI",
            kind: RuntimeKind::Anthropic, // Trae uses Claude / GPT family under the hood
            program: Self::env("SUPER_DEV_TRAE_BIN", "trae-cli"),
            base_args: vec!["run".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: [Plandex](https://github.com/plandex-ai/plandex) —
    /// `plandex tell --skip-menu --stop "<prompt>"`. Flag rationale:
    /// - `tell`: the non-interactive subcommand (vs. interactive REPL).
    /// - `--skip-menu`: don't open the "review changes" interactive menu
    ///   after the response — required for headless runs.
    /// - `--stop` (or `-s`): single response, don't auto-continue. Stops
    ///   Plandex from running multi-turn loops on its own.
    ///
    /// Plandex stores plans server-side (self-hosted or cloud). The
    /// driver assumes you've already run `plandex sign-in` once.
    #[must_use]
    pub fn plandex() -> Self {
        Self {
            backend_id: "plandex",
            display_name: "Plandex CLI",
            kind: RuntimeKind::Anthropic, // Plandex routes to Anthropic by default
            program: Self::env("SUPER_DEV_PLANDEX_BIN", "plandex"),
            base_args: vec![
                "tell".to_string(),
                "--skip-menu".to_string(),
                "--stop".to_string(),
            ],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            // Plandex is also agentic by default — its `tell` subcommand
            // creates plans and edits files. The suffix steers it back to
            // pure text output for the prompts Super Dev sends.
            prompt_suffix: STDOUT_ONLY_SUFFIX,
        }
    }

    /// Factory: Sourcegraph [Cody CLI](https://sourcegraph.com/docs/cody/clients/install-cli)
    /// — `cody chat --message "<prompt>"`. Cody routes through your
    /// Sourcegraph enterprise account (or sourcegraph.com), so it sees
    /// your indexed code as context. Login via `cody auth login` first.
    #[must_use]
    pub fn cody() -> Self {
        Self {
            backend_id: "cody",
            display_name: "Sourcegraph Cody CLI",
            kind: RuntimeKind::Anthropic, // Cody defaults to Claude on Sourcegraph
            program: Self::env("SUPER_DEV_CODY_BIN", "cody"),
            base_args: vec!["chat".to_string(), "--message".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: Block's [Goose](https://github.com/block/goose) —
    /// `goose run -t "<prompt>"`. Headless one-shot mode with text
    /// output. 45k+ stars.
    #[must_use]
    pub fn goose() -> Self {
        Self {
            backend_id: "goose",
            display_name: "Goose CLI (Block)",
            kind: RuntimeKind::Anthropic,
            program: Self::env("SUPER_DEV_GOOSE_BIN", "goose"),
            base_args: vec!["run".to_string(), "-t".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: Moonshot's [Kimi CLI](https://github.com/MoonshotAI/kimi-cli) —
    /// `kimi --print -c "<prompt>"`. Flag rationale:
    /// - `--print`: non-interactive print mode (same as Claude Code).
    /// - `-c` / `--command`: passes the user query. The flag is NOT
    ///   `--prompt` (common mistake — Kimi uses `--command` or `--query`).
    ///
    /// Verified against `kimi 0.53` help output.
    #[must_use]
    pub fn kimi() -> Self {
        Self {
            backend_id: "kimi",
            display_name: "Kimi Code CLI",
            kind: RuntimeKind::Anthropic,
            program: Self::env("SUPER_DEV_KIMI_BIN", "kimi"),
            base_args: vec![
                "--print".to_string(),
                "--output-format".to_string(),
                "text".to_string(),
                "-c".to_string(),
            ],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: Sourcegraph's [Amp](https://sourcegraph.com/amp) —
    /// `amp -x "<prompt>"`. Execute mode: run, print last message, exit.
    #[must_use]
    pub fn amp() -> Self {
        Self {
            backend_id: "amp",
            display_name: "Amp CLI (Sourcegraph)",
            kind: RuntimeKind::Anthropic,
            program: Self::env("SUPER_DEV_AMP_BIN", "amp"),
            base_args: vec!["-x".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: `JetBrains`' [Junie CLI](https://junie.jetbrains.com/docs/junie-cli.html) —
    /// `junie "<prompt>"`. Headless mode exits after processing.
    /// LLM-agnostic, supports BYOK.
    #[must_use]
    pub fn junie() -> Self {
        Self {
            backend_id: "junie",
            display_name: "Junie CLI (JetBrains)",
            kind: RuntimeKind::Anthropic,
            program: Self::env("SUPER_DEV_JUNIE_BIN", "junie"),
            base_args: vec![],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: xAI's [Grok Build](https://x.ai/cli) —
    /// `grok-build -p "<prompt>"`. Headless mode for automation.
    /// Requires `SuperGrok` Heavy subscription or xAI API key.
    #[must_use]
    pub fn grok_build() -> Self {
        Self {
            backend_id: "grok-build",
            display_name: "Grok Build CLI (xAI)",
            kind: RuntimeKind::Openai, // xAI uses its own Grok models
            program: Self::env("SUPER_DEV_GROK_BIN", "grok-build"),
            base_args: vec!["-p".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: AWS [Amazon Q Developer CLI](https://github.com/aws/amazon-q-developer-cli) —
    /// `q chat --no-interactive --trust-all-tools "<prompt>"`.
    /// Headless mode for CI/CD. Uses AWS credentials.
    #[must_use]
    pub fn amazon_q() -> Self {
        Self {
            backend_id: "amazon-q",
            display_name: "Amazon Q Developer CLI",
            kind: RuntimeKind::Anthropic, // Q Developer uses Claude internally
            program: Self::env("SUPER_DEV_AMAZON_Q_BIN", "q"),
            base_args: vec![
                "chat".to_string(),
                "--no-interactive".to_string(),
                "--trust-all-tools".to_string(),
            ],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: `Charmbracelet`'s [Crush](https://github.com/charmbracelet/crush) —
    /// `crush run "<prompt>"`. 24k+ stars, Go, multi-provider, LSP-aware.
    #[must_use]
    pub fn crush() -> Self {
        Self {
            backend_id: "crush",
            display_name: "Crush CLI (Charm)",
            kind: RuntimeKind::Anthropic,
            program: Self::env("SUPER_DEV_CRUSH_BIN", "crush"),
            base_args: vec!["run".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: [gptme](https://gptme.org) — `gptme -n "<prompt>"`.
    /// `-n` = non-interactive (implies `--no-confirm`). Python-based,
    /// multi-provider, supports persistent autonomous sessions.
    #[must_use]
    pub fn gptme() -> Self {
        Self {
            backend_id: "gptme",
            display_name: "gptme CLI",
            kind: RuntimeKind::Anthropic,
            program: Self::env("SUPER_DEV_GPTME_BIN", "gptme"),
            base_args: vec!["-n".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: 腾讯 [CodeBuddy CLI](https://copilot.tencent.com/cli/) —
    /// `codebuddy -p -y "<prompt>"`. Flag rationale:
    /// - `-p` / `--print`: non-interactive print mode (same as Claude Code).
    /// - `-y` / `--dangerously-skip-permissions`: required for headless
    ///   runs; without it, codebuddy prompts for permission on every
    ///   file read/write/exec.
    #[must_use]
    pub fn codebuddy() -> Self {
        Self {
            backend_id: "codebuddy",
            display_name: "CodeBuddy CLI",
            kind: RuntimeKind::Anthropic,
            program: Self::env("SUPER_DEV_CODEBUDDY_BIN", "codebuddy"),
            base_args: vec!["-p".to_string(), "-y".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: [Qoder CLI](https://qoder.com/cli) —
    /// `qodercli -p "<prompt>"`. Print mode for non-interactive headless
    /// use. Qoder uses automatic model routing (Claude/GPT/Gemini/Qwen).
    #[must_use]
    pub fn qoder() -> Self {
        Self {
            backend_id: "qoder",
            display_name: "Qoder CLI",
            kind: RuntimeKind::Anthropic,
            program: Self::env("SUPER_DEV_QODER_BIN", "qodercli"),
            base_args: vec!["-p".to_string()],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }

    /// Factory: [Aider](https://aider.chat) — `aider --message "<prompt>" --yes --no-stream`.
    /// `--yes` auto-accepts confirmation prompts so the process actually
    /// exits in CI; `--no-stream` suppresses incremental tokens.
    #[must_use]
    pub fn aider() -> Self {
        Self {
            backend_id: "aider",
            display_name: "Aider CLI",
            kind: RuntimeKind::Anthropic, // Aider is multi-provider; default Anthropic
            program: Self::env("SUPER_DEV_AIDER_BIN", "aider"),
            base_args: vec![
                "--yes".to_string(),
                "--no-stream".to_string(),
                "--message".to_string(),
            ],
            channel: PromptChannel::Arg,
            version_args: vec!["--version".to_string()],
            timeout: DEFAULT_TIMEOUT,
            prompt_suffix: "",
        }
    }
}

#[async_trait]
impl Runtime for SimpleHostDriver {
    fn kind(&self) -> RuntimeKind {
        self.kind
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, RuntimeError> {
        let mut prompt = merge_prompt(&req);
        if !self.prompt_suffix.is_empty() {
            prompt.push_str(self.prompt_suffix);
        }
        let out = run_subprocess(SubprocessCall {
            program: &self.program,
            args: &self.base_args,
            prompt: &prompt,
            channel: self.channel,
            workspace: &default_workspace(),
            timeout: self.timeout,
        })
        .await
        .map_err(RuntimeError::HostProcess)?;

        // Backend-specific output sanitisation. ANSI / leading whitespace
        // is already gone via `run_subprocess::clean_output`; this strips
        // backend-specific noise that would otherwise pollute the artifact.
        let text = match self.backend_id {
            "opencode" => strip_opencode_header(&out.stdout),
            "kimi" => strip_kimi_debug(&out.stdout),
            _ => out.stdout,
        };

        Ok(CompletionResponse {
            text,
            id: format!("{}-cli", self.backend_id),
            model: req.model,
            usage: Usage::default(),
        })
    }
}

/// Kimi `--print` mode dumps Python repr objects like
/// `TextPart(type='text', text='hello')` instead of plain text. Extract
/// the `text='...'` bodies from all `TextPart` lines and join them.
fn strip_kimi_debug(raw: &str) -> String {
    let mut parts = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("TextPart(") {
            if let Some(start) = rest.find("text='") {
                let after = &rest[start + 6..];
                if let Some(end) = after.find("')") {
                    parts.push(&after[..end]);
                }
            }
        }
    }
    if parts.is_empty() {
        raw.trim().to_string()
    } else {
        parts.join("\n")
    }
}

/// Strip `OpenCode`'s `> build · <model>` preamble that lands above the
/// real response in its `default` output format. If the first non-empty
/// line begins with `> ` it's the header; everything after blank-line
/// separator is the LLM body. If the body is empty (which it should
/// never be in practice), return the full input untouched so we don't
/// silently lose content.
fn strip_opencode_header(text: &str) -> String {
    let trimmed = text.trim_start();
    if !trimmed.starts_with("> ") {
        return text.trim().to_string();
    }
    // Drop the first line (the `> build · model` header) and any blank
    // lines that follow before the actual body.
    let after_header = trimmed
        .lines()
        .skip(1)
        .skip_while(|l| l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let after_header = after_header.trim().to_string();
    if after_header.is_empty() {
        text.trim().to_string()
    } else {
        after_header
    }
}

#[async_trait]
impl HostDriver for SimpleHostDriver {
    fn backend_id(&self) -> &'static str {
        self.backend_id
    }

    fn display_name(&self) -> &'static str {
        self.display_name
    }

    async fn probe(&self) -> ProbeResult {
        let tmp = default_workspace();
        match run_subprocess(SubprocessCall {
            program: &self.program,
            args: &self.version_args,
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
    fn droid_defaults_to_exec_auto_low_no_execute() {
        let d = SimpleHostDriver::droid();
        assert_eq!(d.backend_id(), "droid");
        assert_eq!(d.display_name(), "Droid CLI");
        assert_eq!(
            d.base_args(),
            vec![
                "exec".to_string(),
                "--auto".to_string(),
                "low".to_string(),
                "--disabled-tools".to_string(),
                "Execute".to_string(),
                "-o".to_string(),
                "text".to_string(),
            ]
        );
    }

    #[test]
    fn opencode_defaults_to_run_error_loglevel() {
        let d = SimpleHostDriver::opencode();
        assert_eq!(d.backend_id(), "opencode");
        assert_eq!(
            d.base_args(),
            vec![
                "run".to_string(),
                "--log-level".to_string(),
                "ERROR".to_string()
            ]
        );
    }

    #[test]
    fn gemini_defaults_to_dash_p() {
        let d = SimpleHostDriver::gemini();
        assert_eq!(d.backend_id(), "gemini");
        assert_eq!(d.kind(), RuntimeKind::Antigravity);
        assert_eq!(d.base_args(), vec!["-p".to_string()]);
    }

    #[test]
    fn cursor_agent_uses_text_output_format() {
        let d = SimpleHostDriver::cursor_agent();
        assert_eq!(d.backend_id(), "cursor-agent");
        assert_eq!(
            d.base_args(),
            vec![
                "-p".to_string(),
                "--output-format".to_string(),
                "text".to_string()
            ]
        );
    }

    #[test]
    fn qwen_defaults_to_yolo_dash_p() {
        let d = SimpleHostDriver::qwen();
        assert_eq!(d.backend_id(), "qwen");
        assert_eq!(d.base_args(), vec!["-y".to_string(), "-p".to_string()]);
    }

    #[test]
    fn continue_cli_defaults_to_dash_p() {
        let d = SimpleHostDriver::continue_cli();
        assert_eq!(d.backend_id(), "continue");
        assert_eq!(d.base_args(), vec!["-p".to_string()]);
    }

    #[test]
    fn copilot_allows_tools_for_non_interactive() {
        let d = SimpleHostDriver::copilot();
        assert_eq!(d.backend_id(), "copilot");
        assert!(d.base_args().contains(&"--allow-all-tools".to_string()));
    }

    #[test]
    fn aider_uses_yes_and_no_stream_with_message() {
        let d = SimpleHostDriver::aider();
        assert_eq!(d.backend_id(), "aider");
        let args = d.base_args();
        assert!(args.contains(&"--yes".to_string()));
        assert!(args.contains(&"--no-stream".to_string()));
        assert!(args.contains(&"--message".to_string()));
    }

    #[tokio::test]
    async fn probe_reports_not_installed_for_missing_binary() {
        let d = SimpleHostDriver::gemini().with_program("super-dev-fake-gemini-xyz");
        assert!(matches!(d.probe().await, ProbeResult::NotInstalled { .. }));
    }

    #[test]
    fn droid_appends_stdout_only_suffix() {
        let d = SimpleHostDriver::droid();
        assert!(
            d.prompt_suffix.contains("Do NOT call any tools"),
            "droid must instruct stdout-only"
        );
        assert!(d.prompt_suffix.contains("Edit / Create / Execute"));
    }

    #[test]
    fn non_agentic_backends_have_no_suffix() {
        // The stdout-only suffix is restricted to agentic backends
        // (droid, plandex). Adding it to non-agentic backends would
        // pollute their prompts.
        for d in [
            SimpleHostDriver::opencode(),
            SimpleHostDriver::gemini(),
            SimpleHostDriver::cursor_agent(),
            SimpleHostDriver::qwen(),
            SimpleHostDriver::continue_cli(),
            SimpleHostDriver::copilot(),
            SimpleHostDriver::aider(),
            SimpleHostDriver::trae(),
            SimpleHostDriver::cody(),
        ] {
            assert_eq!(
                d.prompt_suffix, "",
                "{} should not have a prompt suffix",
                d.backend_id
            );
        }
    }

    #[test]
    fn trae_uses_run_subcommand() {
        let d = SimpleHostDriver::trae();
        assert_eq!(d.backend_id(), "trae");
        assert_eq!(d.display_name(), "Trae CLI");
        assert_eq!(d.base_args(), vec!["run".to_string()]);
    }

    #[test]
    fn plandex_uses_tell_with_headless_flags() {
        let d = SimpleHostDriver::plandex();
        assert_eq!(d.backend_id(), "plandex");
        assert_eq!(
            d.base_args(),
            vec![
                "tell".to_string(),
                "--skip-menu".to_string(),
                "--stop".to_string(),
            ]
        );
        // Plandex is agentic — needs the stdout-only suffix.
        assert!(d.prompt_suffix.contains("Do NOT call any tools"));
    }

    #[test]
    fn cody_uses_chat_with_message_flag() {
        let d = SimpleHostDriver::cody();
        assert_eq!(d.backend_id(), "cody");
        assert_eq!(
            d.base_args(),
            vec!["chat".to_string(), "--message".to_string()]
        );
    }

    #[test]
    fn strip_kimi_debug_extracts_text_parts() {
        let raw = "StatusUpdate(status=StatusSnapshot(...))\nTextPart(type='text', text='# PRD')\nTextPart(type='text', text='body here')\nStatusUpdate(...)";
        assert_eq!(strip_kimi_debug(raw), "# PRD\nbody here");
    }

    #[test]
    fn strip_kimi_debug_passes_clean_through() {
        assert_eq!(strip_kimi_debug("just text"), "just text");
    }

    #[test]
    fn strip_opencode_header_removes_build_line() {
        let raw = "> build · glm-5\n\n# PRD\n\nbody here";
        assert_eq!(strip_opencode_header(raw), "# PRD\n\nbody here");
    }

    #[test]
    fn strip_opencode_header_passes_clean_output_through() {
        let raw = "# PRD\n\nno header";
        assert_eq!(strip_opencode_header(raw), "# PRD\n\nno header");
    }

    #[test]
    fn strip_opencode_header_preserves_input_if_body_empty() {
        // Pathological case — header present but nothing after. Don't
        // return an empty string (which would trigger the template
        // fallback in `try_generate`); return the input verbatim so the
        // caller still has something to log.
        let raw = "> build · glm-5";
        assert_eq!(strip_opencode_header(raw), "> build · glm-5");
    }

    #[tokio::test]
    async fn complete_drives_a_fake_binary() {
        // `echo` stands in for the host CLI: it echoes args + prompt.
        let d = SimpleHostDriver::gemini().with_program("echo");
        let req = CompletionRequest {
            model: "gemini-2.5-pro".into(),
            system: None,
            messages: vec![super_dev_runtime::Message {
                role: "user".into(),
                content: "synthesise a brief".into(),
            }],
            max_tokens: None,
            temperature: None,
        };
        let resp = d.complete(req).await.unwrap();
        assert!(resp.text.contains("synthesise a brief"));
        assert_eq!(resp.model, "gemini-2.5-pro");
        assert_eq!(resp.id, "gemini-cli");
    }
}
