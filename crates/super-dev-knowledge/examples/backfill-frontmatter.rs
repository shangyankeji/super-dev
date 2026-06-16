//! Backfill YAML front-matter on knowledge files that lack it.
//!
//! 80% of the `knowledge/` corpus (295 files) has no front-matter, which
//! means the chunker's tag/difficulty/quality_score fields are empty for
//! those files and hybrid retrieval can't use the quality reranking signal.
//!
//! This tool adds a canonical front-matter block to every bare `.md` file
//! under `knowledge/`, deriving `id`, `title`, `domain`, `category`, `tags`
//! from the file path and content. It is **idempotent**: files that already
//! start with `---` are skipped.
//!
//! ## Usage
//! ```sh
//! # Dry run (list what would change, write nothing):
//! cargo run --example backfill-frontmatter -- --dry-run
//!
//! # Apply:
//! cargo run --example backfill-frontmatter
//! ```
//!
//! Run from the repo root so `knowledge/` resolves.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Default quality score for backfilled files. Human-tuneable after the fact
/// via the front-matter `quality_score:` field.
const DEFAULT_QUALITY_SCORE: u32 = 70;

/// Augment an existing front-matter block with missing fields (quality_score,
/// difficulty, domain). Preserves all existing fields.
fn augment_front_matter(content: &str, template: &str) -> String {
    // Extract the closing --- of the existing front-matter.
    let lines: Vec<&str> = content.lines().collect();
    let mut fm_end = 0;
    let mut found_close = false;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            fm_end = i;
            found_close = true;
            break;
        }
    }
    if !found_close {
        return content.to_string(); // can't find FM close — leave as-is
    }

    // Parse the template to get the quality_score + difficulty + domain lines.
    let template_lines: Vec<&str> = template.lines().collect();
    let mut additions = Vec::new();
    for line in &template_lines {
        let t = line.trim();
        if t.starts_with("quality_score:") && !content.contains("quality_score:") {
            additions.push(t.to_string());
        }
        if t.starts_with("difficulty:") && !content.contains("difficulty:") {
            additions.push(t.to_string());
        }
        if t.starts_with("domain:") && !content.contains("domain:") {
            additions.push(t.to_string());
        }
    }
    if additions.is_empty() {
        return content.to_string();
    }

    // Insert the additions before the closing ---.
    let mut new_lines = lines[..fm_end].to_vec();
    for add in &additions {
        new_lines.push(add);
    }
    new_lines.push("---");
    new_lines.extend_from_slice(&lines[fm_end + 1..]);
    new_lines.join("\n")
}

fn main() {
    let dry_run = std::env::args().any(|a| a == "--dry-run");
    let fix_mode = std::env::args().any(|a| a == "--fix");
    let knowledge_dir = PathBuf::from("knowledge");
    if !knowledge_dir.is_dir() {
        eprintln!("error: 'knowledge/' not found — run from the repo root");
        std::process::exit(1);
    }

    let mut files: Vec<PathBuf> = Vec::new();
    walk_md(&knowledge_dir, &mut files, 0);
    files.sort();

    let mut changed = 0;
    let mut skipped = 0;
    for f in &files {
        let content = match fs::read_to_string(f) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("skip (unreadable): {f:?} — {e}");
                skipped += 1;
                continue;
            }
        };
        // In default mode: skip files that already have front-matter.
        // In --fix mode: process them if they're missing quality_score.
        if content.trim_start().starts_with("---") {
            if !fix_mode || content.contains("quality_score:") {
                skipped += 1;
                continue;
            }
            // --fix mode: augment existing front-matter with missing fields.
            let fm = build_front_matter(f, &content);
            let new_content = augment_front_matter(&content, &fm);
            if dry_run {
                println!("WOULD FIX: {}", f.display());
            } else if let Err(e) = fs::write(f, &new_content) {
                eprintln!("error fixing {f:?}: {e}");
                skipped += 1;
                continue;
            } else {
                println!("fixed: {}", f.display());
            }
            changed += 1;
            continue;
        }

        let fm = build_front_matter(f, &content);
        let new_content = format!("{fm}\n{content}");

        if dry_run {
            println!("WOULD BACKFILL: {}", f.display());
        } else {
            if let Err(e) = fs::write(f, &new_content) {
                eprintln!("error writing {f:?}: {e}");
                skipped += 1;
                continue;
            }
            println!("backfilled: {}", f.display());
        }
        changed += 1;
    }

    println!("\n--- summary ---");
    println!("total .md files scanned: {}", files.len());
    println!(
        "{}: {}",
        if dry_run {
            "would backfill"
        } else {
            "backfilled"
        },
        changed
    );
    println!("skipped (already have front-matter / errors): {skipped}");
}

/// Build a canonical front-matter block for a file, deriving all fields
/// from path + content.
fn build_front_matter(path: &Path, content: &str) -> String {
    let rel = path.to_string_lossy().replace('\\', "/");
    let id = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".into());
    let title = extract_h1(content).unwrap_or_else(|| id.clone());

    // Segments after "knowledge/": domain dir, optional category subdir,
    // then the file name. The file name is not a category.
    let segments: Vec<&str> = rel.split('/').skip(1).collect();
    let domain = segments.first().copied().unwrap_or("").to_string();
    let category = segments
        .get(1)
        .filter(|s| !s.ends_with(".md"))
        .copied()
        .unwrap_or("")
        .to_string();

    let tags = derive_tags(path, content, &domain);

    let mut fm = String::from("---\n");
    fm.push_str(&format!("id: {id}\n"));
    fm.push_str(&format!("title: {title}\n"));
    if !domain.is_empty() {
        fm.push_str(&format!("domain: {domain}\n"));
    }
    if !category.is_empty() {
        fm.push_str(&format!("category: {category}\n"));
    }
    fm.push_str("difficulty: intermediate\n");
    fm.push_str(&format!("tags: [{}]\n", tags.join(", ")));
    fm.push_str(&format!("quality_score: {DEFAULT_QUALITY_SCORE}\n"));
    fm.push_str("last_updated: 2026-06-15\n");
    fm.push_str("---");
    fm
}

/// First `# Title` line, stripped of leading `# `. Returns `None` when the
/// H1 is absent OR looks like noisy metadata (maintainer email, template
/// placeholder) — in that case the caller falls back to the file stem.
fn extract_h1(content: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("# ") {
            let title = rest.trim().to_string();
            if is_valid_title(&title) {
                return Some(title);
            }
            return None; // noisy H1 → skip, fall back to file stem
        }
        if !t.is_empty() && !t.starts_with('>') {
            return None;
        }
    }
    None
}

/// A valid title is non-empty and has no email/maintainer metadata. Filters
/// out the `# 开发：Excellent（email@x）` signature that ~90 legacy files carry.
fn is_valid_title(title: &str) -> bool {
    if title.is_empty() {
        return false;
    }
    let lower = title.to_lowercase();
    !lower.contains('@') || !lower.contains("excellent") || !lower.contains("开发")
}

/// Derive tags from: domain dir + path keywords + H2 heading words.
fn derive_tags(path: &Path, content: &str, domain: &str) -> Vec<String> {
    let mut tags: BTreeSet<String> = BTreeSet::new();

    // Domain as a tag.
    if !domain.is_empty() {
        tags.insert(domain.to_string());
    }

    // File stem kebab-cased.
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        for word in stem.split(['-', '_']) {
            if word.len() >= 3 && word.chars().all(|c| c.is_alphanumeric()) {
                tags.insert(word.to_lowercase());
            }
        }
    }

    // H2 heading words (top 5 by first appearance).
    let mut h2_words = 0;
    for line in content.lines() {
        if h2_words >= 8 {
            break;
        }
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("## ") {
            for word in rest.split_whitespace() {
                let clean: String = word
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                    .to_lowercase();
                if clean.len() >= 4
                    && clean.chars().all(|c| c.is_alphanumeric() || c == '-')
                    && tags.insert(clean)
                {
                    h2_words += 1;
                }
            }
        }
    }

    tags.into_iter().take(8).collect()
}

/// Recursively collect `.md` file paths.
fn walk_md(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    if depth > 6 || out.len() >= 5000 {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_md(&p, out, depth + 1);
        } else if p.extension().and_then(|s| s.to_str()) == Some("md") {
            out.push(p);
        }
    }
}
