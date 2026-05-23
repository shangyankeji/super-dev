//! Session-continuity context injection.
//!
//! Implements `SD-FLOW-006`. On every user prompt the host calls
//! [`compose_session_context`] and prepends the returned text to the
//! model's context. This keeps the model anchored to the active Super
//! Dev phase without the user repeating themselves.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Total char budget for the injected block. Trimmed past this length.
const MAX_CHARS: usize = 3000;

/// Number of `SESSION_BRIEF.md` head lines included verbatim.
const BRIEF_HEAD_LINES: usize = 40;

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

fn read_session_brief_head(root: &Path) -> String {
    let p = root.join(".super-dev").join("SESSION_BRIEF.md");
    let Ok(text) = fs::read_to_string(&p) else {
        return String::new();
    };
    text.lines()
        .take(BRIEF_HEAD_LINES)
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
    let mut bundles: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|s| s.to_str()) == Some("json")
                && p.file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|n| n.ends_with("-knowledge-bundle.json"))
        })
        .collect();
    bundles.sort();
    let Some(latest) = bundles.last() else {
        return String::new();
    };
    let Ok(text) = fs::read_to_string(latest) else {
        return String::new();
    };
    let Ok(val) = serde_json::from_str::<Value>(&text) else {
        return String::new();
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
    let brief = read_session_brief_head(project_root);
    let knowledge = read_knowledge_digest(project_root);

    if state.is_none() && brief.is_empty() && knowledge.is_empty() {
        return SessionContext {
            text: String::new(),
        };
    }

    let mut parts: Vec<String> = vec!["[Super Dev ambient context]".to_string()];
    if let Some(state_val) = state {
        let phase = state_val
            .get("phase")
            .or_else(|| state_val.get("current_phase"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
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
    if text.chars().count() > MAX_CHARS {
        let mut buf = String::with_capacity(MAX_CHARS);
        for ch in text.chars().take(MAX_CHARS.saturating_sub(3)) {
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
