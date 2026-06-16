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
const EMOJI_GUARDED_EXTS: &[&str] = &[
    "tsx", "ts", "jsx", "js", "mjs", "cjs", "vue", "svelte", "astro", "html", "htm", "css", "scss",
    "sass", "less", "py", "java", "kt", "go", "rs", "rb", "php", "cs", "swift", "md", "mdx",
];

/// UI source file types guarded by the color (SD-CODE-002) and AI-slop rules.
/// Narrower than EMOJI_GUARDED_EXTS: those quality checks only make sense for
/// frontend/UI source, not docs or backend code. Emoji (SD-CODE-001) is a
/// global prohibition and applies to the broader list above.
const UI_CODE_EXTS: &[&str] = &["tsx", "ts", "jsx", "js", "vue", "svelte", "astro"];

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
        // Comprehensive graphical-emoji ranges. Leaves CJK ideographs and
        // CJK punctuation alone (those are legitimate text, not emoji icons).
        // Covers: misc symbols + dingbats, technical symbols, enclosed
        // alphanumerics (① ⓵), pictographs, transport/map, supplemental
        // symbols, flags, skin-tone modifiers, and the keycap/variation
        // selectors that turn plain chars into emoji.
        Regex::new(concat!(
            r"[",
            r"\x{2300}-\x{23FF}",   // misc technical (⚠ etc.)
            r"\x{2460}-\x{24FF}",   // enclosed alphanumerics (① ⓵)
            r"\x{25A0}-\x{27BF}",   // geometric shapes + misc symbols + dingbats
            r"\x{2B00}-\x{2BFF}",   // misc symbols and arrows
            r"\x{1F000}-\x{1F0FF}", // mahjong + dominoes + playing cards
            r"\x{1F100}-\x{1F1FF}", // enclosed alphanumeric supplement + flags
            r"\x{1F200}-\x{1F2FF}", // enclosed ideographic supplement
            r"\x{1F300}-\x{1F5FF}", // misc symbols and pictographs
            r"\x{1F600}-\x{1F64F}", // emoticons
            r"\x{1F680}-\x{1F6FF}", // transport and map
            r"\x{1F700}-\x{1F77F}", // alchemical symbols
            r"\x{1F780}-\x{1F7FF}", // geometric shapes extended
            r"\x{1F800}-\x{1F8FF}", // supplemental arrows-C
            r"\x{1F900}-\x{1F9FF}", // supplemental symbols and pictographs
            r"\x{1FA00}-\x{1FA6F}", // chess symbols
            r"\x{1FA70}-\x{1FAFF}", // symbols and pictographs extended-A
            r"\x{1F3FB}-\x{1F3FF}", // skin-tone modifiers
            r"]",
        ))
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
    // 4.6: tokenise the source and scan every region EXCEPT comments.
    // Emoji-as-icon violations can legitimately appear in JSX text nodes
    // (`<button>🚀</button>`), string literals (`const ICON = "🚀"`), or
    // code — all of which are kept by `without_comments`. Only comments
    // (`// 🚀 todo`) are documentation noise and must be skipped. Scoping
    // to `jsx_text()` alone would MISS string-literal emoji, so
    // `without_comments` is the correct (broader) view here.
    let tz = crate::tokenizer::Tokenized::new(content);
    let scan_text = tz.without_comments(content);
    if !emoji_regex().is_match(&scan_text) {
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

    // 4.6: scan the tokenised source, skipping comments. A color in a
    // comment (`/* placeholder #fff */`) is documentation, not a violation.
    let tz = crate::tokenizer::Tokenized::new(content);
    let scan_text = tz.without_comments(content);
    let mut violations: Vec<String> = Vec::new();
    for m in hex_color_regex().find_iter(&scan_text) {
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
    if rgb_regex().is_match(&scan_text) && !violations.contains(&"rgb()/rgba()".to_string()) {
        violations.push("rgb()/rgba()".to_string());
    }
    if hsl_regex().is_match(&scan_text) && !violations.contains(&"hsl()/hsla()".to_string()) {
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
    if !UI_CODE_EXTS.contains(&ext.as_str()) {
        return Decision::pass();
    }

    // Tokenize once and scan code+strings+JSX-text (skip comments), the
    // same view `check_emoji` / `check_color_tokens` use. Previously this
    // rule lowercased the RAW source, so a comment like
    // `// TODO: replace the lorem ipsum` would falsely block — the very
    // class of false positive the other two rules were upgraded to avoid.
    let tz = crate::tokenizer::Tokenized::new(content);
    let body = tz.without_comments(content);
    let lower = body.to_ascii_lowercase();

    let mut issues: Vec<&str> = Vec::new();
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

    // Placeholder / fake-data patterns — half-finished markers that must
    // never ship in commercial code.
    if lower.contains("your code here")
        || lower.contains("your message here")
        || lower.contains("your text here")
        || lower.contains("replace this")
        || lower.contains("your-api-key-here")
    {
        issues.push("Unfilled placeholder text");
    }
    if lower.contains("example.com") && !lower.contains("// docs.example.com") {
        issues.push("example.com placeholder URL (use a real domain)");
    }
    if lower.contains("test@test.com")
        || lower.contains("user@example")
        || lower.contains("john@example")
    {
        issues.push("Fake placeholder email (use realistic sample data)");
    }
    // Debug residue left in shipped code.
    if lower.contains("console.log(") {
        issues.push("console.log() debug residue (remove before shipping)");
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
    // Attribute to SD-CODE-002 (hardcoded color literals / design tokens):
    // the design part of this check (purple→pink gradient) IS a color-token
    // violation, and the content part (Lorem ipsum / "Welcome to") shares the
    // same "looks auto-generated" design-quality concern. We deliberately do
    // NOT use SD-CODE-005 — that id is reserved by the spec (§10) for the
    // future V2 accessibility-token clause and is non-normative in V1.
    Decision::block("SD-CODE-002", reason)
}

/// Directory names that mark any write *inside* them as sensitive. Matched
/// as a path segment (so `.git/` matches `a/.git/b` AND `.git/b` but not
/// `digit.ts`). Borrowed from Claude Code's bypass-immune safetyCheck.
const SENSITIVE_DIRS: &[&str] = &[".git", ".ssh", ".aws", ".claude", ".vscode"];

/// Specific sensitive path *suffixes* (file/dir names) matched against the
/// normalized path. Each is matched as a trailing path component so it works
/// for both absolute (`/x/.env`) and relative (`.env`) targets.
const SENSITIVE_PATH_SUFFIXES: &[&str] = &[
    ".env",
    ".env.local",
    ".env.production",
    ".env.development",
    ".superdevrc",
    "credentials",
    "credentials.json",
    "service-account.json",
    ".npmrc",
    ".netrc",
    ".pypirc",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
];

/// Check whether a write targets a security-sensitive path. Implements
/// **SD-SEC-001**: a bypass-immune guard that blocks the host from writing
/// into version-control internals (`.git/`), secret stores (`.env`,
/// `~/.ssh/`, `~/.aws/`), or the host's own configuration (`.claude/settings`,
/// `.vscode/settings`). Unlike the code-style rules this is a SAFETY check,
/// not a quality check — it fires first and is exempt from any future
/// "skip governance" toggle, mirroring Claude Code's bypass-immune
/// safetyCheck (`utils/permissions/permissions.ts` step 1f/1g).
#[must_use]
pub fn check_sensitive_path(file_path: &str, _content: &str) -> Decision {
    let normalized = file_path.replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();
    // 1. Segment match for sensitive directories: any path component equal to
    //    a SENSITIVE_DIRS entry (or settings.json *inside* .claude/.vscode)
    //    is blocked. Splitting on '/' avoids the `digit.ts` false positive a
    //    naive `.contains(".git")` would produce.
    for seg in lower.split('/') {
        if SENSITIVE_DIRS.contains(&seg) {
            return Decision::block(
                "SD-SEC-001",
                format!(
                    "Super Dev: write to sensitive path `{file_path}` blocked (SD-SEC-001).                      A parent segment (`{seg}`) holds version-control internals, secrets,                      or toolchain config — overwriting it can corrupt the repo or leak                      credentials. If this is intentional, exclude this path from the                      governance hook or run the host outside Super Dev's supervision."
                ),
            );
        }
    }
    // 2. Trailing-path-suffix match: `.env`, `id_rsa`, `settings.json`, etc.
    //    matched against the END of the normalized path so both `.env` and
    //    `apps/api/.env` are caught.
    for suffix in SENSITIVE_PATH_SUFFIXES {
        if lower == *suffix || lower.ends_with(&format!("/{suffix}")) {
            return Decision::block(
                "SD-SEC-001",
                format!(
                    "Super Dev: write to sensitive file `{file_path}` blocked (SD-SEC-001).                      `{suffix}` typically holds secrets, credentials, or toolchain config.                      If this is intentional and not a real secret, rename the file or                      exclude it from the governance hook."
                ),
            );
        }
    }
    Decision::pass()
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
    fn emoji_now_also_blocks_in_markdown() {
        // 4.6+: emoji prohibition extends to docs — the user explicitly hates
        // emoji used as icons/markers anywhere, including markdown.
        assert!(check_emoji("README.md", "# Project 🚀").block);
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

    #[test]
    fn emoji_in_comment_not_flagged_ast() {
        // 4.6 upgrade: an emoji in a comment is documentation, not a violation.
        let d = check_emoji(
            "src/Btn.tsx",
            "// 🚀 placeholder
const x = 1;",
        );
        assert!(!d.block, "emoji in comment must not block");
    }

    #[test]
    fn emoji_in_jsx_still_flagged_ast() {
        let d = check_emoji("src/Btn.tsx", "<button>🔍 Search</button>");
        assert!(d.block);
    }

    #[test]
    fn color_in_comment_not_flagged_ast() {
        // 4.6 upgrade: a hex color in a comment must not block.
        let d = check_color_tokens("src/Card.tsx", "/* use #9333ea for primary */ const x = 1;");
        assert!(!d.block, "color in comment must not block");
    }

    #[test]
    fn color_in_string_still_flagged_ast() {
        // A color in a string literal IS still a violation.
        let d = check_color_tokens("src/Card.tsx", "const c = '#9333ea';");
        assert!(d.block);
    }

    #[test]
    fn emoji_in_string_literal_still_flagged() {
        // An emoji in a string literal is a violation (it's a hardcoded
        // icon) — `without_comments` keeps string literals, so this is
        // correctly flagged. Pins the rule's scoping contract: comment →
        // skip, everything else (JSX text + string + code) → scan.
        let d = check_emoji("src/Btn.tsx", "const ICON = \"🚀\";");
        assert!(d.block, "emoji in a string literal must block");
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

    // --- sensitive path (SD-SEC-001) -----------------------------------

    #[test]
    fn slop_blocks_your_code_here_placeholder() {
        let d = check_ai_slop("src/Form.tsx", "<input placeholder='your code here' />");
        assert!(d.block);
        assert!(d.reason.contains("placeholder"));
    }

    #[test]
    fn slop_blocks_example_com_url() {
        let d = check_ai_slop("src/Api.tsx", "fetch('https://example.com/api')");
        assert!(d.block);
        assert!(d.reason.contains("example.com"));
    }

    #[test]
    fn slop_blocks_fake_email() {
        let d = check_ai_slop("src/Login.tsx", "const demo = 'test@test.com'");
        assert!(d.block);
        assert!(d.reason.contains("email"));
    }

    #[test]
    fn slop_blocks_console_log_residue() {
        let d = check_ai_slop("src/utils.ts", "console.log('debugging here');");
        assert!(d.block);
        assert!(d.reason.contains("console.log"));
    }

    #[test]
    fn sensitive_blocks_dotgit_config() {
        let d = check_sensitive_path("repo/.git/config", "x");
        assert!(d.block);
        assert_eq!(d.clause, "SD-SEC-001");
    }

    #[test]
    fn sensitive_blocks_dotgit_objects_nested() {
        // Nested path inside .git must still be caught.
        let d = check_sensitive_path("/home/u/proj/.git/objects/ab/cdef", "x");
        assert!(d.block);
    }

    #[test]
    fn sensitive_blocks_env_basename_any_dir() {
        // `.env` as a basename is sensitive regardless of directory.
        let d = check_sensitive_path("apps/api/.env", "SECRET=123");
        assert!(d.block);
        assert_eq!(d.clause, "SD-SEC-001");
    }

    #[test]
    fn sensitive_blocks_env_local_and_production() {
        assert!(check_sensitive_path(".env.local", "x").block);
        assert!(check_sensitive_path(".env.production", "x").block);
    }

    #[test]
    fn sensitive_blocks_ssh_private_keys() {
        assert!(check_sensitive_path("/root/.ssh/id_rsa", "x").block);
        assert!(check_sensitive_path("/u/.ssh/id_ed25519", "x").block);
    }

    #[test]
    fn sensitive_blocks_claude_settings_and_vscode() {
        assert!(check_sensitive_path(".claude/settings.json", "x").block);
        assert!(check_sensitive_path(".vscode/settings.json", "x").block);
    }

    #[test]
    fn sensitive_blocks_credentials_files() {
        assert!(check_sensitive_path("~/.aws/credentials", "x").block);
        assert!(check_sensitive_path("config/credentials.json", "x").block);
        assert!(check_sensitive_path("service-account.json", "x").block);
    }

    #[test]
    fn sensitive_normalizes_windows_backslash_paths() {
        // Windows-style backslash path to .git must be caught after normalization.
        let d = check_sensitive_path("C:\\repo\\.git\\config", "x");
        assert!(d.block);
    }

    #[test]
    fn sensitive_is_case_insensitive() {
        // `.ENV` / `.Git/` should still match (defense against casing tricks).
        assert!(check_sensitive_path("proj/.GIT/HEAD", "x").block);
        assert!(check_sensitive_path(".ENV", "x").block);
    }

    #[test]
    fn sensitive_passes_normal_source_files() {
        assert!(!check_sensitive_path("src/Button.tsx", "x").block);
        assert!(!check_sensitive_path("output/prd.md", "x").block);
        assert!(!check_sensitive_path("web/package.json", "x").block);
    }

    #[test]
    fn sensitive_does_not_false_positive_on_env_in_name() {
        // A file merely containing "env" in its name is NOT sensitive.
        assert!(!check_sensitive_path("src/environment.ts", "x").block);
        assert!(!check_sensitive_path("docs/envelope.md", "x").block);
    }

    // --- expanded emoji coverage (SD-CODE-001, 4.6+) ---

    #[test]
    fn emoji_blocks_flags() {
        // Regional indicator symbols (flags) — previously missed.
        let d = check_emoji("src/Lang.tsx", "<span>🇨🇳</span>");
        assert!(d.block);
    }

    #[test]
    fn emoji_blocks_skin_tone_modifier() {
        // Skin-tone modifiers + base — previously the modifier range was missed.
        assert!(check_emoji("src/Hand.tsx", "👍🏽").block);
    }

    #[test]
    fn emoji_blocks_check_mark_and_warning() {
        // Misc symbols that are NOT in the old 2600-27BF+1F300 range.
        assert!(check_emoji("src/Status.tsx", "<Icon>✅</Icon>").block);
        assert!(check_emoji("src/Alert.tsx", "⚠️ danger").block);
        assert!(check_emoji("src/Star.tsx", "⭐ featured").block);
    }

    #[test]
    fn emoji_blocks_keycap_numbers() {
        // Enclosed/keycap-style emoji.
        assert!(check_emoji("src/Step.tsx", "① first").block);
        assert!(check_emoji("src/Num.tsx", "🔟").block);
    }

    #[test]
    fn emoji_blocks_in_html() {
        // .html now guarded (was previously missed).
        assert!(check_emoji("index.html", "<button>🔍 Search</button>").block);
    }

    #[test]
    fn emoji_blocks_in_python() {
        // .py now guarded.
        assert!(check_emoji("app/main.py", "# TODO 🚀 ship it").block);
    }

    #[test]
    fn emoji_blocks_in_css_content() {
        // .css now guarded (emoji in content: property).
        assert!(check_emoji("styles.css", ".icon::before { content: \"🎉\"; }").block);
    }

    #[test]
    fn emoji_passes_cjk_text_unchanged() {
        // CJK ideographs must NOT be treated as emoji (false-positive guard).
        assert!(!check_emoji("src/Label.tsx", "<span>登录</span>").block);
        assert!(!check_emoji("README.md", "# 项目说明").block);
    }

    #[test]
    fn emoji_passes_normal_code_symbols() {
        // Arrows/operators that are NOT emoji must pass.
        assert!(!check_emoji("src/logic.ts", "const x = a >= b ? 1 : 0;").block);
        assert!(!check_emoji("src/arrow.ts", "const f = (x) => x;").block);
    }
}
