//! Ratatui rendering — pure function of [`App`] state.
//!
//! Two screens, dispatched on [`AppMode`]:
//!
//! - **Picker** — first-launch backend chooser.
//! - **Chat** — persistent input box + scrolling conversation history,
//!   modelled after Claude Code's REPL feel.

use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, AppMode, ChatRole};

/// Draw one full frame — dispatches on the current screen.
pub fn render(frame: &mut Frame, app: &App) {
    match app.mode {
        AppMode::Picker => render_picker(frame, app),
        AppMode::Chat => render_chat(frame, app),
    }
    // Overlay precedence: scrollable content overlay wins over help.
    if let Some(ov) = &app.overlay {
        render_scroll_overlay(frame, ov);
    } else if app.show_help {
        render_help_overlay(frame, app);
    }
}

/// Render a scrollable, near-fullscreen overlay used by `/spec`,
/// `/verify`, `/doctor`, `/diff`, `/history`.
fn render_scroll_overlay(frame: &mut Frame, ov: &crate::app::Overlay) {
    let area = centered_rect(frame.area(), 88, 88);
    frame.render_widget(Clear, area);

    let inner_height = area.height.saturating_sub(3) as usize;
    let total = ov.lines.len();
    let from = ov.scroll;
    let to = (from + inner_height).min(total);
    let visible: Vec<Line<'static>> = ov
        .lines
        .iter()
        .skip(from)
        .take(to - from)
        .map(|l| Line::from(l.clone()))
        .collect();

    let progress = if total == 0 {
        " (empty) ".to_string()
    } else {
        let pct = if total <= inner_height {
            100
        } else {
            ((from + inner_height).min(total) * 100) / total
        };
        format!(" {}-{} of {} · {}% ", from + 1, to, total, pct)
    };
    let title_full = format!("{}{progress}", ov.title);

    let body = Paragraph::new(visible)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title_full)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(body, area);
}

// ---------- Picker (first launch) -----------------------------------------

fn render_picker(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // banner
            Constraint::Length(2 + u16::try_from(app.picker_items.len()).unwrap_or(3)), // list
            Constraint::Min(2),    // help text
            Constraint::Length(3), // footer
        ])
        .split(frame.area());

    // Banner
    let banner = vec![
        Line::from(vec![
            Span::styled(
                format!(" Super Dev {} ", env!("CARGO_PKG_VERSION")),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "AI 编码的项目经理 · first launch",
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(Span::styled(
            "Pick the worker you want me to drive.",
            Style::default().fg(Color::White),
        )),
    ];
    frame.render_widget(
        Paragraph::new(banner).block(Block::default().borders(Borders::ALL)),
        chunks[0],
    );

    // Backend list
    let items: Vec<ListItem> = app
        .picker_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let arrow = if idx == app.picker_selected {
                "›"
            } else {
                " "
            };
            let (mark, mark_style) = if item.ready {
                ("ready", Style::default().fg(Color::Green))
            } else {
                ("unavailable", Style::default().fg(Color::Red))
            };
            let row_style = if idx == app.picker_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {arrow} "), row_style),
                Span::styled(format!("{:<13}", item.label), row_style),
                Span::styled(format!("{mark:<14}"), mark_style),
                Span::styled(item.detail.clone(), Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title(" Workers ")),
        chunks[1],
    );

    // Help text
    let help = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Your choice is saved to ~/.super-dev/config.toml.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "  Switch later inside the chat with /claude /codex /offline.",
            Style::default().fg(Color::Gray),
        )),
    ];
    frame.render_widget(Paragraph::new(help), chunks[2]);

    // Footer
    let footer = Line::from(Span::styled(
        " ↑↓ navigate · Enter confirm · F1 help · Esc quit ",
        Style::default().fg(Color::Gray),
    ));
    frame.render_widget(
        Paragraph::new(footer).block(Block::default().borders(Borders::ALL)),
        chunks[3],
    );
}

// ---------- Chat (main loop) ----------------------------------------------

fn render_chat(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header / status
            Constraint::Min(6),    // history (grows)
            Constraint::Length(5), // input
            Constraint::Length(2), // footer hint
        ])
        .split(frame.area());

    render_chat_header(frame, chunks[0], app);
    render_chat_history(frame, chunks[1], app);
    render_chat_input(frame, chunks[2], app);
    render_chat_footer(frame, chunks[3], app);

    // Slash-command palette popover floats above the input box when
    // the user is typing a `/`-prefixed command with at least one match.
    let palette = app.palette_matches();
    if !palette.is_empty() {
        render_palette_popover(frame, chunks[2], app, &palette);
    }
}

fn render_chat_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = Span::styled(
        format!(" Super Dev {} ", env!("CARGO_PKG_VERSION")),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    let slug = Span::styled(
        format!(
            "  {} ",
            if app.slug.is_empty() {
                "<workspace>"
            } else {
                &app.slug
            }
        ),
        Style::default().fg(Color::White),
    );
    let status = Span::styled(app.status.clone(), Style::default().fg(Color::Yellow));
    let line = Line::from(vec![title, slug, Span::raw(" · "), status]);
    frame.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn render_chat_history(frame: &mut Frame, area: Rect, app: &App) {
    let inner_height = area.height.saturating_sub(2) as usize;
    // Render the most recent N rendered-lines.
    let mut rendered: Vec<Line<'static>> = Vec::new();
    for msg in &app.history {
        // Gate messages get a visually distinct boxed treatment — the
        // user MUST notice them, they're the pause-and-decide moments.
        if msg.role == ChatRole::Gate {
            let yellow = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD);
            rendered.push(Line::from(Span::styled(
                "  ╔══ GATE ══════════════════════════════════════════════════════════════╗",
                yellow,
            )));
            for cont in msg.body.lines() {
                rendered.push(Line::from(vec![
                    Span::styled("  ║ ", yellow),
                    Span::raw(cont.to_string()),
                ]));
            }
            rendered.push(Line::from(Span::styled(
                "  ╚══════════════════════════════════════════════════════════════════════╝",
                yellow,
            )));
            rendered.push(Line::from(""));
            continue;
        }
        let (label, label_style) = match msg.role {
            ChatRole::You => (
                "you      ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            ChatRole::SuperDev => (
                "super-dev",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            ChatRole::Host => (
                "worker   ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            // Gate handled above with the boxed branch.
            ChatRole::Gate => unreachable!(),
            ChatRole::System => ("system   ", Style::default().fg(Color::Gray)),
        };
        let mut lines = msg.body.lines();
        // First line carries the role badge.
        if let Some(first) = lines.next() {
            rendered.push(Line::from(vec![
                Span::styled(format!(" {label}  "), label_style),
                Span::raw(first.to_string()),
            ]));
        }
        // Continuation lines indent to align under the body.
        for cont in lines {
            rendered.push(Line::from(vec![
                Span::raw("            "),
                Span::raw(cont.to_string()),
            ]));
        }
        // Blank separator between messages keeps the wall readable.
        rendered.push(Line::from(""));
    }
    let total_rendered = rendered.len();
    let take = total_rendered.min(inner_height);
    let hidden_above = total_rendered.saturating_sub(take);
    let visible: Vec<Line<'static>> = rendered
        .into_iter()
        .rev()
        .take(take)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    // Surface "N more lines scrolled off-screen" in the block title so
    // the user is never silently missing context.
    let title = if hidden_above == 0 {
        " Conversation ".to_string()
    } else {
        format!(" Conversation · ↑ {hidden_above} more above ")
    };
    let para = Paragraph::new(visible)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_chat_input(frame: &mut Frame, area: Rect, app: &App) {
    // Splice the cursor glyph at the char-position cursor so multi-line
    // input shows the caret where it actually is.
    let pre: String = app.input.chars().take(app.input_cursor).collect();
    let post: String = app.input.chars().skip(app.input_cursor).collect();
    let display = format!("> {pre}▌{post}");

    let title = if app.input.starts_with('/') {
        " /-command — ↑↓ choose · Tab autocomplete · Enter run ".to_string()
    } else if let Some(gate) = app.active_gate {
        // Gate-aware hint — surface what to do when the pipeline is
        // paused so the user doesn't have to guess.
        format!(
            " ⏸ at gate `{}` — type revision / /continue / /diff ",
            gate.id_str()
        )
    } else if app.input.contains('\n') {
        " multi-line · Enter submit · Shift+Enter newline · ↑↓ recall ".to_string()
    } else if !app.input.is_empty() {
        " Enter submit · Shift+Enter newline · ↑↓ recall history ".to_string()
    } else if app.finished {
        " ✓ pipeline complete — type a new requirement to start another, or /quit ".to_string()
    } else if app.run_started {
        " ⏳ pipeline running — wait for the next gate, or /quit to exit ".to_string()
    } else {
        " type your reply · /help for slash commands · ↑↓ recall history ".to_string()
    };
    let para = Paragraph::new(display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

/// A short popover above the input box that lists matching slash
/// commands. The highlighted row is `app.palette_selected`.
fn render_palette_popover(
    frame: &mut Frame,
    input_area: Rect,
    app: &App,
    matches: &[(&'static str, &'static str)],
) {
    // Cap how tall the popover gets so it never covers the chat.
    let rows_usize = matches.len().min(6);
    if rows_usize == 0 {
        return;
    }
    let rows = u16::try_from(rows_usize).unwrap_or(6);
    let height = rows + 2; // borders
    let width = input_area.width.min(50);
    let x = input_area.x;
    let y = input_area.y.saturating_sub(height);
    let area = Rect {
        x,
        y,
        width,
        height,
    };
    frame.render_widget(Clear, area);

    let selected = app.palette_selected.min(matches.len() - 1);
    let items: Vec<ListItem> = matches
        .iter()
        .take(rows_usize)
        .enumerate()
        .map(|(idx, (verb, hint))| {
            let arrow = if idx == selected { "›" } else { " " };
            let row_style = if idx == selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {arrow} "), row_style),
                Span::styled(format!("/{verb:<10}"), row_style),
                Span::styled((*hint).to_string(), Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" slash commands ")
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(List::new(items).block(block), area);
}

fn render_chat_footer(frame: &mut Frame, area: Rect, _app: &App) {
    let footer = Line::from(Span::styled(
        " Enter submit · /commands list · F1 help · Esc quit ",
        Style::default().fg(Color::Gray),
    ));
    frame.render_widget(Paragraph::new(footer), area);
}

// ---------- Help overlay (both modes) -------------------------------------

fn render_help_overlay(frame: &mut Frame, app: &App) {
    let area = centered_rect(frame.area(), 72, 80);
    frame.render_widget(Clear, area);

    let header = match app.mode {
        AppMode::Picker => " Help — first-launch picker ",
        AppMode::Chat => " Help — slash commands & keys ",
    };

    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new(Line::from(Span::styled(
        " Super Dev — AI 编码的项目经理",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))));
    items.push(ListItem::new(Line::from("")));

    match app.mode {
        AppMode::Picker => {
            push_help_group(
                &mut items,
                "Navigation",
                &[
                    ("↑ / ↓", "navigate the worker list"),
                    ("Enter", "confirm — saves to ~/.super-dev/config.toml"),
                    ("F1", "toggle this help"),
                    ("Esc", "quit"),
                ],
            );
        }
        AppMode::Chat => {
            push_help_group(
                &mut items,
                "Worker",
                &[
                    ("/claude", "Claude Code CLI"),
                    ("/codex", "Codex CLI"),
                    ("/gemini", "Gemini CLI"),
                    ("/droid", "Droid CLI"),
                    ("/opencode", "OpenCode CLI"),
                    ("/qwen", "Qwen Code CLI"),
                    ("/copilot", "Copilot CLI"),
                    ("/trae", "Trae CLI"),
                    ("/codebuddy", "CodeBuddy CLI"),
                    ("/qoder", "Qoder CLI"),
                    ("/kimi", "Kimi Code CLI"),
                    ("/offline", "deterministic templates"),
                    ("/model <id>", "set model id (saved to config.toml)"),
                ],
            );
            push_help_group(
                &mut items,
                "Pipeline & gates",
                &[
                    (
                        "Enter",
                        "submit; non-slash = requirement (or revision at gate)",
                    ),
                    ("/continue or c", "approve the active gate"),
                    ("/revise <txt>", "stay at gate, request changes"),
                    ("/diff [artifact]", "show an artifact (default: PRD)"),
                    ("/run [slug] <req>", "start a new run"),
                    ("/redo", "re-run current requirement"),
                    ("/init", "write super-dev.yaml"),
                ],
            );
            push_help_group(
                &mut items,
                "Design & inspect",
                &[
                    ("/design <name>", "pick a design system"),
                    ("/template <name>", "pick a seed template"),
                    ("/status", "detailed pipeline status"),
                    ("/export", "export latest proof-pack"),
                    ("/knowledge", "list knowledge + design files"),
                    ("/spec", "show spec clauses"),
                    ("/verify", "workspace conformance"),
                    ("/config", "all current configuration"),
                    ("/doctor", "self-test"),
                    ("/history", "conversation history"),
                    ("/version", "versions"),
                    ("/changelog", "CHANGELOG"),
                ],
            );
            push_help_group(
                &mut items,
                "Editing & exit",
                &[
                    (
                        "Shift+Enter",
                        "insert newline (multi-line requirement / revision)",
                    ),
                    ("↑ / ↓", "recall input history"),
                    ("Tab", "autocomplete from slash palette"),
                    ("/clear", "clear chat history (config stays)"),
                    ("/help or /?", "this overlay"),
                    ("/quit or q", "exit"),
                    ("F1", "toggle this overlay"),
                    ("Esc", "close overlay / quit"),
                ],
            );
        }
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(header)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(list, area);
}

fn push_help_group(items: &mut Vec<ListItem<'_>>, title: &str, rows: &[(&str, &str)]) {
    items.push(ListItem::new(Line::from(Span::styled(
        format!(" {title}"),
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD),
    ))));
    for (key, desc) in rows {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("  {key:<22} "),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled((*desc).to_string(), Style::default().fg(Color::Gray)),
        ])));
    }
    items.push(ListItem::new(Line::from("")));
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::config::UserConfig;
    use crossterm::event::KeyCode;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use super_dev_agent::{EngineEvent, Gate};
    use super_dev_spec::Phase;

    fn render_to_string(app: &App) -> String {
        let backend = TestBackend::new(120, 70);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, app)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect()
    }

    fn app_with(backend: Option<&str>) -> App {
        App::new(
            "demo",
            UserConfig {
                backend: backend.map(str::to_string),
                model: None,
                ..Default::default()
            },
            std::path::PathBuf::from("/tmp/sd-test-config.toml"),
            std::path::PathBuf::from("/tmp/sd-test-workspace"),
        )
    }

    // --- Picker ---

    #[test]
    fn picker_renders_all_workers() {
        let app = app_with(None);
        let out = render_to_string(&app);
        assert!(out.contains("first launch"));
        assert!(out.contains("Workers"));
        assert!(out.contains("Claude Code CLI"));
        assert!(out.contains("Codex CLI"));
        assert!(out.contains("offline"));
    }

    #[test]
    fn picker_marks_current_selection() {
        let app = app_with(None);
        let out = render_to_string(&app);
        // The cursor row carries '›'. Confirm at least one chevron rendered.
        assert!(out.contains("›"));
    }

    // --- Chat ---

    #[test]
    fn chat_shows_greeting() {
        let app = app_with(Some("offline"));
        let out = render_to_string(&app);
        assert!(out.contains("Super Dev"));
        assert!(out.contains("Conversation"));
        assert!(out.contains("AI"));
    }

    #[test]
    fn chat_input_box_shows_input_and_cursor() {
        let mut app = app_with(Some("offline"));
        for c in "hello".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let out = render_to_string(&app);
        assert!(out.contains("hello"));
        assert!(out.contains("▌"));
    }

    #[test]
    fn chat_slash_input_shows_palette_popover() {
        let mut app = app_with(Some("offline"));
        for c in "/cla".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let out = render_to_string(&app);
        // Popover lists matching commands above the input.
        assert!(out.contains("slash commands"));
        assert!(out.contains("/claude"));
        // Selection chevron is on the first match by default.
        assert!(out.contains("›"));
    }

    #[test]
    fn chat_input_box_title_changes_with_state() {
        let mut app = app_with(Some("offline"));
        // Empty buffer → "type your reply" hint.
        let empty = render_to_string(&app);
        assert!(empty.contains("type your reply"));
        // Some normal text → "Enter submit · Shift+Enter newline · ↑↓ recall".
        for c in "hello".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        let typed = render_to_string(&app);
        assert!(typed.contains("Enter submit"));
        assert!(typed.contains("Shift+Enter"));
    }

    #[test]
    fn chat_history_title_surfaces_scrolloff_count() {
        let mut app = app_with(Some("offline"));
        // Render at a small viewport so plenty of lines spill above.
        for i in 0..40 {
            app.apply_engine(super_dev_agent::EngineEvent::Note(format!(
                "scroll-content-line-{i}"
            )));
        }
        let backend = TestBackend::new(80, 18);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| crate::ui::render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let mut out = String::new();
        for y in 0..buf.area().height {
            for x in 0..buf.area().width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        assert!(out.contains("more above"), "title was: {out}");
    }

    #[test]
    fn input_title_shows_gate_hint_when_paused() {
        let mut app = app_with(Some("offline"));
        app.apply_engine(super_dev_agent::EngineEvent::GateOpened {
            gate: super_dev_agent::Gate::DocsConfirm,
        });
        let out = render_to_string(&app);
        // The input box title is gate-aware.
        assert!(out.contains("at gate"));
        assert!(out.contains("docs_confirm"));
    }

    #[test]
    fn input_title_shows_running_hint_when_pipeline_active() {
        let mut app = app_with(Some("offline"));
        app.apply_engine(super_dev_agent::EngineEvent::PipelineStarted {
            slug: "demo".into(),
            requirement: "x".into(),
        });
        let out = render_to_string(&app);
        assert!(out.contains("pipeline running"));
    }

    #[test]
    fn cursor_glyph_moves_with_arrow_keys() {
        let mut app = app_with(Some("offline"));
        for c in "abc".chars() {
            let _ = app.apply_key(KeyCode::Char(c));
        }
        // Cursor sits at end → render shows "abc▌"
        let end_view = render_to_string(&app);
        assert!(end_view.contains("abc▌"));
        // Left arrow moves cursor between b and c.
        let _ = app.apply_key(KeyCode::Left);
        let mid_view = render_to_string(&app);
        assert!(mid_view.contains("ab▌c"));
    }

    #[test]
    fn chat_renders_gate_message_with_gate_role() {
        let mut app = app_with(Some("offline"));
        app.apply_engine(EngineEvent::GateOpened {
            gate: Gate::DocsConfirm,
        });
        let out = render_to_string(&app);
        assert!(out.contains("gate"));
        assert!(out.contains("docs_confirm"));
    }

    #[test]
    fn chat_renders_host_output_with_worker_label() {
        let mut app = app_with(Some("offline"));
        app.apply_engine(EngineEvent::HostOutput {
            phase: Phase::Research,
            line: "## Similar products".into(),
        });
        let out = render_to_string(&app);
        assert!(out.contains("worker"));
        assert!(out.contains("Similar products"));
    }

    // --- Help overlay ---

    #[test]
    fn help_overlay_in_picker_lists_navigation_keys() {
        let mut app = app_with(None);
        let _ = app.apply_key(KeyCode::F(1));
        let out = render_to_string(&app);
        assert!(out.contains("first-launch picker"));
        assert!(out.contains("navigate the worker list"));
    }

    #[test]
    fn help_overlay_in_chat_lists_slash_commands() {
        let mut app = app_with(Some("offline"));
        let _ = app.apply_key(KeyCode::F(1));
        let out = render_to_string(&app);
        assert!(out.contains("slash commands"));
        assert!(out.contains("/claude"));
        assert!(out.contains("/quit"));
    }

    #[test]
    fn help_overlay_in_chat_uses_grouped_sections() {
        let mut app = app_with(Some("offline"));
        let _ = app.apply_key(KeyCode::F(1));
        // Render at a taller terminal so the 4-group overlay isn't cropped.
        let backend = TestBackend::new(120, 60);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| crate::ui::render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let mut out = String::new();
        for y in 0..buf.area().height {
            for x in 0..buf.area().width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        // Each group header appears, and verbs are sorted under them.
        assert!(out.contains("Worker"));
        assert!(out.contains("Pipeline & gates"));
        assert!(out.contains("Design & inspect"));
        assert!(out.contains("Editing & exit"));
        // 4.4 surfaces live in the right groups.
        assert!(out.contains("/model"));
        assert!(out.contains("/version"));
        assert!(out.contains("/changelog"));
        assert!(out.contains("Shift+Enter"));
    }
}
