//! Expert prompts — the system/user messages each phase hands to a
//! [`Runtime`] to produce real LLM-driven artifacts.
//!
//! Each expert builds one message pair: a `system` prompt that pins the
//! expert's role + the Super Dev spec constraints, plus a `user`
//! message carrying the requirement and any prior artifacts. The
//! returned [`Prompt`] is provider-agnostic — runners hand it to any
//! [`Runtime`] implementation.
//!
//! Why prompts live here:
//! - They are *part of the agent's policy*, not a runtime concern.
//! - Tests can validate that prompts mention the spec clauses they
//!   need to mention (no LLM call required).
//! - Future tuning (better wording, few-shot examples) is one file.

use super_dev_runtime::{CompletionRequest, Message};

/// A reusable prompt — system + a single user message.
#[derive(Debug, Clone)]
pub struct Prompt {
    /// Top-level system prompt.
    pub system: String,
    /// User-role body.
    pub user: String,
}

impl Prompt {
    /// Convert into a runtime-ready [`CompletionRequest`].
    #[must_use]
    pub fn into_request(self, model: impl Into<String>, max_tokens: u32) -> CompletionRequest {
        CompletionRequest {
            model: model.into(),
            system: Some(self.system),
            messages: vec![Message {
                role: "user".to_string(),
                content: self.user,
            }],
            max_tokens: Some(max_tokens),
            temperature: Some(0.2),
        }
    }
}

const SPEC_PREAMBLE: &str = "\
You are working inside a Super Dev pipeline run governed by \
SUPER_DEV_HOST_SPEC_V1. Every artifact you produce MUST follow these \
non-negotiable rules:\n\
- Use a declared icon library (Lucide / Heroicons / Tabler). NEVER \
emoji as functional icons.\n\
- Use design tokens (CSS vars / theme keys). NEVER hardcoded hex / \
rgb / hsl colors in production UI.\n\
- Frontend fetch URLs MUST match an API path declared in the \
architecture document.\n\
- Wait for explicit user approval at docs_confirm and preview_confirm \
gates.\n\
- Output goes into structured markdown sections — do not invent new \
top-level sections, fill the ones the user asks for.\n";

/// Research expert — produces `output/<slug>-research.md`.
#[must_use]
pub fn research_prompt(slug: &str, requirement: &str, knowledge_digest: &str) -> Prompt {
    let system = format!(
        "{SPEC_PREAMBLE}\n\
         Role: senior product researcher + design strategist.\n\n\
         MUST-DO:\n\
         1. Include a `## Discovery` section with ALL 6 fields answered.\n\
         2. Include `## Design system recommendation`.\n\
         3. Cite 5 REAL products (not made up) with design-specific takeaways.\n\n\
         Required sections (in this EXACT order, ALL mandatory):\n\
         - # Research — {slug}\n\
         - ## Requirement (echo verbatim)\n\
         - ## Discovery (answer ALL): Target audience, Visual tone, \
           Design direction (ONE of: Modern Minimal / Editorial Clean / \
           Tech Utility / Soft Warm / Bold Geometric), Brand constraints, \
           Platform, Complexity\n\
         - ## Similar products — 5 real products with design takeaways\n\
         - ## Domain risks — 5 risks with mitigation strategies\n\
         - ## UI / UX must-haves — 5 non-negotiable patterns\n\
         - ## Design system recommendation — palette direction, \
           typography approach, spacing philosophy, signature detail\n\
         - ## Open questions"
    );
    let user = format!(
        "## Requirement\n\n{requirement}\n\n\
         ## Local knowledge files available\n\n{knowledge_digest}\n\n\
         Write the research brief now."
    );
    Prompt { system, user }
}

/// PM expert — produces `output/<slug>-prd.md`.
#[must_use]
pub fn prd_prompt(slug: &str, requirement: &str, research_excerpt: &str) -> Prompt {
    let system = format!(
        "{SPEC_PREAMBLE}\n\
         Role: senior product manager (10+ years B2B/B2C SaaS).\n\
         Task: write a production-grade PRD that a dev team can implement \
         without coming back to ask questions.\n\n\
         Required sections (ALL mandatory, in order):\n\
         - # PRD — {slug}\n\
         - ## Goal — 2-4 sentences: what + why + for whom + success metric\n\
         - ## Target users — 2-3 user personas with role, context, and pain point\n\
         - ## User journey — step-by-step flow of the primary use case \
           (numbered steps, each with action + expected result)\n\
         - ## Scope\n\
           - ### In scope — bullet list of features to build THIS iteration\n\
           - ### Out of scope — explicitly excluded items (prevents scope creep)\n\
         - ## Functional requirements — detailed feature list, each with:\n\
           - Feature name\n\
           - Description (1-2 sentences)\n\
           - Priority (P0/P1/P2)\n\
           - Acceptance criteria (testable conditions)\n\
         - ## Non-functional requirements\n\
           - Performance (target latency, throughput)\n\
           - Security (auth method, data sensitivity level)\n\
           - Accessibility (WCAG level)\n\
           - Internationalization (if applicable)\n\
           - Browser/device support matrix\n\
         - ## Acceptance criteria — master checkbox list (≥8 items), \
           each MUST be independently testable by QA: `- [ ] Given X, when Y, then Z`\n\
         - ## Success metrics — 2-4 measurable KPIs with target numbers\n\
         - ## Risks & mitigations — each risk with probability + impact + mitigation\n\
         - ## Open questions — unresolved decisions that block implementation"
    );
    let user = format!(
        "## Requirement\n\n{requirement}\n\n\
         ## Research brief (excerpt)\n\n{research_excerpt}\n\n\
         Write the complete PRD now."
    );
    Prompt { system, user }
}

/// Architect expert — produces `output/<slug>-architecture.md`.
#[must_use]
pub fn architecture_prompt(slug: &str, requirement: &str, prd_excerpt: &str) -> Prompt {
    let system = format!(
        "{SPEC_PREAMBLE}\n\
         Role: senior software architect.\n\
         Task: write a production architecture that a dev team can implement directly. \
         The API surface is load-bearing — every frontend `fetch` MUST match a row. \
         Every endpoint MUST specify request/response shapes.\n\n\
         Required sections (ALL mandatory):\n\
         - # Architecture — {slug}\n\
         - ## System overview — component diagram in text, data flow direction, \
           communication protocols (REST/gRPC/WebSocket)\n\
         - ## API surface — table: `| Method | Path | Request | Response | Auth | Description |` \
           At least 8 rows. Every path starts with `/`. Include auth requirements per endpoint.\n\
         - ## API error convention — standard error envelope: \
           `{{ \"error\": {{ \"code\": \"...\", \"message\": \"...\" }} }}`. \
           Table of error codes: `| HTTP | Code | Meaning |` (400/401/403/404/409/422/500)\n\
         - ## Data model — for each entity: field table with types + required + description. \
           Show relationships (1:N, N:M). List indexes needed.\n\
         - ## Authentication & authorization — auth method, token format, \
           role definitions, permission matrix per API endpoint\n\
         - ## Tech-stack — for each choice: what + why + rejected alternatives\n\
         - ## Project structure — recommended directory layout for frontend + backend\n\
         - ## Coding conventions — naming (camelCase/snake_case), error handling pattern, \
           logging format, environment variable naming\n\
         - ## Performance budget — FCP target, API p95 target, caching strategy\n\
         - ## Security considerations — input validation, SQL injection prevention, \
           XSS prevention, CORS policy, rate limiting\n\
         - ## Deployment — environments, CI/CD steps, rollback strategy\n\
         - ## Open trade-offs"
    );
    let user = format!(
        "## Requirement\n\n{requirement}\n\n\
         ## PRD (excerpt)\n\n{prd_excerpt}\n\n\
         Write the complete architecture now."
    );
    Prompt { system, user }
}

/// UI/UX expert — produces `output/<slug>-uiux.md`.
#[must_use]
pub fn uiux_prompt(slug: &str, requirement: &str, prd_excerpt: &str) -> Prompt {
    let system = format!(
        "{SPEC_PREAMBLE}\n\
         Role: senior UI/UX designer — creates design SYSTEMS, not mockups.\n\n\
         MUST-DO:\n\
         1. Output pure markdown with ALL sections below. Do NOT skip any.\n\
         2. The `:root` CSS block must have 10+ semantic color tokens.\n\
         3. Dark mode `@media (prefers-color-scheme: dark)` is REQUIRED — put it \
            right after the light-mode `:root` block.\n\n\
         Required sections (in this EXACT order):\n\
         - # UI/UX — {slug}\n\
         - ## Color palette — `:root` CSS block with: --color-bg, --color-surface, \
           --color-text, --color-text-secondary, --color-primary, --color-primary-hover, \
           --color-accent, --color-border, --color-error, --color-success (minimum 10).\n\
         - ## Dark mode — `@media (prefers-color-scheme: dark)` overriding \
           bg/surface/text/border tokens. NOT optional.\n\
         - ## Typography system — font stack (2 families max), 7-step type scale \
           (--text-xs through --text-3xl), line-height + weight tokens.\n\
         - ## Spacing scale — 4px base, 8+ steps.\n\
         - ## Icon library — exactly ONE: Lucide / Heroicons / Tabler.\n\
         - ## Page hierarchy — nested list with route paths.\n\
         - ## Component inventory — every component with states: \
           default / hover / active / disabled / loading / error.\n\
         - ## Motion guidelines — transition tokens.\n\
         - ## Accessibility notes — contrast, focus rings, ARIA.\n\n\
         Self-check: color palette has 10+ tokens? Dark mode block present? \
         Typography has 7 sizes? Every component has states?"
    );
    let user = format!(
        "## Requirement\n\n{requirement}\n\n\
         ## PRD (excerpt)\n\n{prd_excerpt}\n\n\
         Write the complete UI/UX spec now."
    );
    Prompt { system, user }
}

/// Truncate `text` to at most `max_chars` characters, keeping head.
/// Returns text with a trailing `…` marker when it had to cut.
#[must_use]
pub fn excerpt(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut buf: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    buf.push('…');
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_prompt_mentions_required_sections() {
        let p = research_prompt("demo", "build a login system", "- knowledge/x.md");
        assert!(p.system.contains("Similar products"));
        assert!(p.system.contains("Domain risks"));
        assert!(p.user.contains("build a login system"));
        assert!(p.user.contains("knowledge/x.md"));
    }

    #[test]
    fn prd_prompt_quotes_research() {
        let p = prd_prompt("demo", "x", "excerpt: research goes here");
        assert!(p.user.contains("excerpt: research goes here"));
    }

    #[test]
    fn architecture_prompt_demands_api_table() {
        let p = architecture_prompt("demo", "x", "");
        assert!(p.system.to_lowercase().contains("api surface"));
        assert!(p.system.contains("|"));
    }

    #[test]
    fn uiux_prompt_locks_icon_library() {
        let p = uiux_prompt("demo", "x", "");
        assert!(p.system.contains("Lucide"));
        assert!(p.system.contains("Heroicons"));
        assert!(p.system.contains("Tabler"));
    }

    #[test]
    fn every_prompt_carries_spec_preamble() {
        for p in [
            research_prompt("s", "r", "k"),
            prd_prompt("s", "r", "x"),
            architecture_prompt("s", "r", "x"),
            uiux_prompt("s", "r", "x"),
        ] {
            assert!(p.system.contains("SUPER_DEV_HOST_SPEC_V1"));
            assert!(p.system.contains("emoji"));
            assert!(p.system.contains("design tokens"));
        }
    }

    #[test]
    fn into_request_round_trip() {
        let req = research_prompt("s", "r", "k").into_request("claude-sonnet-4-6", 4096);
        assert_eq!(req.model, "claude-sonnet-4-6");
        assert_eq!(req.max_tokens, Some(4096));
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, "user");
        assert!(req.system.is_some());
    }

    #[test]
    fn excerpt_trims_long_text() {
        let s: String = "a".repeat(5000);
        let e = excerpt(&s, 100);
        assert_eq!(e.chars().count(), 100);
        assert!(e.ends_with('…'));
    }

    #[test]
    fn excerpt_passes_short_text_through() {
        assert_eq!(excerpt("hi", 100), "hi");
    }
}
