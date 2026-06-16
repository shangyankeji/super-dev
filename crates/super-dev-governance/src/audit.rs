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

/// Maximum size an audit JSONL may reach before it is rotated. Default
/// 5 MiB — large enough to hold a full delivery's worth of tool/API calls
/// (typically a few hundred KB), small enough that a long-running session
/// can't bloat `.super-dev/audit/` without bound. Override with the
/// `SUPER_DEV_AUDIT_MAX_BYTES` env var (0 = never rotate).
const DEFAULT_MAX_JSONL_BYTES: u64 = 5 * 1024 * 1024;

/// How many rotated archives to keep per audit file. Older archives
/// (`*.jsonl.<n>` beyond this count) are deleted on rotation so the
/// directory stays bounded.
const MAX_ARCHIVES: usize = 3;

/// Resolve the rotation threshold from the env override, falling back to
/// the default. `0` disables rotation entirely.
fn max_jsonl_bytes() -> u64 {
    std::env::var("SUPER_DEV_AUDIT_MAX_BYTES")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_MAX_JSONL_BYTES)
}

/// If `path` exists and exceeds the size threshold, rotate it:
/// `tool-calls.jsonl` → `tool-calls.jsonl.1` (shifting older `.n` up),
/// keeping at most `MAX_ARCHIVES` copies. Best-effort — rotation errors
/// are swallowed (audit must never block the host).
fn rotate_if_needed(path: &Path) {
    let cap = max_jsonl_bytes();
    if cap == 0 {
        return;
    }
    let Ok(meta) = fs::metadata(path) else {
        return;
    };
    if meta.len() < cap {
        return;
    }
    // Shift archives: .{MAX-1} is dropped, .{n} → .{n+1}, then current → .1.
    // Walk from the oldest kept slot downward so we don't overwrite an
    // archive we still need to shift.
    for n in (1..MAX_ARCHIVES).rev() {
        let src = archive_path(path, n);
        if src.exists() {
            let dst = archive_path(path, n + 1);
            let _ = fs::rename(&src, &dst);
        }
    }
    // Current file becomes .1.
    let archive1 = archive_path(path, 1);
    let _ = fs::rename(path, &archive1);
    // Drop any archive beyond the keep count.
    if MAX_ARCHIVES > 0 {
        let drop_n = MAX_ARCHIVES + 1;
        let beyond = archive_path(path, drop_n);
        let _ = fs::remove_file(&beyond);
    }
}

/// `tool-calls.jsonl` + `.{n}` → `tool-calls.jsonl.{n}`.
fn archive_path(path: &Path, n: usize) -> PathBuf {
    let mut name = path
        .file_name()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    name.push('.');
    name.push_str(&n.to_string());
    path.with_file_name(name)
}

/// One audited frontend API call.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApiCallRecord {
    /// Unix seconds.
    pub ts: i64,
    /// Unix milliseconds — sub-second resolution so two calls in the same
    /// second can still be ordered. `#[serde(default)]` keeps old JSONL rows
    /// (pre-4.6, which only had `ts`) deserialisable.
    #[serde(default)]
    pub ts_ms: i64,
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
    /// Unix milliseconds — sub-second resolution for deterministic ordering
    /// of calls that share a second. `#[serde(default)]` for old rows.
    #[serde(default)]
    pub ts_ms: i64,
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
        // Callers that target an API path. Covers modern patterns:
        // - fetch / axios.METHOD / axios() direct / ky.METHOD / http.METHOD
        // - React Query: useQuery / useMutation / useSWR / useSWRInfinite
        // - Wrapped clients: api.METHOD, httpClient.METHOD, client.METHOD,
        //   request(...), fetcher(...), service.METHOD — common names people
        //   give a typed/SDK wrapper around fetch.
        // The URL must start with `/` and runs to the next quote / ? / # /
        // space / `${` (so template-literal fetch(`/api/${id}`) captures the
        // static prefix `/api/`).
        Regex::new(
            r#"(?x)
                (?:
                    fetch | axios | ky | http
                  | useSWR | useSWRInfinite | useQuery | useMutation
                  | api | httpClient | client | request | fetcher | service
                )
                (?:\.\w+)?
                \s*\(\s*
                ['"`]
                (?P<url>/[^'"`?\#\s$]+)
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

/// Normalize a tool-call decision to the documented vocabulary
/// (`allow` | `block` | `warn` | `audit`). Unknown / empty → `allow`
/// (fail-open: an unrecognized decision must never block the host).
/// Matching is case-insensitive so `BLOCK` / `Block` collapse correctly.
fn normalize_decision(decision: &str) -> String {
    let lower = decision.trim().to_ascii_lowercase();
    match lower.as_str() {
        "block" | "warn" | "audit" => lower,
        _ => "allow".to_string(),
    }
}

fn append_jsonl(path: &Path, line: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Rotate before appending so the file stays under the size cap across
    // long sessions. Best-effort: a rotation failure must not block the
    // append (audit is fail-open).
    rotate_if_needed(path);
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
        ts_ms: Utc::now().timestamp_millis(),
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
    // Normalize the decision to the documented vocabulary
    // (allow | block | warn | audit). An unknown value used to be stored
    // verbatim, which then polluted the decisions BTreeMap in the
    // compliance mapping with arbitrary keys.
    let decision_norm = normalize_decision(decision);
    let record = ToolCallRecord {
        ts: now.unwrap_or_else(|| Utc::now().timestamp()),
        ts_ms: Utc::now().timestamp_millis(),
        tool: tool_name.to_string(),
        file: file_path.to_string(),
        decision: decision_norm,
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
        let _guard = ROTATE_TEST_GUARD
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::env::remove_var("SUPER_DEV_AUDIT_MAX_BYTES");
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
        // Rotation tests mutate SUPER_DEV_AUDIT_MAX_BYTES; run governance
        // tests with --test-threads=1 to avoid env-var races.
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

    #[test]
    fn record_tool_call_normalizes_unknown_decision() {
        // An unrecognized decision must collapse to "allow" (fail-open)
        // rather than polluting the compliance decisions map.
        let tmp = TempDir::new().unwrap();
        let r = record_tool_call(tmp.path(), "Edit", "x", "BANANA", "", "", "", Some(1)).unwrap();
        assert_eq!(r.decision, "allow");
    }

    #[test]
    fn record_tool_call_preserves_known_decisions_case_insensitive() {
        let tmp = TempDir::new().unwrap();
        let r = record_tool_call(tmp.path(), "Edit", "x", "BLOCK", "", "", "", Some(1)).unwrap();
        assert_eq!(r.decision, "block");
        let r2 = record_tool_call(tmp.path(), "Edit", "x", "  Warn ", "", "", "", Some(1)).unwrap();
        assert_eq!(r2.decision, "warn");
        let r3 = record_tool_call(tmp.path(), "Edit", "x", "Audit", "", "", "", Some(1)).unwrap();
        assert_eq!(r3.decision, "audit");
    }

    // NOTE: these mutate the process-global SUPER_DEV_AUDIT_MAX_BYTES env
    // var, so they must run in ONE test (serially) — parallel #[test]s on
    // the same env var race and flake.
    /// Test-only guard serializing the env-mutating rotate test against
    /// `record_api_calls_appends` (which reads the rotation cap). Held for
    /// the whole test body.
    static ROTATE_TEST_GUARD: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn rotate_serial_under_cap_and_disabled() {
        use super::*;
        let _guard = ROTATE_TEST_GUARD
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // --- (1) rotate when over cap ---
        std::env::set_var("SUPER_DEV_AUDIT_MAX_BYTES", "8");
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join(".super-dev/audit");
        fs::create_dir_all(&dir).unwrap();
        let live = dir.join("tool-calls.jsonl");
        fs::write(&live, "already-big-content-here").unwrap();
        append_jsonl(&live, r#"{"ts":1}"#).unwrap();
        let body = fs::read_to_string(&live).unwrap();
        assert!(
            body.contains(r#"{"ts":1}"#),
            "new line must be in live file"
        );
        assert!(
            !body.contains("already-big-content"),
            "old content rotated out"
        );
        let archive1 = archive_path(&live, 1);
        assert!(fs::read_to_string(&archive1)
            .unwrap()
            .contains("already-big-content"));

        // --- (2) keeps at most MAX_ARCHIVES ---
        std::env::set_var("SUPER_DEV_AUDIT_MAX_BYTES", "4");
        for i in 0..(MAX_ARCHIVES + 3) {
            append_jsonl(&live, &format!("line-{i}-content")).unwrap();
        }
        let archived_files: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("tool-calls.jsonl.")
            })
            .collect();
        assert!(
            archived_files.len() <= MAX_ARCHIVES,
            "should keep at most {MAX_ARCHIVES} archives, got {}",
            archived_files.len()
        );

        // --- (3) disabled when cap=0 ---
        std::env::set_var("SUPER_DEV_AUDIT_MAX_BYTES", "0");
        let tmp2 = TempDir::new().unwrap();
        let live2 = tmp2.path().join("tool-calls.jsonl");
        fs::write(&live2, "big-content-that-would-normally-rotate").unwrap();
        append_jsonl(&live2, "new").unwrap();
        let body2 = fs::read_to_string(&live2).unwrap();
        assert!(body2.contains("big-content"), "cap=0 must not rotate");
        assert!(body2.contains("new"));

        // Restore the sentinel so other tests see the real default.
        // Rotation tests mutate SUPER_DEV_AUDIT_MAX_BYTES; run governance
        // tests with --test-threads=1 to avoid env-var races.
    }
}
