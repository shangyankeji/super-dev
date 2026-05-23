//! Confirmation gates — SD-FLOW-002 / SD-FLOW-003.

use serde::{Deserialize, Serialize};

/// Which gate this represents.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gate {
    /// After `docs` phase — wait for explicit user approval of PRD/ARCH/UIUX.
    DocsConfirm,
    /// After `frontend` phase — wait for explicit user approval of preview.
    PreviewConfirm,
}

impl Gate {
    /// Canonical id persisted to `workflow-state.json#active_gate`.
    #[must_use]
    pub const fn id_str(self) -> &'static str {
        match self {
            Self::DocsConfirm => "docs_confirm",
            Self::PreviewConfirm => "preview_confirm",
        }
    }
}

/// What the user did at the gate.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GateOutcome {
    /// User said `确认 / 通过 / 继续 / lgtm / approve / ...`.
    Approved,
    /// User requested revisions (free-form).
    Revise(String),
    /// User explicitly cancelled the pipeline.
    Cancelled,
}

const APPROVAL_TOKENS: &[&str] = &[
    "确认", "通过", "继续", "approved", "approve", "lgtm", "ship it", "ok",
];

/// Classify a free-form user reply into a gate outcome.
///
/// SD-FLOW-002 rules:
/// - exact match against `APPROVAL_TOKENS` (case-insensitive, trimmed) → Approved
/// - "cancel" / "取消" / "重来" → Cancelled
/// - everything else → Revise(text)
#[must_use]
pub fn classify_reply(reply: &str) -> GateOutcome {
    let lower = reply.trim().to_lowercase();
    if lower.is_empty() {
        return GateOutcome::Revise(String::new());
    }
    if APPROVAL_TOKENS
        .iter()
        .any(|t| t.eq_ignore_ascii_case(&lower))
    {
        return GateOutcome::Approved;
    }
    if matches!(lower.as_str(), "cancel" | "取消" | "重来" | "restart") {
        return GateOutcome::Cancelled;
    }
    GateOutcome::Revise(reply.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_tokens_match() {
        for t in [
            "确认", "通过", "继续", "approved", "Approve", "LGTM", "ship it",
        ] {
            assert!(matches!(classify_reply(t), GateOutcome::Approved), "{t}");
        }
    }

    #[test]
    fn cancel_tokens_match() {
        for t in ["cancel", "取消", "重来", "restart"] {
            assert!(matches!(classify_reply(t), GateOutcome::Cancelled), "{t}");
        }
    }

    #[test]
    fn revise_default() {
        let out = classify_reply("把图标库换成 lucide");
        if let GateOutcome::Revise(text) = out {
            assert!(text.contains("lucide"));
        } else {
            panic!("expected Revise");
        }
    }

    #[test]
    fn empty_reply_is_revise_with_empty_text() {
        assert!(matches!(classify_reply(""), GateOutcome::Revise(s) if s.is_empty()));
    }
}
