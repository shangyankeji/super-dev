//! TUI application model.
//!
//! 4.4+ design (Claude Code-style):
//!
//! - **Picker** — shown only on first launch (no `~/.super-dev/config.toml`).
//!   Up/Down through detected backends, Enter to confirm, choice saved.
//! - **Chat** — the main screen. Persistent input box at the bottom,
//!   scrolling message history above (user / super-dev / host outputs /
//!   gate prompts), status bar on top.
//!
//! Slash commands inside Chat (`/claude` `/codex` `/offline` `/init`
//! `/continue` `/revise` `/diff` `/spec` `/verify` `/doctor` `/help`
//! `/quit` `/clear` `/history` `/commands`) plus normal text — which is
//! routed to either "submit as requirement" (if no run is active) or
//! "treat as revision" (if a gate is open).

use std::collections::VecDeque;

use crossterm::event::KeyCode;
use super_dev_agent::{EngineEvent, Gate};
use super_dev_spec::{Phase, PHASE_CHAIN};

use crate::config::UserConfig;

/// Max lines kept in the chat history (older lines roll off).
const HISTORY_CAP: usize = 1000;
/// Max chars in the input box.
const INPUT_CAP: usize = 8192;

/// Which screen the TUI is showing.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AppMode {
    /// First-launch backend chooser. Transitions to `Chat` on Enter.
    Picker,
    /// The conversational main screen.
    Chat,
}

/// What the event loop should do after a key press.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Action {
    /// Nothing — keep looping.
    None,
    /// Tear down and exit.
    Quit,
    /// Approve the named gate and drive the next block.
    Continue(Gate),
    /// User submitted a fresh requirement — start the initial pipeline block.
    StartRun(String),
    /// User submitted text while a gate was active — record as a revision and
    /// re-run the most recent block.
    Revise(String),
    /// Backend was switched (saved to config); the engine task should be
    /// restarted on next `StartRun`.
    BackendChanged,
}

/// Status of one pipeline phase.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PhaseStatus {
    /// Not reached yet.
    Pending,
    /// Currently executing.
    Running,
    /// Finished.
    Done,
}

/// One row in the pipeline status panel (kept compact for the status bar).
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PhaseRow {
    /// The phase this row tracks.
    pub phase: Phase,
    /// Its current status.
    pub status: PhaseStatus,
}

/// Availability of one host backend.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BackendInfo {
    /// Stable backend id (`claude-code` / `codex`).
    pub id: String,
    /// `true` when the host CLI is installed and reachable.
    pub ready: bool,
    /// Version string or failure reason.
    pub detail: String,
}

/// One item in the first-launch backend picker.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PickerItem {
    /// `Some("claude-code")` etc. `None` represents the `offline` choice.
    pub backend_id: Option<String>,
    /// Display label.
    pub label: String,
    /// Probe state — `Ready` for offline always; for hosts only when CLI is on PATH.
    pub ready: bool,
    /// Detail line (version / "not on PATH" / "deterministic templates").
    pub detail: String,
}

/// Source of a chat message — used to colour the role label.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ChatRole {
    /// The end user typing into the input box.
    You,
    /// Super Dev's own meta-messages (pipeline progress, gate prompts).
    SuperDev,
    /// One line of output captured from a host CLI worker.
    Host,
    /// The pipeline reached a gate and is awaiting approval.
    Gate,
    /// A system event (config saved, error, hint).
    System,
}

/// One row in the chat history.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChatMessage {
    /// Who "said" this.
    pub role: ChatRole,
    /// The text body (already cleaned of ANSI etc.).
    pub body: String,
}

/// A scrollable full-screen overlay opened by `/spec` / `/verify` /
/// `/doctor` / `/diff`. Closed with Esc.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Overlay {
    /// Window title shown at the top of the overlay border.
    pub title: String,
    /// Pre-split lines for easy clipping (each may be longer than the
    /// visible width; the renderer wraps).
    pub lines: Vec<String>,
    /// Top-of-window cursor (0 = first line).
    pub scroll: usize,
}

impl Overlay {
    /// Build an overlay from a single body string.
    #[must_use]
    pub fn from_body(title: impl Into<String>, body: &str) -> Self {
        let lines: Vec<String> = body.lines().map(String::from).collect();
        Self {
            title: title.into(),
            lines,
            scroll: 0,
        }
    }

    /// Scroll down by `n` lines, clamped at end.
    pub fn scroll_down(&mut self, n: usize) {
        let max = self.lines.len().saturating_sub(1);
        self.scroll = (self.scroll + n).min(max);
    }

    /// Scroll up by `n` lines, clamped at 0.
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_sub(n);
    }
}

/// The whole TUI state.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct App {
    /// Active screen.
    pub mode: AppMode,

    /// Persisted user config (backend choice etc.).
    pub config: UserConfig,
    /// Path the picker / slash commands write back to.
    pub config_path: std::path::PathBuf,

    /// Picker state (active during `AppMode::Picker`).
    pub picker_items: Vec<PickerItem>,
    /// Cursor in `picker_items`.
    pub picker_selected: usize,

    /// Chat input buffer (UTF-8 String — mutate via cursor helpers,
    /// never via raw push/pop, so multi-byte chars stay intact).
    pub input: String,
    /// Caret position within `input`, measured in **characters** (not bytes).
    /// `0` = before first char; `chars().count()` = after last char.
    pub input_cursor: usize,
    /// Past submitted texts. ↑↓ in an empty input box recalls them.
    pub input_history: VecDeque<String>,
    /// Recall cursor into `input_history`; `None` = editing a fresh draft.
    pub input_history_idx: Option<usize>,
    /// When `input` starts with `/` and matches command verbs, this is
    /// the highlight in the slash-command palette popover.
    pub palette_selected: usize,

    /// Bounded scrolling chat history (older lines roll off).
    pub history: VecDeque<ChatMessage>,

    /// Currently active backend id (matches `config.backend`).
    /// `None` means offline / no host CLI.
    pub backend: Option<String>,
    /// Display label for the worker — `claude-code` / `codex` / `offline`.
    pub backend_label: String,

    /// Workspace slug (filled in by the caller).
    pub slug: String,
    /// The active requirement once the pipeline starts.
    pub requirement: String,

    /// Phase progress, in `PHASE_CHAIN` order.
    pub phases: Vec<PhaseRow>,
    /// The gate the pipeline is currently paused at, if any.
    pub active_gate: Option<Gate>,
    /// `true` once a delivery proof-pack has landed.
    pub finished: bool,
    /// `true` once the user has kicked off a pipeline run in this session.
    pub run_started: bool,

    /// Detected host backends (asynchronously populated).
    pub backends: Vec<BackendInfo>,
    /// `true` while the help overlay is open.
    pub show_help: bool,
    /// A scrollable overlay (from `/spec` / `/verify` / `/doctor` /
    /// `/diff`). When `Some`, key input is routed to the overlay
    /// (scroll, close); when `None`, normal chat input.
    pub overlay: Option<Overlay>,
    /// Workspace root — surfaced in the status bar as a breadcrumb.
    pub project_root: std::path::PathBuf,
    /// When a pipeline is running and the user presses `q` / Esc, we
    /// stash a "press again to confirm" flag instead of quitting
    /// immediately. Cleared on any other keypress.
    pub pending_quit_confirm: bool,

    /// One-line status shown in the top bar.
    pub status: String,
    /// Spinner animation tick.
    pub tick: u8,
    /// `true` when the user asked to quit.
    pub should_quit: bool,
}

impl App {
    /// Build a fresh app. Reads existing config from disk; if no
    /// backend is set, opens on the picker.
    ///
    /// `project_root` is shown in the status bar and used by overlays
    /// like `/diff` that need to read workspace artifacts.
    #[must_use]
    pub fn new(
        slug: impl Into<String>,
        config: UserConfig,
        config_path: std::path::PathBuf,
        project_root: std::path::PathBuf,
    ) -> Self {
        let phases = PHASE_CHAIN
            .iter()
            .map(|&phase| PhaseRow {
                phase,
                status: PhaseStatus::Pending,
            })
            .collect();
        let backend = config.backend.clone().filter(|b| b != "offline");
        let backend_label = backend.clone().unwrap_or_else(|| "offline".to_string());
        let mode = if config.has_backend() {
            AppMode::Chat
        } else {
            AppMode::Picker
        };
        let mut app = Self {
            mode,
            config,
            config_path,
            picker_items: default_picker_items(),
            picker_selected: 0,
            input: String::new(),
            input_cursor: 0,
            input_history: VecDeque::new(),
            input_history_idx: None,
            palette_selected: 0,
            history: VecDeque::new(),
            backend,
            backend_label,
            slug: slug.into(),
            requirement: String::new(),
            phases,
            active_gate: None,
            finished: false,
            run_started: false,
            backends: Vec::new(),
            show_help: false,
            overlay: None,
            project_root,
            pending_quit_confirm: false,
            status: String::new(),
            tick: 0,
            should_quit: false,
        };
        app.load_history();
        if app.mode == AppMode::Chat {
            app.push_greeting();
            app.maybe_push_resume_hint();
        }
        app.refresh_status();
        app
    }

    fn history_path(&self) -> std::path::PathBuf {
        self.project_root
            .join(".super-dev")
            .join("input-history.txt")
    }

    fn load_history(&mut self) {
        if let Ok(body) = std::fs::read_to_string(self.history_path()) {
            for line in body.lines().rev().take(50) {
                if !line.is_empty() {
                    self.input_history.push_front(line.to_string());
                }
            }
        }
    }

    fn persist_history(&self) {
        let path = self.history_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let lines: Vec<&str> = self.input_history.iter().map(String::as_str).collect();
        let _ = std::fs::write(path, lines.join("\n"));
    }

    /// If a `.super-dev/workflow-state.json` exists in the workspace
    /// (meaning a prior session left the pipeline mid-flight), surface
    /// it as a system message so the user can resume with `/continue`
    /// instead of staring at a fresh prompt and wondering "did my
    /// previous work disappear?".
    fn maybe_push_resume_hint(&mut self) {
        let Some(state) = super_dev_agent::read_workflow_state(&self.project_root) else {
            return;
        };
        let gate = state.active_gate.clone();
        let req = state.requirement.clone();
        if gate.is_empty() && state.phase == "delivery" {
            // Last session completed; mention the proof-pack but don't
            // nudge a resume.
            self.push(
                ChatRole::System,
                format!(
                    "📂 workspace 上次跑完了流水线(需求:\"{req}\")。\n  /diff 看上次产出 · 或直接输入新需求开新一轮。",
                ),
            );
            return;
        }
        if !gate.is_empty() {
            self.push(
                ChatRole::System,
                format!(
                    "📂 workspace 上次会话停在 gate `{gate}`(需求:\"{req}\")。\n  /continue 继续推进 · /diff 看产出 · 或开新需求(会重置 workflow state)。",
                ),
            );
        } else if !req.is_empty() {
            self.push(
                ChatRole::System,
                format!(
                    "📂 检测到未完成的会话(phase={}, 需求:\"{req}\")。\n  \
                     输入 /continue 推进上次 · 输入新需求会重新开始 · /status 查看详情",
                    state.phase,
                ),
            );
        }
    }

    fn push(&mut self, role: ChatRole, body: impl Into<String>) {
        self.history.push_back(ChatMessage {
            role,
            body: body.into(),
        });
        while self.history.len() > HISTORY_CAP {
            self.history.pop_front();
        }
    }

    fn push_greeting(&mut self) {
        let ds_label = self.config.design_system.as_deref().unwrap_or("(未选)");
        let tpl_label = self.config.seed_template.as_deref().unwrap_or("(自动)");
        self.push(
            ChatRole::SuperDev,
            format!(
                "Super Dev · AI 编码的项目经理\n\
                 工人: {} · 设计系统: {ds_label} · 模板: {tpl_label}\n\
                 直接告诉我做什么,我会驱动工人按 9 阶段流水线交付。",
                self.backend_label,
            ),
        );
        let ds_list = self.list_design_systems();
        let ds_hint = if ds_list.is_empty() {
            String::new()
        } else {
            format!("\n  · /design <名字> — 选设计风格: {}", ds_list.join(" / "))
        };
        let tpl_list = self.list_seed_templates();
        let tpl_hint = if tpl_list.is_empty() {
            String::new()
        } else {
            format!(
                "\n  · /template <名字> — 选页面模板: {}",
                tpl_list.join(" / ")
            )
        };
        self.push(
            ChatRole::System,
            format!(
                "试试这种问法:\n  \
                 · 做一个登录系统,支持邮箱+OAuth+MFA\n  \
                 · 帮我做一个 SaaS 数据分析仪表盘\n  \
                 · 给现有项目加二步验证(TOTP + 备用码)\n\n\
                 开始前可选:{ds_hint}{tpl_hint}\n  \
                 · /config — 看当前配置 · /help — 看全部命令"
            ),
        );
    }

    fn refresh_status(&mut self) {
        // Compact 9-dot phase progress, e.g. "●●◐○○○○○○".
        // ● done · ◐ running · ○ pending.
        let dots: String = self
            .phases
            .iter()
            .map(|r| match r.status {
                PhaseStatus::Done => '●',
                PhaseStatus::Running => '◐',
                PhaseStatus::Pending => '○',
            })
            .collect();
        let running = self
            .phases
            .iter()
            .find(|r| r.status == PhaseStatus::Running)
            .map(|r| format!(" {} {}", self.spinner(), r.phase.id()))
            .unwrap_or_default();
        let gate_label = self
            .active_gate
            .map(|g| format!(" · ⏸ {}", g.id_str()))
            .unwrap_or_default();
        let done_label = if self.finished {
            " · ✓ delivered".to_string()
        } else {
            String::new()
        };
        let ds_short = self
            .config
            .design_system
            .as_deref()
            .map(|s| format!(" · 🎨 {s}"))
            .unwrap_or_default();
        self.status = format!(
            "● {} · {}{}{}{}{}",
            self.backend_label, dots, running, gate_label, done_label, ds_short
        );
    }

    /// `true` while the user has started a run that hasn't reached
    /// delivery yet. Used by the Esc-to-quit confirmation.
    #[must_use]
    pub fn is_pipeline_active(&self) -> bool {
        self.run_started && !self.finished
    }

    // ---- input editing helpers (char-cursor over a UTF-8 String) ---------

    /// Number of characters in the input buffer.
    #[must_use]
    pub fn input_len(&self) -> usize {
        self.input.chars().count()
    }

    /// Convert a char-position cursor to a byte index into `input`.
    /// Used to splice multi-byte UTF-8 strings safely.
    fn byte_index(&self, char_pos: usize) -> usize {
        self.input
            .char_indices()
            .nth(char_pos)
            .map_or(self.input.len(), |(i, _)| i)
    }

    /// Insert one character at the cursor and advance.
    pub fn insert_at_cursor(&mut self, c: char) {
        if self.input_len() >= INPUT_CAP {
            return;
        }
        let pos = self.byte_index(self.input_cursor);
        self.input.insert(pos, c);
        self.input_cursor += 1;
    }

    /// Delete the character BEFORE the cursor (Backspace).
    pub fn backspace(&mut self) {
        if self.input_cursor == 0 {
            return;
        }
        let end = self.byte_index(self.input_cursor);
        let start = self.byte_index(self.input_cursor - 1);
        self.input.replace_range(start..end, "");
        self.input_cursor -= 1;
    }

    /// Delete the character AT the cursor (forward Delete).
    pub fn forward_delete(&mut self) {
        if self.input_cursor >= self.input_len() {
            return;
        }
        let start = self.byte_index(self.input_cursor);
        let end = self.byte_index(self.input_cursor + 1);
        self.input.replace_range(start..end, "");
    }

    /// Move cursor by `delta` characters, clamped to `[0, len]`.
    pub fn move_cursor(&mut self, delta: isize) {
        let len = self.input_len();
        if delta < 0 {
            self.input_cursor = self.input_cursor.saturating_sub(delta.unsigned_abs());
        } else {
            #[allow(clippy::cast_sign_loss)]
            let fwd = delta as usize;
            self.input_cursor = self.input_cursor.saturating_add(fwd).min(len);
        }
    }

    /// Clear the input buffer + reset cursor + history-recall index.
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.input_cursor = 0;
        self.input_history_idx = None;
    }

    /// Push a submitted line onto the input-history ring. De-dups
    /// consecutive duplicates (typing the same thing twice doesn't
    /// double-pollute the ↑↓ recall). Also persists to disk so history
    /// survives across TUI sessions.
    pub fn remember_submission(&mut self, text: &str) {
        const HISTORY_CAP_PROMPTS: usize = 100;
        if text.trim().is_empty() {
            return;
        }
        if self.input_history.back().map(String::as_str) == Some(text) {
            return;
        }
        self.input_history.push_back(text.to_string());
        while self.input_history.len() > HISTORY_CAP_PROMPTS {
            self.input_history.pop_front();
        }
        self.persist_history();
    }

    /// Step back through input history. Loads the previous prompt into
    /// the input box. Idempotent at the oldest entry.
    pub fn input_history_back(&mut self) {
        if self.input_history.is_empty() {
            return;
        }
        let new_idx = match self.input_history_idx {
            None => self.input_history.len() - 1,
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.input_history_idx = Some(new_idx);
        if let Some(s) = self.input_history.get(new_idx) {
            self.input = s.clone();
            self.input_cursor = self.input_len();
        }
    }

    /// Step forward through input history. At the most-recent entry,
    /// stepping forward once more clears the input (returns to fresh draft).
    pub fn input_history_forward(&mut self) {
        let Some(idx) = self.input_history_idx else {
            return;
        };
        if idx + 1 < self.input_history.len() {
            self.input_history_idx = Some(idx + 1);
            if let Some(s) = self.input_history.get(idx + 1) {
                self.input = s.clone();
                self.input_cursor = self.input_len();
            }
        } else {
            self.input_history_idx = None;
            self.input.clear();
            self.input_cursor = 0;
        }
    }

    // ---- slash command palette ------------------------------------------

    /// Verbs the palette popover suggests, in display order. (verb, hint)
    pub const SLASH_VERBS: &'static [(&'static str, &'static str)] = &[
        ("claude", "switch worker to Claude Code CLI"),
        ("codex", "switch worker to Codex CLI"),
        ("gemini", "switch worker to Gemini CLI"),
        ("droid", "switch worker to Droid CLI"),
        ("opencode", "switch worker to OpenCode CLI"),
        ("qwen", "switch worker to Qwen Code CLI"),
        ("copilot", "switch worker to Copilot CLI"),
        ("trae", "switch worker to Trae CLI"),
        ("codebuddy", "switch worker to CodeBuddy CLI"),
        ("qoder", "switch worker to Qoder CLI"),
        ("kimi", "switch worker to Kimi Code CLI"),
        ("offline", "switch worker to offline templates"),
        ("model", "set the model id (e.g. /model claude-opus-4-7)"),
        (
            "design",
            "pick a design system (e.g. /design modern-minimal)",
        ),
        (
            "template",
            "pick a seed template (e.g. /template dashboard)",
        ),
        ("run", "start a new run (/run [slug] <requirement>)"),
        ("redo", "re-run the current requirement from scratch"),
        ("config", "show all current configuration"),
        ("init", "write super-dev.yaml manifest"),
        ("continue", "approve the active gate"),
        ("revise", "stay at gate, request changes"),
        ("status", "show detailed pipeline status"),
        ("export", "export the latest proof-pack"),
        ("knowledge", "list knowledge + design files"),
        ("spec", "show the SUPER_DEV_HOST_SPEC_V1 spec"),
        ("verify", "show workspace conformance"),
        ("doctor", "self-test"),
        ("diff", "show an artifact (default: PRD)"),
        ("history", "show the conversation history"),
        ("changelog", "show CHANGELOG.md"),
        ("version", "show super-dev / spec / worker versions"),
        ("help", "show all keybindings"),
        ("clear", "clear chat history"),
        ("quit", "exit"),
    ];

    /// Match the verbs prefixed by what comes after `/` in the current
    /// input. Empty input or non-slash input → empty list.
    #[must_use]
    pub fn palette_matches(&self) -> Vec<(&'static str, &'static str)> {
        if !self.input.starts_with('/') {
            return Vec::new();
        }
        let typed = self
            .input
            .strip_prefix('/')
            .unwrap_or("")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        Self::SLASH_VERBS
            .iter()
            .filter(|(verb, _)| verb.starts_with(typed.as_str()))
            .copied()
            .collect()
    }

    /// Replace the input with `/{verb} ` (with trailing space so the
    /// user can immediately type args). Called by Tab autocomplete.
    ///
    /// Second-level: if input is `/design ` or `/template ` (verb +
    /// space + partial arg), Tab completes from the available design
    /// systems / seed templates.
    pub fn autocomplete_palette(&mut self) {
        // Second-level arg completion for /design and /template.
        if let Some(arg_completion) = self.try_arg_completion() {
            self.input = arg_completion;
            self.input_cursor = self.input_len();
            return;
        }
        let matches = self.palette_matches();
        if matches.is_empty() {
            return;
        }
        let selected = self.palette_selected.min(matches.len() - 1);
        let verb = matches[selected].0;
        self.input = format!("/{verb} ");
        self.input_cursor = self.input_len();
        self.palette_selected = 0;
    }

    fn try_arg_completion(&self) -> Option<String> {
        let input = self.input.trim_start();
        let (prefix, partial) = if let Some(rest) = input.strip_prefix("/design ") {
            ("/design ", rest.trim())
        } else if let Some(rest) = input.strip_prefix("/template ") {
            ("/template ", rest.trim())
        } else {
            return None;
        };
        let candidates = if prefix == "/design " {
            self.list_design_systems()
        } else {
            self.list_seed_templates()
        };
        if partial.is_empty() {
            candidates.first().map(|c| format!("{prefix}{c}"))
        } else {
            candidates
                .iter()
                .find(|c| c.starts_with(partial))
                .map(|c| format!("{prefix}{c}"))
        }
    }

    /// Move palette highlight up/down (with wrap-around). Called by
    /// ↑↓ when the palette is showing matches.
    pub fn cycle_palette(&mut self, delta: isize) {
        let count = self.palette_matches().len();
        if count == 0 {
            return;
        }
        // Wrap delta into [0, count). No isize casts → no clippy noise.
        if delta < 0 {
            let back = delta.unsigned_abs() % count;
            self.palette_selected = (self.palette_selected + count - back) % count;
        } else {
            #[allow(clippy::cast_sign_loss)]
            let fwd = delta as usize;
            self.palette_selected = (self.palette_selected + fwd) % count;
        }
    }

    fn set_phase(&mut self, phase: Phase, status: PhaseStatus) {
        if let Some(row) = self.phases.iter_mut().find(|r| r.phase == phase) {
            row.status = status;
        }
    }

    // ---- engine events ----------------------------------------------------

    /// Fold one engine event into the chat history + status bar.
    pub fn apply_engine(&mut self, event: EngineEvent) {
        match event {
            EngineEvent::PipelineStarted { slug, requirement } => {
                self.slug = slug;
                self.requirement.clone_from(&requirement);
                self.run_started = true;
                self.push(
                    ChatRole::SuperDev,
                    format!("流水线启动:{requirement}\n按 9 阶段流程交付,关键节点会停下让你审。"),
                );
            }
            EngineEvent::PhaseStarted { phase } => {
                self.set_phase(phase, PhaseStatus::Running);
                self.push(ChatRole::SuperDev, format!("▶ phase {} 开始…", phase.id()));
            }
            EngineEvent::ArtifactWritten { phase, path } => {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("<artifact>");
                self.push(
                    ChatRole::SuperDev,
                    format!("  · [{}] 写入 {name}", phase.id()),
                );
            }
            EngineEvent::PhaseCompleted { phase } => {
                self.set_phase(phase, PhaseStatus::Done);
                self.push(ChatRole::SuperDev, format!("✓ {}", phase.id()));
            }
            EngineEvent::GateOpened { gate } => {
                self.active_gate = Some(gate);
                self.push(
                    ChatRole::Gate,
                    gate_card(gate, &self.slug, &self.project_root),
                );
            }
            EngineEvent::BlockCompleted {
                final_phase,
                paused_at,
            } => {
                if paused_at.is_none() && final_phase == Phase::Delivery {
                    self.finished = true;
                    self.active_gate = None;
                    let release = self.project_root.join("release");
                    let zip_info = std::fs::read_dir(&release)
                        .ok()
                        .and_then(|rd| {
                            let mut zips: Vec<_> = rd
                                .filter_map(Result::ok)
                                .filter(|e| {
                                    e.path().extension().and_then(|s| s.to_str()) == Some("zip")
                                })
                                .collect();
                            zips.sort_by_key(std::fs::DirEntry::file_name);
                            zips.last().map(|z| {
                                let size =
                                    std::fs::metadata(z.path()).map_or(0, |m| m.len() / 1024);
                                format!(
                                    "\n  最新: {} ({size} KiB)",
                                    z.file_name().to_string_lossy()
                                )
                            })
                        })
                        .unwrap_or_default();
                    self.push(
                        ChatRole::SuperDev,
                        format!(
                            "✓ 流水线完成!{zip_info}\n\n\
                             下一步:\n  \
                             · 输入新需求 → 开新一轮\n  \
                             · /redo → 重跑当前需求\n  \
                             · /export → 查看 proof-pack 详情\n  \
                             · /status → 查看完整状态"
                        ),
                    );
                }
            }
            EngineEvent::BackendProbed {
                backend_id,
                ready,
                detail,
            } => {
                // Update or append the probe row.
                if let Some(existing) = self.backends.iter_mut().find(|b| b.id == backend_id) {
                    existing.ready = ready;
                    existing.detail = detail.clone();
                } else {
                    self.backends.push(BackendInfo {
                        id: backend_id.clone(),
                        ready,
                        detail: detail.clone(),
                    });
                }
                // If we're still on the picker, refresh its labels.
                if self.mode == AppMode::Picker {
                    refresh_picker_with_probes(&mut self.picker_items, &self.backends);
                }
            }
            EngineEvent::VerifyStarted { phase, command } => {
                self.push(
                    ChatRole::SuperDev,
                    format!("✱ verify [{}] running: {command}", phase.id()),
                );
            }
            EngineEvent::VerifySkipped { phase, reason } => {
                self.push(
                    ChatRole::SuperDev,
                    format!("⊘ verify [{}] skipped — {reason}", phase.id()),
                );
            }
            EngineEvent::VerifyPassed { phase, duration_ms } => {
                self.push(
                    ChatRole::SuperDev,
                    format!(
                        "✓ verify [{}] passed in {}.{}s",
                        phase.id(),
                        duration_ms / 1000,
                        (duration_ms % 1000) / 100
                    ),
                );
            }
            EngineEvent::VerifyFailed {
                phase,
                exit_code,
                stderr,
            } => {
                let snippet = stderr.lines().next().unwrap_or("").trim();
                self.push(
                    ChatRole::System,
                    format!(
                        "✗ verify [{}] FAILED (exit {exit_code}): {snippet}",
                        phase.id()
                    ),
                );
            }
            EngineEvent::HostOutput { phase: _, line } => {
                // Cap each line so a 1000-char paragraph doesn't blow the layout.
                let cap = 300;
                let trimmed: String = if line.chars().count() > cap {
                    let cut: String = line.chars().take(cap).collect();
                    format!("{cut}…")
                } else {
                    line
                };
                // Group consecutive host-output lines into the same chat
                // bubble — they belong to one phase's stream and reading
                // them as separate messages is visually noisy.
                if let Some(last) = self.history.back_mut() {
                    if last.role == ChatRole::Host {
                        last.body.push('\n');
                        last.body.push_str(&trimmed);
                    } else {
                        self.push(ChatRole::Host, trimmed);
                    }
                } else {
                    self.push(ChatRole::Host, trimmed);
                }
            }
            EngineEvent::Note(note) => {
                self.push(ChatRole::System, note);
            }
        }
        self.refresh_status();
    }

    // ---- key events -------------------------------------------------------

    /// Fold one key press into the model; return the loop's next action.
    ///
    /// `mods` carries modifiers (Shift, Ctrl, Alt) so multi-line input
    /// via `Shift+Enter` works. Tests that don't care about modifiers
    /// can use [`apply_key`](Self::apply_key) for the no-mods shortcut.
    #[must_use]
    pub fn apply_key_with_mods(
        &mut self,
        key: KeyCode,
        mods: crossterm::event::KeyModifiers,
    ) -> Action {
        // F1 toggles help in any mode.
        if let KeyCode::F(1) = key {
            self.show_help = !self.show_help;
            return Action::None;
        }
        match self.mode {
            AppMode::Picker => self.picker_key(key),
            AppMode::Chat => self.chat_key(key, mods),
        }
    }

    /// Convenience wrapper for tests / call sites that don't care about
    /// modifier state.
    #[must_use]
    pub fn apply_key(&mut self, key: KeyCode) -> Action {
        self.apply_key_with_mods(key, crossterm::event::KeyModifiers::NONE)
    }

    fn picker_key(&mut self, key: KeyCode) -> Action {
        match key {
            KeyCode::Esc => {
                self.should_quit = true;
                Action::Quit
            }
            KeyCode::Up => {
                if self.picker_selected > 0 {
                    self.picker_selected -= 1;
                }
                Action::None
            }
            KeyCode::Down => {
                if self.picker_selected + 1 < self.picker_items.len() {
                    self.picker_selected += 1;
                }
                Action::None
            }
            KeyCode::Enter => {
                let chosen = &self.picker_items[self.picker_selected];
                // Refuse if the chosen host is unavailable.
                if !chosen.ready {
                    self.push(
                        ChatRole::System,
                        format!(
                            "{} 不可用:{}\n请上下选其他选项,或先把宿主装好再来。",
                            chosen.label, chosen.detail
                        ),
                    );
                    return Action::None;
                }
                let backend_id = chosen.backend_id.clone();
                self.commit_backend(backend_id);
                self.mode = AppMode::Chat;
                self.push_greeting();
                self.refresh_status();
                Action::BackendChanged
            }
            _ => Action::None,
        }
    }

    fn chat_key(&mut self, key: KeyCode, mods: crossterm::event::KeyModifiers) -> Action {
        // Overlay routing first — when an overlay is open, everything
        // is scroll / close.
        if self.overlay.is_some() {
            return self.overlay_key(key);
        }
        if self.show_help {
            if let KeyCode::Esc = key {
                self.show_help = false;
                return Action::None;
            }
        }
        let has_palette = !self.palette_matches().is_empty();
        let shift = mods.contains(crossterm::event::KeyModifiers::SHIFT);

        match key {
            // ---- exit handling ----
            KeyCode::Esc => {
                if self.is_pipeline_active() && !self.pending_quit_confirm {
                    self.pending_quit_confirm = true;
                    self.push(
                        ChatRole::System,
                        "流水线运行中。再按一次 Esc 退出(workflow state 已保存,下次 `super-dev` 自动续上)。",
                    );
                    return Action::None;
                }
                self.should_quit = true;
                Action::Quit
            }

            // ---- input editing ----
            KeyCode::Backspace => {
                self.pending_quit_confirm = false;
                self.backspace();
                Action::None
            }
            KeyCode::Delete => {
                self.pending_quit_confirm = false;
                self.forward_delete();
                Action::None
            }
            KeyCode::Left => {
                self.move_cursor(-1);
                Action::None
            }
            KeyCode::Right => {
                self.move_cursor(1);
                Action::None
            }
            KeyCode::Home => {
                self.input_cursor = 0;
                Action::None
            }
            KeyCode::End => {
                self.input_cursor = self.input_len();
                Action::None
            }

            // ---- palette navigation (only when /-prefix has matches) ----
            KeyCode::Up if has_palette => {
                self.cycle_palette(-1);
                Action::None
            }
            KeyCode::Down if has_palette => {
                self.cycle_palette(1);
                Action::None
            }
            KeyCode::Tab if has_palette => {
                self.autocomplete_palette();
                Action::None
            }

            // ---- input history recall (no palette + empty-or-recalling input) ----
            KeyCode::Up
                if !has_palette && (self.input.is_empty() || self.input_history_idx.is_some()) =>
            {
                self.input_history_back();
                Action::None
            }
            KeyCode::Down if !has_palette && self.input_history_idx.is_some() => {
                self.input_history_forward();
                Action::None
            }

            // ---- enter: submit, or insert newline with Shift ----
            KeyCode::Enter => {
                if shift {
                    // Shift+Enter inserts a literal newline so the user
                    // can build multi-line prompts inside the chat box.
                    self.insert_at_cursor('\n');
                    return Action::None;
                }
                self.pending_quit_confirm = false;
                let raw = self.input.trim().to_string();
                self.clear_input();
                if raw.is_empty() {
                    return Action::None;
                }
                self.remember_submission(&raw);
                if let Some(action) = self.try_slash_command(&raw) {
                    return action;
                }
                self.submit_text(raw)
            }

            // ---- printable char ----
            KeyCode::Char(c) => {
                self.pending_quit_confirm = false;
                self.input_history_idx = None;
                self.insert_at_cursor(c);
                Action::None
            }

            _ => Action::None,
        }
    }

    /// Treat non-slash text as either a fresh requirement (if no run is
    /// active) or a revision (if a gate is open). Single-letter `c` at a
    /// gate is the documented shortcut for "approve / continue" — match
    /// the gate card so users don't have to type `/continue` every time.
    fn submit_text(&mut self, text: String) -> Action {
        self.push(ChatRole::You, text.clone());
        if let Some(gate) = self.active_gate {
            if matches!(text.trim(), "c" | "C") {
                self.active_gate = None;
                self.push(
                    ChatRole::SuperDev,
                    format!("✓ approved gate `{}` — continuing…", gate.id_str()),
                );
                return Action::Continue(gate);
            }
            self.push(
                ChatRole::SuperDev,
                format!("收到修订:\"{text}\"。重新跑当前 block…"),
            );
            Action::Revise(text)
        } else if self.run_started && self.finished {
            self.reset_for_new_run();
            self.maybe_suggest_design();
            self.push_preflight(&text);
            Action::StartRun(text)
        } else if !self.run_started {
            self.maybe_suggest_design();
            self.push_preflight(&text);
            Action::StartRun(text)
        } else {
            self.push(
                ChatRole::System,
                "目前正在跑流水线 — 等下一个 gate 暂停后再发指令,或用 /quit 退出。",
            );
            Action::None
        }
    }

    /// Surface a "this is what I'm about to do" preview so the user
    /// isn't left wondering whether their Enter actually did anything.
    /// Lands BEFORE the background pipeline task spawns + emits its
    /// own `PipelineStarted`.
    fn maybe_suggest_design(&mut self) {
        if self.config.design_system.is_some() {
            return;
        }
        let available = self.list_design_systems();
        if available.is_empty() {
            return;
        }
        self.push(
            ChatRole::System,
            format!(
                "提示: 你还没选设计系统。可用 /design <name> 在 run 前锁定视觉方向:\n  \
                 {}。\n  \
                 不选也行 — worker 会自动根据需求推荐一个。",
                available.join(" · ")
            ),
        );
    }

    fn push_preflight(&mut self, text: &str) {
        let ds = self.config.design_system.as_deref().unwrap_or("auto");
        let tpl = self.config.seed_template.as_deref().unwrap_or("auto");
        let plan = format!(
            "收到需求:\"{text}\"\n\n\
             配置: worker `{}` · 设计系统 `{ds}` · 模板 `{tpl}`\n\n\
             9 阶段流水线:\n  \
             1) research(竞品 + discovery) → 2) docs(PRD + Architecture + UIUX) → ⏸ docs_confirm\n  \
             3) spec → 4) frontend(按 UIUX tokens 实现) → ⏸ preview_confirm\n  \
             5) backend → 6) quality(5 维审查) → 7) delivery → proof-pack\n\n\
             两道 gate 我会停下来给你审稿。流水线启动中…",
            self.backend_label,
        );
        self.push(ChatRole::SuperDev, plan);
    }

    fn reset_for_new_run(&mut self) {
        for row in &mut self.phases {
            row.status = PhaseStatus::Pending;
        }
        self.finished = false;
        self.run_started = false;
        self.active_gate = None;
    }

    /// `Some(action)` if `raw` was a recognised slash command; `None`
    /// means "not a slash command, treat as ordinary input".
    fn try_slash_command(&mut self, raw: &str) -> Option<Action> {
        if !raw.starts_with('/') {
            return None;
        }
        let mut parts = raw[1..].splitn(2, char::is_whitespace);
        let verb = parts.next().unwrap_or("").to_ascii_lowercase();
        let rest = parts.next().unwrap_or("").trim();
        self.push(ChatRole::You, raw.to_string());
        let action = match verb.as_str() {
            "help" | "?" | "commands" => {
                self.show_help = true;
                Action::None
            }
            "quit" | "q" | "exit" => {
                self.should_quit = true;
                Action::Quit
            }
            "clear" => {
                self.history.clear();
                self.push(ChatRole::System, "history cleared.");
                Action::None
            }
            "claude" | "claude-code" => self.slash_backend(Some("claude-code")),
            "codex" => self.slash_backend(Some("codex")),
            "gemini" => self.slash_backend(Some("gemini")),
            "droid" => self.slash_backend(Some("droid")),
            "opencode" => self.slash_backend(Some("opencode")),
            "qwen" => self.slash_backend(Some("qwen")),
            "copilot" => self.slash_backend(Some("copilot")),
            "trae" | "trae-cli" => self.slash_backend(Some("trae")),
            "codebuddy" => self.slash_backend(Some("codebuddy")),
            "qoder" | "qodercli" => self.slash_backend(Some("qoder")),
            "kimi" => self.slash_backend(Some("kimi")),
            "offline" => self.slash_backend(None),
            "init" => {
                let slug = if self.slug.is_empty() {
                    self.project_root
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("project")
                        .to_string()
                } else {
                    self.slug.clone()
                };
                let manifest = super_dev_agent::SpecManifest::new(&slug);
                match manifest.write_to(&self.project_root, false) {
                    Ok(path) => {
                        let ds_count = self.scaffold_design_files();
                        let ds_msg = if ds_count > 0 {
                            format!("\n  设计基础设施: {ds_count} 文件写入 knowledge/")
                        } else {
                            String::new()
                        };
                        self.push(
                            ChatRole::SuperDev,
                            format!(
                                "✓ super-dev.yaml 写入 {}\n  level={} profile={} slug={slug}{ds_msg}",
                                path.display(),
                                manifest.level.as_str(),
                                manifest.profile.as_str(),
                            ),
                        );
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                        let ds_count = self.scaffold_design_files();
                        let ds_msg = if ds_count > 0 {
                            format!("设计基础设施: {ds_count} 新文件写入 knowledge/")
                        } else {
                            "super-dev.yaml 已存在。设计文件也已就绪。".to_string()
                        };
                        self.push(ChatRole::System, ds_msg);
                    }
                    Err(e) => self.push(ChatRole::System, format!("/init 失败:{e}")),
                }
                Action::None
            }
            "continue" => {
                if let Some(gate) = self.active_gate.take() {
                    self.push(
                        ChatRole::SuperDev,
                        format!("✓ approved gate `{}` — continuing…", gate.id_str()),
                    );
                    Action::Continue(gate)
                } else {
                    let hint = if self.run_started && !self.finished {
                        "流水线正在跑 — 等下一个 gate 暂停时再 /continue。"
                    } else if self.finished {
                        "流水线已完成。直接输入新需求开新一轮,或 /quit 退出。"
                    } else {
                        "还没启动流水线。直接输入需求一句话即可启动 9 阶段流水线。"
                    };
                    self.push(ChatRole::System, hint);
                    Action::None
                }
            }
            "revise" => {
                if rest.is_empty() {
                    self.push(
                        ChatRole::System,
                        "用法:/revise <修订说明>。\n  示例:/revise 去掉社交登录,只留邮箱密码",
                    );
                    Action::None
                } else if self.active_gate.is_some() {
                    self.push(
                        ChatRole::SuperDev,
                        format!("收到修订:\"{rest}\"。重新跑当前 block…"),
                    );
                    Action::Revise(rest.to_string())
                } else {
                    self.push(
                        ChatRole::System,
                        "目前没有打开的 gate,无法修订 — 要修改产出需要先等流水线停在 gate(docs_confirm / preview_confirm)。",
                    );
                    Action::None
                }
            }
            "spec" => {
                self.open_spec_overlay();
                Action::None
            }
            "verify" => {
                self.open_verify_overlay();
                Action::None
            }
            "doctor" => {
                self.open_doctor_overlay();
                Action::None
            }
            "diff" => {
                self.open_diff_overlay(rest);
                Action::None
            }
            "history" => {
                self.open_history_overlay();
                Action::None
            }
            "model" => self.slash_model(rest),
            "design" => self.slash_design(rest),
            "template" => self.slash_template(rest),
            "run" => self.slash_run(rest),
            "status" => {
                self.open_status_overlay();
                Action::None
            }
            "export" => {
                self.slash_export();
                Action::None
            }
            "knowledge" => {
                self.open_knowledge_overlay();
                Action::None
            }
            "redo" => self.slash_redo(),
            "config" => {
                self.open_config_overlay();
                Action::None
            }
            "version" => {
                self.open_version_overlay();
                Action::None
            }
            "changelog" => {
                self.open_changelog_overlay();
                Action::None
            }
            _ => {
                let hint = Self::did_you_mean(&verb)
                    .map(|s| format!(" 是想用 `/{s}` 吗?"))
                    .unwrap_or_default();
                self.push(
                    ChatRole::System,
                    format!("未知命令 `/{verb}`。{hint} 输入 /help 看完整列表。"),
                );
                Action::None
            }
        };
        Some(action)
    }

    /// Closest recognised slash verb to `typed`, if any sits within a
    /// useful "did-you-mean" radius. Used to suggest a fix when the user
    /// mistypes a command verb.
    fn did_you_mean(typed: &str) -> Option<&'static str> {
        if typed.is_empty() {
            return None;
        }
        // Prefix match first — handles `/c` → `claude` and `/rev` → `revise`.
        if let Some((verb, _)) = Self::SLASH_VERBS.iter().find(|(v, _)| v.starts_with(typed)) {
            return Some(verb);
        }
        // Otherwise Levenshtein ≤ 2 against known verbs.
        let typed_lower = typed.to_ascii_lowercase();
        let (mut best, mut best_dist) = (None, usize::MAX);
        for (verb, _) in Self::SLASH_VERBS {
            let d = lev(&typed_lower, verb);
            if d < best_dist && d <= 2 {
                best = Some(*verb);
                best_dist = d;
            }
        }
        best
    }

    fn slash_backend(&mut self, backend: Option<&str>) -> Action {
        let id = backend.unwrap_or("offline").to_string();
        self.commit_backend(backend.map(str::to_string));
        self.push(
            ChatRole::System,
            format!("已切换工人到 `{id}` — 下一个 run 起用此 backend。"),
        );
        self.refresh_status();
        Action::BackendChanged
    }

    fn slash_model(&mut self, arg: &str) -> Action {
        if arg.is_empty() {
            let current = self
                .config
                .model
                .as_deref()
                .unwrap_or("(default for worker)");
            self.push(
                ChatRole::System,
                format!(
                    "用法:/model <model-id>。当前 model: {current}。\n  \
                     示例:/model claude-opus-4-7 · /model gpt-5 · /model gemini-2.5-pro"
                ),
            );
            return Action::None;
        }
        self.config.model = Some(arg.to_string());
        if let Err(e) = crate::config::save_to(&self.config, &self.config_path) {
            self.push(
                ChatRole::System,
                format!("(无法写入 {}: {e})", self.config_path.display()),
            );
        }
        self.push(
            ChatRole::System,
            format!("model 切换为 `{arg}` — 下一个 run 起用此 model。"),
        );
        Action::None
    }

    fn slash_design(&mut self, arg: &str) -> Action {
        let available = self.list_design_systems();
        if arg.is_empty() {
            // No arg → open overlay listing all design systems with previews
            self.open_design_picker_overlay(&available);
            return Action::None;
        }
        if !available.contains(&arg.to_string()) && !available.is_empty() {
            self.push(
                ChatRole::System,
                format!("未找到设计系统 `{arg}`。可选: {}", available.join(" · ")),
            );
            return Action::None;
        }
        self.config.design_system = Some(arg.to_string());
        if let Err(e) = crate::config::save_to(&self.config, &self.config_path) {
            self.push(
                ChatRole::System,
                format!("(无法写入 {}: {e})", self.config_path.display()),
            );
        }
        // Show a rich preview of the selected design system
        let preview = self.read_design_system_preview(arg);
        self.push(ChatRole::SuperDev, format!("设计系统: `{arg}`\n{preview}"));
        self.refresh_status();
        Action::None
    }

    fn slash_template(&mut self, arg: &str) -> Action {
        let available = self.list_seed_templates();
        if arg.is_empty() {
            let current = self
                .config
                .seed_template
                .as_deref()
                .unwrap_or("(auto-detect)");
            let list = if available.is_empty() {
                "(no seed templates found in knowledge/seed-templates/)".to_string()
            } else {
                available.join(" · ")
            };
            self.push(
                ChatRole::System,
                format!(
                    "用法:/template <name>。当前: {current}。\n  可选: {list}\n  \
                     示例:/template saas-landing · /template dashboard · /template blog-content"
                ),
            );
            return Action::None;
        }
        if !available.contains(&arg.to_string()) && !available.is_empty() {
            self.push(
                ChatRole::System,
                format!("未找到模板 `{arg}`。可选: {}", available.join(" · ")),
            );
            return Action::None;
        }
        self.config.seed_template = Some(arg.to_string());
        if let Err(e) = crate::config::save_to(&self.config, &self.config_path) {
            self.push(
                ChatRole::System,
                format!("(无法写入 {}: {e})", self.config_path.display()),
            );
        }
        self.push(
            ChatRole::SuperDev,
            format!(
                "种子模板切换为 `{arg}` — 下一个 run 的 frontend 阶段将参考此模板的页面结构和质量标准。"
            ),
        );
        self.refresh_status();
        Action::None
    }

    fn open_design_picker_overlay(&mut self, available: &[String]) {
        let current = self.config.design_system.as_deref().unwrap_or("");
        let mut body = String::from("Design Systems\n==============\n\n");
        if available.is_empty() {
            body.push_str("No design systems found.\nRun /init to scaffold them into knowledge/design-systems/\n");
        } else {
            body.push_str("Usage: /design <name>\n\n");
            for name in available {
                let mark = if name == current { "●" } else { "○" };
                let path = self
                    .project_root
                    .join("knowledge/design-systems")
                    .join(format!("{name}.md"));
                let preview = Self::extract_design_preview_static(&path);
                body.push_str(&format!("{mark} {name}\n{preview}\n\n"));
            }
        }
        self.overlay = Some(Overlay::from_body(
            " /design — pick a design system · Esc close ",
            &body,
        ));
    }

    fn read_design_system_preview(&self, name: &str) -> String {
        let path = self
            .project_root
            .join("knowledge/design-systems")
            .join(format!("{name}.md"));
        Self::extract_design_preview_static(&path)
    }

    fn extract_design_preview_static(path: &std::path::Path) -> String {
        let Ok(content) = std::fs::read_to_string(path) else {
            return "  (file not readable)".to_string();
        };
        let mut preview = String::new();

        for line in content.lines() {
            if let Some(desc) = line.strip_prefix("> ") {
                preview.push_str(&format!("  {desc}\n"));
                break;
            }
        }

        // Extract "When to use" section
        let mut in_when = false;
        for line in content.lines() {
            if line.starts_with("## When to use") {
                in_when = true;
                continue;
            }
            if in_when {
                if line.starts_with("## ") {
                    break;
                }
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    preview.push_str(&format!("  Use: {trimmed}\n"));
                    break;
                }
            }
        }

        // Extract key colors from :root block
        let mut colors = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("--color-bg:")
                || trimmed.starts_with("--color-primary:")
                || trimmed.starts_with("--color-text:")
                || trimmed.starts_with("--color-accent:")
            {
                if let Some(val) = trimmed.split(':').nth(1) {
                    let hex = val.trim().trim_end_matches(';').trim();
                    let name = trimmed
                        .split(':')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_start_matches('-');
                    colors.push(format!("{name}: {hex}"));
                }
            }
            if colors.len() >= 4 {
                break;
            }
        }
        if !colors.is_empty() {
            preview.push_str(&format!("  Palette: {}\n", colors.join(" · ")));
        }

        // Extract font families
        for line in content.lines() {
            if line.contains("**Headings**:") || line.contains("**Body**:") {
                let trimmed = line.trim().trim_start_matches("- ");
                preview.push_str(&format!("  {trimmed}\n"));
            }
        }

        // Count total tokens
        let token_count = content.matches("--").count();
        preview.push_str(&format!("  Tokens: {token_count} CSS variables"));

        preview
    }

    fn list_design_systems(&self) -> Vec<String> {
        let dir = self.project_root.join("knowledge/design-systems");
        Self::list_md_stems(&dir)
    }

    fn list_seed_templates(&self) -> Vec<String> {
        let dir = self.project_root.join("knowledge/seed-templates");
        Self::list_md_stems(&dir)
    }

    fn list_md_stems(dir: &std::path::Path) -> Vec<String> {
        let mut names = Vec::new();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        if !stem.starts_with("00-") {
                            names.push(stem.to_string());
                        }
                    }
                }
            }
        }
        names.sort();
        names
    }

    fn slash_run(&mut self, arg: &str) -> Action {
        if arg.is_empty() {
            self.push(
                ChatRole::System,
                "用法:/run [slug] <需求>\n  \
                 示例:/run my-app 做一个登录系统\n  \
                 示例:/run 做一个 todo list (slug 自动用目录名)",
            );
            return Action::None;
        }
        let (slug, req) = if let Some((first, rest)) = arg.split_once(' ') {
            if rest.trim().is_empty() {
                (String::new(), first.to_string())
            } else {
                (first.to_string(), rest.trim().to_string())
            }
        } else {
            (String::new(), arg.to_string())
        };
        if !slug.is_empty() {
            if slug.contains(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_') {
                self.push(
                    ChatRole::System,
                    format!("slug `{slug}` 包含非法字符。只允许字母、数字、`-`、`_`。"),
                );
                return Action::None;
            }
            self.slug = slug;
        }
        if self.run_started {
            self.reset_for_new_run();
        }
        self.maybe_suggest_design();
        self.push_preflight(&req);
        Action::StartRun(req)
    }

    fn open_status_overlay(&mut self) {
        let mut body = String::from("Pipeline Status\n===============\n\n");
        body.push_str(&format!("worker:        {}\n", self.backend_label));
        body.push_str(&format!(
            "design system: {}\n",
            self.config.design_system.as_deref().unwrap_or("(none)")
        ));
        body.push_str(&format!(
            "seed template: {}\n",
            self.config.seed_template.as_deref().unwrap_or("(auto)")
        ));
        body.push_str(&format!(
            "slug:          {}\n",
            if self.slug.is_empty() {
                "(not set)"
            } else {
                &self.slug
            }
        ));
        body.push_str(&format!(
            "requirement:   {}\n",
            if self.requirement.is_empty() {
                "(none yet)"
            } else {
                &self.requirement
            }
        ));
        body.push_str("\n## Pipeline phases\n\n");
        body.push_str("| # | Phase | Status |\n|---|---|---|\n");
        for (i, row) in self.phases.iter().enumerate() {
            let icon = match row.status {
                PhaseStatus::Done => "✓",
                PhaseStatus::Running => "◐",
                PhaseStatus::Pending => "○",
            };
            body.push_str(&format!("| {} | {} | {} |\n", i + 1, row.phase.id(), icon));
        }
        if let Some(gate) = self.active_gate {
            body.push_str(&format!("\n⏸ Active gate: `{}`\n", gate.id_str()));
        }
        if self.finished {
            body.push_str("\n✓ Pipeline complete — proof-pack in release/\n");
        }
        // Artifacts
        let output_dir = self.project_root.join("output");
        if output_dir.is_dir() {
            body.push_str("\n## Artifacts\n\n");
            if let Ok(rd) = std::fs::read_dir(&output_dir) {
                let mut entries: Vec<_> = rd.filter_map(Result::ok).collect();
                entries.sort_by_key(std::fs::DirEntry::file_name);
                for e in entries.iter().take(20) {
                    let name = e.file_name();
                    let size = std::fs::metadata(e.path()).map_or(0, |m| m.len());
                    body.push_str(&format!(
                        "  · {} ({} bytes)\n",
                        name.to_string_lossy(),
                        size
                    ));
                }
            }
        }
        // Quality gate results
        let qg_path = output_dir.join(format!("{}-quality-gate.json", self.slug));
        if qg_path.is_file() {
            body.push_str("\n## Quality gate\n\n");
            if let Ok(qg_content) = std::fs::read_to_string(&qg_path) {
                let score = crate::app::extract_json_number(&qg_content, "score");
                let passed = crate::app::extract_json_bool(&qg_content, "passed");
                body.push_str(&format!(
                    "  Score: {}/100 · {}\n",
                    score.map_or("?".to_string(), |n| n.to_string()),
                    match passed {
                        Some(true) => "PASSED ✓",
                        Some(false) => "BLOCKED ✗",
                        None => "?",
                    }
                ));
            }
        }

        // Knowledge RAG info — show which domains would be queried per phase
        body.push_str("\n## Knowledge RAG mapping\n\n");
        body.push_str("| Phase | Knowledge domains |\n|---|---|\n");
        body.push_str("| research | ALL (308 files, keyword-ranked) |\n");
        body.push_str("| docs | product, architecture, design, frontend, industries |\n");
        body.push_str("| spec | development, governance, product |\n");
        body.push_str("| frontend | frontend, design, design-systems, seed-templates |\n");
        body.push_str("| backend | backend, api, database, security, cloud-native |\n");
        body.push_str("| quality | testing, security, governance |\n");
        body.push_str("| delivery | cicd, operations, governance, security |\n");

        self.overlay = Some(Overlay::from_body(" status — Esc close ", &body));
    }

    fn slash_export(&mut self) {
        let release = self.project_root.join("release");
        if !release.is_dir() {
            self.push(
                ChatRole::System,
                "release/ 目录不存在 — 流水线需要跑到 delivery 才会生成 proof-pack。",
            );
            return;
        }
        let mut zips: Vec<_> = std::fs::read_dir(&release)
            .ok()
            .map(|rd| {
                rd.filter_map(Result::ok)
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("zip"))
                    .collect()
            })
            .unwrap_or_default();
        zips.sort_by_key(std::fs::DirEntry::file_name);
        if zips.is_empty() {
            self.push(
                ChatRole::System,
                "release/ 目录为空 — 需要完成 delivery 阶段才会产出 proof-pack.zip。",
            );
            return;
        }
        let latest = zips.last().unwrap();
        let size = std::fs::metadata(latest.path()).map_or(0, |m| m.len() / 1024);
        self.push(
            ChatRole::SuperDev,
            format!(
                "✓ 最新 proof-pack: {}\n  大小: {size} KiB\n  路径: {}\n\n\
                 可直接分享给审核人员。用 `unzip -l {}` 查看内容。",
                latest.file_name().to_string_lossy(),
                latest.path().display(),
                latest.path().display(),
            ),
        );
    }

    fn open_knowledge_overlay(&mut self) {
        let mut body = String::from("knowledge base\n==============\n\n");
        // Design systems
        body.push_str("## Design systems (knowledge/design-systems/)\n\n");
        let ds = self.list_design_systems();
        if ds.is_empty() {
            body.push_str("  (none found)\n");
        } else {
            let active = self.config.design_system.as_deref().unwrap_or("");
            for name in &ds {
                let mark = if name == active { "●" } else { "○" };
                body.push_str(&format!("  {mark} {name}\n"));
            }
        }
        // Seed templates
        body.push_str("\n## Seed templates (knowledge/seed-templates/)\n\n");
        let tpl = self.list_seed_templates();
        if tpl.is_empty() {
            body.push_str("  (none found)\n");
        } else {
            let active = self.config.seed_template.as_deref().unwrap_or("");
            for name in &tpl {
                let mark = if name == active { "●" } else { "○" };
                body.push_str(&format!("  {mark} {name}\n"));
            }
        }
        // General knowledge
        body.push_str("\n## Knowledge files (knowledge/)\n\n");
        let kdir = self.project_root.join("knowledge");
        if kdir.is_dir() {
            let mut count = 0;
            if let Ok(rd) = std::fs::read_dir(&kdir) {
                let mut dirs: Vec<_> = rd
                    .filter_map(Result::ok)
                    .filter(|e| e.path().is_dir())
                    .collect();
                dirs.sort_by_key(std::fs::DirEntry::file_name);
                for d in &dirs {
                    let name = d.file_name();
                    let n = name.to_string_lossy();
                    if n == "design-systems" || n == "seed-templates" {
                        continue;
                    }
                    let file_count = std::fs::read_dir(d.path()).map_or(0, Iterator::count);
                    body.push_str(&format!("  📁 {n}/ ({file_count} files)\n"));
                    count += file_count;
                }
            }
            body.push_str(&format!("\n  Total: {count} knowledge files\n"));
        } else {
            body.push_str("  (no knowledge/ directory)\n");
        }
        self.overlay = Some(Overlay::from_body(" knowledge — Esc close ", &body));
    }

    fn slash_redo(&mut self) -> Action {
        if self.requirement.is_empty() {
            self.push(
                ChatRole::System,
                "还没有跑过任何需求 — 直接输入需求或用 /run 启动。",
            );
            return Action::None;
        }
        let req = self.requirement.clone();
        self.reset_for_new_run();
        self.push(
            ChatRole::SuperDev,
            format!("重新跑需求:\"{req}\"。流水线从 research 重新开始…"),
        );
        self.push_preflight(&req);
        Action::StartRun(req)
    }

    fn open_config_overlay(&mut self) {
        let mut body = String::from("Configuration\n=============\n\n");
        body.push_str(&format!(
            "worker:          {}\n",
            self.config
                .backend
                .as_deref()
                .unwrap_or("(use picker to select)")
        ));
        body.push_str(&format!(
            "model:           {}\n",
            self.config
                .model
                .as_deref()
                .unwrap_or("(default for worker)")
        ));
        body.push_str(&format!(
            "design system:   {}\n",
            self.config
                .design_system
                .as_deref()
                .unwrap_or("(none — /design to pick)")
        ));
        body.push_str(&format!(
            "seed template:   {}\n",
            self.config
                .seed_template
                .as_deref()
                .unwrap_or("(auto-detect)")
        ));
        body.push_str(&format!(
            "slug:            {}\n",
            if self.slug.is_empty() {
                "(auto from dir name)"
            } else {
                &self.slug
            }
        ));
        body.push_str(&format!(
            "workspace:       {}\n",
            self.project_root.display()
        ));
        body.push_str(&format!(
            "config file:     {}\n",
            self.config_path.display()
        ));
        body.push_str(&format!(
            "input history:   {}\n",
            self.history_path().display()
        ));
        body.push_str(&format!("history entries: {}\n", self.input_history.len()));
        body.push_str("\n## How to change\n\n");
        body.push_str("  /claude /codex /gemini ...    switch worker\n");
        body.push_str("  /model <id>                   switch model\n");
        body.push_str("  /design <name>                switch design system\n");
        body.push_str("  /template <name>              switch seed template\n");
        body.push_str("  /run <slug> <req>             set slug + requirement\n");
        self.overlay = Some(Overlay::from_body(" config — Esc close ", &body));
    }

    fn scaffold_design_files(&self) -> usize {
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
            let target = self.project_root.join(rel);
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

    fn open_version_overlay(&mut self) {
        let mut body = String::new();
        body.push_str("Super Dev — version\n");
        body.push_str("===================\n\n");
        body.push_str(&format!(
            "binary       super-dev {} (built from rev {})\n",
            env!("CARGO_PKG_VERSION"),
            option_env!("VERGEN_GIT_SHA").unwrap_or("unreleased"),
        ));
        body.push_str(&format!("spec         {}\n", super_dev_spec::SPEC_VERSION));
        body.push_str(&format!("worker       {}\n", self.backend_label));
        if let Some(m) = &self.config.model {
            body.push_str(&format!("model        {m}\n"));
        } else {
            body.push_str("model        (default for the worker)\n");
        }
        body.push_str(&format!(
            "design       {}\n",
            self.config
                .design_system
                .as_deref()
                .unwrap_or("(none — use /design to pick)")
        ));
        body.push_str(&format!(
            "template     {}\n",
            self.config
                .seed_template
                .as_deref()
                .unwrap_or("(auto-detect)")
        ));
        body.push_str(&format!("workspace    {}\n", self.project_root.display()));
        body.push_str(&format!("config       {}\n", self.config_path.display()));
        body.push('\n');
        body.push_str("Project home: https://github.com/shangyankeji/super-dev\n");
        self.overlay = Some(Overlay::from_body(" version — Esc close ", &body));
    }

    fn open_changelog_overlay(&mut self) {
        // Embedded at compile time so the overlay matches the binary,
        // not whatever CHANGELOG.md happens to be in the user's cwd.
        let body = include_str!("../../../CHANGELOG.md");
        self.overlay = Some(Overlay::from_body(
            " CHANGELOG — Esc close, ↑↓ scroll ",
            body,
        ));
    }

    // ---- overlays --------------------------------------------------------

    fn overlay_key(&mut self, key: KeyCode) -> Action {
        let Some(ov) = self.overlay.as_mut() else {
            return Action::None;
        };
        match key {
            KeyCode::Esc | KeyCode::Char('q' | 'Q') => {
                self.overlay = None;
            }
            KeyCode::Down | KeyCode::Char('j' | 'J') => ov.scroll_down(1),
            KeyCode::Up | KeyCode::Char('k' | 'K') => ov.scroll_up(1),
            KeyCode::PageDown | KeyCode::Char(' ') => ov.scroll_down(10),
            KeyCode::PageUp => ov.scroll_up(10),
            KeyCode::Home | KeyCode::Char('g') => ov.scroll = 0,
            KeyCode::End | KeyCode::Char('G') => {
                ov.scroll = ov.lines.len().saturating_sub(1);
            }
            _ => {}
        }
        Action::None
    }

    fn open_spec_overlay(&mut self) {
        // Embedded at compile-time via include_str! from the same file
        // `super-dev spec` prints, so the overlay is always fresh.
        let body = include_str!("../../../spec/SUPER_DEV_HOST_SPEC_V1.md");
        self.overlay = Some(Overlay::from_body(
            " SUPER_DEV_HOST_SPEC_V1 — press Esc to close, ↑↓ scroll ",
            body,
        ));
    }

    fn open_doctor_overlay(&mut self) {
        let mut body = String::from(
            "Doctor\n\
             ======\n\n",
        );
        body.push_str(&format!(
            "binary       super-dev {} (spec {})\n",
            env!("CARGO_PKG_VERSION"),
            super_dev_spec::SPEC_VERSION,
        ));
        body.push_str(&format!("workspace    {}\n", self.project_root.display()));
        body.push_str(&format!("worker       {}\n", self.backend_label));
        // Spec manifest
        let manifest = super_dev_agent::SpecManifest::read_from(&self.project_root);
        match manifest {
            Some(m) => body.push_str(&format!(
                "manifest     super-dev.yaml present (level {}, profile {})\n",
                m.level.as_str(),
                m.profile.as_str(),
            )),
            None => body.push_str("manifest     ⚠ no super-dev.yaml — type /init to create one\n"),
        }
        // Backend probes
        body.push_str("\nworker availability:\n");
        if self.backends.is_empty() {
            body.push_str("  (probing…)\n");
        } else {
            for b in &self.backends {
                let mark = if b.ready { "✓" } else { "✗" };
                body.push_str(&format!("  {mark} {:<14} {}\n", b.id, b.detail));
            }
        }
        // Design systems + seed templates
        body.push_str("\ndesign infrastructure:\n");
        let ds_list = self.list_design_systems();
        if ds_list.is_empty() {
            body.push_str("  ⚠ no design systems found in knowledge/design-systems/\n");
        } else {
            let active = self.config.design_system.as_deref().unwrap_or("");
            for ds in &ds_list {
                let mark = if ds == active { "●" } else { "○" };
                body.push_str(&format!("  {mark} {ds}\n"));
            }
        }
        let tpl_list = self.list_seed_templates();
        if !tpl_list.is_empty() {
            let active = self.config.seed_template.as_deref().unwrap_or("");
            body.push_str("  templates: ");
            let labels: Vec<String> = tpl_list
                .iter()
                .map(|t| {
                    if t == active {
                        format!("[{t}]")
                    } else {
                        t.clone()
                    }
                })
                .collect();
            body.push_str(&labels.join(" · "));
            body.push('\n');
        }

        self.overlay = Some(Overlay::from_body(" doctor — press Esc to close ", &body));
    }

    fn open_verify_overlay(&mut self) {
        let mut body = String::from(
            "Workspace Verify\n\
             ================\n\n",
        );
        body.push_str(&format!("workspace: {}\n\n", self.project_root.display()));

        // Spec manifest section
        body.push_str("## Spec manifest (SD-META-001)\n");
        match super_dev_agent::SpecManifest::read_from(&self.project_root) {
            Some(m) => body.push_str(&format!(
                "  version={} level={} profile={} declared_by={}\n",
                m.spec_version,
                m.level.as_str(),
                m.profile.as_str(),
                m.declared_by,
            )),
            None => body.push_str("  <missing — type /init to create>\n"),
        }

        // Workflow state
        body.push_str("\n## Workflow state\n");
        match super_dev_agent::read_workflow_state(&self.project_root) {
            Some(s) => body.push_str(&format!(
                "  phase={} active_gate={} worker={} slug={}\n  requirement={}\n",
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
                s.slug,
                s.requirement,
            )),
            None => body.push_str("  <none — pipeline has not run yet>\n"),
        }

        // Output directory contents
        body.push_str("\n## Artifacts (output/)\n");
        let output_dir = self.project_root.join("output");
        if output_dir.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(&output_dir)
                .ok()
                .map(|rd| rd.filter_map(Result::ok).collect())
                .unwrap_or_default();
            entries.sort_by_key(std::fs::DirEntry::file_name);
            if entries.is_empty() {
                body.push_str("  (empty)\n");
            } else {
                for e in entries.iter().take(20) {
                    body.push_str(&format!("  · {}\n", e.file_name().to_string_lossy()));
                }
            }
        } else {
            body.push_str("  (output/ not yet created)\n");
        }

        // Quality gate — quick verdict so users don't have to open the JSON.
        body.push_str("\n## Quality gate\n");
        let qg_paths: Vec<_> = std::fs::read_dir(&output_dir)
            .ok()
            .map(|rd| {
                rd.filter_map(Result::ok)
                    .filter(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .ends_with("-quality-gate.json")
                    })
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default();
        if qg_paths.is_empty() {
            body.push_str("  (quality phase has not produced a gate report yet)\n");
        } else {
            for p in &qg_paths {
                let score_line = match std::fs::read_to_string(p) {
                    Ok(s) => {
                        let score = extract_json_number(&s, "score")
                            .map_or_else(|| "?".to_string(), |n| n.to_string());
                        let verdict = match extract_json_bool(&s, "passed") {
                            Some(true) => "PASSED",
                            Some(false) => "BLOCKED",
                            None => "?",
                        };
                        format!(
                            "  · {} → {score}/100 ({verdict})\n",
                            p.file_name().unwrap_or_default().to_string_lossy(),
                        )
                    }
                    Err(_) => format!("  · {} (unreadable)\n", p.display()),
                };
                body.push_str(&score_line);
            }
        }

        // Release proof-packs
        body.push_str("\n## Proof packs (release/)\n");
        let release = self.project_root.join("release");
        if release.is_dir() {
            let mut zips: Vec<_> = std::fs::read_dir(&release)
                .ok()
                .map(|rd| {
                    rd.filter_map(Result::ok)
                        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("zip"))
                        .collect()
                })
                .unwrap_or_default();
            zips.sort_by_key(std::fs::DirEntry::file_name);
            if zips.is_empty() {
                body.push_str("  (none — pipeline must reach delivery first)\n");
            } else {
                for z in zips.iter().rev().take(3) {
                    let size = std::fs::metadata(z.path()).map_or(0, |m| m.len() / 1024);
                    body.push_str(&format!(
                        "  · {} ({size} KiB)\n",
                        z.file_name().to_string_lossy()
                    ));
                }
            }
        } else {
            body.push_str("  (release/ not yet created)\n");
        }

        self.overlay = Some(Overlay::from_body(" verify — press Esc to close ", &body));
    }

    fn open_diff_overlay(&mut self, arg: &str) {
        let slug = if self.slug.is_empty() {
            "<slug>"
        } else {
            self.slug.as_str()
        };
        // Pick a sensible artifact: explicit arg → exact name; bare /diff
        // → the PRD (the most-asked-for read).
        let name = if arg.is_empty() { "prd" } else { arg };
        let candidate = self
            .project_root
            .join("output")
            .join(format!("{slug}-{name}.md"));
        let body = if let Ok(text) = std::fs::read_to_string(&candidate) {
            text
        } else {
            // Fallback: list available artifacts so the user can pick.
            let mut hint = format!("找不到 {} — 现有工件:\n\n", candidate.display());
            let output_dir = self.project_root.join("output");
            if let Ok(rd) = std::fs::read_dir(&output_dir) {
                for entry in rd.flatten() {
                    if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                        hint.push_str(&format!("  · {}\n", entry.file_name().to_string_lossy()));
                    }
                }
            } else {
                hint.push_str("  (output/ 还不存在,先跑一段流水线)\n");
            }
            hint.push_str(
                "\n用法:/diff prd · /diff architecture · /diff uiux · /diff frontend-notes …",
            );
            hint
        };
        self.overlay = Some(Overlay::from_body(
            format!(" diff: {} — Esc close ", candidate.display()),
            &body,
        ));
    }

    fn open_history_overlay(&mut self) {
        let mut body = String::new();
        for msg in &self.history {
            let label = match msg.role {
                ChatRole::You => "you",
                ChatRole::SuperDev => "super-dev",
                ChatRole::Host => "worker",
                ChatRole::Gate => "GATE",
                ChatRole::System => "system",
            };
            body.push_str(&format!("[{label}] {}\n", msg.body));
            body.push('\n');
        }
        if body.is_empty() {
            body.push_str("(empty)");
        }
        self.overlay = Some(Overlay::from_body(
            " conversation history — Esc close, ↑↓ scroll ",
            &body,
        ));
    }

    fn commit_backend(&mut self, backend: Option<String>) {
        self.backend.clone_from(&backend);
        self.backend_label = backend.clone().unwrap_or_else(|| "offline".to_string());
        self.config.backend = Some(self.backend_label.clone());
        // Persist; failures only surface as a system message so the TUI
        // never panics on config save errors.
        if let Err(e) = crate::config::save_to(&self.config, &self.config_path) {
            self.push(
                ChatRole::System,
                format!("(无法写入 {}: {e})", self.config_path.display()),
            );
        }
    }

    /// Advance the spinner animation frame; status bar is regenerated
    /// so the spinner glyph actually rotates while a phase is running.
    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.refresh_status();
    }

    /// Current spinner glyph for a running phase.
    #[must_use]
    pub fn spinner(&self) -> char {
        const FRAMES: [char; 4] = ['|', '/', '-', '\\'];
        FRAMES[(self.tick as usize / 2) % FRAMES.len()]
    }
}

/// The fixed set of options the picker shows. Probe results refine the
/// labels at runtime.
fn default_picker_items() -> Vec<PickerItem> {
    let mut items: Vec<PickerItem> = super_dev_host::BACKEND_IDS
        .iter()
        .map(|id| {
            let display = super_dev_host::driver_for(id)
                .map_or_else(|| (*id).to_string(), |d| d.display_name().to_string());
            PickerItem {
                backend_id: Some((*id).to_string()),
                label: display,
                ready: false,
                detail: "detecting…".to_string(),
            }
        })
        .collect();
    items.push(PickerItem {
        backend_id: None,
        label: "offline".to_string(),
        ready: true,
        detail: "deterministic templates (no AI; demos / CI)".to_string(),
    });
    items
}

/// Tiny scalar extractors used by the `/verify` overlay so we don't need
/// a JSON dependency just to surface "score: 95 / passed: true" from the
/// quality-gate file. Returns `None` if the key isn't present or the
/// value isn't shaped like a JSON number / bool.
fn extract_json_number(json: &str, key: &str) -> Option<u32> {
    let needle = format!("\"{key}\"");
    let after = json.split(&needle).nth(1)?;
    let colon = after.find(':')?;
    let rest = &after[colon + 1..];
    let digits: String = rest
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse::<u32>().ok()
}

fn extract_json_bool(json: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\"");
    let after = json.split(&needle).nth(1)?;
    let colon = after.find(':')?;
    let rest = after[colon + 1..].trim_start();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

/// Classic Levenshtein distance — used by the slash command typo
/// "did you mean" suggestion. Kept O(n·m) since `n` and `m` are
/// always under ~15 chars (verb names).
fn lev(a: &str, b: &str) -> usize {
    let a_bytes: Vec<char> = a.chars().collect();
    let b_bytes: Vec<char> = b.chars().collect();
    let n = a_bytes.len();
    let m = b_bytes.len();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = usize::from(a_bytes[i - 1] != b_bytes[j - 1]);
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

/// Build the multi-line card shown in chat history when a Super Dev gate
/// pauses the pipeline. Lists exactly which artifacts are waiting for the
/// user's eyes and which slash commands move it forward — so the user
/// doesn't have to remember what `docs_confirm` vs `preview_confirm`
/// actually means.
fn gate_card(gate: Gate, slug: &str, project_root: &std::path::Path) -> String {
    let slug = if slug.is_empty() { "<slug>" } else { slug };
    let (title, artifacts, next) = match gate {
        Gate::DocsConfirm => (
            "docs_confirm — three core docs are ready for review",
            vec![
                format!("output/{slug}-prd.md"),
                format!("output/{slug}-architecture.md"),
                format!("output/{slug}-uiux.md"),
            ],
            "审核三份核心文档,确认后再进入 spec / frontend。",
        ),
        Gate::PreviewConfirm => (
            "preview_confirm — frontend preview is ready for review",
            vec![
                format!("output/{slug}-frontend-notes.md"),
                format!("output/{slug}-execution-plan.md"),
            ],
            "审核前端可运行预览,确认后再进入 backend / quality / delivery。",
        ),
    };

    let mut out = String::new();
    out.push_str(&format!("⏸ {title}\n"));
    out.push_str("  工件摘要:\n");
    for a in &artifacts {
        let path = project_root.join(a);
        let lines = std::fs::read_to_string(&path).map_or(0, |s| s.lines().count());
        let detail = if lines > 0 {
            format!("{lines} lines")
        } else {
            "missing".to_string()
        };
        out.push_str(&format!("    · {a} ({detail})\n"));
    }
    // Quick quality indicators for docs_confirm
    if matches!(gate, Gate::DocsConfirm) {
        let uiux_path = project_root.join(format!("output/{slug}-uiux.md"));
        if let Ok(content) = std::fs::read_to_string(&uiux_path) {
            let tokens = content.matches("--").count();
            let has_dark = content
                .to_ascii_lowercase()
                .contains("prefers-color-scheme");
            out.push_str(&format!(
                "  质量: {tokens} CSS tokens · dark mode: {}\n",
                if has_dark { "✓" } else { "✗ missing" }
            ));
        }
    }
    out.push_str("  操作:\n");
    out.push_str("    · /continue 或 c        → 通过 gate\n");
    out.push_str("    · /revise <修订说明>     → worker 重做\n");
    out.push_str("    · /diff prd             → 查看 PRD\n");
    out.push_str("    · /diff architecture    → 查看架构\n");
    out.push_str("    · /diff uiux            → 查看设计系统\n");
    out.push_str(&format!("  指引: {next}"));
    out
}

fn refresh_picker_with_probes(items: &mut [PickerItem], probes: &[BackendInfo]) {
    for item in items.iter_mut() {
        if let Some(id) = item.backend_id.as_deref() {
            if let Some(p) = probes.iter().find(|p| p.id == id) {
                item.ready = p.ready;
                item.detail = p.detail.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UserConfig;

    fn fresh_app(backend: Option<&str>) -> App {
        let cfg = UserConfig {
            backend: backend.map(str::to_string),
            model: None,
            ..Default::default()
        };
        App::new(
            "demo",
            cfg,
            std::path::PathBuf::from("/tmp/sd-test-config.toml"),
            std::path::PathBuf::from("/tmp/sd-test-workspace"),
        )
    }

    #[test]
    fn no_backend_opens_picker() {
        let app = fresh_app(None);
        assert_eq!(app.mode, AppMode::Picker);
    }

    #[test]
    fn configured_backend_opens_chat_with_greeting() {
        let app = fresh_app(Some("claude-code"));
        assert_eq!(app.mode, AppMode::Chat);
        assert_eq!(app.backend_label, "claude-code");
        // Greeting is the very first message.
        let first = app.history.front().unwrap();
        assert_eq!(first.role, ChatRole::SuperDev);
        assert!(first.body.contains("claude-code"));
    }

    #[test]
    fn picker_arrow_keys_navigate() {
        let mut app = fresh_app(None);
        let last = app.picker_items.len() - 1;
        assert_eq!(app.picker_selected, 0);
        // Walk all the way down — should clamp at `last`.
        for _ in 0..(app.picker_items.len() + 2) {
            let _ = app.apply_key(KeyCode::Down);
        }
        assert_eq!(app.picker_selected, last);
        let _ = app.apply_key(KeyCode::Up);
        assert_eq!(app.picker_selected, last - 1);
    }

    #[test]
    fn picker_enter_on_offline_transitions_to_chat() {
        let mut app = fresh_app(None);
        // Offline is the LAST row and is always ready.
        let target = app.picker_items.len() - 1;
        while app.picker_selected < target {
            let _ = app.apply_key(KeyCode::Down);
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::BackendChanged);
        assert_eq!(app.mode, AppMode::Chat);
        assert_eq!(app.backend_label, "offline");
    }

    #[test]
    fn picker_enter_on_unavailable_host_stays() {
        let mut app = fresh_app(None);
        // claude-code at idx 0 starts unready until probes arrive.
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::None);
        assert_eq!(app.mode, AppMode::Picker);
        // A system message must appear explaining the refusal.
        assert!(app.history.iter().any(|m| m.role == ChatRole::System));
    }

    #[test]
    fn picker_refreshes_on_backend_probed() {
        let mut app = fresh_app(None);
        app.apply_engine(EngineEvent::BackendProbed {
            backend_id: "claude-code".into(),
            ready: true,
            detail: "claude 1.6.0".into(),
        });
        assert!(app.picker_items[0].ready);
        // Now Enter on idx 0 should succeed.
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::BackendChanged);
        assert_eq!(app.mode, AppMode::Chat);
        assert_eq!(app.backend_label, "claude-code");
    }

    #[test]
    fn chat_plain_text_submits_as_requirement() {
        let mut app = fresh_app(Some("offline"));
        for c in "build a login".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::StartRun("build a login".to_string()));
        // Input is cleared after submit.
        assert!(app.input.is_empty());
    }

    #[test]
    fn chat_empty_enter_is_noop() {
        let mut app = fresh_app(Some("offline"));
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::None);
    }

    #[test]
    fn slash_help_toggles_help_overlay() {
        let mut app = fresh_app(Some("offline"));
        for c in "/help".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::None);
        assert!(app.show_help);
    }

    #[test]
    fn slash_quit_returns_quit() {
        let mut app = fresh_app(Some("offline"));
        for c in "/quit".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::Quit);
        assert!(app.should_quit);
    }

    #[test]
    fn slash_clear_clears_history() {
        let mut app = fresh_app(Some("offline"));
        assert!(!app.history.is_empty()); // greeting present
        for c in "/clear".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let _ = app.apply_key(KeyCode::Enter);
        // After /clear: only the "history cleared." system message remains.
        assert_eq!(app.history.len(), 1);
        assert!(app.history.front().unwrap().body.contains("cleared"));
    }

    #[test]
    fn slash_claude_switches_backend_and_saves() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cfg_path = tmp.path().join("config.toml");
        let cfg = UserConfig {
            backend: Some("offline".to_string()),
            model: None,
            ..Default::default()
        };
        let mut app = App::new(
            "demo",
            cfg,
            cfg_path.clone(),
            std::path::PathBuf::from("/tmp/sd-test-workspace"),
        );
        for c in "/claude".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::BackendChanged);
        assert_eq!(app.backend_label, "claude-code");
        // Config is persisted.
        let loaded = crate::config::load_from(&cfg_path);
        assert_eq!(loaded.backend.as_deref(), Some("claude-code"));
    }

    #[test]
    fn slash_continue_with_open_gate_returns_continue() {
        let mut app = fresh_app(Some("offline"));
        app.apply_engine(EngineEvent::GateOpened {
            gate: Gate::DocsConfirm,
        });
        for c in "/continue".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::Continue(Gate::DocsConfirm));
    }

    #[test]
    fn slash_continue_without_gate_is_noop_with_hint() {
        let mut app = fresh_app(Some("offline"));
        for c in "/continue".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::None);
        assert!(app
            .history
            .iter()
            .any(|m| m.body.contains("还没启动流水线") || m.body.contains("没有打开的 gate")));
    }

    #[test]
    fn slash_revise_at_gate_returns_revise_with_text() {
        let mut app = fresh_app(Some("offline"));
        app.apply_engine(EngineEvent::GateOpened {
            gate: Gate::DocsConfirm,
        });
        for c in "/revise 把 OAuth 删掉".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::Revise("把 OAuth 删掉".to_string()));
    }

    #[test]
    fn slash_revise_without_args_is_noop_with_usage_hint() {
        let mut app = fresh_app(Some("offline"));
        app.apply_engine(EngineEvent::GateOpened {
            gate: Gate::DocsConfirm,
        });
        for c in "/revise".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::None);
        assert!(app.history.iter().any(|m| m.body.contains("/revise")));
    }

    #[test]
    fn slash_unknown_command_hints() {
        let mut app = fresh_app(Some("offline"));
        for c in "/foo".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let _ = app.apply_key(KeyCode::Enter);
        assert!(app
            .history
            .iter()
            .any(|m| m.body.contains("未知命令") && m.body.contains("/foo")));
    }

    #[test]
    fn plain_text_at_open_gate_routes_to_revise() {
        let mut app = fresh_app(Some("offline"));
        app.apply_engine(EngineEvent::GateOpened {
            gate: Gate::DocsConfirm,
        });
        for c in "去掉 OAuth".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::Revise("去掉 OAuth".to_string()));
    }

    #[test]
    fn plain_text_after_delivery_starts_new_run() {
        let mut app = fresh_app(Some("offline"));
        app.run_started = true;
        app.apply_engine(EngineEvent::BlockCompleted {
            final_phase: Phase::Delivery,
            paused_at: None,
        });
        assert!(app.finished);
        for c in "make another tool".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::StartRun("make another tool".to_string()));
        // Phases were reset.
        assert!(app.phases.iter().all(|r| r.status == PhaseStatus::Pending));
        assert!(!app.finished);
    }

    #[test]
    fn host_output_lands_in_history_as_host_role() {
        let mut app = fresh_app(Some("offline"));
        app.apply_engine(EngineEvent::HostOutput {
            phase: Phase::Research,
            line: "## Similar products".into(),
        });
        let last = app.history.back().unwrap();
        assert_eq!(last.role, ChatRole::Host);
        assert!(last.body.contains("Similar products"));
    }

    #[test]
    fn history_is_bounded() {
        let mut app = fresh_app(Some("offline"));
        for i in 0..(HISTORY_CAP + 50) {
            app.apply_engine(EngineEvent::Note(format!("line {i}")));
        }
        assert!(app.history.len() <= HISTORY_CAP);
    }

    #[test]
    fn f1_toggles_help_in_both_modes() {
        let mut a = fresh_app(None);
        assert!(!a.show_help);
        let _ = a.apply_key(KeyCode::F(1));
        assert!(a.show_help);
        let mut b = fresh_app(Some("offline"));
        let _ = b.apply_key(KeyCode::F(1));
        assert!(b.show_help);
    }

    #[test]
    fn slash_spec_opens_overlay() {
        let mut a = fresh_app(Some("offline"));
        for c in "/spec".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        let ov = a.overlay.as_ref().expect("overlay should open");
        assert!(ov.title.contains("SUPER_DEV_HOST_SPEC_V1"));
        assert!(ov
            .lines
            .iter()
            .any(|l| l.contains("SUPER_DEV_HOST_SPEC_V1")));
    }

    #[test]
    fn slash_doctor_opens_overlay_with_binary_line() {
        let mut a = fresh_app(Some("offline"));
        for c in "/doctor".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        let ov = a.overlay.as_ref().expect("doctor overlay");
        assert!(ov.lines.iter().any(|l| l.starts_with("binary")));
        assert!(ov.lines.iter().any(|l| l.starts_with("workspace")));
    }

    #[test]
    fn slash_verify_opens_overlay_with_sections() {
        let mut a = fresh_app(Some("offline"));
        for c in "/verify".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        let ov = a.overlay.as_ref().unwrap();
        let joined = ov.lines.join("\n");
        assert!(joined.contains("## Spec manifest"));
        assert!(joined.contains("## Workflow state"));
        assert!(joined.contains("## Artifacts"));
    }

    #[test]
    fn slash_diff_missing_artifact_shows_available_list() {
        let mut a = fresh_app(Some("offline"));
        for c in "/diff".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        let ov = a.overlay.as_ref().unwrap();
        // Empty workspace → fallback message kicks in.
        assert!(ov
            .lines
            .iter()
            .any(|l| l.contains("找不到") || l.contains("还不存在")));
    }

    #[test]
    fn slash_init_writes_super_dev_yaml() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cfg = UserConfig {
            backend: Some("offline".into()),
            model: None,
            ..Default::default()
        };
        let mut app = App::new(
            "demo",
            cfg,
            std::path::PathBuf::from("/tmp/sd-test-config.toml"),
            tmp.path().to_path_buf(),
        );
        for c in "/init".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let action = app.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::None);
        // Manifest must exist on disk after the slash command.
        assert!(tmp.path().join("super-dev.yaml").is_file());
        // Confirmation message landed in the chat.
        assert!(app
            .history
            .iter()
            .any(|m| m.role == ChatRole::SuperDev && m.body.contains("super-dev.yaml")));
    }

    #[test]
    fn esc_during_active_pipeline_asks_for_confirmation() {
        let mut a = fresh_app(Some("offline"));
        a.apply_engine(EngineEvent::PipelineStarted {
            slug: "demo".into(),
            requirement: "build".into(),
        });
        assert!(a.is_pipeline_active());
        // First Esc → confirmation request, not quit.
        let action = a.apply_key(KeyCode::Esc);
        assert_eq!(action, Action::None);
        assert!(a.pending_quit_confirm);
        assert!(!a.should_quit);
        // Second Esc → actually quit.
        let action = a.apply_key(KeyCode::Esc);
        assert_eq!(action, Action::Quit);
        assert!(a.should_quit);
    }

    #[test]
    fn typing_clears_pending_quit_confirm() {
        let mut a = fresh_app(Some("offline"));
        a.apply_engine(EngineEvent::PipelineStarted {
            slug: "demo".into(),
            requirement: "build".into(),
        });
        let _ = a.apply_key(KeyCode::Esc);
        assert!(a.pending_quit_confirm);
        // Any typing — even one char — clears the pending confirmation.
        let _ = a.apply_key(KeyCode::Char('x'));
        assert!(!a.pending_quit_confirm);
    }

    // ---- resume hint on chat init ----

    #[test]
    fn resume_hint_appears_when_workflow_state_paused_at_gate() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Seed a workflow-state.json that looks like "paused at docs_confirm".
        let state_dir = tmp.path().join(".super-dev");
        std::fs::create_dir_all(&state_dir).unwrap();
        let state_json = r#"{
            "phase": "docs_confirm",
            "active_gate": "docs_confirm",
            "slug": "demo",
            "requirement": "做一个登录系统",
            "last_transition_at": "2026-05-23T10:00:00Z",
            "note": "",
            "spec_version": "SUPER_DEV_HOST_SPEC_V1"
        }"#;
        std::fs::write(state_dir.join("workflow-state.json"), state_json).unwrap();

        let cfg = UserConfig {
            backend: Some("offline".into()),
            model: None,
            ..Default::default()
        };
        let app = App::new(
            "demo",
            cfg,
            std::path::PathBuf::from("/tmp/sd-test-config.toml"),
            tmp.path().to_path_buf(),
        );

        // Greeting + resume hint both land in history.
        let resume_msg = app
            .history
            .iter()
            .find(|m| m.body.contains("docs_confirm"))
            .expect("resume hint should mention the paused gate");
        assert_eq!(resume_msg.role, ChatRole::System);
        assert!(resume_msg.body.contains("做一个登录系统"));
        assert!(resume_msg.body.contains("/continue"));
    }

    #[test]
    fn resume_hint_marks_completed_runs() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state_dir = tmp.path().join(".super-dev");
        std::fs::create_dir_all(&state_dir).unwrap();
        let state_json = r#"{
            "phase": "delivery",
            "active_gate": "",
            "slug": "demo",
            "requirement": "做个 todo",
            "last_transition_at": "2026-05-23T10:00:00Z",
            "note": "Pipeline complete.",
            "spec_version": "SUPER_DEV_HOST_SPEC_V1"
        }"#;
        std::fs::write(state_dir.join("workflow-state.json"), state_json).unwrap();

        let cfg = UserConfig {
            backend: Some("offline".into()),
            model: None,
            ..Default::default()
        };
        let app = App::new(
            "demo",
            cfg,
            std::path::PathBuf::from("/tmp/sd-test-config.toml"),
            tmp.path().to_path_buf(),
        );
        let msg = app
            .history
            .iter()
            .find(|m| m.body.contains("上次跑完了") || m.body.contains("上次会话"))
            .expect("delivery-state should produce a chat hint");
        assert!(msg.body.contains("做个 todo"));
    }

    #[test]
    fn no_resume_hint_for_clean_workspace() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cfg = UserConfig {
            backend: Some("offline".into()),
            model: None,
            ..Default::default()
        };
        let app = App::new(
            "demo",
            cfg,
            std::path::PathBuf::from("/tmp/sd-test-config.toml"),
            tmp.path().to_path_buf(),
        );
        // Greeting still present (always), but no resume hint.
        assert!(!app
            .history
            .iter()
            .any(|m| m.body.contains("docs_confirm") || m.body.contains("上次")));
    }

    // ---- /model + /version + /changelog + typo did-you-mean ----

    #[test]
    fn slash_model_without_arg_prints_usage() {
        let mut a = fresh_app(Some("offline"));
        for c in "/model".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        assert!(a.history.iter().any(|m| m.body.contains("用法:/model")));
        // config.model still None.
        assert!(a.config.model.is_none());
    }

    #[test]
    fn slash_model_with_arg_saves_to_config() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cfg_path = tmp.path().join("config.toml");
        let cfg = UserConfig {
            backend: Some("offline".into()),
            model: None,
            ..Default::default()
        };
        let mut app = App::new(
            "demo",
            cfg,
            cfg_path.clone(),
            std::path::PathBuf::from("/tmp/sd-test-workspace"),
        );
        for c in "/model claude-opus-4-7".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let _ = app.apply_key(KeyCode::Enter);
        assert_eq!(app.config.model.as_deref(), Some("claude-opus-4-7"));
        // Persisted.
        let loaded = crate::config::load_from(&cfg_path);
        assert_eq!(loaded.model.as_deref(), Some("claude-opus-4-7"));
    }

    #[test]
    fn slash_version_opens_overlay_with_binary_info() {
        let mut a = fresh_app(Some("offline"));
        for c in "/version".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        let ov = a.overlay.as_ref().expect("version overlay");
        let joined = ov.lines.join("\n");
        assert!(joined.contains("super-dev"));
        assert!(joined.contains(env!("CARGO_PKG_VERSION")));
        assert!(joined.contains("SUPER_DEV_HOST_SPEC_V1"));
    }

    #[test]
    fn slash_changelog_opens_overlay_with_header() {
        let mut a = fresh_app(Some("offline"));
        for c in "/changelog".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        let ov = a.overlay.as_ref().expect("changelog overlay");
        assert!(ov.lines.iter().any(|l| l.contains("Changelog")));
    }

    #[test]
    fn did_you_mean_suggests_for_typo() {
        // "/quitz" → suggest /quit
        let suggestion = App::did_you_mean("quitz");
        assert_eq!(suggestion, Some("quit"));
    }

    #[test]
    fn did_you_mean_suggests_via_prefix() {
        // "/rev" → /revise (prefix wins)
        let suggestion = App::did_you_mean("rev");
        assert_eq!(suggestion, Some("revise"));
    }

    #[test]
    fn did_you_mean_returns_none_for_garbage() {
        assert_eq!(App::did_you_mean("xxxxxxxxxx"), None);
    }

    #[test]
    fn unknown_slash_command_includes_did_you_mean_hint() {
        let mut a = fresh_app(Some("offline"));
        for c in "/quitz".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        let last = a.history.back().unwrap();
        assert!(last.body.contains("/quitz"));
        assert!(last.body.contains("/quit"));
        assert!(last.body.contains("是想用"));
    }

    #[test]
    fn extract_json_number_pulls_score() {
        let json = r#"{"score": 95, "passed": true, "notes": "ok"}"#;
        assert_eq!(extract_json_number(json, "score"), Some(95));
        assert_eq!(extract_json_number(json, "missing"), None);
    }

    #[test]
    fn extract_json_bool_pulls_passed() {
        let json = r#"{"score": 70, "passed": false}"#;
        assert_eq!(extract_json_bool(json, "passed"), Some(false));
        assert_eq!(extract_json_bool(json, "score"), None);
    }

    #[test]
    fn verify_overlay_surfaces_quality_gate_when_present() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let out_dir = root.join("output");
        std::fs::create_dir_all(&out_dir).unwrap();
        std::fs::write(
            out_dir.join("demo-quality-gate.json"),
            r#"{"score": 88, "passed": true}"#,
        )
        .unwrap();

        let mut app = App::new(
            "demo",
            UserConfig {
                backend: Some("offline".into()),
                model: None,
                ..Default::default()
            },
            std::path::PathBuf::from("/tmp/cfg.toml"),
            root.to_path_buf(),
        );
        for c in "/verify".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let _ = app.apply_key(KeyCode::Enter);
        let ov = app.overlay.as_ref().expect("verify overlay");
        let joined = ov.lines.join("\n");
        assert!(joined.contains("Quality gate"));
        assert!(joined.contains("88/100"));
        assert!(joined.contains("PASSED"));
    }

    #[test]
    fn gate_card_lists_artifacts_and_next_steps() {
        let mut a = fresh_app(Some("offline"));
        a.apply_engine(EngineEvent::PipelineStarted {
            slug: "demo".into(),
            requirement: "x".into(),
        });
        a.apply_engine(EngineEvent::GateOpened {
            gate: Gate::DocsConfirm,
        });
        let card = a
            .history
            .iter()
            .find(|m| m.role == ChatRole::Gate)
            .expect("gate card must land in chat");
        // Lists the three core docs by slug.
        assert!(card.body.contains("output/demo-prd.md"));
        assert!(card.body.contains("output/demo-architecture.md"));
        assert!(card.body.contains("output/demo-uiux.md"));
        // Lists next-step verbs.
        assert!(card.body.contains("/continue"));
        assert!(card.body.contains("/revise"));
        assert!(card.body.contains("/diff"));
    }

    #[test]
    fn gate_card_for_preview_confirm_lists_frontend_artifacts() {
        let mut a = fresh_app(Some("offline"));
        a.apply_engine(EngineEvent::PipelineStarted {
            slug: "shop".into(),
            requirement: "x".into(),
        });
        a.apply_engine(EngineEvent::GateOpened {
            gate: Gate::PreviewConfirm,
        });
        let card = a
            .history
            .iter()
            .find(|m| m.role == ChatRole::Gate)
            .expect("gate card must land in chat");
        assert!(card.body.contains("output/shop-frontend-notes.md"));
        assert!(card.body.contains("output/shop-execution-plan.md"));
    }

    #[test]
    fn bare_c_at_gate_is_treated_as_continue_shortcut() {
        let mut a = fresh_app(Some("offline"));
        a.apply_engine(EngineEvent::GateOpened {
            gate: Gate::DocsConfirm,
        });
        let _ = a.apply_key(KeyCode::Char('c'));
        let action = a.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::Continue(Gate::DocsConfirm));
        assert!(a.active_gate.is_none());
    }

    #[test]
    fn bare_c_without_gate_starts_a_run() {
        let mut a = fresh_app(Some("offline"));
        let _ = a.apply_key(KeyCode::Char('c'));
        let action = a.apply_key(KeyCode::Enter);
        // Outside a gate, "c" is a normal requirement (not magic).
        assert_eq!(action, Action::StartRun("c".to_string()));
    }

    #[test]
    fn slash_continue_no_run_hint_redirects_to_typing_a_requirement() {
        let mut a = fresh_app(Some("offline"));
        for c in "/continue".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        let last = a.history.back().unwrap();
        assert!(
            last.body.contains("还没启动流水线"),
            "expected redirect hint, got: {}",
            last.body
        );
    }

    #[test]
    fn preflight_message_lands_when_starting_run() {
        let mut a = fresh_app(Some("offline"));
        for c in "build me a thing".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let action = a.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::StartRun("build me a thing".to_string()));
        // The SuperDev preflight message includes the 9-phase plan.
        assert!(a.history.iter().any(|m| m.role == ChatRole::SuperDev
            && m.body.contains("9 阶段")
            && m.body.contains("docs_confirm")
            && m.body.contains("preview_confirm")));
    }

    // ---- cursor + editing ----

    #[test]
    fn left_arrow_moves_cursor_back_one_char() {
        let mut a = fresh_app(Some("offline"));
        for c in "abc".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        assert_eq!(a.input_cursor, 3);
        let _ = a.apply_key(KeyCode::Left);
        assert_eq!(a.input_cursor, 2);
    }

    #[test]
    fn home_and_end_jump_cursor() {
        let mut a = fresh_app(Some("offline"));
        for c in "abc".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Home);
        assert_eq!(a.input_cursor, 0);
        let _ = a.apply_key(KeyCode::End);
        assert_eq!(a.input_cursor, 3);
    }

    #[test]
    fn forward_delete_removes_char_at_cursor() {
        let mut a = fresh_app(Some("offline"));
        for c in "abc".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Home);
        let _ = a.apply_key(KeyCode::Delete);
        assert_eq!(a.input, "bc");
        assert_eq!(a.input_cursor, 0);
    }

    #[test]
    fn insertion_in_middle_preserves_surrounding_chars() {
        let mut a = fresh_app(Some("offline"));
        for c in "ac".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Left);
        let _ = a.apply_key(KeyCode::Char('b'));
        assert_eq!(a.input, "abc");
        assert_eq!(a.input_cursor, 2);
    }

    #[test]
    fn backspace_respects_cjk_boundary() {
        let mut a = fresh_app(Some("offline"));
        for c in "做个".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        assert_eq!(a.input, "做个");
        // Backspace once → just one CJK char gone, no panic.
        let _ = a.apply_key(KeyCode::Backspace);
        assert_eq!(a.input, "做");
    }

    // ---- Shift+Enter multi-line ----

    #[test]
    fn shift_enter_inserts_newline_and_does_not_submit() {
        let mut a = fresh_app(Some("offline"));
        for c in "line1".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let action = a.apply_key_with_mods(KeyCode::Enter, crossterm::event::KeyModifiers::SHIFT);
        assert_eq!(action, Action::None);
        assert!(a.input.contains("line1\n"));
        // Cursor advances past the newline.
        assert!(a.input_cursor >= 6);
    }

    #[test]
    fn plain_enter_after_shift_enter_submits_full_multiline_block() {
        let mut a = fresh_app(Some("offline"));
        for c in "line1".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key_with_mods(KeyCode::Enter, crossterm::event::KeyModifiers::SHIFT);
        for c in "line2".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let action = a.apply_key(KeyCode::Enter);
        assert_eq!(action, Action::StartRun("line1\nline2".to_string()));
    }

    // ---- palette ----

    #[test]
    fn palette_matches_filter_by_prefix() {
        let mut a = fresh_app(Some("offline"));
        for c in "/cl".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let matches = a.palette_matches();
        // /claude /clear → 2 matches.
        let verbs: Vec<&str> = matches.iter().map(|(v, _)| *v).collect();
        assert!(verbs.contains(&"claude"));
        assert!(verbs.contains(&"clear"));
    }

    #[test]
    fn arrow_down_navigates_palette_when_active() {
        let mut a = fresh_app(Some("offline"));
        for c in "/c".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let before = a.palette_selected;
        let _ = a.apply_key(KeyCode::Down);
        assert_ne!(a.palette_selected, before);
    }

    #[test]
    fn tab_autocompletes_selected_palette_match() {
        let mut a = fresh_app(Some("offline"));
        for c in "/cla".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Tab);
        assert_eq!(a.input, "/claude ");
    }

    #[test]
    fn arrow_up_with_input_not_in_palette_recalls_history() {
        let mut a = fresh_app(Some("offline"));
        // Submit a prompt to populate history.
        for c in "first request".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        // After submit, input is empty. ↑ should recall it.
        assert!(a.input.is_empty());
        let _ = a.apply_key(KeyCode::Up);
        assert_eq!(a.input, "first request");
    }

    #[test]
    fn arrow_down_at_newest_history_returns_to_fresh_draft() {
        let mut a = fresh_app(Some("offline"));
        for c in "request".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        let _ = a.apply_key(KeyCode::Up);
        assert_eq!(a.input, "request");
        let _ = a.apply_key(KeyCode::Down);
        assert!(a.input.is_empty());
        assert!(a.input_history_idx.is_none());
    }

    #[test]
    fn submit_dedups_consecutive_identical_recalls() {
        let mut a = fresh_app(Some("offline"));
        for c in "same".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        a.finished = true; // simulate end-of-run so the next submit starts fresh
        a.run_started = false;
        for c in "same".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        // Only one "same" remains in history (dedup).
        assert_eq!(
            a.input_history
                .iter()
                .filter(|s| s.as_str() == "same")
                .count(),
            1
        );
    }

    #[test]
    fn esc_when_no_pipeline_quits_immediately() {
        let mut a = fresh_app(Some("offline"));
        let action = a.apply_key(KeyCode::Esc);
        assert_eq!(action, Action::Quit);
    }

    #[test]
    fn slash_history_opens_overlay_with_messages() {
        let mut a = fresh_app(Some("offline"));
        for c in "/history".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        let ov = a.overlay.as_ref().unwrap();
        assert!(ov
            .lines
            .iter()
            .any(|l| l.contains("[super-dev]") || l.contains("[system]")));
    }

    #[test]
    fn overlay_esc_closes() {
        let mut a = fresh_app(Some("offline"));
        for c in "/spec".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        assert!(a.overlay.is_some());
        // Esc should close, NOT quit, when an overlay is open.
        let action = a.apply_key(KeyCode::Esc);
        assert_eq!(action, Action::None);
        assert!(a.overlay.is_none());
        assert!(!a.should_quit);
    }

    #[test]
    fn overlay_scroll_keys() {
        let mut a = fresh_app(Some("offline"));
        for c in "/spec".chars() {
            let _ = a.apply_key(KeyCode::Char(c));
        }
        let _ = a.apply_key(KeyCode::Enter);
        let initial = a.overlay.as_ref().unwrap().scroll;
        // Down + PageDown advance.
        let _ = a.apply_key(KeyCode::Down);
        assert!(a.overlay.as_ref().unwrap().scroll > initial);
        let after_j = a.overlay.as_ref().unwrap().scroll;
        let _ = a.apply_key(KeyCode::PageDown);
        assert!(a.overlay.as_ref().unwrap().scroll > after_j);
        // Up rewinds.
        let _ = a.apply_key(KeyCode::Up);
        // Home resets to 0.
        let _ = a.apply_key(KeyCode::Home);
        assert_eq!(a.overlay.as_ref().unwrap().scroll, 0);
    }

    #[test]
    fn host_output_groups_into_single_bubble() {
        let mut a = fresh_app(Some("offline"));
        let before = a.history.len();
        a.apply_engine(EngineEvent::HostOutput {
            phase: Phase::Research,
            line: "# header".into(),
        });
        a.apply_engine(EngineEvent::HostOutput {
            phase: Phase::Research,
            line: "## section".into(),
        });
        a.apply_engine(EngineEvent::HostOutput {
            phase: Phase::Research,
            line: "body line".into(),
        });
        // All three lines collapse into one Host message.
        let host_msgs: Vec<_> = a
            .history
            .iter()
            .skip(before)
            .filter(|m| m.role == ChatRole::Host)
            .collect();
        assert_eq!(host_msgs.len(), 1);
        let body = &host_msgs[0].body;
        assert!(body.contains("# header"));
        assert!(body.contains("## section"));
        assert!(body.contains("body line"));
    }

    #[test]
    fn host_output_starts_new_bubble_after_super_dev_break() {
        let mut a = fresh_app(Some("offline"));
        a.apply_engine(EngineEvent::HostOutput {
            phase: Phase::Research,
            line: "research line".into(),
        });
        // A SuperDev message between the two streams must break the group.
        a.apply_engine(EngineEvent::PhaseCompleted {
            phase: Phase::Research,
        });
        a.apply_engine(EngineEvent::HostOutput {
            phase: Phase::Docs,
            line: "docs line".into(),
        });
        let host_msgs: Vec<_> = a
            .history
            .iter()
            .filter(|m| m.role == ChatRole::Host)
            .collect();
        assert_eq!(host_msgs.len(), 2);
    }

    #[test]
    fn status_bar_contains_phase_dots() {
        let mut a = fresh_app(Some("offline"));
        a.apply_engine(EngineEvent::PhaseStarted {
            phase: Phase::Research,
        });
        // Phase dots are a contiguous 9-glyph string after the "● {backend} · " prefix.
        // With research running it's ◐○○○○○○○○.
        assert!(a.status.contains("◐○○○○○○○○"));
    }

    #[test]
    fn status_bar_dots_advance_as_phases_complete() {
        let mut a = fresh_app(Some("offline"));
        a.apply_engine(EngineEvent::PhaseStarted {
            phase: Phase::Research,
        });
        a.apply_engine(EngineEvent::PhaseCompleted {
            phase: Phase::Research,
        });
        a.apply_engine(EngineEvent::PhaseStarted { phase: Phase::Docs });
        // After research done + docs running: ●◐○○○○○○○
        assert!(a.status.contains("●◐○○○○○○○"));
    }

    #[test]
    fn spinner_cycles() {
        let mut a = fresh_app(Some("offline"));
        let first = a.spinner();
        for _ in 0..8 {
            a.tick();
        }
        assert_eq!(a.spinner(), first);
    }
}
