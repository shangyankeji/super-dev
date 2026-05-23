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
        if app.mode == AppMode::Chat {
            app.push_greeting();
        }
        app.refresh_status();
        app
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
        self.push(
            ChatRole::SuperDev,
            format!(
                "Super Dev · AI 编码的项目经理 · 工人:{} · 直接告诉我做什么。",
                self.backend_label,
            ),
        );
        // Three concrete starter prompts so a first-time user knows what
        // "tell me what to do" looks like in practice.
        self.push(
            ChatRole::System,
            "试试这种问法:\n  \
             · 做一个登录系统,支持邮箱+OAuth+MFA\n  \
             · 帮我做一个简单的 todo list,前端 React,后端 Rust\n  \
             · 给现有项目加二步验证(TOTP + 备用码)\n\n\
             斜杠命令:/claude /codex /offline 切工人 · /help 看全部 · /quit 退出",
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
        self.status = format!(
            "● {} · {}{}{}{}",
            self.backend_label, dots, running, gate_label, done_label
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
    /// double-pollute the ↑↓ recall).
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
        ("claude", "switch worker to Claude Code"),
        ("codex", "switch worker to Codex"),
        ("offline", "switch worker to offline templates"),
        ("init", "write super-dev.yaml manifest"),
        ("continue", "approve the active gate"),
        ("revise", "stay at gate, request changes"),
        ("spec", "show the SUPER_DEV_HOST_SPEC_V1 spec"),
        ("verify", "show workspace conformance"),
        ("doctor", "self-test"),
        ("diff", "show an artifact (default: PRD)"),
        ("history", "show the conversation history"),
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
    pub fn autocomplete_palette(&mut self) {
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
                    format!(
                        "⏸ 暂停在 gate `{}`。\n  /continue 或 c 通过\n  /revise <说明> 提修订\n  /diff <工件> 看产出",
                        gate.id_str()
                    ),
                );
            }
            EngineEvent::BlockCompleted {
                final_phase,
                paused_at,
            } => {
                if paused_at.is_none() && final_phase == Phase::Delivery {
                    self.finished = true;
                    self.active_gate = None;
                    self.push(
                        ChatRole::SuperDev,
                        "✓ 流水线完成。proof-pack 已在 release/ 目录。",
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
    /// active) or a revision (if a gate is open).
    fn submit_text(&mut self, text: String) -> Action {
        self.push(ChatRole::You, text.clone());
        if self.active_gate.is_some() {
            self.push(
                ChatRole::SuperDev,
                format!("收到修订:\"{text}\"。重新跑当前 block…"),
            );
            Action::Revise(text)
        } else if self.run_started && self.finished {
            // After delivery, treat further text as a fresh kickoff.
            self.push(
                ChatRole::SuperDev,
                format!("收到新需求:\"{text}\"。流水线重新开始…"),
            );
            self.reset_for_new_run();
            Action::StartRun(text)
        } else if !self.run_started {
            self.push(
                ChatRole::SuperDev,
                format!("收到需求:\"{text}\"。流水线启动中…"),
            );
            Action::StartRun(text)
        } else {
            self.push(
                ChatRole::System,
                "目前正在跑流水线 — 等下一个 gate 暂停后再发指令,或用 /quit 退出。",
            );
            Action::None
        }
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
            "offline" => self.slash_backend(None),
            "init" => {
                // Write the SD-META-001 manifest directly from the TUI.
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
                    Ok(path) => self.push(
                        ChatRole::SuperDev,
                        format!(
                            "✓ super-dev.yaml 写入 {}\n  level={} profile={} slug={slug}",
                            path.display(),
                            manifest.level.as_str(),
                            manifest.profile.as_str(),
                        ),
                    ),
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => self.push(
                        ChatRole::System,
                        "super-dev.yaml 已存在且与模板不同——保留你的编辑。需要重写请加 /init --force(待支持)。",
                    ),
                    Err(e) => self.push(
                        ChatRole::System,
                        format!("/init 失败:{e}"),
                    ),
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
                    self.push(ChatRole::System, "目前没有打开的 gate。");
                    Action::None
                }
            }
            "revise" => {
                if rest.is_empty() {
                    self.push(ChatRole::System, "用法:/revise <修订说明>");
                    Action::None
                } else if self.active_gate.is_some() {
                    self.push(
                        ChatRole::SuperDev,
                        format!("收到修订:\"{rest}\"。重新跑当前 block…"),
                    );
                    Action::Revise(rest.to_string())
                } else {
                    self.push(ChatRole::System, "目前没有打开的 gate,无法修订。");
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
            _ => {
                self.push(
                    ChatRole::System,
                    format!("未知命令 `/{verb}` — 输入 /help 看列表。"),
                );
                Action::None
            }
        };
        Some(action)
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
            "super-dev doctor\n\
             ================\n\n",
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
        self.overlay = Some(Overlay::from_body(" doctor — press Esc to close ", &body));
    }

    fn open_verify_overlay(&mut self) {
        let mut body = String::from(
            "super-dev verify\n\
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
                "  phase={} active_gate={} slug={}\n  requirement={}\n",
                s.phase,
                if s.active_gate.is_empty() {
                    "<none>"
                } else {
                    &s.active_gate
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
    vec![
        PickerItem {
            backend_id: Some("claude-code".to_string()),
            label: "claude-code".to_string(),
            ready: false,
            detail: "detecting…".to_string(),
        },
        PickerItem {
            backend_id: Some("codex".to_string()),
            label: "codex".to_string(),
            ready: false,
            detail: "detecting…".to_string(),
        },
        PickerItem {
            backend_id: None,
            label: "offline".to_string(),
            ready: true,
            detail: "deterministic templates (no AI; demos / CI)".to_string(),
        },
    ]
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
        assert_eq!(app.picker_selected, 0);
        let _ = app.apply_key(KeyCode::Down);
        assert_eq!(app.picker_selected, 1);
        let _ = app.apply_key(KeyCode::Down);
        assert_eq!(app.picker_selected, 2);
        // Off-end is clamped.
        let _ = app.apply_key(KeyCode::Down);
        assert_eq!(app.picker_selected, 2);
        let _ = app.apply_key(KeyCode::Up);
        assert_eq!(app.picker_selected, 1);
    }

    #[test]
    fn picker_enter_on_offline_transitions_to_chat() {
        let mut app = fresh_app(None);
        // Offline is the 3rd item (idx 2) and is always ready.
        let _ = app.apply_key(KeyCode::Down);
        let _ = app.apply_key(KeyCode::Down);
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
            .any(|m| m.body.contains("没有打开的 gate")));
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
