//! Persistent workflow state — `.super-dev/workflow-state.json`.
//!
//! Implements the persistence side of `SD-FLOW-001`. The agent updates
//! this file on every phase transition; the prompt-time injection hook
//! reads it back to ground the model.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
            spec_version: super_dev_spec::SPEC_VERSION.to_string(),
        }
    }
}

/// Persist the workflow state to `<project_root>/.super-dev/workflow-state.json`.
///
/// Disk-write errors propagate so callers can decide whether to surface
/// or swallow them.
pub fn write_workflow_state(project_root: &Path, state: &WorkflowState) -> std::io::Result<()> {
    let dir = project_root.join(".super-dev");
    fs::create_dir_all(&dir)?;
    let text = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(dir.join("workflow-state.json"), text)
}

/// Read the workflow state. Returns `None` when the file is missing or
/// malformed.
#[must_use]
pub fn read_workflow_state(project_root: &Path) -> Option<WorkflowState> {
    let path = project_root.join(".super-dev").join("workflow-state.json");
    let text = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&text).ok()
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
}
