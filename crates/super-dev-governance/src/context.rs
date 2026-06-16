//! Session-continuity context injection.
//!
//! Implements `SD-FLOW-006`. On every user prompt the host calls
//! [`compose_session_context`] and prepends the returned text to the
//! model's context. This keeps the model anchored to the active Super
//! Dev phase without the user repeating themselves.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Default char budget for the injected block. The effective cap is
/// [`budget_for_phase`]-adjusted: research/docs (direction-setting) get a
/// larger budget than delivery (where artifacts carry the context).
const MAX_CHARS: usize = 3000;

/// Default number of `SESSION_BRIEF.md` head lines included verbatim.
/// Scaled per-phase via [`brief_lines_for_phase`] so research/docs (which
/// lean on the brief for direction) see more of it than delivery.
const BRIEF_HEAD_LINES: usize = 40;

/// Per-phase SESSION_BRIEF head-line budget. Scales with the phase: the
/// research/docs phases get a larger window (the brief carries direction),
/// delivery gets less. Kept proportional to [`budget_for_phase`]'s intent.
fn brief_lines_for_phase(phase: &str) -> usize {
    if let Ok(v) = std::env::var("SUPER_DEV_CONTEXT_BRIEF_LINES") {
        if let Ok(n) = v.parse::<usize>() {
            if n > 0 {
                return n;
            }
        }
    }
    match phase {
        "research" | "docs" => BRIEF_HEAD_LINES * 2, // 80
        "spec" | "frontend" | "backend" => BRIEF_HEAD_LINES + 10, // 50
        _ => BRIEF_HEAD_LINES,                       // 40
    }
}

/// Per-phase context budget. Research/docs get more room (direction-setting);
/// build phases get a medium budget; gates/delivery get the default. Override
/// everything via `SUPER_DEV_CONTEXT_MAX_CHARS` (>0 = fixed cap).
fn budget_for_phase(phase: &str) -> usize {
    if let Ok(v) = std::env::var("SUPER_DEV_CONTEXT_MAX_CHARS") {
        if let Ok(n) = v.parse::<usize>() {
            if n > 0 {
                return n;
            }
        }
    }
    match phase {
        "research" | "docs" => MAX_CHARS * 2,
        "spec" | "frontend" | "backend" => MAX_CHARS + 1000,
        _ => MAX_CHARS,
    }
}

/// The composed prompt-time injection block.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionContext {
    /// Block text the host prepends to the next user prompt. Empty
    /// when there is no active Super Dev state in the workspace.
    pub text: String,
}

impl SessionContext {
    /// `true` when there is nothing to inject (e.g. fresh workspace).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

fn read_workflow_state(root: &Path) -> Option<Value> {
    let p = root.join(".super-dev").join("workflow-state.json");
    let text = fs::read_to_string(&p).ok()?;
    serde_json::from_str::<Value>(&text).ok()
}

fn read_session_brief_head(root: &Path, line_budget: usize) -> String {
    let p = root.join(".super-dev").join("SESSION_BRIEF.md");
    let Ok(text) = fs::read_to_string(&p) else {
        return String::new();
    };
    text.lines()
        .take(line_budget)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn read_knowledge_digest(root: &Path) -> String {
    let dir = root.join("output").join("knowledge-cache");
    let Ok(entries) = fs::read_dir(&dir) else {
        return String::new();
    };
    // Pick the most-recently-WRITTEN bundle by mtime (falling back to the
    // lexicographically-greatest name on a tie). Previously this sorted by
    // Path lexicographic order and took .last(), so a bundle named
    // `aaa-...json` written LATER than `zzz-...json` was ignored — the
    // digest reflected a stale bundle.
    let mut bundles: Vec<(std::time::SystemTime, std::cmp::Reverse<String>, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            let p = e.path();
            let is_bundle = p.extension().and_then(|s| s.to_str()) == Some("json")
                && p.file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|n| n.ends_with("-knowledge-bundle.json"));
            if !is_bundle {
                return None;
            }
            // mtime; fall back to UNIX_EPOCH on error so the file still sorts.
            let mtime = e
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            Some((mtime, std::cmp::Reverse(name), p))
        })
        .collect();
    // Sort newest-first (mtime descending, tiebreak by name descending).
    bundles.sort_by(|a, b| b.cmp(a));
    // Walk newest→oldest and use the first bundle that reads + parses cleanly.
    // This tolerates a concurrent-write race where the newest bundle is
    // momentarily a partial file (the knowledge phase writing it while we
    // read): instead of silently returning empty, we fall back to the
    // previous (complete) bundle.
    let (latest, val) = {
        let mut parsed: Option<(PathBuf, Value)> = None;
        for (_, _, path) in &bundles {
            if let Ok(text) = fs::read_to_string(path) {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    parsed = Some((path.clone(), v));
                    break;
                }
            }
        }
        match parsed {
            Some((p, v)) => (p, v),
            None => return String::new(),
        }
    };
    let summary = val
        .get("research_summary")
        .or_else(|| val.get("summary"))
        .cloned()
        .unwrap_or(Value::Null);
    let summary_text = match summary {
        Value::String(s) => s,
        Value::Array(arr) => arr
            .iter()
            .take(5)
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    };
    let trimmed = summary_text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let name = latest
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("knowledge-bundle.json");
    let snippet: String = trimmed.chars().take(800).collect();
    format!("Knowledge bundle ({name}):\n{snippet}")
}

/// Build the injection block. Reads `.super-dev/workflow-state.json`,
/// `.super-dev/SESSION_BRIEF.md`, and the latest knowledge bundle.
/// Returns `SessionContext { text: "" }` when there is nothing to
/// inject.
#[must_use]
pub fn compose_session_context(project_root: &Path) -> SessionContext {
    let state = read_workflow_state(project_root);
    let mut brief = read_session_brief_head(project_root, BRIEF_HEAD_LINES); // default; refined per-phase below
    let knowledge = read_knowledge_digest(project_root);

    if state.is_none() && brief.is_empty() && knowledge.is_empty() {
        return SessionContext {
            text: String::new(),
        };
    }

    let mut parts_phase = String::from("unknown");
    let mut parts: Vec<String> = vec!["[Super Dev ambient context]".to_string()];
    if let Some(state_val) = state {
        let phase = state_val
            .get("phase")
            .or_else(|| state_val.get("current_phase"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        parts_phase = phase.to_string();
        // Re-read the brief with the phase-aware line budget now that we
        // know the phase (scales how much of SESSION_BRIEF.md we include).
        brief = read_session_brief_head(project_root, brief_lines_for_phase(&parts_phase));
        let gate = state_val
            .get("active_gate")
            .or_else(|| state_val.get("gate"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if gate.is_empty() {
            parts.push(format!("Active phase: {phase}"));
        } else {
            parts.push(format!("Active phase: {phase} | gate: {gate}"));
        }
    }
    if !brief.is_empty() {
        parts.push("Session brief (head):".to_string());
        parts.push(brief);
    }
    if !knowledge.is_empty() {
        parts.push(knowledge);
    }
    parts.push(
        "Reminder: stay inside the current Super Dev gate; do not exit the \
         pipeline implicitly. Reply 确认 / 通过 / 继续 / 修改 keeps you in stage."
            .to_string(),
    );

    let mut text = parts.join("\n\n");
    let cap = budget_for_phase(&parts_phase);
    if text.chars().count() > cap {
        let mut buf = String::with_capacity(cap);
        for ch in text.chars().take(cap.saturating_sub(3)) {
            buf.push(ch);
        }
        buf.push_str("...");
        text = buf;
    }
    SessionContext { text }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn empty_when_no_state() {
        let tmp = TempDir::new().unwrap();
        let ctx = compose_session_context(tmp.path());
        assert!(ctx.is_empty());
    }

    #[test]
    fn includes_phase_and_gate() {
        let tmp = TempDir::new().unwrap();
        let sd = tmp.path().join(".super-dev");
        fs::create_dir_all(&sd).unwrap();
        fs::write(
            sd.join("workflow-state.json"),
            r#"{"phase":"frontend","active_gate":"preview_confirm"}"#,
        )
        .unwrap();
        let ctx = compose_session_context(tmp.path());
        assert!(ctx.text.contains("frontend"));
        assert!(ctx.text.contains("preview_confirm"));
    }

    #[test]
    fn includes_session_brief() {
        let tmp = TempDir::new().unwrap();
        let sd = tmp.path().join(".super-dev");
        fs::create_dir_all(&sd).unwrap();
        fs::write(
            sd.join("SESSION_BRIEF.md"),
            "# Brief\nWaiting for: docs_confirm\nNext: review three docs\n",
        )
        .unwrap();
        let ctx = compose_session_context(tmp.path());
        assert!(ctx.text.contains("Waiting for"));
        assert!(ctx.text.contains("Session brief"));
    }

    #[test]
    fn caps_total_size() {
        let tmp = TempDir::new().unwrap();
        let sd = tmp.path().join(".super-dev");
        fs::create_dir_all(&sd).unwrap();
        fs::write(sd.join("SESSION_BRIEF.md"), "X".repeat(50_000)).unwrap();
        let ctx = compose_session_context(tmp.path());
        assert!(ctx.text.chars().count() <= MAX_CHARS);
    }

    #[test]
    fn tolerates_corrupt_workflow_state() {
        let tmp = TempDir::new().unwrap();
        let sd = tmp.path().join(".super-dev");
        fs::create_dir_all(&sd).unwrap();
        fs::write(sd.join("workflow-state.json"), "{ not json").unwrap();
        // Should not panic; should return empty because other sources are also empty.
        let ctx = compose_session_context(tmp.path());
        assert!(ctx.is_empty());
    }

    #[test]
    fn reads_knowledge_digest() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("output").join("knowledge-cache");
        fs::create_dir_all(&cache).unwrap();
        fs::write(
            cache.join("demo-knowledge-bundle.json"),
            r#"{"research_summary":"Always read knowledge/frontend/* first."}"#,
        )
        .unwrap();
        let ctx = compose_session_context(tmp.path());
        assert!(ctx.text.contains("Always read knowledge"));
    }
}
