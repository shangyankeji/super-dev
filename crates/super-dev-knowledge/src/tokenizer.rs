//! Tokeniser — the foundation of BM25 quality.
//!
//! Two scripts live in the knowledge corpus:
//! - English / Latin: split on non-alphanumeric, lowercase, drop < 2 chars.
//! - CJK (Chinese/Japanese/Korean): the old code threw CJK away entirely
//!   (it only kept `is_ascii_alphanumeric` runs), so a Chinese requirement
//!   like "做一个登录系统" produced ZERO keywords and the retriever fell back
//!   to dictionary order. That is the bug this module exists to fix.
//!
//! CJK strategy: split into character bigrams. Bigram matching is the
//! classic lightweight CJK retrieval technique — it captures enough local
//! context ("登录" / "系统") to rank relevant documents without a
//! dictionary or segmentation model. Single CJK chars are also emitted as
//! unigrams so a one-char query term still hits.
//!
//! All tokens are lowercased ASCII / verbatim CJK — no stopword removal at
//! this layer (BM25's IDF naturally downweights common tokens).

/// Maximum length of an ASCII token to keep (filters out "the"-sized noise
/// that also drags BM25 quality, but keeps acronyms like "api", "css").
const MIN_ASCII_LEN: usize = 2;

/// One token + whether it originated from CJK (for scoring/debugging).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenScript {
    /// Latin/digit run.
    Ascii,
    /// CJK bigram or unigram.
    Cjk,
}

/// Tokenise `text` into lowercase terms. ASCII runs ≥ 2 chars become one
/// token each; each CJK character becomes a unigram token AND pairs with
/// its successor become bigram tokens.
///
/// Returns a flat `Vec<String>` — the script info is internal; BM25 only
/// needs the term strings. Duplicates are intentionally kept (BM25 term
/// frequency is per-occurrence).
#[must_use]
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut ascii_buf = String::new();

    // Flush the pending ASCII buffer as a single token (if long enough).
    let flush_ascii = |buf: &mut String, out: &mut Vec<String>| {
        if buf.len() >= MIN_ASCII_LEN {
            out.push(buf.clone());
        }
        buf.clear();
    };

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_alphanumeric() {
            // Accumulate into the ASCII buffer.
            ascii_buf.push(c.to_ascii_lowercase());
            i += 1;
            continue;
        }
        // Non-ASCII-alphanumeric: flush any pending ASCII token first.
        flush_ascii(&mut ascii_buf, &mut tokens);

        if is_cjk(c) {
            // CJK unigram.
            tokens.push(c.to_string());
            // CJK bigram with the next char if it's also CJK.
            if i + 1 < chars.len() && is_cjk(chars[i + 1]) {
                tokens.push(format!("{}{}", c, chars[i + 1]));
            }
        }
        // Non-CJK punctuation / whitespace / other scripts: skipped.
        i += 1;
    }
    flush_ascii(&mut ascii_buf, &mut tokens);
    tokens
}

/// Whether a character falls in the common CJK unified ideograph ranges.
/// Covers CJK Unified, Ext-A, and the CJK-compatible ideographs — enough
/// for Chinese/Japanese kanji/Korean hanja content in the knowledge base.
fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x4E00..=0x9FFF   // CJK Unified Ideographs
        | 0x3400..=0x4DBF // CJK Unified Extension A
        | 0xF900..=0xFAFF // CJK Compatibility Ideographs
        | 0x3040..=0x30FF // Hiragana + Katakana
        | 0xAC00..=0xD7AF // Hangul Syllables
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_split_on_punctuation() {
        let t = tokenize("Login, OAuth2 & PKCE!");
        assert_eq!(t, vec!["login", "oauth2", "pkce"]);
    }

    #[test]
    fn ascii_drops_short_tokens() {
        // Single chars and empty runs are dropped.
        assert_eq!(tokenize("a b c api"), vec!["api"]);
    }

    #[test]
    fn ascii_lowercased() {
        assert!(tokenize("PostgreSQL").contains(&"postgresql".to_string()));
    }

    #[test]
    fn cjk_unigrams_emitted() {
        let t = tokenize("登录系统");
        // Each CJK char is a unigram.
        assert!(t.contains(&"登".to_string()));
        assert!(t.contains(&"录".to_string()));
        assert!(t.contains(&"系".to_string()));
        assert!(t.contains(&"统".to_string()));
    }

    #[test]
    fn cjk_bigrams_emitted() {
        let t = tokenize("登录");
        assert!(t.contains(&"登录".to_string()));
    }

    #[test]
    fn cjk_mixed_with_ascii() {
        let t = tokenize("做一个 OAuth2 登录系统");
        assert!(t.contains(&"oauth2".to_string()));
        assert!(t.contains(&"登录".to_string()));
        assert!(t.contains(&"做".to_string()));
    }

    #[test]
    fn cjk_requirement_that_old_code_dropped() {
        // The exact failure case: pure CJK requirement used to yield ZERO
        // tokens under the old ASCII-only keyword extractor.
        let t = tokenize("做一个登录系统");
        assert!(!t.is_empty(), "CJK must produce tokens, not be dropped");
        assert!(t.contains(&"登录".to_string()));
    }

    #[test]
    fn digits_kept() {
        assert!(tokenize("port 8080").contains(&"8080".to_string()));
    }

    #[test]
    fn underscore_split() {
        // snake_case identifiers split into parts.
        let t = tokenize("rate_limiting");
        assert!(t.contains(&"rate".to_string()));
        assert!(t.contains(&"limiting".to_string()));
    }

    #[test]
    fn empty_text_yields_nothing() {
        assert!(tokenize("").is_empty());
        assert!(tokenize("   \n\t  ").is_empty());
    }

    #[test]
    fn mixed_punctuation_between_cjk() {
        let t = tokenize("登录(OAuth)");
        assert!(t.contains(&"登录".to_string()));
        assert!(t.contains(&"oauth".to_string()));
    }
}
