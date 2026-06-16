//! Persistent workflow state — `.super-dev/workflow-state.json`.
//!
//! Implements the persistence side of `SD-FLOW-001`. The agent updates
//! this file on every phase transition; the prompt-time injection hook
//! reads it back to ground the model.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super_dev_spec::Phase;

/// Snapshot of pipeline progress for one project.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Active phase, e.g. `frontend`. Stored as the phase identifier
    /// string so the on-disk schema is stable across versions.
    pub phase: String,
    /// Active gate, e.g. `docs_confirm`. Empty string when no gate is
    /// open.
    #[serde(default)]
    pub active_gate: String,
    /// Project slug used in artifact filenames; persisted so `continue`
    /// invocations resolve the same files as the original `run`.
    #[serde(default)]
    pub slug: String,
    /// Original user requirement (carried through the pipeline so
    /// continuation invocations don't lose context).
    #[serde(default)]
    pub requirement: String,
    /// ISO-8601 UTC timestamp of the last transition.
    pub last_transition_at: String,
    /// Free-form note about the latest action. Empty by default.
    #[serde(default)]
    pub note: String,
    /// Backend id that produced this state (e.g. `claude-code`, `codex`,
    /// or empty for offline). Persisted so `continue` / `revise` resume
    /// against the same worker the original `run` used.
    #[serde(default)]
    pub backend: String,
    /// Spec version the agent is conformant against.
    pub spec_version: String,
}

impl WorkflowState {
    /// Build a fresh state pinned to a given phase.
    #[must_use]
    pub fn new(phase: Phase) -> Self {
        Self {
            phase: phase.id().to_string(),
            active_gate: String::new(),
            slug: String::new(),
            requirement: String::new(),
            last_transition_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            note: String::new(),
            backend: String::new(),
            spec_version: super_dev_spec::SPEC_VERSION.to_string(),
        }
    }
}

/// Persist the workflow state to `<project_root>/.super-dev/workflow-state.json`.
///
/// The write is atomic: the JSON is first written to a sibling `.tmp`
/// file and then renamed into place. A `rename(2)` on the same volume is
/// atomic on POSIX and Windows, so a crash mid-write can never leave a
/// truncated / malformed `workflow-state.json` that would strand the
/// pipeline in an unreadable state. Disk-write errors propagate so
/// callers can decide whether to surface or swallow them.
/// Before overwriting, copy the current `workflow-state.json` (if it exists)
/// into `.super-dev/history/<timestamp>.json`. This makes every transition
/// recoverable — `super-dev rollback` can restore any prior state. Best-effort:
/// a missing current file or a history-write failure is silently skipped
/// (the atomic overwrite below must still proceed).
fn snapshot_previous(project_root: &Path) {
    let current = project_root.join(".super-dev").join("workflow-state.json");
    let Some(text) = fs::read_to_string(&current).ok() else {
        return; // first-ever write — nothing to snapshot
    };
    let history_dir = project_root.join(".super-dev/history");
    let _ = fs::create_dir_all(&history_dir);
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%S%.f");
    let _ = fs::write(history_dir.join(format!("{ts}.json")), text);
}

/// Persist the workflow state to `<project_root>/.super-dev/workflow-state.json`.
///
/// The write is atomic: the JSON is first written to a sibling `.tmp`
/// file and then renamed into place. Before overwriting, the previous state
/// is snapshotted to `.super-dev/history/` (see [`snapshot_previous`]) so
/// every transition is recoverable via `super-dev rollback`.
pub fn write_workflow_state(project_root: &Path, state: &WorkflowState) -> std::io::Result<()> {
    snapshot_previous(project_root);
    let dir = project_root.join(".super-dev");
    fs::create_dir_all(&dir)?;
    let text = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let final_path = dir.join("workflow-state.json");
    let tmp_path = dir.join("workflow-state.json.tmp");
    fs::write(&tmp_path, text)?;
    fs::rename(&tmp_path, &final_path)
}

/// List available rollback snapshots, newest first. Each entry is the
/// timestamp filename stem (e.g. `20260614T120000Z`). Empty when no history.
#[must_use]
pub fn list_snapshots(project_root: &Path) -> Vec<String> {
    let history_dir = project_root.join(".super-dev/history");
    let Ok(rd) = fs::read_dir(&history_dir) else {
        return Vec::new();
    };
    let mut snaps: Vec<String> = rd
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("json") {
                p.file_stem()?.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect();
    snaps.sort_unstable();
    snaps.reverse(); // newest first
    snaps
}

/// Restore the workflow state from a snapshot timestamp (e.g. `20260614T120000Z`).
/// Copies the snapshot over `workflow-state.json` (via the atomic write path so
/// the restore itself is snapshotted). Returns `Err` when the snapshot is missing.
pub fn restore_snapshot(project_root: &Path, timestamp: &str) -> std::io::Result<()> {
    let snap = project_root
        .join(".super-dev/history")
        .join(format!("{timestamp}.json"));
    let text = fs::read_to_string(&snap).map_err(|e| {
        std::io::Error::new(e.kind(), format!("snapshot {timestamp} not found: {e}"))
    })?;
    let state: WorkflowState = serde_json::from_str(&text)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    write_workflow_state(project_root, &state)
}

/// Outcome of [`read_workflow_state_diagnostic`] — distinguishes the three
/// reasons a read can fail so callers (and the CLI) can surface the right
/// message instead of conflating "no run started" with "state corrupted".
#[derive(Debug)]
pub enum ReadState {
    /// The state file exists and parsed.
    Ok(WorkflowState),
    /// No state file — the user has not started a run yet.
    Missing,
    /// The file exists but is unparseable. Carries the file path + parse
    /// error so the CLI can tell the user exactly what's wrong (and that
    /// they should `rollback` or delete it) rather than silently starting
    /// a fresh run over a corrupted state.
    Corrupt {
        /// Path to the unreadable state file.
        path: PathBuf,
        /// The read/parse error message.
        error: String,
    },
}

/// Read the workflow state with a full diagnostic outcome. Prefer this over
/// [`read_workflow_state`] when the caller can act on corruption (the CLI's
/// `continue` / `revise` do); [`read_workflow_state`] remains for
/// best-effort reads where `None` is an acceptable fallback.
pub fn read_workflow_state_diagnostic(project_root: &Path) -> ReadState {
    let path = project_root.join(".super-dev").join("workflow-state.json");
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return ReadState::Missing,
        Err(e) => {
            return ReadState::Corrupt {
                path,
                error: format!("read error: {e}"),
            }
        }
    };
    match serde_json::from_str::<WorkflowState>(&text) {
        Ok(s) => ReadState::Ok(s),
        Err(e) => ReadState::Corrupt {
            path,
            error: format!("parse error: {e}"),
        },
    }
}

/// Read the workflow state. Returns `None` when the file is missing OR
/// malformed. For callers that need to tell those apart (to warn on
/// corruption instead of silently restarting), use
/// [`read_workflow_state_diagnostic`].
#[must_use]
pub fn read_workflow_state(project_root: &Path) -> Option<WorkflowState> {
    match read_workflow_state_diagnostic(project_root) {
        ReadState::Ok(s) => Some(s),
        ReadState::Missing => None,
        ReadState::Corrupt { path, error } => {
            // Surface corruption in the log so a silent restart over a
            // broken state isn't completely invisible. Best-effort: still
            // returns None so fail-open callers behave as before.
            eprintln!(
                "warn: workflow-state.json at {} is corrupt ({error}); \
                 treating as no run started. Run `super-dev rollback latest` \
                 or delete the file to recover.",
                path.display()
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn round_trips_to_disk() {
        let tmp = TempDir::new().unwrap();
        let s = WorkflowState::new(Phase::Frontend);
        write_workflow_state(tmp.path(), &s).unwrap();
        let back = read_workflow_state(tmp.path()).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn read_returns_none_for_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(read_workflow_state(tmp.path()).is_none());
    }

    #[test]
    fn backend_field_roundtrips() {
        let tmp = TempDir::new().unwrap();
        let mut s = WorkflowState::new(Phase::Docs);
        s.backend = "claude-code".into();
        s.active_gate = "docs_confirm".into();
        write_workflow_state(tmp.path(), &s).unwrap();
        let back = read_workflow_state(tmp.path()).unwrap();
        assert_eq!(back.backend, "claude-code");
    }

    #[test]
    fn atomic_write_leaves_no_tmp_file() {
        let tmp = TempDir::new().unwrap();
        let s = WorkflowState::new(Phase::Backend);
        write_workflow_state(tmp.path(), &s).unwrap();
        let tmp_path = tmp.path().join(".super-dev/workflow-state.json.tmp");
        assert!(
            !tmp_path.exists(),
            "temp file should be renamed away, not left behind"
        );
        assert!(tmp.path().join(".super-dev/workflow-state.json").is_file());
    }

    #[test]
    fn overwrite_preserves_readability() {
        // Writing twice must always leave a readable file (the rename is
        // atomic, so the second write never corrupts the first).
        let tmp = TempDir::new().unwrap();
        let mut s = WorkflowState::new(Phase::Docs);
        write_workflow_state(tmp.path(), &s).unwrap();
        s.phase = Phase::Frontend.id().to_string();
        write_workflow_state(tmp.path(), &s).unwrap();
        let back = read_workflow_state(tmp.path()).unwrap();
        assert_eq!(back.phase, "frontend");
    }

    #[test]
    fn transition_snapshots_previous_state() {
        let tmp = TempDir::new().unwrap();
        let mut s = WorkflowState::new(Phase::Docs);
        write_workflow_state(tmp.path(), &s).unwrap();
        // No history yet.
        assert!(list_snapshots(tmp.path()).is_empty());

        // Second write (transition) snapshots the first.
        s.phase = Phase::Frontend.id().to_string();
        write_workflow_state(tmp.path(), &s).unwrap();
        let snaps = list_snapshots(tmp.path());
        assert_eq!(snaps.len(), 1, "previous state should be snapshotted");
        // Current file reflects the new state.
        let back = read_workflow_state(tmp.path()).unwrap();
        assert_eq!(back.phase, "frontend");
    }

    #[test]
    fn multiple_transitions_accumulate_history() {
        let tmp = TempDir::new().unwrap();
        for phase in [Phase::Research, Phase::Docs, Phase::Spec, Phase::Frontend] {
            let s = WorkflowState::new(phase);
            write_workflow_state(tmp.path(), &s).unwrap();
        }
        // 3 transitions after the first → 3 snapshots.
        let snaps = list_snapshots(tmp.path());
        assert_eq!(snaps.len(), 3);
        // Newest first.
        assert!(snaps[0] >= snaps[1]);
    }

    #[test]
    fn rollback_restores_prior_state() {
        let tmp = TempDir::new().unwrap();
        let s1 = WorkflowState::new(Phase::Docs);
        write_workflow_state(tmp.path(), &s1).unwrap();
        let s2 = WorkflowState::new(Phase::Frontend);
        write_workflow_state(tmp.path(), &s2).unwrap();
        // Current is frontend.
        assert_eq!(read_workflow_state(tmp.path()).unwrap().phase, "frontend");

        let snaps = list_snapshots(tmp.path());
        assert_eq!(snaps.len(), 1);
        restore_snapshot(tmp.path(), &snaps[0]).unwrap();
        // Restored to docs.
        assert_eq!(read_workflow_state(tmp.path()).unwrap().phase, "docs");
    }

    #[test]
    fn rollback_missing_snapshot_errors() {
        let tmp = TempDir::new().unwrap();
        assert!(restore_snapshot(tmp.path(), "nonexistent").is_err());
    }

    #[test]
    fn list_snapshots_empty_when_no_history() {
        let tmp = TempDir::new().unwrap();
        assert!(list_snapshots(tmp.path()).is_empty());
    }

    #[test]
    fn legacy_state_without_backend_still_reads() {
        // States written before the `backend` field existed must still
        // deserialize — the new column defaults to empty.
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join(".super-dev");
        std::fs::create_dir_all(&dir).unwrap();
        let legacy = r#"{
            "phase": "frontend",
            "active_gate": "preview_confirm",
            "slug": "old",
            "requirement": "do thing",
            "last_transition_at": "2026-01-01T00:00:00Z",
            "note": "",
            "spec_version": "SUPER_DEV_HOST_SPEC_V1"
        }"#;
        std::fs::write(dir.join("workflow-state.json"), legacy).unwrap();
        let s = read_workflow_state(tmp.path()).expect("legacy state must read");
        assert_eq!(s.backend, "", "missing field must default to empty");
        assert_eq!(s.slug, "old");
    }

    #[test]
    fn diagnostic_distinguishes_missing_and_corrupt() {
        use super::ReadState;
        let tmp = TempDir::new().unwrap();
        // Missing file → Missing, not Corrupt.
        assert!(matches!(
            read_workflow_state_diagnostic(tmp.path()),
            ReadState::Missing
        ));
        // Corrupt JSON → Corrupt (with path + error), NOT silently Missing.
        let dir = tmp.path().join(".super-dev");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("workflow-state.json"), "{ this is not json").unwrap();
        match read_workflow_state_diagnostic(tmp.path()) {
            ReadState::Corrupt { path, error } => {
                assert!(path.ends_with("workflow-state.json"));
                assert!(error.contains("parse"), "error was: {error}");
            }
            other => panic!("expected Corrupt, got {other:?}"),
        }
        // Valid file → Ok.
        std::fs::write(
            dir.join("workflow-state.json"),
            r#"{"phase":"docs","active_gate":"docs_confirm","slug":"x","requirement":"r","last_transition_at":"2026-01-01T00:00:00Z","note":"","spec_version":"SUPER_DEV_HOST_SPEC_V1"}"#,
        )
        .unwrap();
        assert!(matches!(
            read_workflow_state_diagnostic(tmp.path()),
            ReadState::Ok(_)
        ));
    }
}
