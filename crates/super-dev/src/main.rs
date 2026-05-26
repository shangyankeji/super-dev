//! `super-dev` — the single binary entrypoint.
//!
//! 4.4+ verbs (the standalone `tui` subcommand and the legacy
//! `install` / `uninstall` / `hook` verbs were dropped — the TUI is now
//! the default entry, and host injection is gone):
//!
//! - (none)                  launch the chat TUI (recommended entry)
//! - `init`                  write the `super-dev.yaml` spec manifest
//! - `run <req>`             drive research → docs, pause at `docs_confirm`
//! - `continue`              approve the active gate (reuses persisted backend)
//! - `revise <text>`         keep the active gate, request revisions
//! - `spec`                  print `SUPER_DEV_HOST_SPEC_V1`
//! - `verify`                inspect workspace conformance + worker
//! - `report`                emit the SD-EVID-004 compliance mapping
//! - `doctor`                self-test
//! - `examples` / `guide`    cheat-sheet / walkthrough
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

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use super_dev_agent::{
    classify_reply, read_workflow_state, AgentRunner, Gate, GateOutcome, RunOptions, RunReport,
    WorkflowState,
};
use super_dev_governance::{compliance::write_compliance_mapping, record_tool_call};
use super_dev_runtime::{OfflineRuntime, RuntimeKind};
use super_dev_spec::{CLAUSES, PHASE_CHAIN, SPEC_VERSION};

/// Super Dev — a coach for AI coding hosts.
#[derive(Debug, Parser)]
#[command(
    name = "super-dev",
    version,
    about = "AI 编码的项目经理 — drives your logged-in AI coding CLI (11 backends supported) through a 9-phase commercial delivery pipeline. No API key needed.",
    long_about = None,
)]
struct Cli {
    /// Subcommand: `init` / `run` / `continue` / `revise` / `spec` /
    /// `verify` / `report` / `doctor` / `examples` / `guide`.
    /// **Omit to launch the TUI** — that is the recommended user entry.
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
        long_about = "Run the pipeline non-interactively from `research` to the first\n\
                      gate (`docs_confirm`). Workers:\n\
                      \n  \
                      --backend claude-code    Anthropic Claude Code\n  \
                      --backend codex          OpenAI Codex\n  \
                      --backend gemini         Google Gemini CLI\n  \
                      --backend droid          Factory.ai Droid\n  \
                      --backend opencode       OpenCode (open-source)\n  \
                      --backend cursor-agent   Cursor Agent (headless)\n  \
                      --backend qwen           Alibaba Qwen Code\n  \
                      --backend continue       Continue (cn)\n  \
                      --backend copilot        GitHub Copilot CLI\n  \
                      --backend aider          Aider (--message mode)\n  \
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
                      super-dev run \"...\" --backend gemini                   # Google Gemini\n  \
                      super-dev run \"...\" --backend droid                    # Factory Droid\n  \
                      super-dev run \"...\" --backend cursor-agent             # Cursor headless\n  \
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
    /// Print the `SUPER_DEV_HOST_SPEC_V1` specification.
    #[command(
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
        long_about = "Print a curated set of common-workflow examples. Useful as a\n\
                      cheat-sheet — every important command is shown with a real\n\
                      invocation."
    )]
    Examples,
    /// 60-second guided walkthrough for new users (interactive).
    #[command(
        long_about = "Run a short, mode-by-mode walkthrough of Super Dev. Prints what\n\
                      each command does, how the pipeline progresses, and what\n\
                      artifacts you can expect. No side effects."
    )]
    Guide,
}

/// Host CLI backend selector for `super-dev run --backend`.
///
/// Each variant maps 1:1 onto a registered [`super_dev_host::HostDriver`].
/// Adding a new host requires (1) a driver in `super-dev-host`, (2) a
/// variant here, (3) the matching id pair in [`BackendArg::id`] /
/// [`BackendArg::from_id`].
#[derive(Debug, Copy, Clone, clap::ValueEnum)]
enum BackendArg {
    /// Drive Claude Code CLI.
    ClaudeCode,
    /// Drive Codex CLI.
    Codex,
    /// Drive Gemini CLI.
    Gemini,
    /// Drive Droid CLI.
    Droid,
    /// Drive OpenCode CLI.
    Opencode,
    /// Drive Qwen Code CLI.
    Qwen,
    /// Drive Copilot CLI.
    Copilot,
    /// Drive Trae CLI.
    Trae,
    /// Drive CodeBuddy CLI.
    Codebuddy,
    /// Drive Qoder CLI.
    Qoder,
    /// Drive Kimi Code CLI.
    Kimi,
}

impl BackendArg {
    fn id(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::Droid => "droid",
            Self::Opencode => "opencode",
            Self::Qwen => "qwen",
            Self::Copilot => "copilot",
            Self::Trae => "trae",
            Self::Codebuddy => "codebuddy",
            Self::Qoder => "qoder",
            Self::Kimi => "kimi",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        match id {
            "claude-code" => Some(Self::ClaudeCode),
            "codex" => Some(Self::Codex),
            "gemini" => Some(Self::Gemini),
            "droid" => Some(Self::Droid),
            "opencode" => Some(Self::Opencode),
            "qwen" => Some(Self::Qwen),
            "copilot" => Some(Self::Copilot),
            "trae" => Some(Self::Trae),
            "codebuddy" => Some(Self::Codebuddy),
            "qoder" => Some(Self::Qoder),
            "kimi" => Some(Self::Kimi),
            _ => None,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    // No subcommand → launch the TUI. The user types their requirement
    // inside the conversation — this is the recommended user entry.
    let Some(command) = cli.command else {
        return cmd_tui().await;
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
        Command::Spec { clauses } => cmd_spec(clauses),
        Command::Verify { project_root } => cmd_verify(project_root),
        Command::Report { slug, project_root } => cmd_report(slug, project_root),
        Command::Doctor { project_root } => cmd_doctor(project_root),
        Command::Examples => cmd_examples(),
        Command::Guide => cmd_guide(),
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
    println!("  super-dev                          # launch the TUI");
    println!("  super-dev run \"<requirement>\"      # or scripted form");
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
        let report = runner
            .run_initial_block(true)
            .await
            .context("pipeline failure")?;
        (report, label)
    } else {
        let label = "Offline deterministic templates (no AI; demos / CI)".to_string();
        let runner = AgentRunner::new(OfflineRuntime::new(RuntimeKind::Anthropic), opts);
        runner.start().context("failed to start agent")?;
        let report = runner
            .run_initial_block(false)
            .await
            .context("pipeline failure")?;
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

async fn cmd_continue(
    project_root: Option<PathBuf>,
    backend_override: Option<BackendArg>,
) -> Result<()> {
    let project_root = resolve_root(project_root)?;
    let state = read_workflow_state(&project_root).ok_or_else(|| {
        anyhow::anyhow!("no .super-dev/workflow-state.json — run `super-dev run` first")
    })?;
    if state.active_gate.is_empty() {
        anyhow::bail!("no active gate to approve (current phase: {})", state.phase);
    }
    let gate = match state.active_gate.as_str() {
        "docs_confirm" => Gate::DocsConfirm,
        "preview_confirm" => Gate::PreviewConfirm,
        other => anyhow::bail!("unknown active_gate `{other}` in workflow-state.json"),
    };

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

    // Reconstruct RunOptions from the persisted workflow state so the
    // slug and requirement stay consistent across `run` / `continue`.
    let slug = if state.slug.is_empty() {
        infer_slug(&project_root)
    } else {
        state.slug.clone()
    };
    let requirement = if state.requirement.is_empty() {
        state
            .note
            .split_once(": ")
            .map_or("(no requirement recorded)", |x| x.1)
            .to_string()
    } else {
        state.requirement.clone()
    };

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
        project_root: project_root.clone(),
        requirement,
        slug,
        model: "claude-sonnet-4-6".to_string(),
        backend: backend_id.clone().unwrap_or_default(),
        design_system: String::new(),
        seed_template: String::new(),
    };

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
        let report = runner
            .continue_from_gate(gate)
            .await
            .context("pipeline failure")?;
        (report, label)
    } else {
        let runner = AgentRunner::new(OfflineRuntime::new(RuntimeKind::Anthropic), opts);
        let report = runner
            .continue_from_gate(gate)
            .await
            .context("pipeline failure")?;
        (
            report,
            "Offline deterministic templates (no AI; demos / CI)".to_string(),
        )
    };

    print_report(&project_root, &runtime_label, &report);
    Ok(())
}

async fn cmd_revise(text: String, project_root: Option<PathBuf>) -> Result<()> {
    let project_root = resolve_root(project_root)?;
    let state = read_workflow_state(&project_root).ok_or_else(|| {
        anyhow::anyhow!("no .super-dev/workflow-state.json — run `super-dev run` first")
    })?;
    if state.active_gate.is_empty() {
        anyhow::bail!("no active gate; revision applies only inside a gate.");
    }
    let outcome = classify_reply(&text);
    match outcome {
        GateOutcome::Revise(notes) => {
            println!("revision noted; still in gate `{}`.", state.active_gate);
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
            println!("To re-execute the affected phase after editing artifacts, run `super-dev run` again.");
            Ok(())
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
    match write_compliance_mapping(&project_root, &slug) {
        Some((path, doc)) => {
            println!("Wrote {} ({} clauses).", path.display(), doc.clauses.len());
            println!("  frameworks: {}", doc.summary.frameworks.join(", "));
        }
        None => {
            anyhow::bail!(
                "no evidence to map yet — run `super-dev run` and let the agent emit \
                 audit logs / quality report first."
            );
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

fn project_root_from_env() -> PathBuf {
    if let Ok(v) = std::env::var("CLAUDE_PROJECT_DIR") {
        let p = PathBuf::from(v);
        if p.is_dir() {
            return p;
        }
    }
    if let Ok(v) = std::env::var("SUPER_DEV_PROJECT_DIR") {
        let p = PathBuf::from(v);
        if p.is_dir() {
            return p;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn resolve_root(project_root: Option<PathBuf>) -> Result<PathBuf> {
    project_root
        .map_or_else(std::env::current_dir, Ok)
        .context("could not resolve project root")
}

fn infer_slug(project_root: &std::path::Path) -> String {
    project_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_string()
}
