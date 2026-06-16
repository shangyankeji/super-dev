//! Markdown-aware chunker — turns one knowledge file into N retrievable chunks.
//!
//! The corpus has three coexisting header schemas (discovered while mapping
//! `knowledge/`):
//! 1. YAML front-matter (`---\nid: ...\ntitle: ...\ntags: [...]\n---`)
//!    — the newer numbered-subdir docs.
//! 2. Blockquote summary (`# Title\n> Version: ...`) — older standards docs.
//! 3. Plain H1 — design-system / seed-template files.
//!
//! The chunker handles all three: it strips front-matter (extracting tags
//! for metadata), then splits the body on `## H2` headings. Each H2 section
//! becomes one [`Chunk`]. A file with no H2 becomes a single chunk holding
//! the whole body. The H1 title propagates to every chunk as `title`.

use std::path::Path;

use crate::tokenizer::tokenize;

/// Per-chunk metadata parsed from front-matter + path.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChunkMeta {
    /// Path relative to the `knowledge/` root (e.g. `security/01-standards/owasp.md`).
    pub path: String,
    /// The H1 title (first `# ` line), stripped of leading `# `.
    pub title: String,
    /// The `## H2` heading this chunk sits under, or the title if no H2.
    pub section: String,
    /// Tags from YAML front-matter `tags: [...]`, empty when absent.
    pub tags: Vec<String>,
    /// Top-level domain directory (e.g. `security`, `database`), parsed
    /// from the first path segment. Empty for files directly in `knowledge/`.
    pub domain: String,
    /// Difficulty from front-matter (`beginner`/`intermediate`/`advanced`).
    /// `None` when absent — the majority of legacy files have no front-matter.
    #[serde(default)]
    pub difficulty: Option<String>,
}

/// One retrievable unit: metadata + tokenised body + raw text excerpt.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Chunk {
    /// Where this chunk came from.
    pub meta: ChunkMeta,
    /// The raw markdown body of this H2 section (or whole file).
    pub body: String,
    /// Pre-tokenised body — used by the BM25 index without re-tokenising.
    #[serde(default)]
    pub tokens: Vec<String>,
    /// Quality score (0-100) from front-matter, used as a weak reranking
    /// signal in retrieval. `None` (treated as 50 = neutral) when absent.
    #[serde(default)]
    pub quality_score: Option<u32>,
}

impl Chunk {
    /// First `max_chars` of the body, with a trailing ellipsis if trimmed.
    /// Used when rendering chunk hits into a prompt digest.
    #[must_use]
    pub fn excerpt(&self, max_chars: usize) -> String {
        let body = self.body.trim_start();
        // Guard: max_chars=0 previously still appended '…' (because the
        // `<= max_chars` check only short-circuited for empty bodies), and
        // max_chars=1 saturated to 0 chars + '…'. Treat 0 as "no excerpt".
        if max_chars == 0 || body.is_empty() {
            return body.to_string();
        }
        if body.chars().count() <= max_chars {
            return body.to_string();
        }
        // Reserve 1 char for the ellipsis; clamp so a tiny max_chars still
        // shows at least the first char (max_chars=1 → first char + '…').
        let take = max_chars.saturating_sub(1).max(1);
        let mut s: String = body.chars().take(take).collect();
        s.push('…');
        s
    }
}

/// Parsed front-matter: the YAML block (if present) + remaining body text.
struct ParsedFrontMatter {
    /// Tags extracted from `tags: [...]`. Empty when no front-matter.
    tags: Vec<String>,
    /// Difficulty (`beginner`/`intermediate`/`advanced`).
    difficulty: Option<String>,
    /// Quality score (0-100) for reranking.
    quality_score: Option<u32>,
    /// Body with the front-matter block removed.
    body: String,
}

/// Chunk a markdown file from disk. Returns one or more chunks. Empty vec
/// only when the file is unreadable or completely empty.
pub fn chunk_file(knowledge_root: &Path, abs_path: &Path) -> Vec<Chunk> {
    let Ok(body) = std::fs::read_to_string(abs_path) else {
        return Vec::new();
    };
    // Path relative to knowledge/ root (or fall back to the file name).
    let rel = abs_path.strip_prefix(knowledge_root).map_or_else(
        |_| {
            abs_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        },
        |p| p.to_string_lossy().to_string(),
    );
    chunk_text(&rel, &body)
}

/// Pure chunker over in-memory text — exposed for tests and the vector
/// layer (which re-chunks the same text to embed).
#[must_use]
pub fn chunk_text(rel_path: &str, body: &str) -> Vec<Chunk> {
    if body.trim().is_empty() {
        return Vec::new();
    }

    let ParsedFrontMatter {
        tags,
        difficulty,
        quality_score,
        body,
    } = strip_front_matter(body);
    let title = extract_h1_title(&body).unwrap_or_else(|| {
        // Fall back to the last path segment without extension.
        rel_path
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(rel_path)
            .trim_end_matches(".md")
            .to_string()
    });
    let domain = rel_path
        .split(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty() && !s.ends_with(".md"))
        .unwrap_or("")
        .to_string();

    // Strip the H1 line + any blockquote preamble before sectioning, so the
    // Overview chunk holds real intro content, not the title echo.
    let body_without_h1 = strip_h1_and_preamble(&body);

    // Split into H2 sections. A section is everything from one `## ` line
    // up to (but not including) the next `## ` line or EOF.
    let sections = split_on_h2(&body_without_h1);

    sections
        .into_iter()
        .map(|(heading, section_body)| {
            let trimmed = section_body.trim().to_string();
            let tokens = tokenize(&format!("{title} {heading} {trimmed}"));
            Chunk {
                meta: ChunkMeta {
                    path: rel_path.to_string(),
                    title: title.clone(),
                    section: heading,
                    tags: tags.clone(),
                    domain: domain.clone(),
                    difficulty: difficulty.clone(),
                },
                body: trimmed,
                tokens,
                quality_score,
            }
        })
        .collect()
}

/// Detect + strip a leading `---\n...\n---\n` YAML block. Extracts `tags`
/// from a `tags: [a, b, c]` line. Returns the body with the block removed.
///
/// Line-oriented for robustness: byte-index arithmetic on front-matter is
/// fragile (the `---` opener/closer look identical to each other and to a
/// markdown thematic break). Scanning lines avoids that ambiguity.
fn strip_front_matter(body: &str) -> ParsedFrontMatter {
    let mut lines = body.lines().peekable();
    // The first non-blank line must be exactly `---` to count as front-matter.
    let first = loop {
        match lines.peek() {
            Some(l) if l.trim().is_empty() => {
                lines.next();
            }
            Some(l) => break l.trim(),
            None => break "",
        }
    };
    if first != "---" {
        return ParsedFrontMatter {
            tags: Vec::new(),
            difficulty: None,
            quality_score: None,
            body: body.to_string(),
        };
    }
    // Consume the opening `---`.
    lines.next();

    // Collect YAML lines until the closing `---`.
    let mut yaml_lines: Vec<&str> = Vec::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        yaml_lines.push(line);
    }
    if !closed {
        // No closing fence → not valid front-matter; return the body as-is.
        return ParsedFrontMatter {
            tags: Vec::new(),
            difficulty: None,
            quality_score: None,
            body: body.to_string(),
        };
    }

    // Whatever remains in `lines` is the real body.
    let yaml_text = yaml_lines.join("\n");
    let fields = parse_front_matter_fields(&yaml_text);
    let rest: String = lines.collect::<Vec<_>>().join("\n");
    ParsedFrontMatter {
        tags: fields.tags,
        difficulty: fields.difficulty,
        quality_score: fields.quality_score,
        body: rest,
    }
}

/// Fields extracted from a front-matter YAML block.
struct FrontMatterFields {
    tags: Vec<String>,
    difficulty: Option<String>,
    quality_score: Option<u32>,
}

/// Parse the YAML front-matter fields we care about: `tags` (inline array,
/// inline scalar, or block `- item` form), `difficulty`, and `quality_score`.
/// Line-oriented (no serde_yaml dependency) — covers all field shapes present
/// in the `knowledge/` corpus.
fn parse_front_matter_fields(yaml: &str) -> FrontMatterFields {
    let mut tags = Vec::new();
    let mut difficulty = None;
    let mut quality_score = None;
    let mut in_tags_block = false;

    for line in yaml.lines() {
        let trimmed = line.trim();

        // Block-form tags: lines like `  - item` following a bare `tags:`.
        if in_tags_block {
            if let Some(item) = trimmed.strip_prefix("- ") {
                tags.push(item.trim().trim_matches(['\'', '"']).to_string());
                continue;
            }
            // First non-`- ` line ends the tags block.
            in_tags_block = false;
        }

        if let Some(rest) = trimmed.strip_prefix("tags:") {
            let rest = rest.trim();
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                // Inline array form: `tags: [a, b, c]`
                tags = inner
                    .split(',')
                    .map(|s| s.trim().trim_matches(['\'', '"']).to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            } else if !rest.is_empty() {
                // Inline scalar form: `tags: api`
                tags.push(rest.trim_matches(['\'', '"']).to_string());
            } else {
                // Bare `tags:` → block form on following lines.
                in_tags_block = true;
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("difficulty:") {
            let v = rest.trim().trim_matches(['\'', '"']);
            if !v.is_empty() {
                difficulty = Some(v.to_string());
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("quality_score:") {
            let v = rest.trim();
            if let Ok(n) = v.parse::<u32>() {
                quality_score = Some(n);
            }
        }
    }

    FrontMatterFields {
        tags,
        difficulty,
        quality_score,
    }
}

/// First `# Title` line, returning the title without the leading `# `.
fn extract_h1_title(body: &str) -> Option<String> {
    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("# ") {
            return Some(rest.trim().to_string());
        }
        // Skip blank lines / blockquotes before the H1.
        if !trimmed.is_empty() && !trimmed.starts_with('>') {
            // Hit body content before an H1 — no title.
            return None;
        }
    }
    None
}

/// Remove the leading H1 line and any blockquote preamble that precedes
/// real content, so the first chunk's body is actual prose, not a title
/// echo. Lines after the H1 are preserved (including the intro paragraph).
fn strip_h1_and_preamble(body: &str) -> String {
    let mut saw_h1 = false;
    let mut out = String::new();
    for line in body.lines() {
        let trimmed = line.trim_start();
        if !saw_h1 {
            // Skip blockquote preamble (`> Version: ...`) before the H1.
            if trimmed.starts_with('>') || trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("# ") {
                saw_h1 = true;
                continue; // drop the H1 line itself
            }
            // Content before any H1 — keep it (H1 was absent).
            saw_h1 = true;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.trim_start().to_string()
}

/// Split body into `(heading, body)` pairs at each `## ` line. Content
/// before the first `## ` (the H1 line + any intro) is attached to a
/// synthetic "Overview" section so it's not lost.
fn split_on_h2(body: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current_heading = String::from("Overview");
    let mut current_body = String::new();

    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("## ") {
            // Flush the previous section.
            sections.push((
                std::mem::take(&mut current_heading),
                std::mem::take(&mut current_body),
            ));
            current_heading = rest.trim().to_string();
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }
    sections.push((current_heading, current_body));

    // Drop a purely-empty Overview (happens when the file starts with `## `).
    sections.retain(|(_, b)| !b.trim().is_empty());
    if sections.is_empty() {
        // No H2 at all and nothing left — emit the whole body as one chunk.
        // (Must RETURN this, not just build it — the previous code built the
        // vec and discarded it, so an all-empty-H2 document indexed as zero
        // chunks instead of one.)
        return vec![(String::from("Document"), body.to_string())];
    }
    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_h1_splits_on_h2() {
        let md = "# Login Playbook\n\nIntro line.\n\n## OAuth Flow\n\nUse PKCE.\n\n## Risks\n\nToken theft.";
        let chunks = chunk_text("security/login.md", md);
        assert_eq!(chunks.len(), 3); // Overview + OAuth Flow + Risks
        assert_eq!(chunks[0].meta.section, "Overview");
        assert!(chunks[0].body.contains("Intro line"));
        assert_eq!(chunks[1].meta.section, "OAuth Flow");
        assert!(chunks[1].body.contains("PKCE"));
        assert_eq!(chunks[2].meta.section, "Risks");
        assert_eq!(chunks[1].meta.title, "Login Playbook");
        assert_eq!(chunks[1].meta.domain, "security");
        assert_eq!(chunks[1].meta.path, "security/login.md");
    }

    #[test]
    fn yaml_front_matter_stripped_and_tags_kept() {
        let md = "---\nid: x\ntitle: Postgres\ntags: [postgresql, database, optimization]\n---\n# Postgres\n\nIntro about Postgres.\n\n## Tuning\n\nshared_buffers.";
        let chunks = chunk_text("database/postgres.md", md);
        assert_eq!(chunks.len(), 2); // Overview (intro) + Tuning
        assert_eq!(
            chunks[0].meta.tags,
            vec!["postgresql", "database", "optimization"]
        );
        // Front-matter is gone from the body.
        assert!(!chunks[0].body.contains("---"));
        assert!(!chunks[0].body.contains("id: x"));
        assert_eq!(chunks[0].meta.title, "Postgres");
        // Tuning section captured as its own chunk.
        assert!(chunks[1].body.contains("shared_buffers"));
    }

    #[test]
    fn no_h2_emits_single_chunk() {
        let md = "# Tiny\n\nJust one section, no H2.";
        let chunks = chunk_text("notes.md", md);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].body.contains("Just one section"));
    }

    #[test]
    fn blockquote_before_h1_is_skipped_for_title() {
        let md = "> Version: 2\n> Last Updated: 2024\n\n# OWASP Top 10\n\n## A01\n\nBroken Access.";
        let chunks = chunk_text("security/owasp.md", md);
        assert_eq!(chunks[0].meta.title, "OWASP Top 10");
    }

    #[test]
    fn tokens_precomputed() {
        let md = "# Auth\n\n## Login\n\nUse OAuth2 with PKCE.";
        let chunks = chunk_text("api/auth.md", md);
        // After stripping the H1, only the `## Login` chunk remains.
        assert_eq!(chunks.len(), 1);
        let login = &chunks[0];
        assert!(login.tokens.contains(&"oauth2".to_string()));
        assert!(login.tokens.contains(&"pkce".to_string()));
        assert!(login.tokens.contains(&"login".to_string()));
    }

    #[test]
    fn empty_body_yields_nothing() {
        assert!(chunk_text("x.md", "   \n\n   ").is_empty());
        assert!(chunk_text("x.md", "").is_empty());
    }

    #[test]
    fn excerpt_truncates_long_body() {
        let long = "# T\n\n## S\n\n".to_string() + &"content ".repeat(200);
        let chunks = chunk_text("t.md", &long);
        let e = chunks[0].excerpt(50);
        assert!(e.chars().count() <= 51); // 50 + ellipsis
        assert!(e.ends_with('…'));
    }

    #[test]
    fn excerpt_zero_max_chars_returns_body() {
        // Regression: max_chars=0 used to append a stray '…'. Now returns
        // the body unchanged.
        let chunks = chunk_text("t.md", "# T\n\n## S\n\nsome body text");
        let c = &chunks[0];
        assert_eq!(c.excerpt(0), "some body text");
        assert_eq!(c.excerpt(1), "s…"); // max_chars=1 → first char + ellipsis
    }

    #[test]
    fn excerpt_short_body_returned_intact() {
        let md = "# T\n\n## S\n\nshort";
        let chunks = chunk_text("t.md", md);
        assert_eq!(chunks[0].excerpt(100), "short");
    }

    #[test]
    fn domain_extracted_from_first_path_segment() {
        let md = "# X\n\n## S\n\nbody";
        assert_eq!(
            chunk_text("security/sub/x.md", md)[0].meta.domain,
            "security"
        );
        // File directly under knowledge/ root → empty domain.
        assert_eq!(chunk_text("root-level.md", md)[0].meta.domain, "");
    }

    #[test]
    fn inline_scalar_tag_form() {
        let md = "---\ntags: auth\n---\n# X\n\n## S\n\nbody";
        let chunks = chunk_text("x.md", md);
        assert_eq!(chunks[0].meta.tags, vec!["auth"]);
    }

    #[test]
    fn block_form_tags_parsed() {
        let md = "---\ntags:\n  - login\n  - security\n---\n# X\n\n## S\n\nbody";
        let chunks = chunk_text("x.md", md);
        assert_eq!(chunks[0].meta.tags, vec!["login", "security"]);
    }

    #[test]
    fn difficulty_and_quality_score_parsed() {
        let md = "---\nid: x\ndifficulty: advanced\nquality_score: 92\ntags: [api]\n---\n# X\n\n## S\n\nbody";
        let chunks = chunk_text("x.md", md);
        assert_eq!(chunks[0].meta.difficulty.as_deref(), Some("advanced"));
        assert_eq!(chunks[0].quality_score, Some(92));
        assert_eq!(chunks[0].meta.tags, vec!["api"]);
    }

    #[test]
    fn missing_front_matter_fields_are_none() {
        let md = "# X\n\n## S\n\nbody";
        let chunks = chunk_text("x.md", md);
        assert!(chunks[0].meta.difficulty.is_none());
        assert!(chunks[0].quality_score.is_none());
    }

    #[test]
    fn front_matter_fields_backwards_compatible() {
        // Old cache blobs without quality_score must still deserialize.
        let json = r#"{"meta":{"path":"a","title":"A","section":"S","tags":[],"domain":""},"body":"b","tokens":["t"]}"#;
        let chunk: Chunk = serde_json::from_str(json).unwrap();
        assert!(chunk.quality_score.is_none());
        assert!(chunk.meta.difficulty.is_none());
    }
}
