//! `super-dev` — the single binary entrypoint.
//!
//! The recommended entry is to run the binary with **no subcommand**, which
//! launches the chat TUI. From there every feature is reachable via slash
//! commands (`/run`, `/continue`, `/provider`, `/backend`, `/status`, …).
//!
//! Subcommands exist for scripting / CI and for the architectural non-negotiable
//! real-time governance hook (called *by* a host CLI, not a human). The visible
//! surface is intentionally tiny:
//!
//! - (none)                  launch the chat TUI (the recommended entry)
//! - `init`                  write the `super-dev.yaml` spec manifest
//! - `install --host <id>`   wire the real-time governance hook into a host CLI
//!
//! The rest (`run` / `continue` / `revise` / `spec` / `verify` / `report` /
//! `doctor` / `examples` / `guide` / `rollback` / `history`) are hidden from
//! `--help` but still work for scripts, and mirror TUI slash commands.
//! `hook` / `uninstall` are hidden internals.
//!
//! Anything outside this surface is intentionally absent.

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::unused_async,
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    clippy::vec_init_then_push,
    clippy::format_push_string,
    clippy::uninlined_format_args,
    dead_code
)]

mod doctor;
mod hook;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};

use super_dev_agent::ChannelSink;
use super_dev_agent::{
    classify_reply, list_snapshots, read_workflow_state, restore_snapshot, AgentRunner, Gate,
    GateOutcome, RunOptions, RunReport, WorkflowState,
};
use super_dev_governance::{compliance::write_compliance_mapping, record_tool_call};
use super_dev_runtime::{OfflineRuntime, RuntimeKind};
use super_dev_spec::{CLAUSES, PHASE_CHAIN, SPEC_VERSION};

/// Super Dev — a coach for AI coding hosts.
#[derive(Debug, Parser)]
#[command(
    name = "super-dev",
    version,
    about = "AI 编码的项目经理 — 9-phase commercial delivery pipeline. Run `super-dev` (no args) for the TUI. Drive your logged-in Claude Code / Codex CLI, or a custom OpenAI/Anthropic API endpoint. No API key needed for host-CLI mode.",
    long_about = None,
)]
struct Cli {
    /// Subcommand. **Omit to launch the TUI** — that is the recommended entry.
    /// `init` bootstraps a workspace; `install` wires the governance hook.
    /// All other verbs are hidden but still work for scripts/CI.
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Bootstrap a workspace: write the `super-dev.yaml` spec manifest.
    #[command(
        long_about = "Bootstrap a workspace by writing a `super-dev.yaml` spec manifest\n\
                      (SD-META-001). This declares the project to Super Dev so future\n\
                      `super-dev run` / `super-dev` (TUI) invocations have a stable slug,\n\
                      conformance level, and quality threshold.\n\
                      \n\
                      Idempotent — re-running on an initialised workspace is a no-op\n\
                      unless `--force` is passed.",
        after_help = "EXAMPLES:\n  \
                      super-dev init                       # init current dir\n  \
                      super-dev init --slug my-app          # explicit slug\n  \
                      super-dev init --project-root ./app   # initialise a sub-directory\n  \
                      super-dev init --force                # overwrite a hand-edited manifest"
    )]
    Init {
        /// Project slug used in artifact filenames; defaults to the
        /// workspace directory name.
        #[arg(long)]
        slug: Option<String>,
        /// Workspace root; defaults to current directory.
        #[arg(long)]
        project_root: Option<PathBuf>,
        /// Overwrite an existing `super-dev.yaml` that differs from the
        /// shipped template.
        #[arg(long)]
        force: bool,
    },
    /// Drive the pipeline from `research` to the first gate (`docs_confirm`).
    #[command(
        hide = true,
        long_about = "Run the pipeline non-interactively from `research` to the first\n\
                      gate (`docs_confirm`). Workers:\n\
                      \n  \
                      --backend claude-code    Anthropic Claude Code\n  \
                      --backend codex          OpenAI Codex\n  \
                      (default)                offline deterministic templates\n\
                      \n\
                      All host backends drive the user's already-installed,\n\
                      already-logged-in CLI — no API key needed.\n\
                      \n\
                      After `run`, the pipeline pauses at the `docs_confirm` gate.\n\
                      Review `output/*-prd.md` etc., then run `super-dev continue` to\n\
                      proceed, or `super-dev revise \"...\"` to ask for changes.",
        after_help = "EXAMPLES:\n  \
                      super-dev run \"做一个登录系统\"                       # offline\n  \
                      super-dev run \"...\" --backend claude-code              # Claude Code\n  \
                      super-dev run \"...\" --backend codex --slug my-mvp      # explicit slug"
    )]
    Run {
        /// Plain-text requirement, e.g. "做一个登录页".
        requirement: String,
        /// Drive an already-logged-in host CLI as the worker. When
        /// omitted, the pipeline runs offline with deterministic templates.
        #[arg(long, value_enum)]
        backend: Option<BackendArg>,
        /// Model identifier passed to the backend (host drivers may ignore).
        #[arg(long, default_value = "claude-sonnet-4-6")]
        model: String,
        /// Workspace root; defaults to current directory.
        #[arg(long)]
        project_root: Option<PathBuf>,
        /// Project slug used in artifact filenames.
        #[arg(long, default_value = "")]
        slug: String,
    },
    /// Approve the active gate and continue the pipeline.
    #[command(
        hide = true,
        long_about = "Approve the currently-active gate and drive the pipeline to its\n\
                      next pause point. Call after reviewing the artifacts a `run` or\n\
                      `continue` previously produced.\n\
                      \n\
                      Gate progression:\n  \
                      docs_confirm       (after research + docs)      → spec / frontend\n  \
                      preview_confirm    (after spec + frontend)       → backend / quality / delivery\n\
                      \n\
                      `continue` is a no-op if no gate is active.",
        after_help = "EXAMPLES:\n  \
                      super-dev continue\n  \
                      super-dev continue --backend claude-code\n  \
                      super-dev continue --project-root ./app"
    )]
    Continue {
        /// Workspace root; defaults to current directory.
        #[arg(long)]
        project_root: Option<PathBuf>,
        /// Drive the next block via an already-logged-in host CLI. When
        /// omitted, falls back to whatever the original `run` recorded —
        /// or offline templates if no backend was tracked.
        #[arg(long, value_enum)]
        backend: Option<BackendArg>,
    },
    /// Stay in the active gate and record a revision request.
    #[command(
        hide = true,
        long_about = "Record a revision request against the active gate. The pipeline\n\
                      does NOT advance — it stays at the same gate so you can iterate\n\
                      on the artifacts. The revision text is appended to\n\
                      `.super-dev/decisions/<gate>.md` so the audit chain captures\n\
                      every change request.",
        after_help = "EXAMPLES:\n  \
                      super-dev revise \"把 OAuth 那段去掉,先做邮箱+密码 MVP\"\n  \
                      super-dev revise \"前端改成 Vue 而不是 React\""
    )]
    Revise {
        /// What needs to change. Free-form text.
        text: String,
        /// Workspace root; defaults to current directory.
        #[arg(long)]
        project_root: Option<PathBuf>,
    },
    /// Roll the pipeline back to a previous state snapshot.
    #[command(
        hide = true,
        long_about = "Restore workflow-state.json from a previous transition snapshot.\n\n                      Every phase transition is snapshotted to .super-dev/history/. Use\n                      `super-dev history` to list available snapshots, then `super-dev rollback\n                      <timestamp>` to restore one. This undoes the most recent transition(s)\n                      without losing the generated artifacts on disk.",
        after_help = "EXAMPLES:\n                        super-dev history                   # list snapshots\n                        super-dev rollback 20260614T120000  # restore by timestamp\n                        super-dev rollback latest           # undo the last transition"
    )]
    Rollback {
        /// Snapshot timestamp (from `super-dev history`), or `latest`.
        timestamp: String,
        /// Workspace root; defaults to current directory.
        #[arg(long)]
        project_root: Option<PathBuf>,
    },
    /// List available rollback snapshots.
    History {
        /// Workspace root; defaults to current directory.
        #[arg(long)]
        project_root: Option<PathBuf>,
    },
    /// Print the `SUPER_DEV_HOST_SPEC_V1` specification.
    #[command(
        hide = true,
        long_about = "Print the SUPER_DEV_HOST_SPEC_V1 specification — the normative\n\
                      contract Super Dev enforces (25 clauses across 4 layers + 9\n\
                      phases + 2 gates).",
        after_help = "EXAMPLES:\n  \
                      super-dev spec               # full markdown\n  \
                      super-dev spec --clauses     # clause table only\n  \
                      super-dev spec | less        # paged"
    )]
    Spec {
        /// Print only the clause table.
        #[arg(long)]
        clauses: bool,
    },
    /// Verify spec conformance of a workspace.
    #[command(
        hide = true,
        long_about = "Print a structured conformance report for the workspace:\n\
                      spec manifest health, workflow state, evidence chain row counts,\n\
                      latest quality-gate score, and proof-pack zips.",
        after_help = "EXAMPLES:\n  \
                      super-dev verify\n  \
                      super-dev verify --project-root ./app"
    )]
    Verify {
        /// Workspace root; defaults to current directory.
        #[arg(long)]
        project_root: Option<PathBuf>,
    },
    /// Emit the SD-EVID-004 compliance mapping document.
    #[command(
        hide = true,
        long_about = "Generate the SD-EVID-004 compliance mapping document. Takes the\n\
                      in-workspace evidence files (audit JSONLs + quality report) and\n\
                      maps every clause that fired to the corresponding controls in\n\
                      SOC 2 (2017 TSC), ISO/IEC 27001:2022 Annex A, and the EU AI Act.\n\
                      \n\
                      Output: `output/<slug>-compliance-mapping.json`.",
        after_help = "EXAMPLES:\n  \
                      super-dev report\n  \
                      super-dev report --slug my-app"
    )]
    Report {
        /// Project slug used in artifact filenames.
        #[arg(long)]
        slug: Option<String>,
        /// Workspace root; defaults to current directory.
        #[arg(long)]
        project_root: Option<PathBuf>,
    },
    /// Self-test: binary integrity, workspace permissions, manifest.
    #[command(
        hide = true,
        long_about = "Run a self-test that diagnoses common 'installed but not working'\n\
                      situations: binary identity, embedded spec validity, workspace\n\
                      writability, SD-META-001 manifest health.\n\
                      \n\
                      Exit code is non-zero if any check FAILs (suitable for CI smoke).",
        after_help = "EXAMPLES:\n  \
                      super-dev doctor\n  \
                      super-dev doctor --project-root ./app"
    )]
    Doctor {
        /// Workspace root; defaults to current directory.
        #[arg(long)]
        project_root: Option<PathBuf>,
    },
    /// Show common-workflow examples for new users.
    #[command(
        hide = true,
        long_about = "Print a curated set of common-workflow examples. Useful as a\n\
                      cheat-sheet — every important command is shown with a real\n\
                      invocation."
    )]
    Examples,
    /// 60-second guided walkthrough for new users (interactive).
    #[command(
        hide = true,
        long_about = "Run a short, mode-by-mode walkthrough of Super Dev. Prints what\n\
                      each command does, how the pipeline progresses, and what\n\
                      artifacts you can expect. No side effects."
    )]
    Guide,
    /// Pre-write governance hook (called by Claude Code's PreToolUse).
    ///
    /// Reads a PreToolUse JSON payload from stdin, runs the governance rules
    /// (emoji / color / AI-slop), and prints a permission decision. This is
    /// NOT meant to be called by humans — Claude Code invokes it via the
    /// hook registered by `super-dev install`.
    #[command(hide = true)]
    Hook {
        /// Which governance check to run: `pre-write` (all code rules) or
        /// `check-emoji` / `check-color` / `check-slop` (individual).
        check: String,
    },
    /// Install the Super Dev pre-write governance hook into a host CLI.
    ///
    /// Currently supports Claude Code (writes `.claude/settings.json` with a
    /// PreToolUse hook pointing at this binary). Codex is honestly
    /// reported as unsupported — they rely on the quality-gate hard block
    /// instead of real-time interception.
    #[command(
        long_about = "Install the Super Dev pre-write governance hook into a host CLI.\n\
                      \n\
                      Supported hosts:\n  \
                      claude-code   writes .claude/settings.json PreToolUse hook\n\
                      \n\
                      The hook intercepts every Write/Edit tool call and refuses\n\
                      emoji-as-icon / hardcoded-color / AI-slop code in real time\n\
                      (SD-CODE-001/002/005). Codex lacks\n\
                      a PreToolUse hook surface — they rely on the quality-gate\n\
                      hard block instead."
    )]
    Install {
        /// Host to install into: `claude-code` (default).
        #[arg(long, default_value = "claude-code")]
        host: String,
        /// Workspace root; defaults to current directory.
        #[arg(long)]
        project_root: Option<PathBuf>,
    },
    /// Remove the Super Dev governance hook from a host CLI.
    #[command(hide = true)]
    Uninstall {
        /// Host to uninstall from.
        #[arg(long, default_value = "claude-code")]
        host: String,
        /// Workspace root; defaults to current directory.
        #[arg(long)]
        project_root: Option<PathBuf>,
    },
}

/// Host CLI backend selector for `super-dev run --backend`.
///
/// Super Dev drives exactly two host CLIs: Claude Code and Codex. A unit test
/// asserts [`BACKEND_ARG_IDS`] stays equal to [`super_dev_host::BACKEND_IDS`].
#[derive(Debug, Copy, Clone, clap::ValueEnum)]
enum BackendArg {
    /// Drive Claude Code CLI.
    ClaudeCode,
    /// Drive Codex CLI.
    Codex,
}

impl BackendArg {
    fn id(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        match id {
            "claude-code" => Some(Self::ClaudeCode),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }

    /// Every id this enum can produce. Kept in sync with
    /// [`super_dev_host::BACKEND_IDS`] by the `backend_arg_ids_match_host` test.
    fn all_ids() -> &'static [&'static str] {
        &["claude-code", "codex"]
    }
}

/// Re-export so the sync test and help text can reference the canonical list.
const BACKEND_ARG_IDS: &[&str] = &["claude-code", "codex"];

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    // No subcommand → launch the TUI (the recommended interactive entry).
    // In a non-TTY environment (piped output, CI, docker), fall back to
    // printing help instead of crashing on terminal setup.
    let Some(command) = cli.command else {
        if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            return cmd_tui().await;
        }
        eprintln!(
            "super-dev: no terminal detected — showing help.
"
        );
        Cli::command().print_help()?;
        return Ok(());
    };

    match command {
        Command::Init {
            slug,
            project_root,
            force,
        } => cmd_init(slug, project_root, force),
        Command::Run {
            requirement,
            backend,
            model,
            project_root,
            slug,
        } => {
            cmd_run(RunArgs {
                requirement,
                backend,
                model,
                project_root,
                slug,
            })
            .await
        }
        Command::Continue {
            project_root,
            backend,
        } => cmd_continue(project_root, backend).await,
        Command::Revise { text, project_root } => cmd_revise(text, project_root).await,
        Command::Rollback {
            timestamp,
            project_root,
        } => cmd_rollback(timestamp, project_root),
        Command::History { project_root } => cmd_history(project_root),
        Command::Spec { clauses } => cmd_spec(clauses),
        Command::Verify { project_root } => cmd_verify(project_root),
        Command::Report { slug, project_root } => cmd_report(slug, project_root),
        Command::Doctor { project_root } => cmd_doctor(project_root),
        Command::Examples => cmd_examples(),
        Command::Guide => cmd_guide(),
        Command::Hook { check } => cmd_hook(check),
        Command::Install { host, project_root } => cmd_install(host, project_root),
        Command::Uninstall { host, project_root } => cmd_uninstall(host, project_root),
    }
}

fn cmd_examples() -> Result<()> {
    println!("{}", include_str!("../templates/examples.txt"));
    Ok(())
}

fn cmd_guide() -> Result<()> {
    println!("{}", include_str!("../templates/guide.txt"));
    Ok(())
}

fn cmd_hook(check: String) -> Result<()> {
    // Read the PreToolUse payload from stdin.
    use std::io::Read;
    let mut stdin = String::new();
    let _ = std::io::stdin().read_to_string(&mut stdin);
    match check.as_str() {
        "pre-write" | "check-emoji" | "check-color" | "check-slop" => {
            let decision = hook::run_pre_write(&stdin);
            hook::print_decision(&decision);
        }
        _ => {
            anyhow::bail!(
                "unknown hook check: {check} (expected one of: pre-write, check-emoji, check-color, check-slop)"
            );
        }
    }
    Ok(())
}

fn cmd_install(host: String, project_root: Option<PathBuf>) -> Result<()> {
    let root = project_root.unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    match host.as_str() {
        "claude-code" => {
            let path = hook::install_claude_hook(&root)?;
            println!("✓ Installed Super Dev PreToolUse hook for Claude Code.");
            println!("  → {}", path.display());
            println!();
            println!("Every Write/Edit tool call will now be checked for:");
            println!("  • emoji-as-functional-icons (SD-CODE-001)");
            println!("  • hardcoded color literals   (SD-CODE-002)");
            println!("  • AI-slop / placeholders     (SD-CODE-002)");
            println!("  • sensitive-path writes      (SD-SEC-001) — .git/.env/.ssh bypass-immune");
            println!();
            println!("To remove: super-dev uninstall --host claude-code");
        }
        other => {
            anyhow::bail!(
                "Real-time hook interception is only supported for Claude Code.\n\
                 Host '{other}' lacks a PreToolUse hook surface. It relies on the\n\
                 quality-gate hard block (SD-EVID-003) instead — which is always\n\
                 active for every backend."
            );
        }
    }
    Ok(())
}

fn cmd_uninstall(host: String, project_root: Option<PathBuf>) -> Result<()> {
    let root = project_root.unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    match host.as_str() {
        "claude-code" => {
            hook::uninstall_claude_hook(&root)?;
            println!("✓ Removed Super Dev PreToolUse hook from Claude Code settings.");
        }
        other => {
            anyhow::bail!("uninstall not applicable for host '{other}'");
        }
    }
    Ok(())
}

fn cmd_doctor(project_root: Option<PathBuf>) -> Result<()> {
    let workspace = resolve_root(project_root)?;
    let results = doctor::run_all(&workspace);
    print!("{}", doctor::render_report(&workspace, &results));
    if results.iter().any(|r| r.status == doctor::Status::Failed) {
        anyhow::bail!("super-dev doctor: one or more checks failed");
    }
    Ok(())
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn,super_dev=info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

fn cmd_init(slug: Option<String>, project_root: Option<PathBuf>, force: bool) -> Result<()> {
    let workspace = resolve_root(project_root)?;
    let slug = match slug {
        Some(s) if !s.is_empty() => s,
        _ => infer_slug(&workspace),
    };
    let manifest = super_dev_agent::SpecManifest::new(&slug);
    let path = manifest
        .write_to(&workspace, force)
        .with_context(|| format!("write {}", workspace.join("super-dev.yaml").display()))?;

    // Scaffold design infrastructure into the workspace so RAG and
    // /design work out of the box. Uses include_str! so the files are
    // embedded in the binary — no external data directory needed.
    let scaffolded = scaffold_design_infrastructure(&workspace);

    // Write a default .superdevrc config template so users can discover the
    // knowledge / quality / pipeline configuration surface. Idempotent.
    let superdevrc = workspace.join(".superdevrc");
    if !superdevrc.is_file() {
        let template = "# Super Dev project configuration. Edit and re-run to take effect.\n\
# Docs: https://github.com/anthropics/super-dev/blob/main/crates/super-dev-agent/src/config.rs\n\
\n[quality]\nthreshold = 90           # minimum weighted score to pass the quality gate\nskip_checks = []         # e.g. [\"Dark mode support\"]\n\
\n[pipeline]\nskip_phases = []         # e.g. [\"research\"]\nmax_review_rounds = 3    # doc structural review retries\n\
\n[knowledge]\nenabled = true           # enable BM25 / hybrid expert-knowledge retrieval\nengine = \"bm25\"          # bm25 (offline) or hybrid (needs OPENAI_API_KEY)\ntop_k = 6                # knowledge chunks injected per phase\n\
\n# Custom API provider for THIS project (overrides the global default_provider).\n\
# The named provider must be defined in ~/.super-dev/config.toml under [providers.<name>].\n\
# Empty string = explicitly disable any custom provider for this project.\n\
#[model]\n\
#provider = \"deepseek\"\n";
        let _ = std::fs::write(&superdevrc, template);
        println!("  config:  {}", superdevrc.display());
    }

    println!("Super Dev workspace initialised.");
    println!("  manifest: {}", path.display());
    println!(
        "  spec: {} | level: {} | profile: {} | slug: {slug}",
        super_dev_spec::SPEC_VERSION,
        manifest.level.as_str(),
        manifest.profile.as_str(),
    );
    if scaffolded > 0 {
        println!("  design: {scaffolded} files scaffolded into knowledge/");
    }
    println!("\nNext steps:");
    println!("  super-dev                          # launch the TUI (recommended)");
    println!("  super-dev run \"<requirement>\"      # or scripted / CI form");
    println!();
    println!("Inside the TUI:");
    println!("  /claude  or  /codex    drive a logged-in host CLI (Claude Code / Codex)");
    println!("  /provider add <name> openai <base_url> <model>  custom API (DeepSeek/OpenRouter/Ollama/...)");
    println!(
        "  /provider key <name> ${{ENV_VAR}}                  set the API key via env reference"
    );
    println!("  /provider <name>                                  switch to that custom provider");
    Ok(())
}

fn scaffold_design_infrastructure(workspace: &Path) -> usize {
    let files: &[(&str, &str)] = &[
        (
            "knowledge/design-systems/modern-minimal.md",
            include_str!("../../../knowledge/design-systems/modern-minimal.md"),
        ),
        (
            "knowledge/design-systems/editorial-clean.md",
            include_str!("../../../knowledge/design-systems/editorial-clean.md"),
        ),
        (
            "knowledge/design-systems/tech-utility.md",
            include_str!("../../../knowledge/design-systems/tech-utility.md"),
        ),
        (
            "knowledge/design-systems/soft-warm.md",
            include_str!("../../../knowledge/design-systems/soft-warm.md"),
        ),
        (
            "knowledge/design-systems/bold-geometric.md",
            include_str!("../../../knowledge/design-systems/bold-geometric.md"),
        ),
        (
            "knowledge/design-systems/00-craft-rules.md",
            include_str!("../../../knowledge/design-systems/00-craft-rules.md"),
        ),
        (
            "knowledge/seed-templates/saas-landing.md",
            include_str!("../../../knowledge/seed-templates/saas-landing.md"),
        ),
        (
            "knowledge/seed-templates/dashboard.md",
            include_str!("../../../knowledge/seed-templates/dashboard.md"),
        ),
        (
            "knowledge/seed-templates/blog-content.md",
            include_str!("../../../knowledge/seed-templates/blog-content.md"),
        ),
        (
            "knowledge/seed-templates/e-commerce.md",
            include_str!("../../../knowledge/seed-templates/e-commerce.md"),
        ),
        (
            "knowledge/seed-templates/auth-system.md",
            include_str!("../../../knowledge/seed-templates/auth-system.md"),
        ),
        (
            "knowledge/seed-templates/settings-page.md",
            include_str!("../../../knowledge/seed-templates/settings-page.md"),
        ),
        (
            "knowledge/seed-templates/docs-site.md",
            include_str!("../../../knowledge/seed-templates/docs-site.md"),
        ),
        // Expert methodology knowledge
        (
            "knowledge/experts/product-manager/methodology.md",
            include_str!("../../../knowledge/experts/product-manager/methodology.md"),
        ),
        (
            "knowledge/experts/architect/api-design.md",
            include_str!("../../../knowledge/experts/architect/api-design.md"),
        ),
        (
            "knowledge/experts/architect/security.md",
            include_str!("../../../knowledge/experts/architect/security.md"),
        ),
        (
            "knowledge/experts/frontend-lead/methodology.md",
            include_str!("../../../knowledge/experts/frontend-lead/methodology.md"),
        ),
        (
            "knowledge/experts/backend-lead/methodology.md",
            include_str!("../../../knowledge/experts/backend-lead/methodology.md"),
        ),
        (
            "knowledge/experts/qa-lead/test-strategy.md",
            include_str!("../../../knowledge/experts/qa-lead/test-strategy.md"),
        ),
        (
            "knowledge/experts/uiux-designer/methodology.md",
            include_str!("../../../knowledge/experts/uiux-designer/methodology.md"),
        ),
        (
            "knowledge/experts/devops/methodology.md",
            include_str!("../../../knowledge/experts/devops/methodology.md"),
        ),
    ];
    let mut count = 0;
    for (rel, content) in files {
        let target = workspace.join(rel);
        if target.exists() {
            continue;
        }
        if let Some(parent) = target.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::write(&target, content).is_ok() {
            count += 1;
        }
    }
    count
}

/// Launch the conversational TUI. With no CLI flags this is the
/// recommended entry — same as running `super-dev` bare.
async fn cmd_tui() -> Result<()> {
    let project_root = resolve_root(None)?;
    let opts = super_dev_tui::LaunchOptions {
        project_root,
        slug: String::new(),
        model: "claude-sonnet-4-6".to_string(),
    };
    super_dev_tui::run(opts).await.context("TUI session failed")
}

/// Bundled arguments for `cmd_run` (keeps the signature readable).
struct RunArgs {
    requirement: String,
    backend: Option<BackendArg>,
    model: String,
    project_root: Option<PathBuf>,
    slug: String,
}

/// Attach a live event sink to the runner and spawn a background printer.
/// Returns the sink-equipped runner and a JoinHandle for the printer task.
/// The caller must `drop(runner)` then `await` the handle after their block
/// finishes, so the channel closes cleanly.
fn attach_live_sink<R: super_dev_runtime::Runtime>(
    runner: AgentRunner<R>,
) -> (AgentRunner<R>, tokio::task::JoinHandle<()>) {
    let (sink, mut rx) = ChannelSink::new();
    let sink = Arc::new(sink);
    let runner = runner.with_event_sink(sink);
    let printer = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            print_engine_event(&event);
        }
    });
    (runner, printer)
}

/// Render a single engine event to stderr for live CLI progress.
fn print_engine_event(event: &super_dev_agent::EngineEvent) {
    use super_dev_agent::EngineEvent;
    match event {
        EngineEvent::PipelineStarted { slug, .. } => {
            eprintln!("▶ Pipeline started: {slug}");
        }
        EngineEvent::PhaseStarted { phase } => {
            eprintln!("▶ Phase: {}", phase.id());
        }
        EngineEvent::Note(msg) => {
            eprintln!("  {msg}");
        }
        EngineEvent::HostOutput { phase, line } => {
            eprintln!("  │ [{phase:?}] {line}");
        }
        EngineEvent::ArtifactWritten { phase, path } => {
            eprintln!("  ✓ {} → {}", phase.id(), path.display());
        }
        EngineEvent::PhaseCompleted { phase } => {
            eprintln!("  ✓ {} complete", phase.id());
        }
        EngineEvent::GateOpened { gate } => {
            eprintln!(
                "
⏸  Gate: {} — run `super-dev continue` to proceed.",
                gate.id_str()
            );
        }
        EngineEvent::BlockCompleted { final_phase, .. } => {
            eprintln!("✓ Block complete at {}", final_phase.id());
        }
        EngineEvent::VerifyPassed { phase, duration_ms } => {
            eprintln!("  ✓ {} verify OK ({}ms)", phase.id(), duration_ms);
        }
        EngineEvent::VerifyFailed { phase, .. } => {
            eprintln!("  ✗ {} verify FAILED", phase.id());
        }
        EngineEvent::VerifySkipped { phase, .. } => {
            eprintln!("  ⊘ {} verify skipped", phase.id());
        }
        _ => {}
    }
    let _ = std::io::stderr().flush();
}

async fn cmd_run(args: RunArgs) -> Result<()> {
    let project_root = resolve_root(args.project_root)?;
    let opts = RunOptions {
        project_root: project_root.clone(),
        requirement: args.requirement,
        slug: args.slug,
        model: args.model,
        backend: args
            .backend
            .as_ref()
            .map_or(String::new(), |b| b.id().to_string()),
        design_system: String::new(),
        seed_template: String::new(),
    };

    // Two modes:
    //   --backend <host>  → drive a logged-in host CLI as the worker
    //   (default)         → offline deterministic templates
    let (report, runtime_label) = if let Some(backend) = args.backend {
        let driver = super_dev_host::driver_for(backend.id())
            .ok_or_else(|| anyhow::anyhow!("unknown backend `{}`", backend.id()))?;
        match driver.probe().await {
            super_dev_host::ProbeResult::Ready { version } => {
                println!("Backend {} ready ({version}).", driver.display_name());
            }
            super_dev_host::ProbeResult::NotInstalled { program } => {
                anyhow::bail!(
                    "backend `{}` not available: `{program}` is not on PATH. \
                     Install / log in to the host CLI first, or omit --backend to run offline.",
                    backend.id()
                );
            }
            super_dev_host::ProbeResult::Unhealthy { detail } => {
                anyhow::bail!("backend `{}` is unhealthy: {detail}", backend.id());
            }
        }
        let label = format!(
            "Host CLI worker — {} ({})",
            driver.display_name(),
            backend.id()
        );
        let runner = AgentRunner::new(driver, opts);
        runner.start().context("failed to start agent")?;
        let (runner, printer) = attach_live_sink(runner);
        let report = runner
            .run_initial_block(true)
            .await
            .context("pipeline failure")?;
        drop(runner);
        let _ = printer.await;
        (report, label)
    } else {
        let label = "Offline deterministic templates (no AI; demos / CI)".to_string();
        let runner = AgentRunner::new(OfflineRuntime::new(RuntimeKind::Anthropic), opts);
        runner.start().context("failed to start agent")?;
        let (runner, printer) = attach_live_sink(runner);
        let report = runner
            .run_initial_block(false)
            .await
            .context("pipeline failure")?;
        drop(runner);
        let _ = printer.await;
        (report, label)
    };

    print_report(&project_root, &runtime_label, &report);
    Ok(())
}

fn print_report(project_root: &Path, runtime_label: &str, report: &RunReport) {
    println!(
        "Super Dev — {}.",
        if report.paused_at.is_some() {
            "pipeline paused"
        } else {
            "pipeline complete"
        }
    );
    println!("  workspace: {}", project_root.display());
    println!("  runtime: {runtime_label}");
    println!(
        "  final phase: {} | active gate: {}",
        report.final_phase.id(),
        report.paused_at.map_or("none", Gate::id_str)
    );
    println!("  artifacts:");
    for phase_out in &report.completed {
        for a in &phase_out.artifacts {
            if let Ok(rel) = a.strip_prefix(project_root) {
                println!("    - {}", rel.display());
            } else {
                println!("    - {}", a.display());
            }
        }
    }
    match report.paused_at {
        Some(Gate::DocsConfirm) => {
            println!("\nReview the three core docs, then run:");
            println!("  super-dev continue       # approve and advance");
            println!("  super-dev revise \"…\"     # request changes inside the gate");
        }
        Some(Gate::PreviewConfirm) => {
            println!("\nVerify the preview matches the UIUX doc, then run:");
            println!(
                "  super-dev continue       # approve and advance to backend / quality / delivery"
            );
            println!("  super-dev revise \"…\"     # request changes inside the gate");
        }
        None => {
            println!("\nDelivery complete. The proof pack lives in `release/`.");
            println!("Run `super-dev report` if you want to refresh the compliance mapping.");
        }
    }
}

/// Which block to drive relative to the active gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GateBlock {
    /// Approve the gate and advance past it.
    Continue,
    /// Keep the gate; regenerate the artifacts that produced it.
    Revise,
}

/// Parse the persisted `active_gate` string into a [`Gate`], or bail with
/// a clear message when none is open / the value is unrecognised.
fn resolve_active_gate(state: &super_dev_agent::WorkflowState) -> Result<Gate> {
    if state.active_gate.is_empty() {
        anyhow::bail!("no active gate to approve (current phase: {})", state.phase);
    }
    // Delegate to the spec crate's typed parser (case-insensitive,
    // fail-open) instead of hand-matching the two ids here — keeps this
    // call site in sync with Gate if a third gate is ever added.
    match Gate::from_id(&state.active_gate) {
        Some(g) => Ok(g),
        None => anyhow::bail!(
            "unknown active_gate `{}` in workflow-state.json",
            state.active_gate
        ),
    }
}

/// Shared driver for `continue` / `revise`: reconstruct `RunOptions` from
/// the persisted state, resolve + probe the backend, run the appropriate
/// block, and print the report. `requirement_override` lets `revise` fold
/// the user's feedback into the worker prompt.
async fn drive_gate_block(
    project_root: &Path,
    state: &super_dev_agent::WorkflowState,
    gate: Gate,
    backend_override: Option<BackendArg>,
    requirement_override: Option<String>,
    mode: GateBlock,
) -> Result<()> {
    // Reconstruct slug + requirement so `continue` / `revise` resolve the
    // same artifacts the original `run` produced.
    let slug = if state.slug.is_empty() {
        infer_slug(project_root)
    } else {
        state.slug.clone()
    };
    let requirement = requirement_override.unwrap_or_else(|| {
        if state.requirement.is_empty() {
            state
                .note
                .split_once(": ")
                .map_or("(no requirement recorded)", |x| x.1)
                .to_string()
        } else {
            state.requirement.clone()
        }
    });

    // Resolve backend: explicit flag > persisted state > offline.
    let backend_id: Option<String> = backend_override
        .as_ref()
        .map(|b| b.id().to_string())
        .or_else(|| {
            if state.backend.is_empty() {
                None
            } else {
                Some(state.backend.clone())
            }
        });

    let opts = RunOptions {
        project_root: project_root.to_path_buf(),
        requirement,
        slug,
        model: "claude-sonnet-4-6".to_string(),
        backend: backend_id.clone().unwrap_or_default(),
        design_system: String::new(),
        seed_template: String::new(),
    };

    let use_runtime = backend_id.is_some();
    let (report, runtime_label) = if let Some(id) = backend_id {
        let backend = BackendArg::from_id(&id)
            .ok_or_else(|| anyhow::anyhow!("unknown backend `{id}` in workflow-state.json"))?;
        let driver = super_dev_host::driver_for(backend.id())
            .ok_or_else(|| anyhow::anyhow!("no driver registered for `{}`", backend.id()))?;
        match driver.probe().await {
            super_dev_host::ProbeResult::Ready { version } => {
                println!("Backend {} ready ({version}).", driver.display_name());
            }
            super_dev_host::ProbeResult::NotInstalled { program } => {
                anyhow::bail!(
                    "backend `{}` not available: `{program}` is not on PATH. \
                     Pass --backend offline (or no --backend) to fall back.",
                    backend.id()
                );
            }
            super_dev_host::ProbeResult::Unhealthy { detail } => {
                anyhow::bail!("backend `{}` is unhealthy: {detail}", backend.id());
            }
        }
        let label = format!(
            "Host CLI worker — {} ({})",
            driver.display_name(),
            backend.id()
        );
        let runner = AgentRunner::new(driver, opts);
        runner.start().context("failed to start agent")?;
        let (runner, printer) = attach_live_sink(runner);
        let report = if mode == GateBlock::Continue {
            runner.continue_from_gate(gate).await
        } else {
            runner.revise_at_gate(gate, use_runtime).await
        }
        .context("pipeline failure")?;
        drop(runner);
        let _ = printer.await;
        (report, label)
    } else {
        let runner = AgentRunner::new(OfflineRuntime::new(RuntimeKind::Anthropic), opts);
        runner.start().context("failed to start agent")?;
        let (runner, printer) = attach_live_sink(runner);
        let report = if mode == GateBlock::Continue {
            runner.continue_from_gate(gate).await
        } else {
            runner.revise_at_gate(gate, use_runtime).await
        }
        .context("pipeline failure")?;
        drop(runner);
        let _ = printer.await;
        (
            report,
            "Offline deterministic templates (no AI; demos / CI)".to_string(),
        )
    };

    print_report(project_root, &runtime_label, &report);
    Ok(())
}

async fn cmd_continue(
    project_root: Option<PathBuf>,
    backend_override: Option<BackendArg>,
) -> Result<()> {
    let project_root = resolve_root(project_root)?;
    let state = match super_dev_agent::read_workflow_state_diagnostic(&project_root) {
        super_dev_agent::ReadState::Ok(s) => s,
        super_dev_agent::ReadState::Missing => anyhow::bail!(
            "no .super-dev/workflow-state.json — run `super-dev run` first"
        ),
        super_dev_agent::ReadState::Corrupt { path, error } => anyhow::bail!(
            "workflow-state.json at {} is corrupt ({error}).              Run `super-dev rollback latest` or delete it, then `super-dev run` again.",
            path.display()
        ),
    };
    let gate = resolve_active_gate(&state)?;

    // Record the approval as evidence
    let clause = match gate {
        Gate::DocsConfirm => "SD-FLOW-002",
        Gate::PreviewConfirm => "SD-FLOW-003",
    };
    let _ = record_tool_call(
        &project_root,
        "super-dev/cli.continue",
        "",
        "approved",
        clause,
        &format!("user approved gate {}", gate.id_str()),
        "",
        None,
    );

    drive_gate_block(
        &project_root,
        &state,
        gate,
        backend_override,
        None,
        GateBlock::Continue,
    )
    .await
}

async fn cmd_revise(text: String, project_root: Option<PathBuf>) -> Result<()> {
    let project_root = resolve_root(project_root)?;
    let state = match super_dev_agent::read_workflow_state_diagnostic(&project_root) {
        super_dev_agent::ReadState::Ok(s) => s,
        super_dev_agent::ReadState::Missing => anyhow::bail!(
            "no .super-dev/workflow-state.json — run `super-dev run` first"
        ),
        super_dev_agent::ReadState::Corrupt { path, error } => anyhow::bail!(
            "workflow-state.json at {} is corrupt ({error}).              Run `super-dev rollback latest` or delete it, then `super-dev run` again.",
            path.display()
        ),
    };
    let gate = resolve_active_gate(&state)?;
    let outcome = classify_reply(&text);
    match outcome {
        GateOutcome::Revise(notes) => {
            let _ = record_tool_call(
                &project_root,
                "super-dev/cli.revise",
                "",
                "revise",
                "SD-FLOW-004",
                &notes,
                "",
                None,
            );
            // 4.8 auto-sediment: capture this revision as an ADR + lesson.
            let _ = super_dev_agent::capture_gate_revision(
                &project_root,
                &state.active_gate,
                &notes,
                &state.requirement,
            );
            println!(
                "Revising at gate `{}` — regenerating artifacts with your feedback…",
                state.active_gate
            );
            // Fold the revision into the requirement so the worker
            // actually incorporates the feedback, then re-run the block
            // that produced this gate (docs for docs_confirm, frontend
            // for preview_confirm). The pipeline pauses at the same gate.
            let base_req = if state.requirement.is_empty() {
                state
                    .note
                    .split_once(": ")
                    .map_or(String::new(), |x| x.1.to_string())
            } else {
                state.requirement.clone()
            };
            let revised = format!("{base_req}\n\n## Revision request\n{notes}");
            drive_gate_block(
                &project_root,
                &state,
                gate,
                None,
                Some(revised),
                GateBlock::Revise,
            )
            .await
        }
        GateOutcome::Approved => {
            // Defensive: user said "继续" via revise — treat as approval.
            println!("input parsed as approval; treating as `continue`.");
            cmd_continue(Some(project_root), None).await
        }
        GateOutcome::Cancelled => {
            anyhow::bail!("user cancelled the pipeline");
        }
    }
}

fn cmd_spec(clauses_only: bool) -> Result<()> {
    if clauses_only {
        println!("# {SPEC_VERSION} — clause table\n");
        println!("| ID | Layer | Level | Section | Title |");
        println!("|---|---|---|---|---|");
        for c in CLAUSES {
            println!(
                "| {} | {:?} | {:?} | {} | {} |",
                c.id, c.layer, c.level, c.section, c.title
            );
        }
        let chain: Vec<&str> = PHASE_CHAIN.iter().map(|p| p.id()).collect();
        println!("\nPhase chain: {}", chain.join(" → "));
        return Ok(());
    }
    println!(
        "{}",
        include_str!("../../../spec/SUPER_DEV_HOST_SPEC_V1.md")
    );
    Ok(())
}

fn cmd_history(project_root: Option<PathBuf>) -> Result<()> {
    let project_root = resolve_root(project_root)?;
    let snaps = list_snapshots(&project_root);
    if snaps.is_empty() {
        println!(
            "No snapshots yet. Snapshots are created automatically on every phase transition."
        );
        println!("Run `super-dev run` / `super-dev continue` to advance the pipeline.");
        return Ok(());
    }
    println!(
        "Available rollback snapshots (newest first):
"
    );
    let state = read_workflow_state(&project_root);
    for (i, ts) in snaps.iter().enumerate() {
        // Try to show what phase each snapshot would restore.
        let snap_path = project_root
            .join(".super-dev/history")
            .join(format!("{ts}.json"));
        let phase = std::fs::read_to_string(&snap_path)
            .ok()
            .and_then(|t| serde_json::from_str::<WorkflowState>(&t).ok())
            .map_or("?".to_string(), |s| s.phase);
        let marker = if i == 0 { " (latest)" } else { "" };
        println!("  {ts}  →  phase: {phase}{marker}");
    }
    let _ = state;
    println!(
        "
Roll back with:  super-dev rollback latest"
    );
    Ok(())
}

fn cmd_rollback(timestamp: String, project_root: Option<PathBuf>) -> Result<()> {
    let project_root = resolve_root(project_root)?;
    let snaps = list_snapshots(&project_root);
    if snaps.is_empty() {
        anyhow::bail!("no snapshots available — run `super-dev history` to check");
    }
    let target = if timestamp == "latest" {
        snaps[0].clone()
    } else {
        // Allow partial match (e.g. user passes 20260614T12 to match 20260614T120000.123).
        let matches: Vec<&String> = snaps.iter().filter(|s| s.starts_with(&timestamp)).collect();
        match matches.len() {
            0 => anyhow::bail!("no snapshot matches `{timestamp}`. Run `super-dev history`."),
            1 => matches[0].clone(),
            _ => anyhow::bail!(
                "`{timestamp}` is ambiguous ({} matches). Use more digits.",
                matches.len()
            ),
        }
    };
    let before = read_workflow_state(&project_root).map_or("none".to_string(), |s| s.phase);
    restore_snapshot(&project_root, &target)?;
    let after = read_workflow_state(&project_root).map_or("none".to_string(), |s| s.phase);
    println!("Rolled back to snapshot {target}.");
    println!("  phase: {before} → {after}");
    println!("Note: artifacts on disk are NOT reverted — only workflow-state.json.");
    println!(
        "      The pipeline now resumes from phase `{after}` on the next `super-dev continue`."
    );
    Ok(())
}

fn cmd_verify(project_root: Option<PathBuf>) -> Result<()> {
    let project_root = resolve_root(project_root)?;
    println!("workspace: {}", project_root.display());
    println!(
        "super-dev: {} (spec {SPEC_VERSION})",
        env!("CARGO_PKG_VERSION")
    );

    // --- spec manifest (SD-META-001) ---
    println!("\n## Spec manifest");
    if let Some(m) = super_dev_agent::SpecManifest::read_from(&project_root) {
        println!(
            "  super-dev.yaml: version={} level={} profile={} declared_by={}",
            m.spec_version,
            m.level.as_str(),
            m.profile.as_str(),
            m.declared_by,
        );
    } else {
        println!("  <no super-dev.yaml — run `super-dev init` (SD-META-001)>");
    }

    // --- workflow state ---
    println!("\n## Workflow state");
    if let Some(s) = read_workflow_state(&project_root) {
        print_state(&s);
    } else {
        println!("  <no .super-dev/workflow-state.json — run `super-dev run \"<requirement>\"`>");
    }

    // --- evidence chain ---
    println!("\n## Evidence chain");
    let audit = project_root.join(".super-dev/audit");
    let api_log = audit.join("frontend-api-calls.jsonl");
    let tool_log = audit.join("tool-calls.jsonl");
    if api_log.is_file() {
        println!(
            "  - SD-EVID-001 frontend-api-calls.jsonl ({} rows)",
            line_count(&api_log)
        );
    }
    if tool_log.is_file() {
        println!(
            "  - SD-EVID-002 tool-calls.jsonl ({} rows)",
            line_count(&tool_log)
        );
    }
    if !api_log.is_file() && !tool_log.is_file() {
        println!("  <no audit logs yet>");
    }

    // --- latest quality gate ---
    println!("\n## Quality gate");
    if let Some((path, passed, total)) = latest_quality_report(&project_root) {
        println!(
            "  {} → {}/100 ({})",
            path.display(),
            total,
            if passed { "PASSED" } else { "FAILED" }
        );
    } else {
        println!("  <no quality report yet — runs at the `quality` phase>");
    }

    // --- proof packs ---
    println!("\n## Proof packs");
    let release = project_root.join("release");
    let mut packs: Vec<_> = std::fs::read_dir(&release)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .filter(|e| {
                    let n = e.file_name();
                    let s = n.to_string_lossy();
                    s.starts_with("proof-pack-") && s.ends_with(".zip")
                })
                .map(|e| e.path())
                .collect()
        })
        .unwrap_or_default();
    packs.sort();
    if packs.is_empty() {
        println!("  <no proof pack yet — runs at the `delivery` phase>");
    } else {
        for p in packs.iter().rev().take(3) {
            let size = std::fs::metadata(p).map_or(0, |m| m.len());
            println!("  - {} ({} KiB)", p.display(), size / 1024);
        }
    }
    Ok(())
}

fn latest_quality_report(root: &Path) -> Option<(PathBuf, bool, i64)> {
    let dir = root.join("output");
    let mut candidates: Vec<_> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|s| s.to_str()) == Some("json")
                && p.file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|n| n.ends_with("-quality-gate.json"))
        })
        .collect();
    candidates.sort();
    let latest = candidates.last()?.clone();
    let body = std::fs::read_to_string(&latest).ok()?;
    let v: serde_json::Value = serde_json::from_str(&body).ok()?;
    let passed = v.get("passed")?.as_bool().unwrap_or(false);
    let total = v.get("total_score")?.as_i64().unwrap_or(0);
    Some((latest, passed, total))
}

fn cmd_report(slug: Option<String>, project_root: Option<PathBuf>) -> Result<()> {
    let project_root = resolve_root(project_root)?;
    let slug = match slug {
        Some(s) if !s.is_empty() => s,
        _ => infer_slug(&project_root),
    };

    // 1. Compliance mapping (SD-EVID-004)
    match write_compliance_mapping(&project_root, &slug) {
        Some((path, doc)) => {
            println!("Wrote {} ({} clauses).", path.display(), doc.clauses.len());
            println!("  frameworks: {}", doc.summary.frameworks.join(", "));
        }
        None => {
            println!("(no compliance mapping yet — run `super-dev run` first)");
        }
    }

    // 2. Project health summary — tech-debt trend + learned lessons.
    println!("\n--- project health ---");
    let output_dir = project_root.join("output");
    let current_debt = super_dev_agent::tech_debt::scan_debt(&output_dir);
    let ledger = super_dev_agent::tech_debt::read_ledger(&project_root);
    if !ledger.is_empty() || !current_debt.is_empty() {
        let trend = super_dev_agent::tech_debt::diff_against_ledger(&current_debt, &ledger);
        println!(
            "tech-debt: {} open ({} new, {} resolved, net {:+})",
            trend.current_count, trend.new_count, trend.resolved_count, trend.net_change
        );
        let summary = super_dev_agent::tech_debt::summarise(&current_debt);
        if summary.total > 0 {
            let kinds: Vec<String> = summary
                .by_kind
                .iter()
                .map(|(k, n)| format!("{k}: {n}"))
                .collect();
            println!("  by kind: {}", kinds.join(", "));
            println!("  severity score: {}", summary.severity_total);
        }
    } else {
        println!("tech-debt: none detected");
    }

    let lessons = super_dev_agent::list_sedimented_lessons(&project_root);
    if lessons.is_empty() {
        println!("learned lessons: none yet");
    } else {
        println!("learned lessons: {} sedimented file(s)", lessons.len());
        for p in lessons.iter().take(5) {
            if let Some(name) = p.file_name() {
                println!("  • {}", name.to_string_lossy());
            }
        }
        if lessons.len() > 5 {
            println!("  ... and {} more", lessons.len() - 5);
        }
    }
    Ok(())
}

fn print_state(s: &WorkflowState) {
    println!(
        "workflow-state: phase={} active_gate={} worker={} last_transition_at={}",
        s.phase,
        if s.active_gate.is_empty() {
            "<none>"
        } else {
            &s.active_gate
        },
        if s.backend.is_empty() {
            "offline-templates"
        } else {
            s.backend.as_str()
        },
        s.last_transition_at
    );
    if !s.note.is_empty() {
        println!("note: {}", s.note);
    }
}

fn line_count(path: &std::path::Path) -> usize {
    std::fs::read_to_string(path).map_or(0, |t| t.lines().filter(|l| !l.trim().is_empty()).count())
}

fn resolve_root(project_root: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(root) = project_root {
        return Ok(root);
    }
    // Default to cwd. We deliberately do NOT honour CLAUDE_PROJECT_DIR /
    // SUPER_DEV_PROJECT_DIR here: when the user runs `super-dev` from a
    // directory, that directory IS the workspace they mean. The env vars
    // would override cwd even when the user cd'd elsewhere (e.g. a smoke
    // test in /tmp while CLAUDE_PROJECT_DIR still points at the real repo),
    // which is surprising and wrong for the CLI entry.
    std::env::current_dir().context("could not resolve project root (cwd unreadable)")
}

fn infer_slug(project_root: &std::path::Path) -> String {
    project_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BackendArg must cover EXACTLY the ids that super-dev-host registers
    /// as drivers. If you add a host driver, you must add the selector
    /// variant too — otherwise `--backend <new>` is unreachable from the
    /// CLI even though `driver_for` builds it.
    #[test]
    fn backend_arg_ids_match_host() {
        let host_ids: std::collections::HashSet<&str> =
            super_dev_host::BACKEND_IDS.iter().copied().collect();
        let arg_ids: std::collections::HashSet<&str> = BACKEND_ARG_IDS.iter().copied().collect();
        assert_eq!(
            host_ids,
            arg_ids,
            "BackendArg and super-dev-host::BACKEND_IDS drifted.\n\
             only in host: {:?}\n\
             only in arg : {:?}",
            host_ids.difference(&arg_ids).collect::<Vec<_>>(),
            arg_ids.difference(&host_ids).collect::<Vec<_>>(),
        );
    }

    /// Every selector id must resolve through driver_for — i.e. selecting a
    /// backend in the CLI can never 404 at driver-build time.
    #[test]
    fn every_backend_arg_has_a_driver() {
        for id in BACKEND_ARG_IDS {
            assert!(
                super_dev_host::driver_for(id).is_some(),
                "BackendArg id `{id}` has no driver in super-dev-host::driver_for"
            );
        }
    }

    /// id() / from_id() must be round-trippable for every variant.
    #[test]
    fn backend_arg_id_roundtrip() {
        for id in BACKEND_ARG_IDS {
            let var =
                BackendArg::from_id(id).unwrap_or_else(|| panic!("from_id({id}) returned None"));
            assert_eq!(var.id(), *id, "id()/from_id() not inverse for {id}");
        }
    }
}
