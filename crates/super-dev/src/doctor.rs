//! `super-dev doctor` — self-test that diagnoses common
//! "installed-but-not-working" situations.
//!
//! Checks performed (4.2 — post-plugin-removal):
//! 1. Binary identity (`CARGO_PKG_VERSION` + spec version).
//! 2. Embedded spec markdown is non-empty + carries the version marker.
//! 3. Workspace is writable (write + delete a tmp file).
//! 4. SD-META-001 spec manifest present and version-aligned.
//!
//! Plugin-related checks (installed bundles, version drift) were
//! removed in 4.2 when the injection-style plugin architecture was
//! dropped — Super Dev now drives host CLIs externally and does not
//! install files into them.

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
    vec![
        check_binary_identity(),
        check_embedded_spec(),
        check_workspace_writable(workspace),
        check_spec_manifest(workspace),
    ]
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
    fn run_all_returns_four_checks_on_empty_workspace() {
        let tmp = TempDir::new().unwrap();
        let results = run_all(tmp.path());
        assert_eq!(results.len(), 4);
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
}
