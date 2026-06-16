//! Auto-sediment capture layer — turns development experience into persistent
//! knowledge that makes the tool stronger with every run.
//!
//! Until 4.8 Super Dev was stateless across runs: quality-gate failures were
//! overwritten, gate-revision feedback was consumed once then discarded, and
//! the `.super-dev/decisions/` directory the spec promised was never written
//! to. This module closes that loop:
//!
//! - [`capture_quality_failures`] — appends every failed/warning quality
//!   check to `.super-dev/learned/_raw/quality-failures.jsonl`.
//! - [`capture_gate_revision`] — writes a real ADR (Architecture Decision
//!   Record) to `.super-dev/decisions/<gate>-<ts>.md`, fulfilling the spec's
//!   long-standing empty promise. Also appends a raw lesson.
//! - [`capture_validated_patterns`] — records schemas/decisions that passed
//!   the quality gate, so future runs can reuse proven patterns.
//!
//! All captures are fail-open: a write error is logged but never blocks the
//! pipeline. The raw JSONL files are consumed by [`sediment_lessons`] (step
//! 2) which turns them into retrievable markdown.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::phases::QualityCheck;
use super_dev_contract::ApiSpec;

/// Where raw captured experience lives (before sediment turns it into .md).
pub const RAW_DIR: &str = ".super-dev/learned/_raw";
/// Where the ADR (decision) records live — read by the proof-pack.
pub const DECISIONS_DIR: &str = ".super-dev/decisions";
/// Where sedimented markdown lessons live (project-level).
pub const LEARNED_DIR: &str = ".super-dev/learned";
/// Where global (cross-project) lessons live, under the user's home.
pub const GLOBAL_LEARNED_DIRNAME: &str = ".super-dev/learned";

/// The kind of captured experience.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonKind {
    /// A quality-gate check failed or warned.
    Failure,
    /// A human revision request at a gate (what the user wanted changed).
    Revision,
    /// A pattern that passed the quality gate (positive experience).
    ValidatedPattern,
}

/// One captured lesson — written to raw JSONL, later sedimented to .md.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    /// What kind of experience.
    pub kind: LessonKind,
    /// Domain directory (api, database, frontend, ...). Derived from the
    /// requirement entities so the sedimented file lands in the right place.
    pub domain: String,
    /// Short, human-readable title (becomes the H1 + operationId).
    pub title: String,
    /// Detailed body — symptom, fix, root cause. Keywords that future BM25
    /// queries should match MUST appear in this text (tags alone aren't
    /// indexed by BM25).
    pub body: String,
    /// The actionable fix / recommendation.
    pub fix: String,
    /// The root-cause explanation.
    pub root_cause: String,
    /// Search keywords (also embedded in body for BM25 discoverability).
    pub keywords: Vec<String>,
    /// The requirement that triggered this lesson.
    pub source_requirement: String,
    /// ISO-8601 UTC timestamp when first seen.
    pub first_seen: String,
}

/// Capture quality-gate failures + warnings as raw lessons.
///
/// Called at the end of `run_quality`. Writes one JSONL line per failed or
/// warning check to `RAW_DIR/quality-failures.jsonl`. Fail-open: any I/O
/// error is silently ignored.
pub fn capture_quality_failures(
    project_root: &Path,
    checks: &[QualityCheck],
    slug: &str,
    requirement: &str,
) {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut lessons: Vec<Lesson> = Vec::new();
    for check in checks
        .iter()
        .filter(|c| c.status == "failed" || c.status == "warning")
    {
        let domain = domain_for_check(&check.name);
        let keywords = extract_keywords(&check.name, &check.details, requirement);
        lessons.push(Lesson {
            kind: LessonKind::Failure,
            domain: domain.clone(),
            title: format!("Quality gate: {} ({})", check.name, check.status),
            body: format!(
                "During the {slug} run, the quality check \"{name}\" scored {score}/100 \
                 with status {status}.\n\nDetails: {details}\n\nRequirement: {requirement}",
                slug = slug,
                name = check.name,
                score = check.score,
                status = check.status,
                details = check.details,
                requirement = requirement,
            ),
            fix: fix_suggestion_for_check(&check.name),
            root_cause: format!(
                "The {} check scored {}/100 (status: {}). This is a {} issue — {}",
                check.name,
                check.score,
                check.status,
                if check.status == "failed" {
                    "blocking"
                } else {
                    "quality"
                },
                if check.score < 40 {
                    "the artifact is substantially incomplete"
                } else if check.score < 70 {
                    "the artifact is partially complete"
                } else {
                    "the artifact is mostly complete but needs polish"
                }
            ),
            keywords: keywords.clone(),
            source_requirement: requirement.to_string(),
            first_seen: now.clone(),
        });
    }
    append_raw_lessons(project_root, "quality-failures.jsonl", &lessons);
}

/// Capture a gate revision as both an ADR record AND a raw lesson.
///
/// Called from `cmd_revise`. Writes a real ADR markdown file to
/// `DECISIONS_DIR/<gate>-<timestamp>.md` (fulfilling the spec's promise),
/// then appends a Revision lesson to the raw ledger.
pub fn capture_gate_revision(
    project_root: &Path,
    gate: &str,
    revision_text: &str,
    requirement: &str,
) -> PathBuf {
    let now = Utc::now();
    let ts = now.format("%Y%m%dT%H%M%SZ");
    let date = now.format("%Y-%m-%d");

    // 1. Write the ADR (decision record) — fulfills spec §5.4.
    let dec_dir = project_root.join(DECISIONS_DIR);
    let _ = fs::create_dir_all(&dec_dir);
    let adr_path = dec_dir.join(format!("{gate}-{ts}.md"));
    let adr_body = format!(
        "# ADR: {gate} revision\n\n\
         **Date:** {date}\n\n\
         **Status:** Revised\n\n\
         **Requirement:** {requirement}\n\n\
         ## Decision\n\n\
         The user requested the following revision at the {gate} gate:\n\n\
         > {revision_text}\n\n\
         ## Context\n\n\
         This revision feedback is captured as a decision record so future runs \
         of the pipeline understand why the artifacts changed at this gate. The \
         underlying worker will regenerate the block with this feedback folded \
         into the requirement.\n",
    );
    let _ = fs::write(&adr_path, adr_body);

    // 2. Append a raw Revision lesson.
    let domain = if gate.contains("docs") {
        "docs"
    } else {
        "frontend"
    };
    let keywords = extract_keywords(gate, revision_text, requirement);
    let lesson = Lesson {
        kind: LessonKind::Revision,
        domain: domain.to_string(),
        title: format!("{gate} revision: {}", truncate(revision_text, 80)),
        body: format!(
            "At the {gate} gate, the user revised with: \"{revision_text}\".\n\n\
             This indicates the generated artifacts did not meet expectations in \
             this area. The worker should address this feedback directly.\n\n\
             Requirement context: {requirement}",
        ),
        fix: format!("Address the revision feedback: {revision_text}"),
        root_cause: "The generated artifact did not meet the user's expectations at this gate."
            .to_string(),
        keywords,
        source_requirement: requirement.to_string(),
        first_seen: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    };
    append_raw_lessons(project_root, "gate-revisions.jsonl", &[lesson]);

    adr_path
}

/// Capture validated patterns (schemas/decisions that passed the gate) as
/// positive experience. Called at delivery completion.
pub fn capture_validated_patterns(
    project_root: &Path,
    slug: &str,
    requirement: &str,
    spec: &ApiSpec,
) {
    if spec.is_empty() {
        return;
    }
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let entity_summary = spec
        .declared_paths()
        .iter()
        .map(|(_, p)| (*p).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let keywords = extract_keywords(slug, &entity_summary, requirement);
    let lesson = Lesson {
        kind: LessonKind::ValidatedPattern,
        domain: "api".to_string(),
        title: format!("Validated API contract for {slug}"),
        body: format!(
            "The {slug} run produced a validated OpenAPI contract with these endpoints:\n\
             {entity_summary}\n\n\
             This schema passed the quality gate. Reuse this entity decomposition \
             for similar requirements.\n\nRequirement: {requirement}",
        ),
        fix: "Reuse this proven entity decomposition for similar projects.".to_string(),
        root_cause: "This contract was generated from the requirement and validated.".to_string(),
        keywords,
        source_requirement: requirement.to_string(),
        first_seen: now,
    };
    append_raw_lessons(project_root, "validated-decisions.jsonl", &[lesson]);
}

/// Append lessons to a raw JSONL file. Fail-open (best-effort write).
fn append_raw_lessons(project_root: &Path, filename: &str, lessons: &[Lesson]) {
    if lessons.is_empty() {
        return;
    }
    let raw_dir = project_root.join(RAW_DIR);
    let _ = fs::create_dir_all(&raw_dir);
    let path = raw_dir.join(filename);
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        for lesson in lessons {
            if let Ok(line) = serde_json::to_string(lesson) {
                let _ = writeln!(f, "{line}");
            }
        }
    }
}

/// Read all raw lessons from a file. Returns empty vec on missing/malformed.
#[must_use]
pub fn read_raw_lessons(project_root: &Path, filename: &str) -> Vec<Lesson> {
    let path = project_root.join(RAW_DIR).join(filename);
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<Lesson>(l).ok())
        .collect()
}

/// Read ALL raw lessons across all files.
#[must_use]
pub fn read_all_raw_lessons(project_root: &Path) -> Vec<Lesson> {
    let mut all = Vec::new();
    for f in &[
        "quality-failures.jsonl",
        "gate-revisions.jsonl",
        "validated-decisions.jsonl",
    ] {
        all.extend(read_raw_lessons(project_root, f));
    }
    all
}

/// Map a quality-check name to a domain directory slug.
fn domain_for_check(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.contains("api") || lower.contains("contract") || lower.contains("openapi") {
        "api".to_string()
    } else if lower.contains("color")
        || lower.contains("emoji")
        || lower.contains("design")
        || lower.contains("dark")
        || lower.contains("uiux")
        || lower.contains("ui/")
    {
        "frontend".to_string()
    } else if lower.contains("placeholder") || lower.contains("slop") {
        "governance".to_string()
    } else if lower.contains("ops") || lower.contains("docker") || lower.contains("ci") {
        "devops".to_string()
    } else if lower.contains("architecture") || lower.contains("alignment") {
        "architecture".to_string()
    } else if lower.contains("acceptance") || lower.contains("prd") {
        "product".to_string()
    } else {
        "general".to_string()
    }
}

/// Extract search keywords from text (for BM25 discoverability).
fn extract_keywords(source: &str, details: &str, requirement: &str) -> Vec<String> {
    let mut kws: Vec<String> = Vec::new();
    for text in [source, details, requirement] {
        // ASCII words: split on non-alphanumeric, keep len>=3.
        for word in text.split(|c: char| !c.is_alphanumeric()) {
            let w = word.trim().to_ascii_lowercase();
            if w.len() >= 3 && !kws.contains(&w) {
                kws.push(w);
            }
        }
        // CJK: the split above yields one giant token per CJK run (all CJK
        // chars are alphanumeric), which is useless for BM25 discoverability.
        // Emit CJK unigrams + bigrams so a Chinese requirement like
        // "登录系统" produces "登录" / "系统" / "登录系统" keywords. Mirrors the
        // knowledge crate's tokenizer strategy.
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if is_cjk_char(chars[i]) {
                // unigram
                let uni = chars[i].to_string();
                if !kws.contains(&uni) {
                    kws.push(uni);
                }
                // bigram with next CJK char
                if i + 1 < chars.len() && is_cjk_char(chars[i + 1]) {
                    let bi: String = chars[i..=i + 1].iter().collect();
                    if !kws.contains(&bi) {
                        kws.push(bi);
                    }
                }
            }
            i += 1;
        }
    }
    kws.truncate(20);
    kws
}

/// Whether a char is in the common CJK unified ideograph ranges (same set
/// the knowledge tokenizer uses). Inline copy to avoid a cross-crate dep.
fn is_cjk_char(c: char) -> bool {
    matches!(c as u32,
        0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0xF900..=0xFAFF
        | 0x3040..=0x30FF | 0xAC00..=0xD7AF
    )
}

/// Generate an actionable fix suggestion based on the check name.
fn fix_suggestion_for_check(name: &str) -> String {
    let l = name.to_ascii_lowercase();
    if l.contains("placeholder") {
        "Replace EVERY TODO/placeholder marker with real content. Use --backend claude-code so the worker fills in actual requirements and API details.".to_string()
    } else if l.contains("conformance") {
        "Ensure every frontend fetch/axios call hits a declared OpenAPI endpoint with the correct method. Run the contract validator before submitting.".to_string()
    } else if l.contains("openapi") || l.contains("contract") {
        "Generate .super-dev/contracts/openapi.yaml from the architecture API table. Verify frontend calls map to declared endpoints (method + path templates).".to_string()
    } else if l.contains("consistency") || l.contains("alignment") {
        "Cross-check three artifacts: PRD routes, OpenAPI paths, and frontend calls must all reference the same entities (e.g. /api/articles).".to_string()
    } else if l.contains("color") {
        "Replace hardcoded hex/rgb/hsl with CSS custom properties (design tokens). Define --color-primary in :root. Only #fff/#000 allowed.".to_string()
    } else if l.contains("emoji") {
        "Replace emoji-as-icons with a declared icon library (Lucide, Heroicons). Emoji in JSX text is blocked.".to_string()
    } else if l.contains("slop") {
        "Remove Lorem ipsum and generic 'Welcome to App' titles. Write real, requirement-specific copy.".to_string()
    } else if l.contains("acceptance") {
        "Write 3+ Given/When/Then criteria per entity: GET returns list, POST creates with id, DELETE removes.".to_string()
    } else if l.contains("discovery") {
        "Add a ## Discovery section: target audience, similar products, design direction. This grounds the PRD.".to_string()
    } else if l.contains("uiux") || l.contains("ui/ux") || l.contains("design system") {
        "Complete the UIUX doc: color tokens, typography, icon set, interactive states (hover/focus/disabled).".to_string()
    } else if l.contains("dark") {
        "Add @media (prefers-color-scheme: dark) overrides for all color tokens. Test both themes."
            .to_string()
    } else if l.contains("ops") {
        "Generate: Dockerfile (multi-stage, non-root), docker-compose (app+postgres), CI workflow (lint+test+quality gate), migrations, .env.example.".to_string()
    } else if l.contains("audit") {
        "Ensure audit JSONL logs are populated. frontend-api-calls.jsonl records every fetch(). tool-calls.jsonl records governance decisions.".to_string()
    } else if l.contains("research") {
        "Enrich research doc: domain risks, similar products, discovery (audience + design direction).".to_string()
    } else if l.contains("prd") {
        "Complete PRD: Goal (what+why+metric), personas, Scope, functional requirements table, acceptance criteria.".to_string()
    } else if l.contains("architecture") {
        "Complete architecture: API surface table, data model (entity field tables), auth method, tech-stack rationale.".to_string()
    } else {
        format!("Address the '{name}' check — see details in quality-gate.json and fix the specific issue.")
    }
}

/// Truncate a string to `max` chars with an ellipsis.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
    t.push('…');
    t
}

// =====================================================================
// Step 2: Sediment — turn raw JSONL lessons into retrievable markdown.
// =====================================================================

/// Resolve the global learned dir: `~/.super-dev/learned/`.
/// Returns None when no home directory can be determined (fail-open).
///
/// Cross-platform: prefers `HOME` (Unix + most shells), falls back to
/// `USERPROFILE` (Windows). Previously only `HOME` was checked, which is
/// usually unset on Windows — so global experience silently never loaded.
#[must_use]
pub fn global_learned_dir() -> Option<PathBuf> {
    let home = home_dir()?;
    let dir = home.join(GLOBAL_LEARNED_DIRNAME);
    // Bootstrap: create the dir so a fresh machine can accumulate global
    // experience (before this fix, promote_to_global silently did nothing
    // on machines where ~/.super-dev/learned/ didn't exist yet).
    if dir.is_dir() {
        Some(dir)
    } else {
        let _ = std::fs::create_dir_all(&dir);
        Some(dir)
    }
}

/// Cross-platform home directory: `HOME` then `USERPROFILE` (Windows).
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

/// Sediment all raw lessons into markdown knowledge files under
/// `.super-dev/learned/<domain>/`. Each unique `(domain, title)` produces one
/// file (latest wins). Called from `run_quality` after the capture step.
///
/// Total-ordering predicate for sediment dedup: `true` if `a` should be
/// considered "before / less desirable to keep" than `b`. Newer
/// `first_seen` wins; on a same-second tie, the record with the richer
/// `fix` (longer) wins; on a fix-length tie, the lexicographically-larger
/// title wins as a final stable deterministic tiebreak. This makes dedup
/// fully deterministic even when timestamps collide at second resolution.
fn lesson_precedes(a: &Lesson, b: &Lesson) -> bool {
    match a.first_seen.cmp(&b.first_seen) {
        std::cmp::Ordering::Equal => {
            // Same second → compare richness, then title.
            match a.fix.len().cmp(&b.fix.len()) {
                std::cmp::Ordering::Equal => a.title < b.title,
                ord => ord.is_lt(),
            }
        }
        ord => ord.is_lt(),
    }
}

/// Returns the number of markdown files written. Fail-open: errors return 0.
#[must_use]
pub fn sediment_lessons(project_root: &Path) -> usize {
    let lessons = read_all_raw_lessons(project_root);
    if lessons.is_empty() {
        return 0;
    }

    // Dedupe by (domain, title) — keep the latest first_seen. On a
    // same-second tie (first_seen has only second resolution), break
    // deterministically by the richer record: longer `fix` text wins, then
    // lexicographically-greater `title` as a final stable tiebreak. This
    // replaces the previous `existing.first_seen >= lesson.first_seen`
    // guard, which on equal timestamps kept whichever happened to iterate
    // first — deterministic given a fixed Vec order, but with no signal
    // that the kept record was actually the "latest" content.
    let mut by_key: std::collections::HashMap<String, &Lesson> = std::collections::HashMap::new();
    for lesson in &lessons {
        let key = format!("{}::{}", lesson.domain, lesson.title);
        match by_key.get(&key) {
            Some(existing) if lesson_precedes(lesson, existing) => {}
            _ => {
                by_key.insert(key, lesson);
            }
        }
    }

    let learned_root = project_root.join(LEARNED_DIR);
    let _ = fs::create_dir_all(&learned_root);
    let mut written = 0usize;
    let mut seq_by_domain: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for lesson in by_key.values() {
        let domain_dir = learned_root.join(&lesson.domain);
        let _ = fs::create_dir_all(&domain_dir);
        let seq = seq_by_domain.entry(lesson.domain.clone()).or_insert(0);
        *seq += 1;
        let path = domain_dir.join(format!("lesson-{domain}-{seq}.md", domain = lesson.domain));
        let body = render_lesson_markdown(lesson);
        if fs::write(&path, body).is_ok() {
            written += 1;
        }
    }

    // Promote frequently-occurring lessons to the global dir.
    let _ = promote_to_global(project_root, &lessons);

    written
}

/// Render a Lesson as a markdown knowledge file matching the chunker's
/// expectations: YAML front-matter (tags), H1 title, H2 sections (症状/修复/原因).
/// Keywords are deliberately embedded in the body text so BM25 can find them
/// (front-matter tags alone are NOT indexed).
fn render_lesson_markdown(lesson: &Lesson) -> String {
    let date = &lesson.first_seen[..10.min(lesson.first_seen.len())];
    let kind_label = match lesson.kind {
        LessonKind::Failure => "⚠️ Failure",
        LessonKind::Revision => "✏️ Revision",
        LessonKind::ValidatedPattern => "✓ Validated pattern",
    };
    let keywords_inline = lesson.keywords.join(", ");
    format!(
        "---\nid: lesson-{domain}\ntitle: {title}\ndomain: {domain}\ncategory: learned\ntags: [{tags}]\nmaintainer: auto-sediment\nlast_updated: {date}\n---\n\
# {kind_label}: {title}\n\n\
## Symptom\n\n{body}\n\n\
Keywords: {keywords_inline}\n\n\
## Fix\n\n{fix}\n\n\
## Root cause\n\n{root_cause}\n",
        domain = lesson.domain,
        title = lesson.title,
        tags = lesson.keywords.join(", "),
        date = date,
        kind_label = kind_label,
        body = lesson.body,
        keywords_inline = keywords_inline,
        fix = lesson.fix,
        root_cause = lesson.root_cause,
    )
}

/// Promote lessons that appear across multiple distinct requirements to the
/// global `~/.super-dev/learned/` dir, so all projects benefit. A lesson is
/// "global-worthy" if its domain+title appears with ≥2 different source
/// requirements (indicating it's a general pattern, not project-specific).
fn promote_to_global(_project_root: &Path, lessons: &[Lesson]) -> usize {
    let Some(global_dir) = global_learned_dir() else {
        return 0; // HOME unset or dir doesn't exist yet — skip.
    };

    // Group by (domain, title) and count distinct requirements.
    let mut groups: std::collections::HashMap<String, Vec<&Lesson>> =
        std::collections::HashMap::new();
    for lesson in lessons {
        let key = format!("{}::{}", lesson.domain, lesson.title);
        groups.entry(key).or_default().push(lesson);
    }

    let mut promoted = 0usize;
    for (key, group) in &groups {
        let distinct_reqs: std::collections::HashSet<&str> = group
            .iter()
            .map(|l| l.source_requirement.as_str())
            .collect();
        if distinct_reqs.len() < 2 {
            continue; // only 1 requirement — not general enough.
        }
        // Promote the latest lesson in this group. Use the deterministic
        // total-order from lesson_precedes (first_seen → fix length → title)
        // so same-second timestamps don't make the choice non-deterministic
        // (matches the sediment_lessons dedup policy).
        let latest = group
            .iter()
            .copied()
            .reduce(|acc, l| if lesson_precedes(acc, l) { l } else { acc });
        if let Some(lesson) = latest {
            let dir = global_dir.join(&lesson.domain);
            let _ = fs::create_dir_all(&dir);
            let slug = key.replace("::", "-").replace(' ', "-");
            let path = dir.join(format!("{slug}.md"));
            let body = render_lesson_markdown(lesson);
            if fs::write(&path, body).is_ok() {
                promoted += 1;
            }
        }
    }
    promoted
}

/// List all sedimented lesson files (project + global), for reporting.
#[must_use]
pub fn list_sedimented_lessons(project_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let project_learned = project_root.join(LEARNED_DIR);
    if project_learned.is_dir() {
        collect_md_files(&project_learned, &mut files);
    }
    if let Some(global) = global_learned_dir() {
        collect_md_files(&global, &mut files);
    }
    files
}

fn collect_md_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            // Skip the _raw dir (raw JSONL, not retrievable markdown).
            if p.file_name().is_some_and(|n| n == "_raw") {
                continue;
            }
            collect_md_files(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("md") {
            out.push(p);
        }
    }
}

// =====================================================================
// Step 4: Feed back — render lessons into the coach prompt.
// =====================================================================

/// Render the most relevant prior-run lessons for the current phase's prompt.
/// Returns a formatted markdown block (empty string when no lessons exist —
/// so the prompt is unchanged for first-ever runs).
///
/// Uses simple keyword overlap against the requirement to pick the top 2-3
/// lessons (we don't call BM25 here to avoid a circular dependency between
/// the agent crate and the knowledge crate at prompt-assembly time — the
/// BM25 index already picks up learned/ files during phase_knowledge_digest).
#[must_use]
pub fn relevant_lessons_for_prompt(project_root: &Path, requirement: &str) -> String {
    let lessons = read_all_raw_lessons(project_root);
    if lessons.is_empty() {
        return String::new();
    }

    // Two-tier recall:
    // 1. Keyword-matched lessons (precisely relevant).
    // 2. Universal fallback — quality-gate failure modes (API consistency,
    //    placeholder, contract) apply to ALL projects, so inject recent
    //    failures even when keywords don't overlap.
    let req_lower = requirement.to_ascii_lowercase();
    let req_words: std::collections::HashSet<&str> = req_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3)
        .collect();

    let mut scored: Vec<(usize, &Lesson)> = lessons
        .iter()
        .map(|l| {
            let score = l
                .keywords
                .iter()
                .filter(|k| req_words.contains(k.as_str()))
                .count();
            (score, l)
        })
        .collect();
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.first_seen.cmp(&a.1.first_seen))
    });

    // Tier 1: keyword-matched (up to 2).
    let mut top_idx: Vec<usize> = scored
        .iter()
        .enumerate()
        .filter(|(_, (s, _))| *s > 0)
        .take(2)
        .map(|(i, _)| i)
        .collect();
    // Tier 2: universal fallback — recent failures.
    if top_idx.len() < 3 {
        for (i, (s, l)) in scored.iter().enumerate() {
            if top_idx.len() >= 3 {
                break;
            }
            if *s == 0 && l.kind == LessonKind::Failure && !top_idx.contains(&i) {
                top_idx.push(i);
            }
        }
    }
    if top_idx.is_empty() {
        return String::new();
    }
    let mut out = String::from(
        "
## Lessons from prior runs

",
    );
    out.push_str("Experiences captured from previous runs on this project. ");
    out.push_str(
        "Apply these to avoid repeating mistakes:

",
    );
    for &i in &top_idx {
        let lesson = scored[i].1;
        let icon = match lesson.kind {
            LessonKind::Failure => "⚠️",
            LessonKind::Revision => "✏️",
            LessonKind::ValidatedPattern => "✓",
        };
        out.push_str(&format!(
            "{icon} **{}**
   {}

",
            lesson.title, lesson.fix
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phases::QualityCheck;
    use tempfile::TempDir;

    fn check(name: &str, status: &str, score: i32) -> QualityCheck {
        QualityCheck {
            name: name.to_string(),
            category: "contract".to_string(),
            description: "test".to_string(),
            status: status.to_string(),
            score,
            details: format!("details for {name}"),
            weight: 2.0,
        }
    }

    #[test]
    fn capture_quality_failures_writes_raw_jsonl() {
        let tmp = TempDir::new().unwrap();
        let checks = vec![
            check("API URL consistency", "failed", 30),
            check("OpenAPI contract", "passed", 100),
            check("No placeholder content", "warning", 60),
        ];
        capture_quality_failures(tmp.path(), &checks, "demo", "博客系统");
        let raw = read_raw_lessons(tmp.path(), "quality-failures.jsonl");
        // 2 lessons (failed + warning; passed is skipped).
        assert_eq!(raw.len(), 2);
        assert_eq!(raw[0].kind, LessonKind::Failure);
        assert!(raw[0].title.contains("API URL consistency"));
        assert!(raw[1].title.contains("placeholder"));
    }

    #[test]
    fn capture_quality_failures_no_failures_writes_nothing() {
        let tmp = TempDir::new().unwrap();
        let checks = vec![check("All good", "passed", 100)];
        capture_quality_failures(tmp.path(), &checks, "demo", "x");
        assert!(read_raw_lessons(tmp.path(), "quality-failures.jsonl").is_empty());
    }

    #[test]
    fn capture_gate_revision_writes_adr_and_lesson() {
        let tmp = TempDir::new().unwrap();
        let adr_path = capture_gate_revision(
            tmp.path(),
            "docs_confirm",
            "需要更多数据库设计的细节",
            "博客系统",
        );
        // ADR file written.
        assert!(adr_path.is_file());
        let adr = fs::read_to_string(&adr_path).unwrap();
        assert!(adr.contains("ADR"));
        assert!(adr.contains("docs_confirm"));
        assert!(adr.contains("数据库设计"));
        // Raw lesson written.
        let lessons = read_raw_lessons(tmp.path(), "gate-revisions.jsonl");
        assert_eq!(lessons.len(), 1);
        assert_eq!(lessons[0].kind, LessonKind::Revision);
        assert!(lessons[0].body.contains("数据库设计"));
    }

    #[test]
    fn capture_validated_patterns_records_contract() {
        let tmp = TempDir::new().unwrap();
        let spec = super_dev_contract::parse_architecture(
            "| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/articles | - | - | none | List |\n",
            "demo",
        );
        capture_validated_patterns(tmp.path(), "demo", "博客系统", &spec);
        let lessons = read_raw_lessons(tmp.path(), "validated-decisions.jsonl");
        assert_eq!(lessons.len(), 1);
        assert_eq!(lessons[0].kind, LessonKind::ValidatedPattern);
        assert!(lessons[0].body.contains("/api/articles"));
    }

    #[test]
    fn capture_validated_patterns_empty_spec_skips() {
        let tmp = TempDir::new().unwrap();
        capture_validated_patterns(tmp.path(), "demo", "x", &ApiSpec::default());
        assert!(read_raw_lessons(tmp.path(), "validated-decisions.jsonl").is_empty());
    }

    #[test]
    fn domain_for_check_maps_correctly() {
        assert_eq!(domain_for_check("API URL consistency"), "api");
        assert_eq!(domain_for_check("OpenAPI contract"), "api");
        assert_eq!(domain_for_check("No placeholder content"), "governance");
        assert_eq!(domain_for_check("Hardcoded color block events"), "frontend");
        assert_eq!(domain_for_check("Ops artifacts present"), "devops");
        assert_eq!(
            domain_for_check("PRD↔Architecture alignment"),
            "architecture"
        );
        assert_eq!(domain_for_check("Unknown check"), "general");
    }

    #[test]
    fn fix_suggestion_is_actionable() {
        let fix = fix_suggestion_for_check("No placeholder content");
        assert!(fix.contains("TODO"));
        let fix = fix_suggestion_for_check("OpenAPI contract");
        assert!(fix.contains("contract"));
    }

    #[test]
    fn keywords_extracted_from_multiple_sources() {
        let kws = extract_keywords(
            "API URL consistency",
            "frontend calls /api/x",
            "博客系统 articles",
        );
        assert!(kws.contains(&"api".to_string()));
        assert!(kws.contains(&"consistency".to_string()));
        assert!(kws.contains(&"articles".to_string()));
    }

    #[test]
    fn raw_lessons_persist_across_calls() {
        let tmp = TempDir::new().unwrap();
        let checks1 = vec![check("Check A", "failed", 20)];
        let checks2 = vec![check("Check B", "failed", 10)];
        capture_quality_failures(tmp.path(), &checks1, "demo", "req");
        capture_quality_failures(tmp.path(), &checks2, "demo", "req");
        let raw = read_raw_lessons(tmp.path(), "quality-failures.jsonl");
        assert_eq!(raw.len(), 2);
    }

    #[test]
    fn read_all_raw_lessons_merges_files() {
        let tmp = TempDir::new().unwrap();
        capture_quality_failures(tmp.path(), &[check("X", "failed", 10)], "d", "r");
        capture_gate_revision(tmp.path(), "docs_confirm", "fix it", "r");
        let all = read_all_raw_lessons(tmp.path());
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn adr_filename_includes_gate_and_timestamp() {
        let tmp = TempDir::new().unwrap();
        let path = capture_gate_revision(tmp.path(), "preview_confirm", "redo", "req");
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with("preview_confirm-"));
        assert!(name.ends_with(".md"));
    }

    #[test]
    fn read_missing_file_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(read_raw_lessons(tmp.path(), "nonexistent.jsonl").is_empty());
    }

    #[test]
    fn sediment_creates_markdown_files() {
        let tmp = TempDir::new().unwrap();
        let checks = vec![
            check("API URL consistency", "failed", 30),
            check("No placeholder content", "warning", 60),
        ];
        capture_quality_failures(tmp.path(), &checks, "demo", "博客系统 articles api");
        let count = sediment_lessons(tmp.path());
        assert_eq!(count, 2, "should write 2 markdown files");
        // Files exist under learned/<domain>/.
        let learned = tmp.path().join(".super-dev/learned");
        assert!(learned.join("api").is_dir() || learned.join("governance").is_dir());
    }

    #[test]
    fn sediment_dedupes_by_domain_title() {
        let tmp = TempDir::new().unwrap();
        let checks = vec![check("API URL consistency", "failed", 30)];
        // Capture the same failure twice.
        capture_quality_failures(tmp.path(), &checks, "demo", "req");
        capture_quality_failures(tmp.path(), &checks, "demo", "req");
        let count = sediment_lessons(tmp.path());
        assert_eq!(count, 1, "dedupe should produce 1 file for repeated lesson");
    }

    #[test]
    fn sediment_markdown_has_correct_structure() {
        let tmp = TempDir::new().unwrap();
        capture_quality_failures(
            tmp.path(),
            &[check("OpenAPI contract", "failed", 0)],
            "d",
            "api contract openapi",
        );
        let _ = sediment_lessons(tmp.path());
        let files = list_sedimented_lessons(tmp.path());
        assert!(!files.is_empty());
        let content = fs::read_to_string(&files[0]).unwrap();
        // Has front-matter tags.
        assert!(content.contains("tags:"));
        // Has H1 + H2 sections.
        assert!(content.contains("# "));
        assert!(content.contains("## Symptom"));
        assert!(content.contains("## Fix"));
        assert!(content.contains("## Root cause"));
        // Keywords in body (for BM25).
        assert!(content.contains("Keywords:"));
        assert!(content.contains("openapi"));
    }

    #[test]
    fn sediment_empty_raw_writes_nothing() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(sediment_lessons(tmp.path()), 0);
        assert!(list_sedimented_lessons(tmp.path()).is_empty());
    }

    #[test]
    fn list_sedimented_skips_raw_dir() {
        let tmp = TempDir::new().unwrap();
        capture_quality_failures(tmp.path(), &[check("X", "failed", 0)], "d", "r");
        let _ = sediment_lessons(tmp.path());
        let files = list_sedimented_lessons(tmp.path());
        // No file should be under _raw.
        assert!(files.iter().all(|f| !f.to_string_lossy().contains("_raw")));
    }
}
