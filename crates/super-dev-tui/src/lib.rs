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
use super_dev_runtime::http::{AnthropicHttpRuntime, OpenAiHttpRuntime};
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

    // Install a panic hook BEFORE entering raw mode. If anything in the
    // event loop panics, the default hook would print the backtrace but
    // LEAVE THE TERMINAL IN RAW MODE — the user's shell becomes unusable
    // (no echo, no line buffering) until they run `reset`. Our hook
    // restores the terminal first, then forwards to the original hook so
    // the panic message + backtrace still print normally.
    install_panic_hook();
    let mut terminal = setup_terminal().context("failed to set up terminal")?;
    let result = event_loop(&mut terminal, &mut app, opts).await;
    // Graceful cleanup: kill any preview dev server the user started via
    // /preview, so quitting Super Dev never leaves an orphaned process.
    if let Ok(mut g) = app.preview_server.lock() {
        if let Some(mut child) = g.take() {
            let _ = child.start_kill();
        }
    }
    restore_terminal(&mut terminal).ok();
    result
}

/// Replace the global panic hook with one that restores the terminal
/// (disable raw mode, leave the alternate screen, show the cursor) before
/// the panic unwinds. Idempotent: the prior hook is chained so repeated
/// installs don't stack indefinitely.
fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Best-effort restoration — ignore errors, we're panicking anyway.
        let _ = disable_raw_mode();
        let _ = std::io::stdout().execute(LeaveAlternateScreen);
        let _ = std::io::stdout().execute(crossterm::cursor::Show);
        // Print a visible marker so the user knows it was a panic, not a
        // clean exit, then defer to the previous hook for the backtrace.
        eprintln!("\n\nsuper-dev: panic — terminal restored.\n");
        prev(info);
    }));
}

/// Resolved decision of which "brain" runs the pipeline, captured up-front so
/// the spawn path has everything it needs without re-reading config. Produced
/// by [`App::brain_spec`]; consumed by [`build_brain`] / [`spawn_block`].
///
/// Order of precedence (matches [`crate::config::UserConfig::effective_provider`]):
/// project-level provider > global provider > host CLI backend > offline.
#[derive(Debug, Clone)]
pub enum BrainSpec {
    /// Drive a logged-in host CLI subprocess (Claude Code / Codex).
    HostCli(String),
    /// Call a custom OpenAI-compatible or Anthropic HTTP endpoint directly.
    CustomApi(crate::config::ProviderConfig),
    /// Deterministic templates, no AI.
    Offline,
}

impl BrainSpec {
    /// Human-facing label for status / error messages.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::HostCli(id) => id.clone(),
            Self::CustomApi(p) => format!("{} ({})", p.model, p.kind),
            Self::Offline => "offline".to_string(),
        }
    }

    /// `true` when this brain is a real AI (host CLI or custom API), i.e. the
    /// pipeline should use the runtime path rather than offline templates.
    #[must_use]
    pub fn is_runtime(&self) -> bool {
        matches!(self, Self::HostCli(_) | Self::CustomApi(_))
    }
}

fn build_brain(spec: &BrainSpec) -> Result<Box<dyn Runtime>> {
    match spec {
        BrainSpec::Offline => Ok(Box::new(OfflineRuntime::new(RuntimeKind::Anthropic))),
        BrainSpec::HostCli(id) => {
            let driver = driver_for(id).ok_or_else(|| anyhow::anyhow!("unknown backend `{id}`"))?;
            Ok(Box::new(driver))
        }
        BrainSpec::CustomApi(p) => {
            if !p.kind_is_known() {
                anyhow::bail!(
                    "provider kind `{}` 未知 —— 只支持 `openai` 或 `anthropic`。用 /provider 查看。",
                    p.kind
                );
            }
            match p.kind.as_str() {
                "anthropic" => Ok(Box::new(AnthropicHttpRuntime::new(
                    &p.base_url,
                    &p.api_key,
                    &p.model,
                ))),
                _ => Ok(Box::new(OpenAiHttpRuntime::new(
                    &p.base_url,
                    &p.api_key,
                    &p.model,
                ))),
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Block {
    Initial,
    Continue(Gate),
}

/// Split a worker-recorded run command like `cd web && npm run dev` into
/// (`working_dir`, `program`, `args`). Falls back to running the whole string via
/// `sh -c` when it does not match the `cd X && ...` shape.
fn parse_run_command(
    command: &str,
    project_root: &std::path::Path,
) -> (std::path::PathBuf, String, Vec<String>) {
    // Strip a leading `cd <dir> &&` and resolve it relative to the workspace.
    if let Some(after_cd) = command.trim().strip_prefix("cd ") {
        if let Some((dir, rest)) = after_cd.split_once("&&") {
            let dir = dir.trim().trim_matches(|c| c == '\'' || c == '"');
            let resolved = if std::path::Path::new(dir).is_absolute() {
                std::path::PathBuf::from(dir)
            } else {
                project_root.join(dir)
            };
            let rest = rest.trim();
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if let Some((prog, args)) = parts.split_first() {
                let args: Vec<String> = args.iter().map(std::string::ToString::to_string).collect();
                return (resolved, prog.to_string(), args);
            }
        }
    }
    // Fallback: shell out with `sh -c "<command>"` in the workspace root.
    (
        project_root.to_path_buf(),
        "sh".to_string(),
        vec!["-c".to_string(), command.to_string()],
    )
}

/// Cross-platform best-effort browser open (sync variant for the event loop).
fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()?;
    }
    Ok(())
}

fn spawn_block(options: RunOptions, spec: BrainSpec, sink: Arc<ChannelSink>, block: Block) {
    tokio::spawn(async move {
        let label = spec.label();
        let brain = match build_brain(&spec) {
            Ok(b) => b,
            Err(e) => {
                sink.emit(EngineEvent::Note(format!(
                    "⚠ 无法初始化 worker `{label}`: {e}\n  \
                     请检查: 自定义 provider 的 kind/base_url/key 是否正确? \
                     或用 /backend 选一个已登录的 CLI; /offline 切离线模板。"
                )));
                return;
            }
        };
        let use_runtime = spec.is_runtime();
        let runner = AgentRunner::new(brain, options).with_event_sink(sink.clone());
        let outcome = match block {
            Block::Initial => {
                if let Err(e) = runner.start() {
                    sink.emit(EngineEvent::Note(format!(
                        "⚠ 流水线启动失败: {e}\n  \
                         请检查: 工作目录是否可写? 磁盘空间是否充足?"
                    )));
                    return;
                }
                runner.run_initial_block(use_runtime).await
            }
            Block::Continue(gate) => runner.continue_from_gate(gate).await,
        };
        if let Err(e) = outcome {
            let err_str = e.to_string();
            let hint = if err_str.contains("timed out") {
                format!(
                    "Worker `{label}` 调用超时(5 分钟)。排查顺序:\n  \
                     1) 先在终端跑一次该 CLI / 直接 curl 该 API 确认能响应;\n  \
                     2) 若是大需求,拆小后重试;\n  \
                     3) 用 /doctor 检查 worker 健康,或 /offline 临时切到离线模板继续。"
                )
            } else if err_str.contains("not found on PATH") {
                format!(
                    "Worker CLI `{label}` 不在 PATH 里。\n  \
                     用 /doctor 看哪些 worker 可用,或安装该 CLI 后重试;\n  \
                     也可 /provider 切到自定义 API,或 /offline 切离线模板。"
                )
            } else if err_str.contains("returned 4") || err_str.contains("returned 5") {
                "自定义 API 返回了 HTTP 错误。检查:\n  \
                 1) api_key 是否有效(未过期、有额度)?\n  \
                 2) base_url 是否指向正确的 endpoint?\n  \
                 3) model 名称该 provider 是否支持?\n  \
                 用 /provider 查看当前配置,/offline 临时切离线继续。"
                    .to_string()
            } else if err_str.contains("exited with code") {
                "Worker 进程异常退出。查看上方 worker 输出定位原因;\n  \
                 常见是未登录或额度用尽 —— 先在终端单独跑一次该 CLI 验证。"
                    .to_string()
            } else {
                "流水线遇到错误。已回退到 offline 模板继续(如适用)。用 /status 查看当前状态。"
                    .to_string()
            };
            sink.emit(EngineEvent::Note(format!("⚠ 流水线错误: {e}\n  {hint}")));
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
                                    app.brain_spec(),
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
                                    app.brain_spec(),
                                    sink.clone(),
                                    Block::Initial,
                                );
                            }
                            Action::Revise(text) => {
                                // Re-run the block that PRODUCED the current
                                // gate, with the revision feedback folded into
                                // the requirement so the worker actually
                                // incorporates it. Branch on the active gate:
                                //   - docs_confirm  → re-run Initial (regen docs)
                                //   - preview_confirm→ re-run Continue(DocsConfirm)
                                //     (regen spec → frontend), NOT the docs.
                                // Re-running Initial unconditionally was a bug:
                                // a UI revision at preview_confirm would have
                                // thrown away the approved docs and regenerated
                                // them instead of redoing the frontend.
                                sink.emit(EngineEvent::Note(format!("user revision: {text}")));
                                let revised_requirement = format!(
                                    "{}\n\n## Revision request\n{text}",
                                    app.requirement
                                );
                                let run_opts = RunOptions {
                                    project_root: opts.project_root.clone(),
                                    requirement: revised_requirement,
                                    slug: opts.slug.clone(),
                                    model: opts.model.clone(),
                                    backend: app.backend.clone().unwrap_or_default(),
                                    design_system: app.config.design_system.clone().unwrap_or_default(),
                                    seed_template: app.config.seed_template.clone().unwrap_or_default(),
                                };
                                let block = match app.active_gate {
                                    Some(Gate::PreviewConfirm) => {
                                        Block::Continue(Gate::DocsConfirm)
                                    }
                                    // docs_confirm or unknown → regenerate docs
                                    _ => Block::Initial,
                                };
                                spawn_block(
                                    run_opts,
                                    app.brain_spec(),
                                    sink.clone(),
                                    block,
                                );
                            }
                            Action::ProbeProvider {
                                name,
                                kind,
                                base_url,
                                api_key,
                                model,
                            } => {
                                // Spawn a short probe: build the matching HTTP
                                // runtime, send an 8-token ping, emit the result.
                                let sink2 = sink.clone();
                                tokio::spawn(async move {
                                    sink2.emit(EngineEvent::Note(format!(
                                        "正在验证 {name} ({model}) 连通性…"
                                    )));
                                    let brain: Box<dyn Runtime> = if kind == "anthropic" {
                                        Box::new(AnthropicHttpRuntime::new(
                                            &base_url, &api_key, &model,
                                        ))
                                    } else {
                                        Box::new(OpenAiHttpRuntime::new(
                                            &base_url, &api_key, &model,
                                        ))
                                    };
                                    let ping = super_dev_runtime::CompletionRequest {
                                        model: model.clone(),
                                        messages: vec![super_dev_runtime::Message {
                                            role: "user".into(),
                                            content: "Reply with the single word: ok".into(),
                                        }],
                                        max_tokens: Some(8),
                                        temperature: Some(0.0),
                                        system: None,
                                    };
                                    let result = tokio::time::timeout(
                                        std::time::Duration::from_secs(30),
                                        brain.complete(ping),
                                    )
                                    .await;
                                    let (ok, detail) = match result {
                                        Ok(Ok(resp)) => (
                                            true,
                                            format!(
                                                "返回 {} tokens,id={}",
                                                resp.usage.input_tokens
                                                    + resp.usage.output_tokens,
                                                resp.id
                                            ),
                                        ),
                                        Ok(Err(e)) => (false, e.to_string()),
                                        Err(_) => (
                                            false,
                                            "探测超时(30s)—— 服务无响应或网络不通".to_string(),
                                        ),
                                    };
                                    sink2.emit(EngineEvent::ProviderVerified {
                                        name,
                                        model,
                                        ok,
                                        detail,
                                    });
                                });
                            }
                            Action::StartPreview { url, command } => {
                                // Parse a "cd <dir> && <cmd>" form the worker
                                // recorded; spawn the dev server detached with
                                // kill_on_drop, open the browser, stash the child.
                                let (dir, prog, args) = parse_run_command(&command, &opts.project_root);
                                let mut cmd = tokio::process::Command::new(prog);
                                cmd.args(&args)
                                    .current_dir(&dir)
                                    .stdin(std::process::Stdio::null())
                                    .stdout(std::process::Stdio::null())
                                    .stderr(std::process::Stdio::null())
                                    .kill_on_drop(true);
                                match cmd.spawn() {
                                    Ok(child) => {
                                        if let Ok(mut g) = app.preview_server.lock() {
                                            *g = Some(child);
                                        }
                                        let _ = open_url(&url);
                                        sink.emit(EngineEvent::Note(format!(
                                            "✓ dev server 已启动:{url}\n  /stop-preview 停止"
                                        )));
                                    }
                                    Err(e) => {
                                        sink.emit(EngineEvent::Note(format!(
                                            "⚠ 无法启动 dev server ({command}): {e}\n  \
                                             请手动运行该命令,然后刷新 {url}"
                                        )));
                                    }
                                }
                            }
                            Action::RunDeploy { command } => {
                                // Deploy runs in a background task: `sh -c` the
                                // recorded command in the workspace, capture
                                // output, surface success/failure + the live URL.
                                let sink2 = sink.clone();
                                let root = opts.project_root.clone();
                                tokio::spawn(async move {
                                    sink2.emit(EngineEvent::Note(format!(
                                        "🚀 部署中,执行:`{command}` …"
                                    )));
                                    let out = tokio::process::Command::new("sh")
                                        .arg("-c")
                                        .arg(&command)
                                        .current_dir(&root)
                                        .output()
                                        .await;
                                    match out {
                                        Ok(o) if o.status.success() => {
                                            let stdout = String::from_utf8_lossy(&o.stdout);
                                            // Many deploy CLIs print the live URL.
                                            let url = stdout
                                                .lines()
                                                .find(|l| l.contains("https://"))
                                                .map(str::to_string);
                                            sink2.emit(EngineEvent::Note(format!(
                                                "✓ 部署完成。{}",
                                                url.clone().unwrap_or_else(|| "查看上方输出确认地址".into())
                                            )));
                                        }
                                        Ok(o) => {
                                            let stderr = String::from_utf8_lossy(&o.stderr);
                                            sink2.emit(EngineEvent::Note(format!(
                                                "⚠ 部署失败(退出码 {}): {}",
                                                o.status.code().unwrap_or(-1),
                                                stderr.chars().take(500).collect::<String>()
                                            )));
                                        }
                                        Err(e) => {
                                            sink2.emit(EngineEvent::Note(format!(
                                                "⚠ 无法执行部署命令 ({command}): {e}"
                                            )));
                                        }
                                    }
                                });
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
    fn parse_run_command_cd_form() {
        let root = std::path::PathBuf::from("/proj");
        let (dir, prog, args) = parse_run_command("cd web && npm run dev", &root);
        assert_eq!(dir, std::path::PathBuf::from("/proj/web"));
        assert_eq!(prog, "npm");
        assert_eq!(args, vec!["run".to_string(), "dev".into()]);
    }

    #[test]
    fn parse_run_command_absolute_dir() {
        let root = std::path::PathBuf::from("/proj");
        let (dir, prog, args) = parse_run_command("cd /abs/app && pnpm dev", &root);
        assert_eq!(dir, std::path::PathBuf::from("/abs/app"));
        assert_eq!(prog, "pnpm");
        assert_eq!(args, vec!["dev".to_string()]);
    }

    #[test]
    fn parse_run_command_fallback_shells() {
        let root = std::path::PathBuf::from("/proj");
        let (dir, prog, args) = parse_run_command("npm run dev", &root);
        // No `cd &&` prefix → fallback to sh -c in the workspace root.
        assert_eq!(dir, root);
        assert_eq!(prog, "sh");
        assert_eq!(args, vec!["-c".to_string(), "npm run dev".into()]);
    }

    #[test]
    fn parse_run_command_npx_vercel_deploy() {
        // The canonical /deploy command. No `cd &&` → sh -c fallback,
        // preserving the full command (flags included).
        let root = std::path::PathBuf::from("/proj");
        let (dir, prog, args) = parse_run_command("npx vercel --prod", &root);
        assert_eq!(dir, root);
        assert_eq!(prog, "sh");
        assert_eq!(args, vec!["-c".to_string(), "npx vercel --prod".into()]);
    }

    #[test]
    fn parse_run_command_cd_with_npm_exec_flags() {
        // `cd web && npm exec -- vite` — flags after the program must survive.
        let root = std::path::PathBuf::from("/proj");
        let (dir, prog, args) = parse_run_command("cd web && npm exec -- vite", &root);
        assert_eq!(dir, std::path::PathBuf::from("/proj/web"));
        assert_eq!(prog, "npm");
        assert_eq!(args, vec!["exec".to_string(), "--".into(), "vite".into()]);
    }

    #[test]
    fn parse_run_command_trims_whitespace() {
        let root = std::path::PathBuf::from("/proj");
        let (dir, _, _) = parse_run_command("   cd app   &&   npm run dev   ", &root);
        assert_eq!(dir, std::path::PathBuf::from("/proj/app"));
    }

    #[test]
    fn parse_run_command_single_quoted_dir() {
        // Quoted directory names should be unquoted.
        let root = std::path::PathBuf::from("/proj");
        let (dir, prog, _) = parse_run_command("cd 'my app' && npm run dev", &root);
        assert_eq!(dir, std::path::PathBuf::from("/proj/my app"));
        assert_eq!(prog, "npm");
    }

    #[test]
    fn build_brain_offline_default() {
        let brain = build_brain(&BrainSpec::Offline).unwrap();
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
                build_brain(&BrainSpec::HostCli((*id).to_string())).is_ok(),
                "TUI cannot build brain for registered backend {id}"
            );
        }
    }

    #[test]
    fn build_brain_rejects_unknown_host_cli() {
        assert!(build_brain(&BrainSpec::HostCli("not-a-host".into())).is_err());
    }

    #[test]
    fn build_brain_builds_openai_custom_api() {
        let spec = BrainSpec::CustomApi(crate::config::ProviderConfig {
            kind: "openai".into(),
            base_url: "https://api.deepseek.com/v1".into(),
            api_key: "sk-test".into(),
            model: "deepseek-chat".into(),
        });
        let brain = build_brain(&spec).unwrap();
        assert_eq!(brain.kind(), RuntimeKind::Openai);
    }

    #[test]
    fn build_brain_builds_anthropic_custom_api() {
        let spec = BrainSpec::CustomApi(crate::config::ProviderConfig {
            kind: "anthropic".into(),
            base_url: "https://api.anthropic.com".into(),
            api_key: "sk-ant".into(),
            model: "claude-sonnet-4".into(),
        });
        let brain = build_brain(&spec).unwrap();
        assert_eq!(brain.kind(), RuntimeKind::Anthropic);
    }

    #[test]
    fn build_brain_rejects_unknown_provider_kind() {
        let spec = BrainSpec::CustomApi(crate::config::ProviderConfig {
            kind: "quantum".into(),
            base_url: "https://x".into(),
            api_key: "k".into(),
            model: "m".into(),
        });
        assert!(build_brain(&spec).is_err());
    }

    #[test]
    fn brain_spec_label_and_is_runtime() {
        assert_eq!(BrainSpec::Offline.label(), "offline");
        assert!(!BrainSpec::Offline.is_runtime());
        assert!(BrainSpec::HostCli("codex".into()).is_runtime());
        let api = BrainSpec::CustomApi(crate::config::ProviderConfig {
            kind: "openai".into(),
            model: "deepseek-chat".into(),
            ..Default::default()
        });
        assert!(api.is_runtime());
        assert_eq!(api.label(), "deepseek-chat (openai)");
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
