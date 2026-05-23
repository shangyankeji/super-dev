//! Audit trails — the evidence half of `SUPER_DEV_HOST_SPEC_V1` layer 4.
//!
//! Two append-only logs live at:
//!
//! - `<project_root>/.super-dev/audit/frontend-api-calls.jsonl`
//!   (Implements `SD-EVID-001`)
//! - `<project_root>/.super-dev/audit/tool-calls.jsonl`
//!   (Implements `SD-EVID-002`)
//!
//! Both are JSONL. Both fail open: a filesystem error here MUST NOT
//! break the host.

use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const FRONTEND_EXTS: &[&str] = &["tsx", "ts", "jsx", "js", "vue", "svelte", "astro"];

/// One audited frontend API call.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApiCallRecord {
    /// Unix seconds.
    pub ts: i64,
    /// Workspace-relative path of the file being written.
    pub file: String,
    /// Host tool name, e.g. `Write` or `Edit`.
    pub tool: String,
    /// Sorted, deduped list of API paths extracted from `content`.
    pub urls: Vec<String>,
    /// Opaque host session identifier; empty when absent.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub session_id: String,
}

/// One audited host tool call (a wider trail than just API audit).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolCallRecord {
    /// Unix seconds.
    pub ts: i64,
    /// Host tool name (e.g. `Write`, `Edit`, `Bash`).
    pub tool: String,
    /// Workspace-relative target file (empty when not applicable).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub file: String,
    /// Outcome: `allow` | `block` | `warn` | `audit`.
    pub decision: String,
    /// Firing clause id (e.g. `SD-CODE-001`); empty when not gated.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub clause: String,
    /// Human-readable note shown to the model.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reason: String,
    /// Opaque host session identifier; empty when absent.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub session_id: String,
}

fn api_url_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Common fetch-shaped callers + an absolute path leading with `/`.
        // Path is captured non-greedily up to the next quote / ?,#,space.
        Regex::new(
            r#"(?x)
                (?:fetch|axios\.\w+|ky\.\w+|useSWR|useQuery|http\.\w+)
                \s*\(\s*
                ['"`]
                (?P<url>/[^'"`?\#\s]+)
            "#,
        )
        .expect("api url regex is well-formed")
    })
}

fn ext_of(file_path: &str) -> String {
    file_path
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default()
}

/// Extract sorted, deduplicated frontend API paths from `content`.
///
/// Implements the extraction half of `SD-CODE-003`. Returns an empty
/// `Vec` for non-frontend file extensions.
#[must_use]
pub fn extract_api_urls(file_path: &str, content: &str) -> Vec<String> {
    let ext = ext_of(file_path);
    if !FRONTEND_EXTS.contains(&ext.as_str()) {
        return Vec::new();
    }
    let mut urls: Vec<String> = Vec::new();
    for cap in api_url_regex().captures_iter(content) {
        if let Some(url) = cap.name("url") {
            let s = url.as_str().to_string();
            if !urls.contains(&s) {
                urls.push(s);
            }
        }
    }
    urls.sort();
    urls
}

fn audit_dir(project_root: &Path) -> PathBuf {
    project_root.join(".super-dev").join("audit")
}

fn append_jsonl(path: &Path, line: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    f.write_all(line.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}

/// Append an API-audit record. Implements `SD-EVID-001`.
///
/// Returns `Some(record)` when something was extracted (regardless of
/// disk-write success — audit failure must never bubble up). Returns
/// `None` when the file has no URLs to log.
#[must_use]
pub fn record_api_calls(
    project_root: &Path,
    file_path: &str,
    content: &str,
    tool_name: &str,
    session_id: &str,
    now: Option<i64>,
) -> Option<ApiCallRecord> {
    let urls = extract_api_urls(file_path, content);
    if urls.is_empty() {
        return None;
    }
    let record = ApiCallRecord {
        ts: now.unwrap_or_else(|| Utc::now().timestamp()),
        file: file_path.to_string(),
        tool: tool_name.to_string(),
        urls,
        session_id: session_id.to_string(),
    };
    let log_path = audit_dir(project_root).join("frontend-api-calls.jsonl");
    if let Ok(line) = serde_json::to_string(&record) {
        let _ = append_jsonl(&log_path, &line);
    }
    Some(record)
}

/// Append a tool-call audit record. Implements `SD-EVID-002`.
///
/// Returns `None` for an empty `tool_name` (nothing to log); otherwise
/// returns the record. Disk-write errors are swallowed by design.
#[must_use]
pub fn record_tool_call(
    project_root: &Path,
    tool_name: &str,
    file_path: &str,
    decision: &str,
    clause: &str,
    reason: &str,
    session_id: &str,
    now: Option<i64>,
) -> Option<ToolCallRecord> {
    if tool_name.is_empty() {
        return None;
    }
    let record = ToolCallRecord {
        ts: now.unwrap_or_else(|| Utc::now().timestamp()),
        tool: tool_name.to_string(),
        file: file_path.to_string(),
        decision: if decision.is_empty() {
            "allow".to_string()
        } else {
            decision.to_string()
        },
        clause: clause.to_string(),
        reason: reason.to_string(),
        session_id: session_id.to_string(),
    };
    let log_path = audit_dir(project_root).join("tool-calls.jsonl");
    if let Ok(line) = serde_json::to_string(&record) {
        let _ = append_jsonl(&log_path, &line);
    }
    Some(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    #[test]
    fn extract_fetch_axios_ky_swr() {
        let urls = extract_api_urls(
            "src/X.tsx",
            "fetch('/api/users'); axios.post('/api/orders', body); ky.get('/api/k'); useSWR('/api/s', f)",
        );
        assert_eq!(urls, vec!["/api/k", "/api/orders", "/api/s", "/api/users"]);
    }

    #[test]
    fn extract_dedupes() {
        let urls = extract_api_urls("src/X.tsx", "fetch('/api/u'); fetch('/api/u')");
        assert_eq!(urls, vec!["/api/u"]);
    }

    #[test]
    fn extract_ignores_external() {
        let urls = extract_api_urls("src/X.tsx", "fetch('https://cdn.example.com/i.png')");
        assert!(urls.is_empty());
    }

    #[test]
    fn extract_ignores_non_frontend_extension() {
        let urls = extract_api_urls("server.py", "fetch('/api/u')");
        assert!(urls.is_empty());
    }

    #[test]
    fn extract_handles_empty_content() {
        assert!(extract_api_urls("src/x.tsx", "").is_empty());
    }

    #[test]
    fn record_api_calls_persists_jsonl() {
        let tmp = TempDir::new().unwrap();
        let r = record_api_calls(
            tmp.path(),
            "src/U.tsx",
            "fetch('/api/users'); axios.post('/api/orders', b)",
            "Write",
            "sess-123",
            Some(1_700_000_000),
        )
        .unwrap();
        assert_eq!(r.urls, vec!["/api/orders", "/api/users"]);
        let log = tmp.path().join(".super-dev/audit/frontend-api-calls.jsonl");
        assert!(log.exists());
        let text = std::fs::read_to_string(&log).unwrap();
        assert!(text.contains("/api/users"));
        assert!(text.contains("sess-123"));
    }

    #[test]
    fn record_api_calls_skips_when_empty() {
        let tmp = TempDir::new().unwrap();
        let r = record_api_calls(tmp.path(), "src/X.tsx", "const x = 1", "Write", "", None);
        assert!(r.is_none());
        assert!(!tmp.path().join(".super-dev/audit").exists());
    }

    #[test]
    fn record_api_calls_appends() {
        let tmp = TempDir::new().unwrap();
        let _ = record_api_calls(
            tmp.path(),
            "src/A.tsx",
            "fetch('/api/a')",
            "Write",
            "",
            Some(1),
        );
        let _ = record_api_calls(
            tmp.path(),
            "src/B.tsx",
            "fetch('/api/b')",
            "Write",
            "",
            Some(2),
        );
        let log = tmp.path().join(".super-dev/audit/frontend-api-calls.jsonl");
        let lines = std::fs::read_to_string(&log).unwrap();
        assert_eq!(lines.lines().count(), 2);
    }

    #[test]
    fn record_tool_call_full_record() {
        let tmp = TempDir::new().unwrap();
        let r = record_tool_call(
            tmp.path(),
            "Write",
            "src/X.tsx",
            "block",
            "SD-CODE-001",
            "emoji used",
            "sess-xyz",
            Some(1_700_000_001),
        )
        .unwrap();
        assert_eq!(r.tool, "Write");
        assert_eq!(r.decision, "block");
        assert_eq!(r.clause, "SD-CODE-001");
        let log = tmp.path().join(".super-dev/audit/tool-calls.jsonl");
        assert!(log.exists());
    }

    #[test]
    fn record_tool_call_empty_tool_name_skipped() {
        let tmp = TempDir::new().unwrap();
        let r = record_tool_call(tmp.path(), "", "x", "block", "", "", "", None);
        assert!(r.is_none());
    }

    #[test]
    fn record_tool_call_default_decision_is_allow() {
        let tmp = TempDir::new().unwrap();
        let r = record_tool_call(tmp.path(), "Edit", "x", "", "", "", "", Some(1)).unwrap();
        assert_eq!(r.decision, "allow");
    }
}
