//! Verify loop — the engine's "project manager runs build/tests"
//! capability. After a code-producing phase, this module:
//!
//! 1. Detects what kind of project sits in the workspace
//!    (`deno.json` → Deno, `package.json` → Node, `Cargo.toml` → Rust,
//!    `go.mod` → Go, `pyproject.toml` / `requirements.txt` → Python).
//!    For Node, the package manager (pnpm / yarn / bun / npm) is chosen
//!    from the lockfile so verify matches the scaffolded project.
//! 2. Runs a **sequence** of verify steps (install → lint → typecheck →
//!    test → build) for that project type as subprocesses, each with its
//!    own timeout and captured output. A step whose binary is missing
//!    (e.g. `ruff` not installed) is recorded as `skipped`, not failure.
//! 3. Returns a structured [`VerifyOutcome`] per step. The runner emits
//!    these as engine events and appends to `.super-dev/audit/verify.jsonl`.
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

/// Default per-step timeout (seconds) when neither the step's own
/// [`VerifyStep::timeout_secs`] nor the `SUPER_DEV_VERIFY_TIMEOUT_SECS`
/// env override is set.
pub const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Larger default budget (seconds) for steps that do real work —
/// `install` (network), `test` (suite), `build` (compile). Used as the
/// per-step default for those categories so a global 120s cap doesn't
/// falsely time out a legitimate `cargo build --release`.
const SLOW_STEP_TIMEOUT_SECS: u64 = 600;

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
    /// `go.mod` present — Go.
    Go,
    /// `deno.json` / `deno.jsonc` present — Deno.
    Deno,
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
            Self::Go => "go",
            Self::Deno => "deno",
            Self::None => "none",
        }
    }

    /// The **single canonical** build/verify command for this project type
    /// (kept for backwards compatibility with older callers and the TUI).
    /// For the full multi-step sequence, use [`verify_steps`].
    ///
    /// For Node the package manager is chosen by lockfile (pnpm / yarn /
    /// bun / npm) so the install matches what the worker actually scaffolded.
    #[must_use]
    pub fn verify_command(self, workspace: &Path) -> Option<(String, Vec<String>)> {
        let steps = verify_steps(self, workspace)?;
        // Return the first step (install for most, check for rust/go/deno).
        steps.into_iter().next().map(|s| (s.program, s.args))
    }
}

/// One verify step: a named command to run (e.g. "install", "lint", "test").
#[derive(Debug, Clone)]
pub struct VerifyStep {
    /// Short label for the audit row (e.g. "install", "lint", "test").
    pub name: &'static str,
    /// The program to run.
    pub program: String,
    /// The arguments.
    pub args: Vec<String>,
    /// Whether a missing binary should be `skipped` (true) or `failed`
    /// (false). E.g. `ruff` not installed → skipped; `cargo` missing → failed.
    pub skippable: bool,
    /// Per-step timeout budget in seconds. `0` means "use the global
    /// default" ([`DEFAULT_TIMEOUT_SECS`] or the
    /// `SUPER_DEV_VERIFY_TIMEOUT_SECS` env override). Lets slow steps
    /// (`build`, `test`, `install`) declare a larger budget than fast ones
    /// (`fmt`, `lint`), which a single global timeout can't express.
    pub timeout_secs: u64,
}

/// The ordered verify step sequence for a project type. Each step runs
/// independently with its own timeout; a failing step does NOT abort the
/// remaining steps (we want full visibility into what works and what
/// doesn't). Returns `None` when the project type is unrecognised.
///
/// **Sequence per stack:**
/// - Node → install → lint (if script exists) → typecheck (if tsc) →
///   test (if script exists) → build (if script exists)
/// - Rust → fmt-check → clippy → test → build
/// - Python → install → ruff check → mypy (if configured) → pytest
/// - Go → vet → test → build
/// - Deno → lint → test → check
#[must_use]
pub fn verify_steps(kind: ProjectKind, workspace: &Path) -> Option<Vec<VerifyStep>> {
    // Helper: fast step (lint/fmt/vet) — default timeout.
    let s = |name: &'static str, program: &str, args: &[&str], skippable: bool| VerifyStep {
        name,
        program: program.to_string(),
        args: args.iter().map(|a| (*a).to_string()).collect(),
        skippable,
        timeout_secs: 0, // 0 → global default
    };
    // Helper: slow step (install/test/build) — larger budget.
    let slow = |name: &'static str, program: &str, args: &[&str], skippable: bool| VerifyStep {
        name,
        program: program.to_string(),
        args: args.iter().map(|a| (*a).to_string()).collect(),
        skippable,
        timeout_secs: SLOW_STEP_TIMEOUT_SECS,
    };
    match kind {
        ProjectKind::Node => {
            let (pm, install_args) = node_package_manager(workspace);
            let mut steps = vec![slow(
                "install",
                pm,
                install_args,
                false, // missing pm = hard fail
            )];
            // Lint: only if a lint script exists in package.json.
            if has_node_script(workspace, "lint") {
                steps.push(s("lint", pm, &["run", "lint"], true));
            }
            // Typecheck: tsc or vue-tsc. Decide by what package.json DECLARES
            // (not by whether node_modules/.bin/<x> exists at step-build time),
            // because install runs as the FIRST step at runtime — so the binary
            // may not exist yet when we build this list, and we'd wrongly pick
            // `tsc` for a Vue project whose install would have provided vue-tsc.
            if workspace.join("tsconfig.json").is_file() {
                let tsc = if package_json_depends_on(workspace, "vue-tsc") {
                    "vue-tsc"
                } else {
                    "tsc"
                };
                steps.push(s("typecheck", tsc, &["--noEmit"], true));
            }
            // Test: only if a test script exists.
            if has_node_script(workspace, "test") {
                steps.push(slow("test", pm, &["run", "test"], false));
            }
            // Build: only if a build script exists.
            if has_node_script(workspace, "build") {
                steps.push(slow("build", pm, &["run", "build"], false));
            }
            Some(steps)
        }
        ProjectKind::Rust => Some(vec![
            s("fmt", "cargo", &["fmt", "--check", "--quiet"], true),
            s(
                "clippy",
                "cargo",
                &["clippy", "--quiet", "--", "-D", "warnings"],
                true,
            ),
            slow("test", "cargo", &["test", "--quiet"], false),
            slow("build", "cargo", &["build", "--release", "--quiet"], false),
        ]),
        ProjectKind::Python => {
            let mut steps = Vec::new();
            // Install: prefer uv (fast), fall back to pip.
            if which("uv") {
                steps.push(slow("install", "uv", &["sync"], false));
            } else {
                steps.push(slow("install", "pip", &["install", "-e", "."], true));
            }
            steps.push(s("lint", "ruff", &["check"], true));
            if workspace.join("mypy.ini").is_file()
                || (workspace.join("pyproject.toml").is_file()
                    && file_contains(workspace, "pyproject.toml", "[tool.mypy]"))
            {
                steps.push(s("typecheck", "mypy", &["."], true));
            }
            steps.push(slow("test", "pytest", &[], true));
            Some(steps)
        }
        ProjectKind::Go => Some(vec![
            s("vet", "go", &["vet", "./..."], true),
            slow("test", "go", &["test", "./..."], false),
            slow("build", "go", &["build", "./..."], false),
        ]),
        ProjectKind::Deno => Some(vec![
            s("lint", "deno", &["lint"], true),
            s("test", "deno", &["test"], true),
            s("check", "deno", &["check", "."], false),
        ]),
        ProjectKind::None => None,
    }
}

/// Check whether a PATH-resolvable binary exists. Used to decide whether a
/// step is genuinely missing (→ skip) vs the project being broken (→ fail).
///
/// Splits `PATH` on the platform-native separator (`:` on Unix, `;` on
/// Windows). On Windows also honours `PATHEXT` so `which("cargo")` finds
/// `cargo.exe`. Previously this split on `:` unconditionally, which meant
/// every step was reported "skipped" on Windows.
fn which(bin: &str) -> bool {
    let Ok(path_var) = std::env::var("PATH") else {
        return false;
    };
    let separator = if cfg!(windows) { ';' } else { ':' };
    // On Windows a bare "cargo" resolves via PATHEXT (`.exe`, `.bat`, …).
    let exts: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.BAT;.CMD;.COM".to_string())
            .split(';')
            .map(str::to_string)
            .collect()
    } else {
        vec![String::new()]
    };
    for dir in path_var.split(separator) {
        if dir.is_empty() {
            continue;
        }
        for ext in &exts {
            let candidate = Path::new(dir).join(format!("{bin}{ext}"));
            if candidate.is_file() {
                return true;
            }
        }
    }
    false
}

/// Read a file and check whether it contains a substring (best-effort).
fn file_contains(workspace: &Path, file: &str, needle: &str) -> bool {
    std::fs::read_to_string(workspace.join(file)).is_ok_and(|content| content.contains(needle))
}

/// Whether `package.json` declares a dependency (in `dependencies` or
/// `devDependencies`) on `pkg`. Used to pick the right typechecker (vue-tsc
/// vs tsc) WITHOUT relying on `node_modules/.bin/` existing at step-build
/// time, since install runs first at runtime.
fn package_json_depends_on(workspace: &Path, pkg: &str) -> bool {
    let Ok(content) = std::fs::read_to_string(workspace.join("package.json")) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    let in_obj = |key: &str| {
        json.get(key)
            .and_then(|v| v.as_object())
            .is_some_and(|o| o.contains_key(pkg))
    };
    in_obj("dependencies") || in_obj("devDependencies") || in_obj("peerDependencies")
}

/// Check whether `package.json` has a given script (e.g. "lint", "test").
fn has_node_script(workspace: &Path, script: &str) -> bool {
    let Ok(content) = std::fs::read_to_string(workspace.join("package.json")) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    json.get("scripts").and_then(|s| s.get(script)).is_some()
}

/// Pick the Node package manager + install args from the workspace's
/// lockfile. Falls back to `npm` when no lockfile is present.
fn node_package_manager(workspace: &Path) -> (&'static str, &'static [&'static str]) {
    if workspace.join("pnpm-lock.yaml").is_file() {
        ("pnpm", &["install", "--silent"])
    } else if workspace.join("yarn.lock").is_file() {
        // yarn classic (v1) AND yarn berry (v2+) both use `yarn.lock`; the
        // install command is identical, so no version branch is needed here.
        ("yarn", &["install", "--silent"])
    } else if workspace.join("bun.lock").is_file() || workspace.join("bun.lockb").is_file() {
        // Bun 1.2+ migrated from binary `bun.lockb` to text `bun.lock`; check
        // the text form first so a repo with BOTH (transition state) picks the
        // current one. Install command is the same either way.
        ("bun", &["install"])
    } else {
        ("npm", &["install", "--no-audit", "--no-fund", "--silent"])
    }
}

/// Detect the project kind from workspace files.
///
/// Order matters: a workspace containing both `package.json` and
/// `Cargo.toml` is reported as Node — projects with a frontend take
/// priority because frontend errors are louder. Deno is checked before
/// Node because a `deno.json` repo may also carry a `package.json` for
/// editor tooling while building with Deno.
#[must_use]
pub fn detect_project(workspace: &Path) -> ProjectKind {
    if workspace.join("deno.json").is_file() || workspace.join("deno.jsonc").is_file() {
        ProjectKind::Deno
    } else if workspace.join("package.json").is_file() {
        ProjectKind::Node
    } else if workspace.join("Cargo.toml").is_file() {
        ProjectKind::Rust
    } else if workspace.join("go.mod").is_file() {
        ProjectKind::Go
    } else if workspace.join("pyproject.toml").is_file()
        || workspace.join("requirements.txt").is_file()
    {
        ProjectKind::Python
    } else {
        ProjectKind::None
    }
}

/// Result of running a single verify step.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct VerifyOutcome {
    /// What kind of project we ran against.
    pub project_kind: ProjectKind,
    /// Which step this was (e.g. "install", "lint", "test"). Pre-4.6 runs
    /// that had no multi-step concept default to "default".
    #[serde(default = "default_step")]
    pub step: String,
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
    /// `true` when the step was skipped because its binary wasn't on PATH.
    #[serde(default)]
    pub skipped: bool,
}

/// Default value for the `step` field in old JSONL rows.
fn default_step() -> String {
    "default".to_string()
}

impl VerifyOutcome {
    /// Build an outcome that records a non-spawnable command (e.g.
    /// `npm` not on PATH). If the step is `skippable`, this becomes a
    /// `skipped` row instead of a failure.
    fn from_spawn_error(
        project: ProjectKind,
        step: &str,
        command: String,
        err: &str,
        ms: u64,
        skippable: bool,
    ) -> Self {
        let mut stderr = format!("failed to spawn: {err}");
        truncate_in_place(&mut stderr, CAPTURE_CAP);
        // A skippable step whose binary is absent is neutral (passed=true,
        // skipped=true so the gate downweights it). A NON-skippable step
        // whose binary is absent is a real failure — it MUST report
        // passed=false so the quality gate surfaces it, not silently green.
        Self {
            project_kind: project,
            step: step.to_string(),
            command,
            exit_code: -1,
            duration_ms: ms,
            stdout: String::new(),
            stderr,
            passed: skippable,
            skipped: skippable,
        }
    }

    fn from_timeout(
        project: ProjectKind,
        step: &str,
        command: String,
        secs: u64,
        partial_stdout: String,
        partial_stderr: String,
    ) -> Self {
        let mut stdout = partial_stdout;
        truncate_in_place(&mut stdout, CAPTURE_CAP);
        let mut stderr = partial_stderr;
        // Append the timeout marker AFTER any partial output so the
        // auditor sees what the process printed before being killed.
        if stderr.is_empty() {
            stderr = format!("timed out after {secs}s");
        } else {
            stderr.push_str(&format!("\n...[timed out after {secs}s]"));
            truncate_in_place(&mut stderr, CAPTURE_CAP);
        }
        Self {
            project_kind: project,
            step: step.to_string(),
            command,
            exit_code: -1,
            duration_ms: secs * 1000,
            stdout,
            stderr,
            passed: false,
            skipped: false,
        }
    }
}

/// Resolve the effective per-step timeout: the env override wins, then
/// the step's own budget (if non-zero), then the global default.
/// Resolve the effective per-step timeout. Semantics:
/// - A step with its own budget (`step_timeout_secs != 0`, i.e. the slow
///   install/test/build steps) always gets AT LEAST that budget — even when
///   `SUPER_DEV_VERIFY_TIMEOUT_SECS` is set low, so a user who lowers the
///   global default doesn't artificially time out `cargo build --release`.
/// - The env override otherwise raises the default-budget steps (fmt/clippy).
/// - Final fallback: [`DEFAULT_TIMEOUT_SECS`].
fn effective_timeout(step_timeout_secs: u64, global_override: Option<u64>) -> u64 {
    let baseline = global_override.unwrap_or(DEFAULT_TIMEOUT_SECS);
    if step_timeout_secs != 0 {
        baseline.max(step_timeout_secs)
    } else {
        baseline
    }
}

/// Run the full verify step sequence for `workspace`. Returns one
/// [`VerifyOutcome`] per step. Returns an empty vec when no project manifest
/// is present (verify is genuinely meaningless).
///
/// Each step runs independently — a failing step does NOT abort the
/// remaining steps, so the quality gate sees the complete picture (e.g.
/// "lint failed but test + build passed"). The timeout for each step is
/// `SUPER_DEV_VERIFY_TIMEOUT_SECS` (global override, applies to ALL steps)
/// → else the step's own [`VerifyStep::timeout_secs`] → else
/// [`DEFAULT_TIMEOUT_SECS`].
///
/// On timeout the partial stdout/stderr already produced is still captured
/// (up to [`CAPTURE_CAP`]) so the auditor sees the build's last words
/// before the kill, rather than an empty buffer.
pub async fn run_verify(workspace: &Path) -> Vec<VerifyOutcome> {
    let kind = detect_project(workspace);
    let Some(steps) = verify_steps(kind, workspace) else {
        return Vec::new();
    };

    // A global env override, when set, overrides EVERY step's budget.
    let global_override = std::env::var("SUPER_DEV_VERIFY_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok());

    let mut outcomes = Vec::with_capacity(steps.len());
    for step in steps {
        let command_str = format!("{} {}", step.program, step.args.join(" "));
        let started = Instant::now();
        let timeout_secs = effective_timeout(step.timeout_secs, global_override);

        // If the binary isn't on PATH and the step is skippable, record
        // a skip row without spawning (fast + honest).
        if step.skippable && !which(&step.program) {
            outcomes.push(VerifyOutcome::from_spawn_error(
                kind,
                step.name,
                command_str,
                &format!("{} not found on PATH", step.program),
                0,
                true,
            ));
            continue;
        }

        // Take the pipes up-front so we can read whatever the process
        // produced even when it times out. `wait_with_output` would own the
        // pipes and drop partial output on timeout; instead we detach the
        // readers, race wait() against the timer, then drain the buffers.
        let mut child = match Command::new(&step.program)
            .args(&step.args)
            .current_dir(workspace)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                outcomes.push(VerifyOutcome::from_spawn_error(
                    kind,
                    step.name,
                    command_str,
                    &e.to_string(),
                    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
                    step.skippable,
                ));
                continue;
            }
        };

        // Detach the stdout/stderr handles into async read tasks so we can
        // collect partial output regardless of whether the step completes or
        // times out. Each task reads to EOF (which happens when the child
        // exits OR when it's killed on drop after timeout).
        let stdout_handle = child.stdout.take();
        let stderr_handle = child.stderr.take();
        let stdout_task = tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut buf = Vec::with_capacity(CAPTURE_CAP);
            if let Some(mut h) = stdout_handle {
                // Read up to CAPTURE_CAP+1 so we know to truncate.
                let mut chunk = vec![0u8; CAPTURE_CAP + 1];
                loop {
                    match h.read(&mut chunk).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    }
                    if buf.len() > CAPTURE_CAP {
                        buf.truncate(CAPTURE_CAP);
                        break;
                    }
                }
            }
            buf
        });
        let stderr_task = tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut buf = Vec::with_capacity(CAPTURE_CAP);
            if let Some(mut h) = stderr_handle {
                let mut chunk = vec![0u8; CAPTURE_CAP + 1];
                loop {
                    match h.read(&mut chunk).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    }
                    if buf.len() > CAPTURE_CAP {
                        buf.truncate(CAPTURE_CAP);
                        break;
                    }
                }
            }
            buf
        });

        // Race the child's exit against the timeout.
        let wait_result =
            tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait()).await;

        // Whether it exited or timed out, the pipe tasks eventually
        // complete (EOF on exit, EOF after kill_on_drop). Join them.
        let raw_stdout = stdout_task.await.unwrap_or_default();
        let raw_stderr = stderr_task.await.unwrap_or_default();
        let mut stdout = String::from_utf8_lossy(&raw_stdout).into_owned();
        let mut stderr = String::from_utf8_lossy(&raw_stderr).into_owned();
        truncate_in_place(&mut stdout, CAPTURE_CAP);
        truncate_in_place(&mut stderr, CAPTURE_CAP);

        let outcome = match wait_result {
            Ok(Ok(status)) => VerifyOutcome {
                project_kind: kind,
                step: step.name.to_string(),
                command: command_str,
                exit_code: status.code().unwrap_or(-1),
                duration_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
                stdout,
                stderr,
                passed: status.success(),
                skipped: false,
            },
            Ok(Err(e)) => VerifyOutcome::from_spawn_error(
                kind,
                step.name,
                command_str,
                &e.to_string(),
                started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
                step.skippable,
            ),
            // Timed out: kill_on_drop(true) kills the child when `child`
            // drops at end of scope, which closes the pipes → the read
            // tasks return the partial output they collected. Pass it into
            // from_timeout so the audit row shows the build's last words.
            Err(_) => {
                // Best-effort explicit kill so the pipe tasks don't block.
                let _ = child.start_kill();
                VerifyOutcome::from_timeout(
                    kind,
                    step.name,
                    command_str,
                    timeout_secs,
                    stdout,
                    stderr,
                )
            }
        };
        outcomes.push(outcome);
    }

    outcomes
}

/// Append an outcome (plus timestamp + phase tag) to
/// `.super-dev/audit/verify.jsonl`. The runner calls this after every
/// verify step so auditors get a complete chain.
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
        assert_eq!(ProjectKind::Go.as_str(), "go");
        assert_eq!(ProjectKind::Deno.as_str(), "deno");
        assert_eq!(ProjectKind::None.as_str(), "none");
    }

    #[test]
    fn verify_command_returns_none_for_no_project() {
        let tmp = TempDir::new().unwrap();
        assert!(ProjectKind::None.verify_command(tmp.path()).is_none());
        assert!(ProjectKind::Rust.verify_command(tmp.path()).is_some());
    }

    #[test]
    fn node_pm_picks_pnpm_from_lockfile() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), r#"{"name":"x"}"#).unwrap();
        fs::write(tmp.path().join("pnpm-lock.yaml"), "lockfileVersion: 9").unwrap();
        let (pm, _) = ProjectKind::Node.verify_command(tmp.path()).unwrap();
        assert_eq!(pm, "pnpm");
    }

    #[test]
    fn node_pm_picks_yarn_from_lockfile() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), r#"{"name":"x"}"#).unwrap();
        fs::write(tmp.path().join("yarn.lock"), "# yarn lockfile").unwrap();
        let (pm, _) = ProjectKind::Node.verify_command(tmp.path()).unwrap();
        assert_eq!(pm, "yarn");
    }

    #[test]
    fn node_pm_picks_bun_from_lockfile() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), r#"{"name":"x"}"#).unwrap();
        fs::write(tmp.path().join("bun.lockb"), "").unwrap();
        let (pm, _) = ProjectKind::Node.verify_command(tmp.path()).unwrap();
        assert_eq!(pm, "bun");
    }

    #[test]
    fn node_pm_defaults_to_npm_without_lockfile() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), r#"{"name":"x"}"#).unwrap();
        let (pm, _) = ProjectKind::Node.verify_command(tmp.path()).unwrap();
        assert_eq!(pm, "npm");
    }

    #[test]
    fn detect_go_project() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("go.mod"), "module x\n\ngo 1.22").unwrap();
        assert_eq!(detect_project(tmp.path()), ProjectKind::Go);
    }

    #[test]
    fn detect_deno_project() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("deno.json"), "{}").unwrap();
        assert_eq!(detect_project(tmp.path()), ProjectKind::Deno);
    }

    #[test]
    fn deno_takes_priority_over_node() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("deno.json"), "{}").unwrap();
        fs::write(tmp.path().join("package.json"), r#"{"name":"x"}"#).unwrap();
        assert_eq!(detect_project(tmp.path()), ProjectKind::Deno);
    }

    #[tokio::test]
    async fn run_verify_returns_empty_when_no_manifest() {
        let tmp = TempDir::new().unwrap();
        assert!(run_verify(tmp.path()).await.is_empty());
    }

    #[tokio::test]
    async fn run_verify_python_produces_steps() {
        // A Python project produces multiple steps (install, lint, test...).
        // Steps whose binary is missing are `skipped`, not failures.
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("pyproject.toml"), "[project]\nname=\"x\"").unwrap();
        let outcomes = run_verify(tmp.path()).await;
        assert!(!outcomes.is_empty(), "python project must produce ≥1 step");
        // Every outcome has a step name.
        assert!(outcomes.iter().all(|o| !o.step.is_empty()));
    }

    #[tokio::test]
    async fn run_verify_rust_produces_step_sequence() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"",
        )
        .unwrap();
        let outcomes = run_verify(tmp.path()).await;
        let names: Vec<&str> = outcomes.iter().map(|o| o.step.as_str()).collect();
        assert!(
            names.contains(&"test"),
            "rust sequence must include test: {names:?}"
        );
        assert!(
            names.contains(&"build"),
            "rust sequence must include build: {names:?}"
        );
    }

    #[tokio::test]
    async fn record_outcome_writes_jsonl_line() {
        let tmp = TempDir::new().unwrap();
        let outcome = VerifyOutcome {
            project_kind: ProjectKind::Rust,
            step: "test".into(),
            command: "cargo test".into(),
            exit_code: 1,
            duration_ms: 42,
            stdout: String::new(),
            stderr: "error[E0599]: no method named foo".into(),
            passed: false,
            skipped: false,
        };
        let path = record_verify_outcome(tmp.path(), "frontend", &outcome).unwrap();
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("\"phase\":\"frontend\""));
        assert!(body.contains("\"project_kind\":\"rust\""));
        assert!(body.contains("\"passed\":false"));
        assert!(body.contains("\"step\":\"test\""));
        assert!(body.contains("E0599"));
    }

    #[tokio::test]
    async fn record_outcome_appends_multiple_lines() {
        let tmp = TempDir::new().unwrap();
        for i in 0..3 {
            let outcome = VerifyOutcome {
                project_kind: ProjectKind::Node,
                step: "install".into(),
                command: format!("npm install (run {i})"),
                exit_code: 0,
                duration_ms: 10,
                stdout: String::new(),
                stderr: String::new(),
                passed: true,
                skipped: false,
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

    #[test]
    fn verify_steps_rust_has_four_steps() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
        let steps = verify_steps(ProjectKind::Rust, tmp.path()).unwrap();
        assert_eq!(steps.len(), 4);
        assert_eq!(steps[0].name, "fmt");
        assert_eq!(steps[1].name, "clippy");
        assert_eq!(steps[2].name, "test");
        assert_eq!(steps[3].name, "build");
    }

    #[test]
    fn verify_steps_rust_assigns_larger_budget_to_slow_steps() {
        // Slow steps (test, build) must get the SLOW budget; fast steps
        // (fmt, clippy) keep the 0 → global-default sentinel.
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
        let steps = verify_steps(ProjectKind::Rust, tmp.path()).unwrap();
        let by_name: std::collections::HashMap<&str, u64> =
            steps.iter().map(|s| (s.name, s.timeout_secs)).collect();
        assert_eq!(by_name["fmt"], 0, "fmt is fast → 0 (global default)");
        assert_eq!(by_name["clippy"], 0, "clippy is fast → 0 (global default)");
        assert_eq!(
            by_name["test"], SLOW_STEP_TIMEOUT_SECS,
            "test must get the slow budget"
        );
        assert_eq!(
            by_name["build"], SLOW_STEP_TIMEOUT_SECS,
            "build must get the slow budget"
        );
    }

    #[test]
    fn effective_timeout_slow_step_keeps_its_budget() {
        // A slow step keeps AT LEAST its own budget, even when the env
        // override is lower — so a low global cap can't time out
        // `cargo build --release`.
        assert_eq!(effective_timeout(0, None), DEFAULT_TIMEOUT_SECS);
        assert_eq!(
            effective_timeout(300, None),
            300,
            "slow step default = its budget"
        );
        assert_eq!(effective_timeout(0, Some(45)), 45, "fast step honours env");
        // Slow budget (300) wins over a lower env (45):
        assert_eq!(
            effective_timeout(300, Some(45)),
            300,
            "slow step must keep its 300s budget even when env is 45"
        );
        // Higher env (900) wins over slow budget (300):
        assert_eq!(
            effective_timeout(300, Some(900)),
            900,
            "a higher env override raises the slow step too"
        );
    }

    #[test]
    fn verify_steps_node_includes_test_only_when_script_exists() {
        let tmp = TempDir::new().unwrap();
        // No test/build scripts → just install step.
        fs::write(tmp.path().join("package.json"), r#"{"name":"x"}"#).unwrap();
        let steps = verify_steps(ProjectKind::Node, tmp.path()).unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].name, "install");

        // With test + build scripts → 3 steps.
        fs::write(
            tmp.path().join("package.json"),
            r#"{"name":"x","scripts":{"test":"jest","build":"tsc"}}"#,
        )
        .unwrap();
        let steps = verify_steps(ProjectKind::Node, tmp.path()).unwrap();
        let names: Vec<&str> = steps.iter().map(|s| s.name).collect();
        assert!(names.contains(&"test"));
        assert!(names.contains(&"build"));
    }

    #[test]
    fn verify_steps_none_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(verify_steps(ProjectKind::None, tmp.path()).is_none());
    }

    #[test]
    fn package_json_depends_on_detects_declared_deps() {
        // Regression: the typecheck picker used to check
        // node_modules/.bin/vue-tsc (which doesn't exist at step-BUILD time,
        // before install runs). Now it reads package.json declarations.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"name":"x","devDependencies":{"vue-tsc":"^2.0.0","typescript":"^5.0.0"}}"#,
        )
        .unwrap();
        assert!(package_json_depends_on(tmp.path(), "vue-tsc"));
        assert!(package_json_depends_on(tmp.path(), "typescript"));
        assert!(!package_json_depends_on(tmp.path(), "eslint"));
    }

    #[test]
    fn has_node_script_detects_scripts() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"scripts":{"lint":"eslint .","test":"jest"}}"#,
        )
        .unwrap();
        assert!(has_node_script(tmp.path(), "lint"));
        assert!(has_node_script(tmp.path(), "test"));
        assert!(!has_node_script(tmp.path(), "build"));
    }

    #[test]
    fn spawn_error_non_skippable_is_failure() {
        // Regression: a missing NON-skippable binary (e.g. `cargo` not on
        // PATH for a Rust project) MUST report passed=false so the gate
        // surfaces it. Previously `from_spawn_error` returned
        // `passed = !skippable` which inverted this case to passed=true.
        let outcome = VerifyOutcome::from_spawn_error(
            ProjectKind::Rust,
            "build",
            "cargo build".to_string(),
            "No such file or directory",
            0,
            false, // non-skippable
        );
        assert!(!outcome.passed, "non-skippable spawn miss must fail");
        assert!(!outcome.skipped);
    }

    #[test]
    fn spawn_error_skippable_is_neutral() {
        let outcome = VerifyOutcome::from_spawn_error(
            ProjectKind::Rust,
            "fmt",
            "cargo fmt --check".to_string(),
            "No such file or directory",
            0,
            true, // skippable
        );
        assert!(
            outcome.passed,
            "skippable spawn miss is neutral (pass+skip)"
        );
        assert!(outcome.skipped);
    }

    #[test]
    fn from_timeout_appends_marker_to_partial_stderr() {
        // When partial output exists, the timeout marker is appended so the
        // auditor sees the build's last words + the cause.
        let o = VerifyOutcome::from_timeout(
            ProjectKind::Rust,
            "build",
            "cargo build".into(),
            30,
            "Compiling foo\n".into(),
            "warning: unused".into(),
        );
        assert!(o.stderr.contains("warning: unused"));
        assert!(
            o.stderr.contains("timed out after 30s"),
            "stderr was: {}",
            o.stderr
        );
        assert!(
            o.stdout.contains("Compiling foo"),
            "partial stdout preserved"
        );
    }

    #[test]
    fn from_timeout_uses_marker_when_no_partial_output() {
        let o = VerifyOutcome::from_timeout(
            ProjectKind::Rust,
            "build",
            "cargo build".into(),
            30,
            String::new(),
            String::new(),
        );
        assert_eq!(o.stderr, "timed out after 30s");
    }
}
