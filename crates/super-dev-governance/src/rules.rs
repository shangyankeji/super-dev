//! Pre-write enforcement rules — refuse a tool call before it lands on disk.
//!
//! Each rule is a pure function: takes `(file_path, content)`, returns a
//! [`Decision`] describing whether to pass or block, with a human-
//! readable reason. The host wires these into its `PreToolUse` / pre-edit
//! hook.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Outcome of a governance rule.
///
/// A `block` decision is conveyed to the host as JSON so it refuses the
/// tool call; the `reason` is shown to the model so it can self-correct
/// on retry.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    /// `true` when the host MUST refuse the tool call.
    pub block: bool,
    /// Human-readable explanation shown to the model; empty when pass.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reason: String,
    /// Clause that fired, e.g. `SD-CODE-001`. Empty on pass.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub clause: String,
}

impl Decision {
    /// Build a passing decision.
    #[must_use]
    pub const fn pass() -> Self {
        Self {
            block: false,
            reason: String::new(),
            clause: String::new(),
        }
    }

    /// Build a blocking decision with `reason` and the firing clause id.
    #[must_use]
    pub fn block(clause: &str, reason: impl Into<String>) -> Self {
        Self {
            block: true,
            reason: reason.into(),
            clause: clause.to_string(),
        }
    }
}

/// File extensions guarded by the emoji rule (SD-CODE-001).
const EMOJI_GUARDED_EXTS: &[&str] = &["tsx", "ts", "jsx", "js", "vue", "svelte", "astro"];

/// File extensions guarded by the color rule (SD-CODE-002).
const COLOR_GUARDED_EXTS: &[&str] = &[
    "tsx", "ts", "jsx", "js", "vue", "svelte", "astro", "css", "scss", "sass",
];

/// Path fragments that exempt a file from the color rule.
const COLOR_EXEMPT_FRAGMENTS: &[&str] = &[
    "/tokens/",
    "/theme/",
    "/themes/",
    "/design-system/",
    "/design-tokens/",
    "/.storybook/",
    ".stories.",
    ".test.",
    ".spec.",
    "/fixtures/",
    "/mocks/",
];

/// Achromatic literals tolerated under the color rule.
const COLOR_ALLOWED: &[&str] = &["#fff", "#ffffff", "#000", "#000000"];

fn emoji_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Conservative pictograph ranges — leaves CJK punctuation and
        // CJK ideographs alone, only catches graphical emoji.
        Regex::new(r"[\x{2600}-\x{27BF}\x{1F300}-\x{1FAFF}\x{1F900}-\x{1F9FF}\x{1FA70}-\x{1FAFF}]")
            .expect("emoji regex is well-formed at compile time")
    })
}

fn hex_color_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"#[0-9a-fA-F]{3,8}\b").expect("hex regex is well-formed"))
}

fn rgb_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\brgba?\s*\(").expect("rgb regex is well-formed"))
}

fn hsl_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\bhsla?\s*\(").expect("hsl regex is well-formed"))
}

fn extension_of(file_path: &str) -> String {
    file_path
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default()
}

/// Check whether `content` would land emoji-as-functional-icons in a UI file.
///
/// Implements **SD-CODE-001** (`SUPER_DEV_HOST_SPEC_V1` §3.1).
#[must_use]
pub fn check_emoji(file_path: &str, content: &str) -> Decision {
    let ext = extension_of(file_path);
    if !EMOJI_GUARDED_EXTS.contains(&ext.as_str()) {
        return Decision::pass();
    }
    if !emoji_regex().is_match(content) {
        return Decision::pass();
    }
    let reason = format!(
        "Super Dev: emoji detected in {ext} source file ({file_path}). \
         Use a declared icon library (Lucide / Heroicons / Tabler) instead \
         of emoji as functional icons. Replace the emoji before retrying."
    );
    Decision::block("SD-CODE-001", reason)
}

/// Check whether `content` contains hardcoded chromatic literals in a UI file.
///
/// Implements **SD-CODE-002** (`SUPER_DEV_HOST_SPEC_V1` §3.2).
#[must_use]
pub fn check_color_tokens(file_path: &str, content: &str) -> Decision {
    let ext = extension_of(file_path);
    if !COLOR_GUARDED_EXTS.contains(&ext.as_str()) {
        return Decision::pass();
    }
    let lower_path = file_path.to_ascii_lowercase();
    if COLOR_EXEMPT_FRAGMENTS
        .iter()
        .any(|frag| lower_path.contains(frag))
    {
        return Decision::pass();
    }

    let mut violations: Vec<String> = Vec::new();
    for m in hex_color_regex().find_iter(content) {
        let token = m.as_str().to_ascii_lowercase();
        if COLOR_ALLOWED.contains(&token.as_str()) {
            continue;
        }
        if !violations.contains(&token) {
            violations.push(token);
        }
        if violations.len() >= 5 {
            break;
        }
    }
    if rgb_regex().is_match(content) && !violations.contains(&"rgb()/rgba()".to_string()) {
        violations.push("rgb()/rgba()".to_string());
    }
    if hsl_regex().is_match(content) && !violations.contains(&"hsl()/hsla()".to_string()) {
        violations.push("hsl()/hsla()".to_string());
    }

    if violations.is_empty() {
        return Decision::pass();
    }

    let reason = format!(
        "Super Dev: hardcoded colors detected in {file_path}: {}. \
         Use design tokens (CSS vars, theme constants, or Tailwind theme \
         keys) from output/*-uiux.md instead. If this is a tokens / theme \
         / design-system file, move it under tokens/ or theme/ to exempt \
         the check.",
        violations.join(", ")
    );
    Decision::block("SD-CODE-002", reason)
}

/// Check for common "AI slop" visual anti-patterns in UI source files.
///
/// P0-level checks (cardinal sins that make output look AI-generated):
/// - Purple/violet gradient backgrounds (`linear-gradient` containing purple hues)
/// - "Lorem ipsum" placeholder text
/// - "Welcome to [App]" generic hero headings
///
/// Implements an extension of **SD-CODE-001/002** focused on visual
/// quality beyond just emoji and color tokens.
#[must_use]
pub fn check_ai_slop(file_path: &str, content: &str) -> Decision {
    let ext = extension_of(file_path);
    if !EMOJI_GUARDED_EXTS.contains(&ext.as_str()) {
        return Decision::pass();
    }

    let mut issues: Vec<&str> = Vec::new();

    let lower = content.to_ascii_lowercase();
    if lower.contains("lorem ipsum") || lower.contains("dolor sit amet") {
        issues.push("Lorem ipsum placeholder text");
    }
    if lower.contains("welcome to")
        && (lower.contains("<h1") || lower.contains("<h2") || lower.contains("heading"))
    {
        issues.push("Generic 'Welcome to [App]' heading");
    }
    if lower.contains("linear-gradient") {
        let has_purple = lower.contains("#7c3aed")
            || lower.contains("#8b5cf6")
            || lower.contains("#a855f7")
            || lower.contains("#9333ea")
            || lower.contains("purple")
            || lower.contains("violet");
        let has_pink = lower.contains("#ec4899")
            || lower.contains("#f472b6")
            || lower.contains("pink")
            || lower.contains("fuchsia");
        if has_purple && has_pink {
            issues.push("Purple-to-pink gradient (classic AI template pattern)");
        }
    }

    if issues.is_empty() {
        return Decision::pass();
    }

    let reason = format!(
        "Super Dev anti-slop: {} detected in {file_path}. \
         These patterns make output look AI-generated. \
         Use real content and design tokens from output/*-uiux.md.",
        issues.join("; ")
    );
    Decision::block("SD-CODE-005", reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // --- emoji ----------------------------------------------------------

    #[test]
    fn emoji_blocks_in_tsx() {
        let d = check_emoji("src/Btn.tsx", "<button>🔍 Search</button>");
        assert!(d.block);
        assert_eq!(d.clause, "SD-CODE-001");
        assert!(d.reason.contains("src/Btn.tsx"));
        assert!(d.reason.contains("icon library"));
    }

    #[test]
    fn emoji_blocks_in_jsx_vue_svelte_astro() {
        for path in ["App.jsx", "App.vue", "App.svelte", "page.astro"] {
            assert!(
                check_emoji(path, "<div>🚀</div>").block,
                "expected block for {path}"
            );
        }
    }

    #[test]
    fn emoji_passes_when_clean() {
        assert!(!check_emoji("src/Btn.tsx", "<button>Search</button>").block);
    }

    #[test]
    fn emoji_passes_in_markdown() {
        assert!(!check_emoji("README.md", "# Project 🚀").block);
    }

    #[test]
    fn emoji_passes_when_no_extension() {
        assert!(!check_emoji("Makefile", "🚀").block);
    }

    #[test]
    fn emoji_passes_empty_content() {
        assert!(!check_emoji("src/x.tsx", "").block);
    }

    #[test]
    fn emoji_extension_case_insensitive() {
        assert!(check_emoji("src/Btn.TSX", "🔍").block);
    }

    // --- color ----------------------------------------------------------

    #[test]
    fn color_blocks_hex_in_tsx() {
        let d = check_color_tokens("src/Card.tsx", "color:#9333ea");
        assert!(d.block);
        assert_eq!(d.clause, "SD-CODE-002");
        assert!(d.reason.contains("#9333ea"));
    }

    #[test]
    fn color_blocks_rgb() {
        let d = check_color_tokens("src/Card.tsx", "background: rgba(255,0,0,0.5)");
        assert!(d.block);
        assert!(d.reason.to_lowercase().contains("rgb"));
    }

    #[test]
    fn color_blocks_hsl() {
        let d = check_color_tokens("src/Card.tsx", "color: hsl(120 50% 50%)");
        assert!(d.block);
    }

    #[test]
    fn color_passes_neutral() {
        for c in ["#fff", "#ffffff", "#000", "#000000"] {
            let d = check_color_tokens("src/Card.tsx", &format!("color:{c}"));
            assert!(!d.block, "expected pass for {c}");
        }
    }

    #[test]
    fn color_passes_css_var() {
        assert!(!check_color_tokens("src/Card.tsx", "color: var(--primary)").block);
    }

    #[test]
    fn color_passes_exempt_paths() {
        for path in [
            "src/tokens/colors.ts",
            "src/theme/dark.css",
            "src/design-system/palette.tsx",
            "src/Button.stories.tsx",
            "src/Button.test.tsx",
            "src/fixtures/colors.ts",
        ] {
            assert!(
                !check_color_tokens(path, "export = '#9333ea'").block,
                "expected pass for exempt path {path}"
            );
        }
    }

    #[test]
    fn color_passes_non_ui_files() {
        assert!(!check_color_tokens("config.json", "#9333ea").block);
    }

    #[test]
    fn color_caps_examples_at_five() {
        let content = "a:#111 b:#222 c:#333 d:#444 e:#555 f:#666 g:#777";
        let d = check_color_tokens("src/Card.tsx", content);
        assert!(d.block);
        // hash count in reason should be <= 5 distinct hex literals
        let hash_count = d.reason.matches('#').count();
        assert!(hash_count <= 5, "expected <=5 examples, got {hash_count}");
    }

    #[test]
    fn color_blocks_in_css_file() {
        assert!(check_color_tokens("src/styles.css", ".btn { color: #ff0000 }").block);
    }

    // --- AI slop --------------------------------------------------------

    #[test]
    fn slop_blocks_lorem_ipsum() {
        let d = check_ai_slop("src/Hero.tsx", "<p>Lorem ipsum dolor sit amet</p>");
        assert!(d.block);
        assert!(d.reason.contains("Lorem ipsum"));
    }

    #[test]
    fn slop_blocks_welcome_heading() {
        let d = check_ai_slop("src/Hero.tsx", "<h1>Welcome to MyApp</h1>");
        assert!(d.block);
        assert!(d.reason.contains("Welcome to"));
    }

    #[test]
    fn slop_blocks_purple_pink_gradient() {
        let d = check_ai_slop(
            "src/Hero.tsx",
            "background: linear-gradient(135deg, #7c3aed, #ec4899)",
        );
        assert!(d.block);
        assert!(d.reason.contains("gradient"));
    }

    #[test]
    fn slop_passes_clean_code() {
        assert!(!check_ai_slop("src/Hero.tsx", "<h1>Ship faster</h1>").block);
    }

    #[test]
    fn slop_ignores_non_ui_files() {
        assert!(!check_ai_slop("README.md", "Lorem ipsum in docs is fine").block);
    }
}
