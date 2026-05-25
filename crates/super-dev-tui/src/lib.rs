//! `super-dev-tui` — Claude Code-style terminal app that drives the
//! Super Dev pipeline.
//!
//! Two screens:
//!
//! 1. **Picker** (first launch only) — `↑↓` to choose a worker
//!    (claude-code / codex / offline), Enter to save to
//!    `~/.super-dev/config.toml`.
//! 2. **Chat** — persistent input box + scrolling conversation history.
//!    Type a requirement, watch the pipeline narrate. Slash commands
//!    (`/claude` `/codex` `/offline` `/init` `/continue` `/revise`
//!    `/spec` `/verify` `/doctor` `/help` `/quit` `/clear`) switch
//!    worker, drive gates, etc.
//!
//! Pipeline blocks run in background `tokio` tasks; each emits
//! [`EngineEvent`]s through a shared [`ChannelSink`]. The event loop
//! folds those events + key presses into [`App`] state and redraws.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::assigning_clones,
    clippy::format_push_string
)]

pub mod app;
pub mod config;
pub mod ui;

use std::io::Stdout;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{Event, EventStream, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use super_dev_agent::{AgentRunner, ChannelSink, EngineEvent, EventSink, Gate, RunOptions};
use super_dev_host::driver_for;
use super_dev_runtime::{OfflineRuntime, Runtime, RuntimeKind};

use crate::app::{Action, App};

/// Launch parameters for [`run`].
#[derive(Debug, Clone)]
pub struct LaunchOptions {
    /// Workspace root.
    pub project_root: PathBuf,
    /// Project slug (empty → inferred from workspace dir name).
    pub slug: String,
    /// Model identifier (host drivers may ignore).
    pub model: String,
}

impl LaunchOptions {
    /// Effective slug — uses cwd dir name when `slug` is empty.
    #[must_use]
    pub fn effective_slug(&self) -> String {
        if !self.slug.is_empty() {
            return self.slug.clone();
        }
        self.project_root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project")
            .to_string()
    }
}

/// Launch the TUI. Blocks until the user quits.
pub async fn run(opts: LaunchOptions) -> Result<()> {
    let config_path = config::default_path();
    let cfg = config::load_from(&config_path);
    let mut app = App::new(
        opts.effective_slug(),
        cfg,
        config_path,
        opts.project_root.clone(),
    );

    let mut terminal = setup_terminal().context("failed to set up terminal")?;
    let result = event_loop(&mut terminal, &mut app, opts).await;
    restore_terminal(&mut terminal).ok();
    result
}

fn build_brain(backend: Option<&str>) -> Result<Box<dyn Runtime>> {
    match backend {
        None | Some("offline") => Ok(Box::new(OfflineRuntime::new(RuntimeKind::Anthropic))),
        Some(id) => {
            let driver = driver_for(id).ok_or_else(|| anyhow::anyhow!("unknown backend `{id}`"))?;
            Ok(Box::new(driver))
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Block {
    Initial,
    Continue(Gate),
}

fn spawn_block(options: RunOptions, backend: Option<String>, sink: Arc<ChannelSink>, block: Block) {
    tokio::spawn(async move {
        let brain = match build_brain(backend.as_deref()) {
            Ok(b) => b,
            Err(e) => {
                sink.emit(EngineEvent::Note(format!("backend error: {e}")));
                return;
            }
        };
        // `use_runtime=true` only when a real host CLI is configured.
        // `Some("offline")` resolves to `OfflineRuntime` (empty bodies);
        // calling `try_generate` there is wasteful — N empty LLM calls
        // per run that all fall back to template anyway.
        let use_runtime =
            matches!(backend.as_deref(), Some(id) if id != "offline" && !id.is_empty());
        let runner = AgentRunner::new(brain, options).with_event_sink(sink.clone());
        let outcome = match block {
            Block::Initial => {
                if let Err(e) = runner.start() {
                    sink.emit(EngineEvent::Note(format!("start failed: {e}")));
                    return;
                }
                runner.run_initial_block(use_runtime).await
            }
            Block::Continue(gate) => runner.continue_from_gate(gate).await,
        };
        if let Err(e) = outcome {
            sink.emit(EngineEvent::Note(format!("pipeline error: {e}")));
        }
    });
}

fn spawn_probe(sink: Arc<ChannelSink>) {
    tokio::spawn(async move {
        for status in super_dev_host::probe_all().await {
            let (ready, detail) = match status.probe {
                super_dev_host::ProbeResult::Ready { version } => (true, version),
                super_dev_host::ProbeResult::NotInstalled { program } => {
                    (false, format!("`{program}` not on PATH"))
                }
                super_dev_host::ProbeResult::Unhealthy { detail } => (false, detail),
            };
            sink.emit(EngineEvent::BackendProbed {
                backend_id: status.id.to_string(),
                ready,
                detail,
            });
        }
    });
}

type Term = Terminal<CrosstermBackend<Stdout>>;

fn setup_terminal() -> Result<Term> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Term) -> Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

async fn event_loop(terminal: &mut Term, app: &mut App, opts: LaunchOptions) -> Result<()> {
    let (sink, mut engine_rx) = ChannelSink::new();
    let sink = Arc::new(sink);

    // Probe in the background so the picker labels refresh as data arrives.
    spawn_probe(sink.clone());

    let mut keys = EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_millis(120));

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        tokio::select! {
            maybe_event = engine_rx.recv() => {
                if let Some(ev) = maybe_event {
                    app.apply_engine(ev);
                }
            }
            maybe_key = keys.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_key {
                    if key.kind == KeyEventKind::Press {
                        match app.apply_key_with_mods(key.code, key.modifiers) {
                            Action::Quit => break,
                            Action::None | Action::BackendChanged => {
                                // BackendChanged only affects later spawns;
                                // no immediate side-effect on running tasks.
                            }
                            Action::Continue(gate) => {
                                let run_opts = current_run_options(app, &opts);
                                spawn_block(
                                    run_opts,
                                    app.backend.clone(),
                                    sink.clone(),
                                    Block::Continue(gate),
                                );
                            }
                            Action::StartRun(req) => {
                                let run_opts = RunOptions {
                                    project_root: opts.project_root.clone(),
                                    requirement: req,
                                    slug: opts.slug.clone(),
                                    model: opts.model.clone(),
                                    backend: app.backend.clone().unwrap_or_default(),
                                    design_system: app.config.design_system.clone().unwrap_or_default(),
                                    seed_template: app.config.seed_template.clone().unwrap_or_default(),
                                };
                                spawn_block(
                                    run_opts,
                                    app.backend.clone(),
                                    sink.clone(),
                                    Block::Initial,
                                );
                            }
                            Action::Revise(text) => {
                                // Record the revision as a note + re-run the
                                // current block. The engine has no "revise"
                                // primitive yet; for v1 we re-emit the same
                                // initial block which re-generates artifacts
                                // taking the new note into account.
                                sink.emit(EngineEvent::Note(format!("user revision: {text}")));
                                let run_opts = RunOptions {
                                    project_root: opts.project_root.clone(),
                                    requirement: app.requirement.clone(),
                                    slug: opts.slug.clone(),
                                    model: opts.model.clone(),
                                    backend: app.backend.clone().unwrap_or_default(),
                                    design_system: app.config.design_system.clone().unwrap_or_default(),
                                    seed_template: app.config.seed_template.clone().unwrap_or_default(),
                                };
                                spawn_block(
                                    run_opts,
                                    app.backend.clone(),
                                    sink.clone(),
                                    Block::Initial,
                                );
                            }
                        }
                    }
                }
            }
            _ = tick.tick() => app.tick(),
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn current_run_options(app: &App, opts: &LaunchOptions) -> RunOptions {
    RunOptions {
        project_root: opts.project_root.clone(),
        requirement: app.requirement.clone(),
        slug: opts.slug.clone(),
        model: opts.model.clone(),
        backend: app.backend.clone().unwrap_or_default(),
        design_system: app.config.design_system.clone().unwrap_or_default(),
        seed_template: app.config.seed_template.clone().unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> LaunchOptions {
        LaunchOptions {
            project_root: std::env::temp_dir(),
            slug: "demo".into(),
            model: "claude-sonnet-4-6".into(),
        }
    }

    #[test]
    fn build_brain_offline_default() {
        let brain = build_brain(None).unwrap();
        assert_eq!(brain.kind(), RuntimeKind::Anthropic);
    }

    #[test]
    fn build_brain_explicit_offline_string() {
        let brain = build_brain(Some("offline")).unwrap();
        assert_eq!(brain.kind(), RuntimeKind::Anthropic);
    }

    #[test]
    fn build_brain_accepts_every_registered_backend() {
        // Lock the TUI ↔ super-dev-host wiring. If `BACKEND_IDS` adds an
        // entry but the TUI dispatch (`build_brain` → `driver_for`)
        // doesn't reach it, the user picks the backend in the picker and
        // it silently falls back to offline — this test makes that
        // mismatch loud at test time.
        for id in super_dev_host::BACKEND_IDS {
            assert!(
                build_brain(Some(id)).is_ok(),
                "TUI cannot build brain for registered backend {id}"
            );
        }
    }

    #[test]
    fn build_brain_rejects_unknown() {
        assert!(build_brain(Some("not-a-host")).is_err());
    }

    #[test]
    fn launch_options_effective_slug_uses_explicit_first() {
        assert_eq!(opts().effective_slug(), "demo");
    }

    #[test]
    fn launch_options_effective_slug_falls_back_to_dir_name() {
        let mut o = opts();
        o.slug.clear();
        o.project_root = PathBuf::from("/tmp/my-project");
        assert_eq!(o.effective_slug(), "my-project");
    }
}
