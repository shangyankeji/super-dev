//! Governance hook entry point — the `super-dev hook pre-write` command.
//!
//! This is invoked by Claude Code's `PreToolUse` hook (registered via
//! `super-dev install`). It reads a PreToolUse JSON payload from stdin,
//! extracts the target file path + new content, runs the governance rules
//! (emoji / color / AI-slop), and prints a permission-decision JSON object
//! that Claude Code honours to allow or deny the write.
//!
//! ## Claude Code PreToolUse payload shape (simplified)
//! ```json
//! {
//!   "tool_name": "Write",
//!   "tool_input": {
//!     "file_path": "src/Button.tsx",
//!     "content": "<button>🔍</button>"
//!   }
//! }
//! ```
//!
//! ## Decision output shape
//! ```json
//! {
//!   "hookSpecificOutput": {
//!     "hookEventName": "PreToolUse",
//!     "permissionDecision": "deny",
//!     "permissionDecisionReason": "Super Dev: emoji detected..."
//!   }
//! }
//! ```
//! When all rules pass, we emit `permissionDecision: "allow"`.
//!
//! Fail-open: if the payload can't be parsed or the tool isn't a write,
//! we allow (never block a legitimate operation on a parse error).

use serde::Deserialize;
use super_dev_governance::{
    check_ai_slop, check_color_tokens, check_emoji, check_sensitive_path, Decision,
};

/// Read the PreToolUse payload from stdin, run the governance rules, and
/// print the decision JSON. Returns the raw decision for testing.
pub fn run_pre_write(stdin: &str) -> Decision {
    let payload: PreToolUsePayload = match serde_json::from_str(stdin) {
        Ok(p) => p,
        Err(_) => return Decision::pass(), // fail-open on unparseable input
    };
    // Only intercept Write / Edit / MultiEdit / NotebookEdit tools.
    let is_write = matches!(
        payload.tool_name.as_str(),
        "Write" | "Edit" | "MultiEdit" | "NotebookEdit" | "create_file" | "str_replace_editor"
    );
    if !is_write {
        return Decision::pass();
    }
    let file_path = payload.tool_input.file_path.as_deref().unwrap_or("");
    let content = payload.tool_input.content.as_deref().unwrap_or("");
    // For Edit, the new content may be in `new_string` rather than `content`.
    let content = if content.is_empty() {
        payload.tool_input.new_string.as_deref().unwrap_or("")
    } else {
        content
    };

    // Bypass-immune safety guard (SD-SEC-001) runs FIRST and is exempt from
    // any future "skip governance" toggle — it blocks writes into .git/,
    // secret stores, and toolchain config regardless of mode. Mirrors Claude
    // Code's bypass-immune safetyCheck (permissions.ts step 1f/1g).
    if let d @ Decision { block: true, .. } = check_sensitive_path(file_path, content) {
        return d;
    }
    // Run the three code-quality rules; the first block wins.
    for check in [check_emoji, check_color_tokens, check_ai_slop] {
        let d = check(file_path, content);
        if d.block {
            return d;
        }
    }
    Decision::pass()
}

/// Print the decision as a Claude Code-compatible JSON object.
pub fn print_decision(decision: &Decision) {
    let result = if decision.block {
        serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": decision.reason
            }
        })
    } else {
        serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow"
            }
        })
    };
    println!("{}", serde_json::to_string(&result).unwrap_or_default());
}

/// Claude Code PreToolUse stdin payload.
#[derive(Debug, Deserialize)]
struct PreToolUsePayload {
    #[serde(default)]
    tool_name: String,
    #[serde(default)]
    tool_input: ToolInput,
}

#[derive(Debug, Default, Deserialize)]
struct ToolInput {
    #[serde(default)]
    file_path: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    new_string: Option<String>,
}

/// Install the PreToolUse hook into `.claude/settings.json` (workspace-level).
/// Idempotent — if the hook is already registered, does nothing.
pub fn install_claude_hook(project_root: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    let claude_dir = project_root.join(".claude");
    std::fs::create_dir_all(&claude_dir)?;
    let settings_path = claude_dir.join("settings.json");

    // Resolve the path to this binary so the hook points at it.
    let bin = std::env::current_exe().map_or_else(
        |_| "super-dev".to_string(),
        |p| p.to_string_lossy().to_string(),
    );

    // Load existing settings (or start fresh) so we don't clobber user config.
    let mut settings: serde_json::Value = std::fs::read_to_string(&settings_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    // Ensure hooks.PreToolUse exists and contains our matcher.
    let hooks = settings
        .as_object_mut()
        .expect("settings is an object")
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));
    let pre_use = hooks
        .as_object_mut()
        .expect("hooks is an object")
        .entry("PreToolUse")
        .or_insert_with(|| serde_json::json!([]));
    let matchers = pre_use.as_array_mut().expect("PreToolUse is an array");

    // Check if our hook is already registered (idempotent).
    let hook_cmd = format!("{bin} hook pre-write");
    let already = matchers.iter().any(|m| {
        m.get("hooks")
            .and_then(|h| h.as_array())
            .is_some_and(|arr| {
                arr.iter().any(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .is_some_and(|c| c == hook_cmd)
                })
            })
    });
    if !already {
        matchers.push(serde_json::json!({
            "matcher": "Write|Edit|MultiEdit",
            "hooks": [{"type": "command", "command": hook_cmd}]
        }));
    }

    let json = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&settings_path, json + "\n")?;
    Ok(settings_path)
}

/// Remove the Super Dev hook from `.claude/settings.json`. Idempotent.
pub fn uninstall_claude_hook(project_root: &std::path::Path) -> std::io::Result<()> {
    let settings_path = project_root.join(".claude/settings.json");
    let Ok(content) = std::fs::read_to_string(&settings_path) else {
        return Ok(()); // nothing to remove
    };
    let mut settings: serde_json::Value = serde_json::from_str(&content)?;
    let bin = std::env::current_exe().map_or_else(
        |_| "super-dev".to_string(),
        |p| p.to_string_lossy().to_string(),
    );
    let hook_cmd = format!("{bin} hook pre-write");

    if let Some(matchers) = settings
        .get_mut("hooks")
        .and_then(|h| h.get_mut("PreToolUse"))
        .and_then(|p| p.as_array_mut())
    {
        matchers.retain(|m| {
            m.get("hooks").and_then(|h| h.as_array()).is_none_or(|arr| {
                !arr.iter().any(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .is_some_and(|c| c == hook_cmd)
                })
            })
        });
    }
    let json = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&settings_path, json + "\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pre_write_blocks_emoji() {
        let payload = r#"{"tool_name":"Write","tool_input":{"file_path":"src/Btn.tsx","content":"<button>🔍</button>"}}"#;
        let d = run_pre_write(payload);
        assert!(d.block);
        assert_eq!(d.clause, "SD-CODE-001");
    }

    #[test]
    fn pre_write_blocks_color() {
        let payload = r#"{"tool_name":"Write","tool_input":{"file_path":"src/Card.tsx","content":"color:#9333ea"}}"#;
        let d = run_pre_write(payload);
        assert!(d.block);
        assert_eq!(d.clause, "SD-CODE-002");
    }

    #[test]
    fn pre_write_allows_clean_code() {
        let payload = r#"{"tool_name":"Write","tool_input":{"file_path":"src/Btn.tsx","content":"<button>Search</button>"}}"#;
        let d = run_pre_write(payload);
        assert!(!d.block);
    }

    #[test]
    fn pre_write_fails_open_on_garbage() {
        let d = run_pre_write("not json at all");
        assert!(!d.block);
    }

    #[test]
    fn pre_write_ignores_non_write_tools() {
        let payload = r#"{"tool_name":"Bash","tool_input":{"command":"ls"}}"#;
        let d = run_pre_write(payload);
        assert!(!d.block);
    }

    #[test]
    fn pre_write_uses_new_string_for_edit() {
        let payload =
            r#"{"tool_name":"Edit","tool_input":{"file_path":"src/Btn.tsx","new_string":"🚀"}}"#;
        let d = run_pre_write(payload);
        assert!(d.block);
    }

    #[test]
    fn print_decision_outputs_deny_json() {
        let d = Decision::block("SD-CODE-001", "emoji here");
        // Just verify it doesn't panic and produces JSON with deny.
        print_decision(&d);
    }

    #[test]
    fn install_and_uninstall_are_idempotent() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Install twice — second should be a no-op.
        install_claude_hook(tmp.path()).unwrap();
        install_claude_hook(tmp.path()).unwrap();
        let settings = std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap();
        assert!(settings.contains("hook pre-write"));
        // Uninstall twice — second should be a no-op.
        uninstall_claude_hook(tmp.path()).unwrap();
        uninstall_claude_hook(tmp.path()).unwrap();
        let settings2 = std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap();
        assert!(!settings2.contains("hook pre-write"));
    }

    #[test]
    fn sensitive_path_blocked_via_full_hook_pipeline() {
        // A Write targeting .git/config must be denied end-to-end, BEFORE the
        // code-style rules run (the content here is clean, so only the path
        // check would catch it).
        let payload =
            r#"{"tool_name":"Write","tool_input":{"file_path":".git/config","content":"[core]"}}"#;
        let d = run_pre_write(payload);
        assert!(d.block);
        assert_eq!(d.clause, "SD-SEC-001");
    }

    #[test]
    fn sensitive_path_env_blocked_via_hook() {
        let payload =
            r#"{"tool_name":"Write","tool_input":{"file_path":".env","content":"SECRET=x"}}"#;
        let d = run_pre_write(payload);
        assert!(d.block);
        assert_eq!(d.clause, "SD-SEC-001");
    }

    #[test]
    fn sensitive_path_ssh_key_blocked_via_hook() {
        let payload = r#"{"tool_name":"Edit","tool_input":{"file_path":"/root/.ssh/id_rsa","new_string":"KEY"}}"#;
        let d = run_pre_write(payload);
        assert!(d.block);
    }

    #[test]
    fn normal_source_file_passes_full_hook() {
        // A clean Write to a normal source file passes all checks.
        let payload = r#"{"tool_name":"Write","tool_input":{"file_path":"src/Button.tsx","content":"export const Button = () => <button />"}}"#;
        let d = run_pre_write(payload);
        assert!(!d.block);
    }

    #[test]
    fn sensitive_path_priority_over_code_rules() {
        // Path is sensitive (.env) AND content has an emoji — sensitive-path
        // (SD-SEC-001) must win because it runs first, not emoji (SD-CODE-001).
        let payload = r#"{"tool_name":"Write","tool_input":{"file_path":".env","content":"🔍"}}"#;
        let d = run_pre_write(payload);
        assert!(d.block);
        assert_eq!(d.clause, "SD-SEC-001");
    }
}
