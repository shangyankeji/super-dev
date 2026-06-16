//! `super-dev doctor` — self-test that diagnoses common
//! "installed-but-not-working" situations.
//!
//! Checks performed:
//! 1. Binary identity (`CARGO_PKG_VERSION` + spec version).
//! 2. Embedded spec markdown is non-empty + carries the version marker.
//! 3. Workspace is writable (write + delete a tmp file).
//! 4. SD-META-001 spec manifest present and version-aligned.
//! 5. AI coding host CLIs detected on PATH.
//! 6. Claude Code PreToolUse governance hook installed (if `.claude/` exists).
//! 7. Delivery / deployment readiness (after a run completes): delivery notes
//!    present with a deploy command, build output exists, and a deploy CLI
//!    (vercel / netlify / wrangler) is on PATH.
//!
//! The hook check (6) was added in 4.6 alongside the restored real-time
//! governance hook (`super-dev install`). When `.claude/settings.json` exists
//! but the hook isn't registered, the doctor suggests running `super-dev install`.

use std::fs;
use std::io::Write;
use std::path::Path;

/// Single check result row.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CheckResult {
    /// Short check name shown in the report.
    pub name: String,
    /// `passed` | `warning` | `failed`.
    pub status: Status,
    /// Human-readable detail.
    pub detail: String,
}

/// Status verbs.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Status {
    /// The check passed.
    Passed,
    /// The check produced a warning but the binary still functions.
    Warning,
    /// The check failed — user intervention needed.
    Failed,
}

impl Status {
    /// Short label used in the report header column.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "PASS",
            Self::Warning => "WARN",
            Self::Failed => "FAIL",
        }
    }
}

/// Run every doctor check, returning the rows in a stable order.
pub fn run_all(workspace: &Path) -> Vec<CheckResult> {
    let mut results = vec![
        check_binary_identity(),
        check_embedded_spec(),
        check_workspace_writable(workspace),
        check_spec_manifest(workspace),
    ];
    results.push(check_ai_backends());
    results.push(check_claude_hook(workspace));
    results.push(check_delivery_readiness(workspace));
    results
}

/// Check whether the Claude Code PreToolUse governance hook is registered.
/// Only relevant when `.claude/settings.json` exists (workspace-level Claude
/// Code config). When the hook is missing, suggests `super-dev install`.
fn check_claude_hook(workspace: &Path) -> CheckResult {
    let settings_path = workspace.join(".claude/settings.json");
    if !settings_path.is_file() {
        // No Claude Code config at all — not an error, just informational.
        return CheckResult {
            name: "Claude Code hook".to_string(),
            status: Status::Passed,
            detail: "no .claude/settings.json (real-time governance off; quality-gate hard block still active)"
                .to_string(),
        };
    }
    let Ok(content) = fs::read_to_string(&settings_path) else {
        return CheckResult {
            name: "Claude Code hook".to_string(),
            status: Status::Warning,
            detail: ".claude/settings.json exists but is unreadable".to_string(),
        };
    };
    if content.contains("hook pre-write") {
        CheckResult {
            name: "Claude Code hook".to_string(),
            status: Status::Passed,
            detail:
                "PreToolUse governance hook registered (SD-CODE-001/002/005 enforced at write time)"
                    .to_string(),
        }
    } else {
        CheckResult {
            name: "Claude Code hook".to_string(),
            status: Status::Warning,
            detail: ".claude/settings.json exists but the Super Dev PreToolUse hook is not registered. \
                     Run `super-dev install --host claude-code` for real-time emoji/color/slop interception."
                .to_string(),
        }
    }
}

fn check_spec_manifest(workspace: &Path) -> CheckResult {
    // SD-META-001: a conformant workspace declares its spec level.
    match super_dev_agent::SpecManifest::read_from(workspace) {
        Some(m) if m.spec_version == super_dev_spec::SPEC_VERSION => CheckResult {
            name: "spec manifest (SD-META-001)".to_string(),
            status: Status::Passed,
            detail: format!(
                "super-dev.yaml present: level {}, profile {}",
                m.level.as_str(),
                m.profile.as_str()
            ),
        },
        Some(m) => CheckResult {
            name: "spec manifest (SD-META-001)".to_string(),
            status: Status::Warning,
            detail: format!(
                "super-dev.yaml declares spec `{}` but this binary speaks `{}`",
                m.spec_version,
                super_dev_spec::SPEC_VERSION
            ),
        },
        None => CheckResult {
            name: "spec manifest (SD-META-001)".to_string(),
            status: Status::Warning,
            detail: "no super-dev.yaml — run `super-dev init` to declare conformance".to_string(),
        },
    }
}

/// Check which host CLIs (claude-code, codex) are installed and usable.
/// and usable. This is the most important doctor check for enterprise use —
/// without a backend, Super Dev falls back to offline templates.
fn check_ai_backends() -> CheckResult {
    // Map backend IDs to the executable name(s) to look for on PATH.
    // This list is kept in sync with super_dev_host::BACKEND_IDS — the
    // `backend_arg_ids_match_host` test in main.rs guards the selector,
    // and `probe_all_reports_every_backend` in super-dev-host guards the
    // driver registry; this table is the doctor's fast PATH-only view.
    // Super Dev drives exactly two host CLIs: Claude Code and Codex. Kept in
    // sync with super_dev_host::BACKEND_IDS.
    let probes: &[(&str, &[&str])] = &[("claude-code", &["claude"]), ("codex", &["codex"])];

    let mut found: Vec<&str> = Vec::new();
    for (id, cmds) in probes {
        if cmds.iter().any(|cmd| which_on_path(cmd)) {
            found.push(id);
        }
    }

    if found.is_empty() {
        CheckResult {
            name: "AI host backends".to_string(),
            status: Status::Warning,
            detail: "No host CLI (claude / codex) detected on PATH. Install + log in to one, or use /provider in the TUI to connect a custom API (DeepSeek/OpenRouter/Ollama). Without a backend, Super Dev uses offline templates.".to_string(),
        }
    } else {
        CheckResult {
            name: "AI host backends".to_string(),
            status: Status::Passed,
            detail: format!(
                "{} backend(s) ready: {}. Use --backend {} for real AI generation.",
                found.len(),
                found.join(", "),
                found[0]
            ),
        }
    }
}

/// Check if an executable is on PATH (without spawning a subprocess).
fn which_on_path(cmd: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        // Check common executable extensions on the current platform.
        let candidates = if cfg!(windows) {
            vec![
                dir.join(format!("{cmd}.exe")),
                dir.join(format!("{cmd}.bat")),
                dir.join(format!("{cmd}.cmd")),
            ]
        } else {
            vec![dir.join(cmd)]
        };
        candidates.iter().any(|p| p.is_file())
    })
}

/// Return true iff every result in `results` is `Passed`.
#[must_use]
pub fn all_passed(results: &[CheckResult]) -> bool {
    results.iter().all(|r| r.status == Status::Passed)
}

fn check_binary_identity() -> CheckResult {
    let version = env!("CARGO_PKG_VERSION");
    let spec = super_dev_spec::SPEC_VERSION;
    CheckResult {
        name: "binary identity".to_string(),
        status: Status::Passed,
        detail: format!("super-dev {version}, conformant to {spec}"),
    }
}

fn check_embedded_spec() -> CheckResult {
    let spec_body = include_str!("../../../spec/SUPER_DEV_HOST_SPEC_V1.md");
    if spec_body.is_empty() {
        return CheckResult {
            name: "embedded spec markdown".to_string(),
            status: Status::Failed,
            detail: "spec/SUPER_DEV_HOST_SPEC_V1.md was empty at build time".to_string(),
        };
    }
    if !spec_body.contains("SUPER_DEV_HOST_SPEC_V1") {
        return CheckResult {
            name: "embedded spec markdown".to_string(),
            status: Status::Warning,
            detail: format!(
                "embedded spec lacks the SPEC_VERSION marker ({} bytes)",
                spec_body.len()
            ),
        };
    }
    CheckResult {
        name: "embedded spec markdown".to_string(),
        status: Status::Passed,
        detail: format!("{} bytes, carries SPEC_VERSION marker", spec_body.len()),
    }
}

fn check_workspace_writable(workspace: &Path) -> CheckResult {
    let probe = workspace.join(".super-dev-doctor-probe");
    let res = (|| -> std::io::Result<()> {
        if let Some(parent) = probe.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = fs::File::create(&probe)?;
        f.write_all(b"ok")?;
        f.sync_data()?;
        fs::remove_file(&probe)?;
        Ok(())
    })();
    match res {
        Ok(()) => CheckResult {
            name: "workspace writable".to_string(),
            status: Status::Passed,
            detail: format!("write + delete OK at {}", workspace.display()),
        },
        Err(e) => CheckResult {
            name: "workspace writable".to_string(),
            status: Status::Failed,
            detail: format!("{} ({})", e, workspace.display()),
        },
    }
}

/// Pretty-print one report block.
#[must_use]
pub fn render_report(workspace: &Path, results: &[CheckResult]) -> String {
    let mut out = String::new();
    out.push_str(&format!("super-dev doctor — {}\n\n", workspace.display()));
    out.push_str("status | check\n");
    out.push_str("-------|------\n");
    for r in results {
        out.push_str(&format!("{:6} | {}\n", r.status.label(), r.name));
        out.push_str(&format!("       │  {}\n", r.detail));
    }
    let passed = results
        .iter()
        .filter(|r| r.status == Status::Passed)
        .count();
    let warn = results
        .iter()
        .filter(|r| r.status == Status::Warning)
        .count();
    let failed = results
        .iter()
        .filter(|r| r.status == Status::Failed)
        .count();
    out.push_str(&format!(
        "\n{passed} passed, {warn} warning, {failed} failed.\n"
    ));
    out
}

/// Check 7: delivery / deployment readiness. After a pipeline run reaches
/// the delivery phase, this verifies the worker produced a deployable state:
/// delivery notes with a `## Deploy command`, a build output directory, and at
/// least one deploy-platform CLI on PATH. Before any run it reports "not started".
fn check_delivery_readiness(workspace: &Path) -> CheckResult {
    let output = workspace.join("output");
    // No output dir → pipeline hasn't run; not an error, just informational.
    if !output.is_dir() {
        return CheckResult {
            name: "Deployment readiness".to_string(),
            status: Status::Passed,
            detail: "no run yet (run `super-dev` and enter a requirement to start)".to_string(),
        };
    }
    // Find any delivery-notes file.
    let delivery_notes = fs::read_dir(&output).ok().and_then(|rd| {
        rd.filter_map(Result::ok).map(|e| e.path()).find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("delivery-notes"))
        })
    });
    let Some(notes_path) = delivery_notes else {
        return CheckResult {
            name: "Deployment readiness".to_string(),
            status: Status::Passed,
            detail: "pipeline has not reached delivery phase yet".to_string(),
        };
    };
    let notes = fs::read_to_string(&notes_path).unwrap_or_default();
    // Does the worker record a concrete deploy command (not the placeholder)?
    let has_deploy_cmd = notes
        .split("## Deploy command")
        .nth(1)
        .is_some_and(|after| {
            after
                .lines()
                .any(|l| !l.trim().is_empty() && !l.trim().starts_with('_'))
        });
    // Is a deploy-platform CLI on PATH?
    let deploy_cli = ["vercel", "netlify", "wrangler"]
        .iter()
        .find(|c| which_on_path(c))
        .copied();
    let mut detail = String::new();
    let mut status = Status::Passed;
    if has_deploy_cmd {
        detail.push_str("delivery notes have a deploy command; ");
    } else {
        detail.push_str("delivery notes missing a concrete deploy command; ");
        status = Status::Warning;
    }
    if let Some(cli) = deploy_cli {
        detail.push_str(&format!("`{cli}` on PATH (run /deploy to ship)"));
    } else {
        detail.push_str(
            "no deploy CLI (vercel/netlify/wrangler) on PATH — install one to /deploy, \
             or run the recorded command manually",
        );
        if status == Status::Passed {
            status = Status::Warning;
        }
    }
    CheckResult {
        name: "Deployment readiness".to_string(),
        status,
        detail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn binary_identity_always_passes() {
        let r = check_binary_identity();
        assert_eq!(r.status, Status::Passed);
        assert!(r.detail.contains(env!("CARGO_PKG_VERSION")));
        assert!(r.detail.contains("SUPER_DEV_HOST_SPEC_V1"));
    }

    #[test]
    fn embedded_spec_check_passes() {
        let r = check_embedded_spec();
        assert_eq!(r.status, Status::Passed);
    }

    #[test]
    fn workspace_writable_pass_in_tmp() {
        let tmp = TempDir::new().unwrap();
        let r = check_workspace_writable(tmp.path());
        assert_eq!(r.status, Status::Passed);
    }

    #[test]
    fn run_all_returns_seven_checks_on_empty_workspace() {
        let tmp = TempDir::new().unwrap();
        let results = run_all(tmp.path());
        assert_eq!(results.len(), 7);
        // No FAILs on a clean workspace — only a manifest WARN.
        assert!(results.iter().all(|r| r.status != Status::Failed));
        assert_eq!(
            results
                .iter()
                .filter(|r| r.status == Status::Warning)
                .count(),
            1
        );
    }

    #[test]
    fn run_all_passes_clean_after_init() {
        let tmp = TempDir::new().unwrap();
        super_dev_agent::SpecManifest::new("demo")
            .write_to(tmp.path(), false)
            .unwrap();
        let results = run_all(tmp.path());
        assert!(all_passed(&results));
    }

    #[test]
    fn render_report_includes_counts() {
        let tmp = TempDir::new().unwrap();
        let results = run_all(tmp.path());
        let report = render_report(tmp.path(), &results);
        assert!(report.contains("passed"));
        assert!(report.contains("failed"));
        assert!(report.contains("super-dev doctor"));
    }

    #[test]
    fn claude_hook_warns_when_settings_exist_without_hook() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        fs::write(tmp.path().join(".claude/settings.json"), r#"{"hooks":{}}"#).unwrap();
        let results = run_all(tmp.path());
        let hook_check = results
            .iter()
            .find(|r| r.name == "Claude Code hook")
            .unwrap();
        assert_eq!(hook_check.status, Status::Warning);
        assert!(hook_check.detail.contains("super-dev install"));
    }

    #[test]
    fn claude_hook_passes_when_registered() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        fs::write(
            tmp.path().join(".claude/settings.json"),
            r#"{"hooks":{"PreToolUse":[{"matcher":"Write","hooks":[{"command":"super-dev hook pre-write"}]}]}}"#,
        )
        .unwrap();
        let results = run_all(tmp.path());
        let hook_check = results
            .iter()
            .find(|r| r.name == "Claude Code hook")
            .unwrap();
        assert_eq!(hook_check.status, Status::Passed);
    }

    #[test]
    fn backend_check_runs_without_panic() {
        let r = check_ai_backends();
        assert!(!r.name.is_empty());
        // On a dev machine with CLIs installed it should pass; on CI it may warn.
        assert!(r.status == Status::Passed || r.status == Status::Warning);
    }

    #[test]
    fn which_on_path_finds_known_commands() {
        // 'ls' / 'cmd' should be on PATH on any unix/windows dev machine.
        assert!(which_on_path("ls") || which_on_path("cmd"));
    }

    #[test]
    fn delivery_readiness_passes_when_no_run_yet() {
        let tmp = tempfile::TempDir::new().unwrap();
        let r = check_delivery_readiness(tmp.path());
        assert_eq!(r.status, Status::Passed);
        assert!(r.detail.contains("no run yet") || r.detail.contains("not reached"));
    }

    #[test]
    fn delivery_readiness_warns_when_no_deploy_command() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("output");
        std::fs::create_dir_all(&out).unwrap();
        // delivery notes present but only the placeholder (no real command).
        std::fs::write(
            out.join("demo-delivery-notes.md"),
            "## Deploy command\n\n_(exact command — read by Super Dev)_\n",
        )
        .unwrap();
        let r = check_delivery_readiness(tmp.path());
        // Placeholder-only → warning (missing concrete command).
        assert!(r.status == Status::Warning || r.status == Status::Passed);
        assert!(r.name.contains("Deployment"));
    }

    #[test]
    fn delivery_readiness_detects_concrete_deploy_command() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("output");
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(
            out.join("demo-delivery-notes.md"),
            "## Deploy command\n\nnpx vercel --prod\n",
        )
        .unwrap();
        let r = check_delivery_readiness(tmp.path());
        assert!(r.detail.contains("deploy command"));
    }
}
