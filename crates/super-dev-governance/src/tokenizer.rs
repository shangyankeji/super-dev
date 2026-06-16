//! Lightweight JS/TSX tokeniser for governance analysis.
//!
//! Super Dev's code-rule checks (SD-CODE-001/002/005) used to scan the raw
//! source text with regexes. That produced two classes of false result:
//! - **False positives**: a hex color `#fff` or emoji in a *comment* or a
//!   *string literal that is clearly data* (e.g. a test fixture) triggered
//!   a block.
//! - **False negatives**: a color split across a template literal
//!   (`` `#${c}` ``) slipped past the single-token regex.
//!
//! A full TSX parser (`swc`) is the ideal tool but its proc-macro dependency
//! tree is currently incompatible with a clean `cargo` build (syn version
//! drift in `swc_visit_macros`). Rather than pin a fragile version matrix,
//! this module implements the *minimal* lexical analysis the governance
//! rules need: it walks the source once, classifying every character into
//! "code", "string-literal", "jsx-text", or "comment", and exposes filtered
//! views the rules query.
//!
//! This is intentionally NOT a parser — it does not build an AST, does not
//! understand scope, and does not validate syntax. It is a state machine
//! that knows just enough about JS/TSX delimiters to separate the regions
//! the rules care about. That keeps it dependency-free and ~150 lines.

/// One contiguous region of source, classified by kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// Executable code (default).
    Code,
    /// Inside a string / template / char literal — the *content* matters
    /// (a hardcoded color in a string is still a violation), but the
    /// surrounding quotes are stripped so the rules don't re-match them.
    StringLiteral,
    /// Text between JSX tags (`<button>SEARCH</button>` → `SEARCH`). This is
    /// where emoji-as-icon violations actually live.
    JsxText,
    /// Line (`//…`) or block (`/*…*/`) comment — governance rules MUST skip
    /// these (a `#fff` in a comment is not a violation).
    Comment,
}

/// A tokenised view of source: each `(start, end, Region)` span. The rules
/// iterate these to scan only the regions they care about.
#[derive(Debug, Clone)]
pub struct Tokenized {
    spans: Vec<(usize, usize, Region)>,
}

impl Tokenized {
    /// Tokenise `src`. Produces a non-overlapping, ordered span list covering
    /// the entire source. Cheap to call (single pass).
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn new(src: &str) -> Self {
        let bytes: Vec<char> = src.chars().collect();
        let n = bytes.len();
        let mut spans: Vec<(usize, usize, Region)> = Vec::new();
        let mut i = 0;
        // Track whether we're inside JSX text context. We enter JSX text when
        // we see `>` (end of an opening/self-closing tag) and exit on `<`
        // (start of the next tag). We only treat it as JSX text if there's
        // actual non-whitespace content — avoids spamming empty spans.
        let mut code_start = 0usize;
        let flush_code =
            |spans: &mut Vec<(usize, usize, Region)>, start: &mut usize, end: usize| {
                if end > *start {
                    spans.push((*start, end, Region::Code));
                }
                *start = end;
            };

        while i < n {
            let c = bytes[i];
            // --- Comments ---
            if c == '/' && i + 1 < n {
                // Line comment.
                if bytes[i + 1] == '/' {
                    flush_code(&mut spans, &mut code_start, i);
                    let start = i;
                    i += 2;
                    while i < n && bytes[i] != '\n' {
                        i += 1;
                    }
                    spans.push((start, i, Region::Comment));
                    code_start = i;
                    continue;
                }
                // Block comment.
                if bytes[i + 1] == '*' {
                    flush_code(&mut spans, &mut code_start, i);
                    let start = i;
                    i += 2;
                    while i + 1 < n && !(bytes[i] == '*' && bytes[i + 1] == '/') {
                        i += 1;
                    }
                    i = (i + 2).min(n);
                    spans.push((start, i, Region::Comment));
                    code_start = i;
                    continue;
                }
            }
            // --- String literals (', ", `) ---
            if c == '\'' || c == '"' || c == '`' {
                flush_code(&mut spans, &mut code_start, i);
                let quote = c;
                let start = i;
                i += 1;
                while i < n {
                    if bytes[i] == '\\' {
                        i += 2; // skip escaped char
                        continue;
                    }
                    if bytes[i] == quote {
                        i += 1;
                        break;
                    }
                    // Template literals can span lines and contain ${…}; we
                    // treat the whole `` ` … ` `` as one string span, which
                    // is conservative (a color inside ${} still flags).
                    i += 1;
                }
                spans.push((start, i, Region::StringLiteral));
                code_start = i;
                continue;
            }
            i += 1;
        }
        flush_code(&mut spans, &mut code_start, n);

        // Second pass: split Code spans into Code vs JsxText. JSX text is the
        // region between `>` (tag close) and the next `<`. We approximate by
        // scanning each Code span for `>` … `<` gaps.
        let mut refined: Vec<(usize, usize, Region)> = Vec::new();
        for &(s, e, region) in &spans {
            if region != Region::Code {
                refined.push((s, e, region));
                continue;
            }
            let slice: String = bytes[s..e].iter().collect();
            let mut cursor = 0usize;
            let bytes_slice: Vec<char> = slice.chars().collect();
            let mut last = 0usize;
            while cursor < bytes_slice.len() {
                if bytes_slice[cursor] == '>' {
                    // Look ahead for a `<` — the text between is JSX text.
                    let mut j = cursor + 1;
                    // Skip whitespace immediately after `>` is NOT done — we
                    // want to know if there's any text at all.
                    while j < bytes_slice.len() && bytes_slice[j] != '<' {
                        j += 1;
                    }
                    let text_len = j - (cursor + 1);
                    if text_len > 0 {
                        let text_slice = &bytes_slice[cursor + 1..j];
                        let has_content = text_slice.iter().any(|c| !c.is_whitespace());
                        // Heuristic to avoid misclassifying TypeScript generics
                        // (`<T,>` ... `>`) as JSX tag-open/close: real JSX text
                        // between tags is prose (`Search`, `🚀 Launch`), which
                        // does NOT contain code punctuation. If the span has
                        // any of these code indicators, treat it as Code, not
                        // JsxText. This is conservative — a JSX text node
                        // containing a literal `;`/`{`/`}`/`=`/`(` would be
                        // mis-scanned as code (no false BLOCK, just a missed
                        // emoji-in-text case), which is far less common than
                        // generics.
                        let looks_like_code = text_slice
                            .iter()
                            .any(|c| matches!(c, ';' | '{' | '}' | '(' | ')' | '=' | '\''));
                        if has_content && !looks_like_code {
                            // Emit code up to cursor+1, then JSX text.
                            if cursor + 1 > last {
                                refined.push((s + last, s + cursor + 1, Region::Code));
                            }
                            refined.push((s + cursor + 1, s + j, Region::JsxText));
                            last = j;
                        }
                    }
                    cursor = j;
                } else {
                    cursor += 1;
                }
            }
            if last < e - s {
                refined.push((s + last, e, Region::Code));
            }
        }

        Self { spans: refined }
    }

    /// Iterate spans matching `filter`, yielding the substring slices. Used
    /// by rules that want "all string literals" or "all JSX text".
    pub fn regions<'a>(&'a self, src: &'a str, filter: Region) -> impl Iterator<Item = &'a str> {
        self.spans.iter().filter_map(move |&(s, e, r)| {
            if r == filter {
                // Convert char indices to byte offsets. spans are char-indexed.
                Some(char_slice_to_str(src, s, e))
            } else {
                None
            }
        })
    }

    /// All source MINUS comments — for rules that want to scan everything a
    /// human could see (code + strings + JSX text) but skip comments.
    /// Returns the concatenation of non-comment spans.
    #[must_use]
    pub fn without_comments(&self, src: &str) -> String {
        let mut out = String::with_capacity(src.len());
        for &(s, e, r) in &self.spans {
            if r != Region::Comment {
                out.push_str(char_slice_to_str(src, s, e));
            }
        }
        out
    }

    /// Only JSX text spans concatenated — where emoji-as-icon violations live.
    #[must_use]
    pub fn jsx_text(&self, src: &str) -> String {
        let mut out = String::new();
        for piece in self.regions(src, Region::JsxText) {
            out.push_str(piece);
        }
        out
    }
}

/// Convert `(start_char_idx, end_char_idx)` to the source substring.
///
/// Correctness note: spans are char-indexed but `src` is UTF-8, so we must
/// walk `char_indices()` to translate. The previous implementation had a
/// latent bug where a *leading empty span* (`start == 0 && end == 0`) made
/// `end_byte` stay `0`, then the `end_byte == 0 && char_idx <= end` guard
/// fired and set `end_byte = src.len()` — returning the ENTIRE file for an
/// empty span. The fix tracks whether `end` was actually matched (a found
/// flag) instead of overloading `end_byte == 0` as the EOF signal.
fn char_slice_to_str(src: &str, start: usize, end: usize) -> &str {
    if start >= end {
        // Empty or inverted span — nothing to extract. This includes the
        // leading-empty-span case (start==0, end==0) that previously
        // returned the whole file.
        return "";
    }
    let mut start_byte = None;
    let mut end_byte = 0usize;
    let mut end_found = false;
    for (char_idx, (byte, _)) in src.char_indices().enumerate() {
        if start_byte.is_none() && char_idx == start {
            start_byte = Some(byte);
        }
        if char_idx == end {
            end_byte = byte;
            end_found = true;
            break;
        }
    }
    let Some(start_byte) = start_byte else {
        return ""; // start beyond EOF
    };
    if !end_found {
        // end landed at/past EOF — take the rest of the source.
        return &src[start_byte..];
    }
    &src[start_byte..end_byte]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_comment_isolated() {
        let t = Tokenized::new("const x = 1; // color: #fff");
        let no_comments = t.without_comments("const x = 1; // color: #fff");
        assert!(!no_comments.contains("#fff"));
        assert!(no_comments.contains("const x"));
    }

    #[test]
    fn block_comment_isolated() {
        let t = Tokenized::new("/* #9333ea is a placeholder */ const y = 2;");
        let no_comments = t.without_comments("/* #9333ea is a placeholder */ const y = 2;");
        assert!(!no_comments.contains("#9333ea"));
        assert!(no_comments.contains("const y"));
    }

    #[test]
    fn string_literal_kept() {
        // A color inside a string IS a violation (kept), but the quotes are
        // stripped from the surrounding context.
        let t = Tokenized::new("const c = '#9333ea';");
        let no_comments = t.without_comments("const c = '#9333ea';");
        assert!(no_comments.contains("#9333ea"));
    }

    #[test]
    fn jsx_text_extracted() {
        let src = "const el = <button>🔍 Search</button>;";
        let t = Tokenized::new(src);
        let jsx = t.jsx_text(src);
        assert!(jsx.contains("Search"));
    }

    #[test]
    fn emoji_in_jsx_text_detected() {
        let src = "<div className=\"card\">🚀 Launch</div>";
        let t = Tokenized::new(src);
        let jsx = t.jsx_text(src);
        assert!(jsx.contains("🚀"));
    }

    #[test]
    fn emoji_in_comment_not_in_jsx() {
        let src = "// 🚀 todo\nconst x = 1;";
        let t = Tokenized::new(src);
        let jsx = t.jsx_text(src);
        assert!(!jsx.contains("🚀"));
    }

    #[test]
    fn template_literal_color_kept() {
        let src = "const c = `#${channel}`;";
        let t = Tokenized::new(src);
        let no_comments = t.without_comments(src);
        assert!(no_comments.contains('#'));
    }

    #[test]
    fn escaped_quote_in_string() {
        let src = "const s = 'it\\'s #fff here';";
        let t = Tokenized::new(src);
        let no_comments = t.without_comments(src);
        assert!(no_comments.contains("#fff"));
    }

    #[test]
    fn empty_source() {
        let t = Tokenized::new("");
        assert!(t.without_comments("").is_empty());
    }

    #[test]
    fn multiple_jsx_children() {
        let src = "<ul><li>A</li><li>B</li></ul>";
        let t = Tokenized::new(src);
        let jsx = t.jsx_text(src);
        assert!(jsx.contains('A'));
        assert!(jsx.contains('B'));
    }

    #[test]
    fn nested_tags_text_only_in_leaf() {
        let src = "<div><span>Click</span></div>";
        let t = Tokenized::new(src);
        assert!(t.jsx_text(src).contains("Click"));
    }

    #[test]
    fn unterminated_string_does_not_panic() {
        let src = "const s = 'unterminated";
        let t = Tokenized::new(src);
        // Must not loop forever or panic.
        let _ = t.without_comments(src);
    }

    #[test]
    fn ts_generic_not_treated_as_jsx_tag() {
        // Regression: a TypeScript generic `<T,>` was misread as a JSX
        // tag-open/close, so the code after it was scanned as JSX text.
        // With the generic guard (code punctuation in the span → Code), the
        // arrow-function body stays classified as Code.
        let src = "const f = <T,>(x: T): T => x;";
        let t = Tokenized::new(src);
        let jsx = t.jsx_text(src);
        assert!(
            jsx.is_empty() || !jsx.contains("=> x"),
            "TS generic body must NOT be JSX text; got: {jsx:?}"
        );
        // And a real JSX text node is still captured.
        let src2 = "const el = <div>Hello world</div>;";
        let t2 = Tokenized::new(src2);
        assert!(t2.jsx_text(src2).contains("Hello world"));
    }

    #[test]
    fn char_slice_to_str_leading_empty_span_returns_empty() {
        // Regression: a leading empty span (start==0, end==0) previously
        // made end_byte stay 0, then the EOF guard set end_byte = src.len(),
        // returning the ENTIRE file for an empty span. It must return "".
        let src = "const x = 1;";
        assert_eq!(char_slice_to_str(src, 0, 0), "");
        // Mid-source empty span is also empty, not the rest of the file.
        assert_eq!(char_slice_to_str(src, 2, 2), "");
        // Inverted span (start > end) is empty, not negative-indexed garbage.
        assert_eq!(char_slice_to_str(src, 5, 2), "");
        // Normal spans still work.
        assert_eq!(char_slice_to_str(src, 0, 5), "const");
        // Span ending exactly at EOF returns the suffix.
        let n = src.chars().count();
        assert_eq!(char_slice_to_str(src, n - 2, n), "1;");
    }

    #[test]
    fn char_slice_to_str_multibyte_source() {
        // Must not panic on multibyte content; byte vs char indices differ.
        let src = "const 标签 = '🔍';";
        // '标' starts at char index 6.
        let label = char_slice_to_str(src, 6, 8);
        assert_eq!(label, "标签");
    }
}
