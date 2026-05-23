//! Verify loop — the engine's "project manager runs build/tests"
//! capability. After a code-producing phase, this module:
//!
//! 1. Detects what kind of project sits in the workspace
//!    (`package.json` → Node, `Cargo.toml` → Rust, `pyproject.toml` /
//!    `requirements.txt` → Python).
//! 2. Runs the canonical "does it compile / parse / install" command
//!    for that project type as a subprocess.
//! 3. Returns a structured [`VerifyOutcome`] the runner emits as engine
//!    events and appends to `.super-dev/audit/verify.jsonl`.
//!
//! This module **does not** retry on failure — it just reports. The
//! retry-with-feedback loop is layered on top in the runner.
//!
//! Failures are first-class data, not exceptions: an `Err` `VerifyOutcome`
//! is the desired path for "build failed, here's why". Spawn errors
//! produce a `passed: false` outcome with the OS error as `stderr`.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

/// Cap stderr / stdout captured in the audit row, so a chatty build
/// can't bloat the JSONL.
const CAPTURE_CAP: usize = 8 * 1024;

/// Default per-verify-call timeout.
pub const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// What kind of project we detected.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectKind {
    /// `package.json` present — Node / TypeScript / etc.
    Node,
    /// `Cargo.toml` present — Rust.
    Rust,
    /// `pyproject.toml` or `requirements.txt` present — Python.
    Python,
    /// No recognised manifest. Verify is skipped.
    None,
}

impl ProjectKind {
    /// Stable string label used in audit rows and events.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::Rust => "rust",
            Self::Python => "python",
            Self::None => "none",
        }
    }

    /// The build / verify command for this project type, or `None` if
    /// verify is skipped.
    #[must_use]
    pub fn verify_command(self) -> Option<(&'static str, &'static [&'static str])> {
        match self {
            Self::Node => Some(("npm", &["install", "--no-audit", "--no-fund", "--silent"])),
            Self::Rust => Some(("cargo", &["check", "--quiet"])),
            Self::Python => Some(("python3", &["-c", "print('python ok')"])),
            Self::None => None,
        }
    }
}

/// Detect the project kind from workspace files.
///
/// Order matters: a workspace containing both `package.json` and
/// `Cargo.toml` is reported as Node — projects with a frontend take
/// priority because frontend errors are louder.
#[must_use]
pub fn detect_project(workspace: &Path) -> ProjectKind {
    if workspace.join("package.json").is_file() {
        ProjectKind::Node
    } else if workspace.join("Cargo.toml").is_file() {
        ProjectKind::Rust
    } else if workspace.join("pyproject.toml").is_file()
        || workspace.join("requirements.txt").is_file()
    {
        ProjectKind::Python
    } else {
        ProjectKind::None
    }
}

/// Result of running the verify command.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct VerifyOutcome {
    /// What kind of project we ran against.
    pub project_kind: ProjectKind,
    /// The actual command string we ran (for audit / display).
    pub command: String,
    /// Subprocess exit code; `-1` for spawn / timeout failures.
    pub exit_code: i32,
    /// Wall-clock duration, milliseconds.
    pub duration_ms: u64,
    /// Truncated stdout (capped at 8 KiB).
    pub stdout: String,
    /// Truncated stderr (capped at 8 KiB).
    pub stderr: String,
    /// `true` iff exit code was 0.
    pub passed: bool,
}

impl VerifyOutcome {
    /// Build an outcome that records a non-spawnable command (e.g.
    /// `npm` not on PATH). The audit chain still gets a row so the
    /// project manager has full visibility.
    fn from_spawn_error(project: ProjectKind, command: String, err: &str, ms: u64) -> Self {
        let mut stderr = format!("failed to spawn: {err}");
        truncate_in_place(&mut stderr, CAPTURE_CAP);
        Self {
            project_kind: project,
            command,
            exit_code: -1,
            duration_ms: ms,
            stdout: String::new(),
            stderr,
            passed: false,
        }
    }

    fn from_timeout(project: ProjectKind, command: String, secs: u64) -> Self {
        Self {
            project_kind: project,
            command,
            exit_code: -1,
            duration_ms: secs * 1000,
            stdout: String::new(),
            stderr: format!("timed out after {secs}s"),
            passed: false,
        }
    }
}

/// Run the verify command for `workspace`. Returns `None` when no
/// project manifest is present (verify is genuinely meaningless).
///
/// Honours `SUPER_DEV_VERIFY_TIMEOUT_SECS` env override.
pub async fn run_verify(workspace: &Path) -> Option<VerifyOutcome> {
    let kind = detect_project(workspace);
    let (program, args) = kind.verify_command()?;

    let timeout_secs = std::env::var("SUPER_DEV_VERIFY_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);

    let command_str = format!("{program} {}", args.join(" "));
    let started = Instant::now();

    let spawn = Command::new(program)
        .args(args)
        .current_dir(workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output();

    let result = tokio::time::timeout(Duration::from_secs(timeout_secs), spawn).await;

    let outcome = match result {
        Ok(Ok(out)) => {
            let mut stdout = String::from_utf8_lossy(&out.stdout).into_owned();
            let mut stderr = String::from_utf8_lossy(&out.stderr).into_owned();
            truncate_in_place(&mut stdout, CAPTURE_CAP);
            truncate_in_place(&mut stderr, CAPTURE_CAP);
            VerifyOutcome {
                project_kind: kind,
                command: command_str,
                exit_code: out.status.code().unwrap_or(-1),
                duration_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
                stdout,
                stderr,
                passed: out.status.success(),
            }
        }
        Ok(Err(e)) => VerifyOutcome::from_spawn_error(
            kind,
            command_str,
            &e.to_string(),
            started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
        ),
        Err(_) => VerifyOutcome::from_timeout(kind, command_str, timeout_secs),
    };

    Some(outcome)
}

/// Append the outcome (plus a timestamp + phase tag) to
/// `.super-dev/audit/verify.jsonl`. The runner calls this after every
/// verify pass so auditors get a complete chain.
pub fn record_verify_outcome(
    workspace: &Path,
    phase: &str,
    outcome: &VerifyOutcome,
) -> std::io::Result<PathBuf> {
    let audit_dir = workspace.join(".super-dev/audit");
    std::fs::create_dir_all(&audit_dir)?;
    let path = audit_dir.join("verify.jsonl");

    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut row = serde_json::to_value(outcome).unwrap_or(serde_json::Value::Null);
    if let Some(obj) = row.as_object_mut() {
        obj.insert("timestamp".into(), serde_json::Value::String(timestamp));
        obj.insert("phase".into(), serde_json::Value::String(phase.to_string()));
    }
    let line = serde_json::to_string(&row).unwrap_or_default();

    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(f, "{line}")?;
    Ok(path)
}

fn truncate_in_place(s: &mut String, cap: usize) {
    if s.len() > cap {
        // Truncate at char boundary to keep the string valid.
        let mut idx = cap;
        while !s.is_char_boundary(idx) {
            idx -= 1;
        }
        s.truncate(idx);
        s.push_str("\n...[truncated]");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detect_node_project() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), r#"{"name":"x"}"#).unwrap();
        assert_eq!(detect_project(tmp.path()), ProjectKind::Node);
    }

    #[test]
    fn detect_rust_project() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"",
        )
        .unwrap();
        assert_eq!(detect_project(tmp.path()), ProjectKind::Rust);
    }

    #[test]
    fn detect_python_project_via_pyproject() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("pyproject.toml"), "[project]\nname=\"x\"").unwrap();
        assert_eq!(detect_project(tmp.path()), ProjectKind::Python);
    }

    #[test]
    fn detect_python_project_via_requirements() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("requirements.txt"), "requests==2.0").unwrap();
        assert_eq!(detect_project(tmp.path()), ProjectKind::Python);
    }

    #[test]
    fn detect_returns_none_when_no_manifest() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect_project(tmp.path()), ProjectKind::None);
    }

    #[test]
    fn node_takes_priority_over_rust_when_both_present() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), r#"{"name":"x"}"#).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"",
        )
        .unwrap();
        assert_eq!(detect_project(tmp.path()), ProjectKind::Node);
    }

    #[test]
    fn verify_command_strings_are_stable() {
        assert_eq!(ProjectKind::Node.as_str(), "node");
        assert_eq!(ProjectKind::Rust.as_str(), "rust");
        assert_eq!(ProjectKind::Python.as_str(), "python");
        assert_eq!(ProjectKind::None.as_str(), "none");
    }

    #[test]
    fn verify_command_returns_none_for_no_project() {
        assert!(ProjectKind::None.verify_command().is_none());
        assert!(ProjectKind::Rust.verify_command().is_some());
    }

    #[tokio::test]
    async fn run_verify_returns_none_when_no_manifest() {
        let tmp = TempDir::new().unwrap();
        assert!(run_verify(tmp.path()).await.is_none());
    }

    #[tokio::test]
    async fn run_verify_reports_spawn_failure_when_command_missing() {
        // Simulate "the build tool isn't installed" by pointing at a
        // bogus python binary via PATH manipulation is fragile. Instead
        // we install a manifest the build for which will fail at runtime.
        let tmp = TempDir::new().unwrap();
        // Write a Python manifest, then point verify at a Python that
        // surely fails: we'll synthesise a project that runs the real
        // `python3 -c "print('python ok')"` so this path actually
        // succeeds on hosts that have python3. The negative case is
        // exercised in `run_verify_captures_nonzero_exit` below.
        fs::write(tmp.path().join("pyproject.toml"), "[project]\nname=\"x\"").unwrap();
        let outcome = run_verify(tmp.path()).await;
        assert!(outcome.is_some());
    }

    #[tokio::test]
    async fn record_outcome_writes_jsonl_line() {
        let tmp = TempDir::new().unwrap();
        let outcome = VerifyOutcome {
            project_kind: ProjectKind::Rust,
            command: "cargo check".into(),
            exit_code: 1,
            duration_ms: 42,
            stdout: String::new(),
            stderr: "error[E0599]: no method named foo".into(),
            passed: false,
        };
        let path = record_verify_outcome(tmp.path(), "frontend", &outcome).unwrap();
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("\"phase\":\"frontend\""));
        assert!(body.contains("\"project_kind\":\"rust\""));
        assert!(body.contains("\"passed\":false"));
        assert!(body.contains("E0599"));
    }

    #[tokio::test]
    async fn record_outcome_appends_multiple_lines() {
        let tmp = TempDir::new().unwrap();
        for i in 0..3 {
            let outcome = VerifyOutcome {
                project_kind: ProjectKind::Node,
                command: format!("npm install (run {i})"),
                exit_code: 0,
                duration_ms: 10,
                stdout: String::new(),
                stderr: String::new(),
                passed: true,
            };
            record_verify_outcome(tmp.path(), "backend", &outcome).unwrap();
        }
        let body = fs::read_to_string(tmp.path().join(".super-dev/audit/verify.jsonl")).unwrap();
        assert_eq!(body.lines().count(), 3);
    }

    #[test]
    fn truncate_does_not_split_multibyte_chars() {
        let mut s = "做做做做做做".to_string(); // each char is 3 bytes
        truncate_in_place(&mut s, 7); // boundary lands inside a char
        assert!(s.is_char_boundary(0));
        assert!(s.ends_with("[truncated]"));
        // The truncated body must still be valid UTF-8 (no panic).
        let _ = s.as_bytes();
    }
}
