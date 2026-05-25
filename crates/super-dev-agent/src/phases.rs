//! Phase implementations — one function per Super Dev phase.
//!
//! V1 phases are *deterministic templates*: they read knowledge / the
//! user requirement and write the artifacts required by
//! `SUPER_DEV_HOST_SPEC_V1` §5 to disk, plus the evidence required by
//! §6. Future milestones swap deterministic bodies for LLM-driven ones
//! without changing the architecture.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use super_dev_governance::{
    compliance::write_compliance_mapping, extract_api_urls, record_tool_call,
};
use super_dev_spec::Phase;

use crate::runner::RunOptions;

/// What the phase produced. Returned for tracing / tests.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PhaseOutput {
    /// The phase that just finished.
    pub phase: Phase,
    /// Workspace-relative paths of the files written.
    pub artifacts: Vec<PathBuf>,
    /// Whether this phase ends at a gate (caller must pause).
    pub gate: Option<crate::gates::Gate>,
}

// =====================================================================
// research (SD-ART-001)
// =====================================================================

/// Knowledge digest summary about the workspace's `knowledge/` dir.
///
/// As of 4.2+, this is **requirement-aware** — it ranks the workspace's
/// `knowledge/*.md` files by how well each path / filename matches the
/// user's requirement keywords, picks the top-K most relevant ones, and
/// includes a short excerpt from each. Falls back to a flat listing when
/// no keyword overlap is found (e.g. CJK-only requirement against
/// English knowledge files).
///
/// Exposed so the async runner can compute it once and feed it into the
/// research expert prompt before delegating back into [`run_research`].
#[must_use]
pub fn knowledge_digest(opts: &RunOptions) -> String {
    let dir = opts.project_root.join("knowledge");
    smart_knowledge_digest(&dir, &opts.requirement)
}

/// Phase-aware knowledge digest — each pipeline phase gets knowledge
/// from its relevant domain subdirectories, keyword-ranked against the
/// user requirement. This is the "virtual expert's professional library".
#[must_use]
pub fn phase_knowledge_digest(opts: &RunOptions, phase: Phase) -> String {
    let base = opts.project_root.join("knowledge");
    if !base.is_dir() {
        return String::new();
    }
    let subdirs: &[&str] = match phase {
        Phase::Research => return knowledge_digest(opts),
        Phase::Docs => &[
            "product",
            "architecture",
            "design",
            "frontend",
            "industries",
        ],
        Phase::DocsConfirm | Phase::PreviewConfirm => return String::new(),
        Phase::Spec => &["development", "00-governance", "product"],
        Phase::Frontend => &["frontend", "design", "design-systems", "seed-templates"],
        Phase::Backend => &["backend", "api", "database", "security", "cloud-native"],
        Phase::Quality => &["testing", "security", "00-governance"],
        Phase::Delivery => &["cicd", "operations", "00-governance", "security"],
    };

    let mut all_paths = Vec::new();
    for sub in subdirs {
        let dir = base.join(sub);
        if dir.is_dir() {
            walk_md(&dir, &mut all_paths, 0);
        }
    }
    if all_paths.is_empty() {
        return String::new();
    }

    let keywords = extract_keywords(&opts.requirement);
    let mut scored: Vec<(usize, &String)> = all_paths
        .iter()
        .map(|p| {
            let full = base.join(p).to_string_lossy().to_string();
            (score_path(&full, &keywords), p)
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));

    let top_k = 4;
    let chosen: Vec<&String> = scored
        .iter()
        .filter(|(s, _)| *s > 0)
        .take(top_k)
        .map(|(_, p)| *p)
        .collect();

    if chosen.is_empty() {
        let mut sorted: Vec<&String> = all_paths.iter().collect();
        sorted.sort();
        let fallback: Vec<&String> = sorted.into_iter().take(3).collect();
        if fallback.is_empty() {
            return String::new();
        }
        let mut out = format!(
            "\n\n## Expert knowledge ({} phase)\n\n\
             Top {} files from {} domain knowledge (no keyword match, showing first files):\n\n",
            phase.id(),
            fallback.len(),
            subdirs.join("/")
        );
        for rel in fallback {
            let full = base.join(rel);
            let excerpt = read_excerpt(&full, 400);
            out.push_str(&format!("### `{rel}`\n\n{excerpt}\n\n"));
        }
        return out;
    }

    let mut out = format!(
        "\n\n## Expert knowledge ({} phase)\n\n\
         Top {} of {} domain files (keyword-ranked from {}):\n\n",
        phase.id(),
        chosen.len(),
        all_paths.len(),
        subdirs.join(", ")
    );
    for rel in chosen {
        let full = base.join(rel);
        let excerpt = read_excerpt(&full, 400);
        out.push_str(&format!("### `{rel}`\n\n{excerpt}\n\n"));
    }
    out
}

/// The file paths (`knowledge/*.md`, workspace-relative) the digest
/// would surface to the worker for this requirement. Used by the runner
/// to emit a chat-visible "I'm reading X, Y, Z" event so the user can
/// see Super Dev is doing context retrieval, not flying blind.
///
/// Returns `(chosen_paths, total_scanned)` where `total_scanned` is the
/// full corpus size — handy for showing "selected 6 of 306" in the UI.
#[must_use]
pub fn knowledge_top_files(opts: &RunOptions) -> (Vec<String>, usize) {
    let dir = opts.project_root.join("knowledge");
    if !dir.is_dir() {
        return (Vec::new(), 0);
    }
    let mut paths = Vec::new();
    walk_md(&dir, &mut paths, 0);
    if paths.is_empty() {
        return (Vec::new(), 0);
    }
    let total = paths.len();
    let keywords = extract_keywords(&opts.requirement);
    let mut scored: Vec<(usize, &String)> = paths
        .iter()
        .map(|p| (score_path(p, &keywords), p))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
    let top_k = 6;
    let top: Vec<String> = scored
        .iter()
        .filter(|(s, _)| *s > 0)
        .take(top_k)
        .map(|(_, p)| (*p).clone())
        .collect();
    let chosen = if top.is_empty() {
        let mut sorted: Vec<String> = paths.clone();
        sorted.sort();
        sorted.into_iter().take(top_k).collect()
    } else {
        top
    };
    (chosen, total)
}

/// Smart digest: rank knowledge files against `requirement`, then emit
/// the top-K with a short excerpt. Pure-text scoring, no embeddings.
fn smart_knowledge_digest(dir: &Path, requirement: &str) -> String {
    if !dir.is_dir() {
        return "_no `knowledge/` directory in this workspace._".to_string();
    }
    let mut paths = Vec::new();
    walk_md(dir, &mut paths, 0);
    if paths.is_empty() {
        return "_knowledge directory is empty._".to_string();
    }

    let keywords = extract_keywords(requirement);
    // Score every path (path-and-name match counts).
    let mut scored: Vec<(usize, &String)> = paths
        .iter()
        .map(|p| (score_path(p, &keywords), p))
        .collect();
    // Highest score first; ties broken by lex order for determinism.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));

    let top_k = 6;
    let top: Vec<&String> = scored
        .iter()
        .filter(|(s, _)| *s > 0)
        .take(top_k)
        .map(|(_, p)| *p)
        .collect();

    // If no keyword overlap (e.g. all-CJK requirement, all-English
    // filenames), fall back to a stable lex-sorted preview of K files
    // so the prompt still gets something useful.
    let chosen: Vec<&String> = if top.is_empty() {
        let mut sorted: Vec<&String> = paths.iter().collect();
        sorted.sort();
        sorted.into_iter().take(top_k).collect()
    } else {
        top
    };

    let mut out = String::new();
    out.push_str(&format!(
        "Selected {} of {} `knowledge/*.md` files (keyword-ranked against requirement):\n\n",
        chosen.len(),
        paths.len()
    ));
    for rel in chosen {
        let full = dir.join(rel);
        let excerpt = read_excerpt(&full, 600);
        out.push_str(&format!("### `{rel}`\n\n{excerpt}\n\n"));
    }
    out
}

/// Tokenize requirement into 3+ char ASCII / digit tokens, lowercased
/// and de-duplicated. Skips a tiny set of stopwords.
fn extract_keywords(requirement: &str) -> Vec<String> {
    const STOPWORDS: &[&str] = &[
        "the", "and", "for", "with", "that", "from", "this", "have", "into", "make", "build",
        "create", "needs", "want", "system", "support",
    ];
    let mut seen = std::collections::BTreeSet::new();
    requirement
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter_map(|tok| {
            let t = tok.to_ascii_lowercase();
            if t.len() >= 3 && !STOPWORDS.contains(&t.as_str()) && seen.insert(t.clone()) {
                Some(t)
            } else {
                None
            }
        })
        .collect()
}

/// Score a knowledge file against the requirement keywords.
/// Checks both the file path AND the first 500 chars of content.
/// Path hits are weighted 2x (filename is a strong signal).
fn score_path(path: &str, keywords: &[String]) -> usize {
    let p = path.to_ascii_lowercase();
    let path_hits = keywords.iter().filter(|k| p.contains(k.as_str())).count();
    // Content-level check: read first 500 chars for keyword matches.
    let content_hits = std::fs::read_to_string(path).map_or(0, |body| {
        let lower: String = body
            .chars()
            .take(500)
            .collect::<String>()
            .to_ascii_lowercase();
        keywords
            .iter()
            .filter(|k| lower.contains(k.as_str()))
            .count()
    });
    path_hits * 2 + content_hits
}

/// Read the first `limit` chars from `file`, trimmed and cleaned.
/// Returns a placeholder if the file is unreadable.
fn read_excerpt(file: &Path, limit: usize) -> String {
    match fs::read_to_string(file) {
        Ok(body) => {
            let trimmed = body.trim_start();
            let mut excerpt: String = trimmed.chars().take(limit).collect();
            if trimmed.chars().count() > limit {
                excerpt.push_str("\n…");
            }
            excerpt
        }
        Err(_) => "_(unreadable)_".to_string(),
    }
}

/// Run the `research` phase (`SD-ART-001`).
///
/// When `generated_body` is `Some` and non-empty, that text replaces the
/// deterministic template — this is how the runner injects LLM-driven
/// content. The deterministic fallback always carries the
/// requirement and knowledge digest so the artifact is never empty.
pub fn run_research(opts: &RunOptions, generated_body: Option<&str>) -> io::Result<PhaseOutput> {
    let slug = opts.effective_slug();
    let output_dir = opts.project_root.join("output");
    fs::create_dir_all(&output_dir)?;
    let cache_dir = output_dir.join("knowledge-cache");
    fs::create_dir_all(&cache_dir)?;

    let knowledge_digest = summarise_knowledge_dir(&opts.project_root.join("knowledge"));

    let research_path = output_dir.join(format!("{slug}-research.md"));
    let existing_on_disk = fs::read_to_string(&research_path).unwrap_or_default();
    let research_body = match generated_body {
        Some(text) if !text.trim().is_empty() => prefer_richer(text, &existing_on_disk),
        _ => {
            if !existing_on_disk.is_empty() && existing_on_disk.len() > 200 {
                existing_on_disk
            } else {
                format!(
                    "# Research — {slug}\n\n\
                     > Offline scaffold — pass `--backend claude-code` or `--backend codex` to fill this in with real worker-generated content.\n\n\
                     ## Requirement\n\n{}\n\n\
                     ## Local knowledge available\n\n{}\n\n\
                     ## Open questions for the model\n\n\
                     - Which similar products exist? What do they do well / badly?\n\
                     - What domain risks should the architecture mitigate?\n\
                     - What UI patterns are non-negotiable in this domain?\n",
                    opts.requirement, knowledge_digest,
                )
            }
        }
    };
    fs::write(&research_path, &research_body)?;

    let bundle_path = cache_dir.join(format!("{slug}-knowledge-bundle.json"));
    let bundle = serde_json::json!({
        "slug": slug,
        "generated_at": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "requirement": opts.requirement,
        "knowledge_files_scanned": knowledge_digest.lines().count(),
        "research_summary": format!("Stub research bundle for: {}", opts.requirement),
    });
    if let Ok(text) = serde_json::to_string_pretty(&bundle) {
        fs::write(&bundle_path, text)?;
    }

    audit(
        opts,
        "super-dev/agent.research",
        &research_path,
        "SD-ART-001",
        "research artifact written",
    );

    Ok(PhaseOutput {
        phase: Phase::Research,
        artifacts: vec![research_path, bundle_path],
        gate: None,
    })
}

// =====================================================================
// docs (SD-ART-002) → docs_confirm
// =====================================================================

/// Optional LLM-generated bodies for the three core documents. Any
/// `None` falls back to a deterministic template.
#[derive(Debug, Default, Clone)]
pub struct DocsContent {
    /// LLM-generated PRD body.
    pub prd: Option<String>,
    /// LLM-generated architecture body.
    pub architecture: Option<String>,
    /// LLM-generated UI/UX body.
    pub uiux: Option<String>,
}

/// Run the `docs` phase (`SD-ART-002`). Ends at `docs_confirm`.
pub fn run_docs(opts: &RunOptions, content: &DocsContent) -> io::Result<PhaseOutput> {
    let slug = opts.effective_slug();
    let output_dir = opts.project_root.join("output");
    fs::create_dir_all(&output_dir)?;

    let prd = output_dir.join(format!("{slug}-prd.md"));
    let arch = output_dir.join(format!("{slug}-architecture.md"));
    let uiux = output_dir.join(format!("{slug}-uiux.md"));

    // For each doc: prefer the richer of (worker stdout, worker disk file,
    // offline template). This handles the case where the worker writes a
    // full document to disk via Edit tool but returns only a summary to
    // stdout — we keep the richer disk version.
    write_preferring_richer(&prd, &content.prd, || render_prd(&slug, &opts.requirement))?;
    write_preferring_richer(&arch, &content.architecture, || {
        render_architecture(&slug, &opts.requirement)
    })?;
    write_preferring_richer(&uiux, &content.uiux, || {
        render_uiux(&slug, &opts.requirement)
    })?;

    for p in [&prd, &arch, &uiux] {
        audit(
            opts,
            "super-dev/agent.docs",
            p,
            "SD-ART-002",
            "core doc written",
        );
    }

    Ok(PhaseOutput {
        phase: Phase::Docs,
        artifacts: vec![prd, arch, uiux],
        gate: Some(crate::gates::Gate::DocsConfirm),
    })
}

// =====================================================================
// spec (SD-ART-003)
// =====================================================================

/// Run the `spec` phase (`SD-ART-003`). Writes the execution plan and
/// the machine-trackable task list.
pub fn run_spec(opts: &RunOptions) -> io::Result<PhaseOutput> {
    let slug = opts.effective_slug();
    let output_dir = opts.project_root.join("output");
    fs::create_dir_all(&output_dir)?;
    let change_id = format!("{}-{}", slug, Utc::now().format("%Y%m%d%H%M%S"));
    let changes_dir = opts
        .project_root
        .join(".super-dev/changes")
        .join(&change_id);
    fs::create_dir_all(&changes_dir)?;

    let plan = output_dir.join(format!("{slug}-execution-plan.md"));
    let tasks = changes_dir.join("tasks.md");

    fs::write(&plan, render_execution_plan(&slug, &opts.requirement))?;
    fs::write(&tasks, render_tasks(&slug))?;

    audit(
        opts,
        "super-dev/agent.spec",
        &plan,
        "SD-ART-003",
        "execution plan written",
    );
    audit(
        opts,
        "super-dev/agent.spec",
        &tasks,
        "SD-ART-003",
        "task list written",
    );

    Ok(PhaseOutput {
        phase: Phase::Spec,
        artifacts: vec![plan, tasks],
        gate: None,
    })
}

// =====================================================================
// frontend → preview_confirm
// =====================================================================

/// Run the `frontend` phase. V1 only records the phase transition;
/// real implementation work belongs to the LLM milestone.
pub fn run_frontend(opts: &RunOptions) -> io::Result<PhaseOutput> {
    let slug = opts.effective_slug();
    let output_dir = opts.project_root.join("output");
    fs::create_dir_all(&output_dir)?;
    let note = output_dir.join(format!("{slug}-frontend-notes.md"));
    let body = format!(
        "# Frontend notes — {slug}\n\n\
         > Instruction checklist for the interactive worker session.\n\
         > Open Claude Code / Codex in this workspace and follow each item.\n\n\
         ## Sources of truth\n\n\
         - `output/{slug}-prd.md` (acceptance criteria)\n\
         - `output/{slug}-architecture.md` (API surface)\n\
         - `output/{slug}-uiux.md` (design tokens + page hierarchy)\n\n\
         ## Build & verify checklist\n\n\
         - [ ] icon library declared and imported (Lucide / Heroicons / Tabler)\n\
         - [ ] color tokens loaded from `output/{slug}-uiux.md`\n\
         - [ ] every `fetch` URL appears in `output/{slug}-architecture.md`\n\
         - [ ] runtime smoke screenshot attached for review\n",
    );
    fs::write(&note, body)?;
    audit(
        opts,
        "super-dev/agent.frontend",
        &note,
        "SD-CODE-001",
        "frontend notes recorded",
    );

    Ok(PhaseOutput {
        phase: Phase::Frontend,
        artifacts: vec![note],
        gate: Some(crate::gates::Gate::PreviewConfirm),
    })
}

// =====================================================================
// backend
// =====================================================================

/// Run the `backend` phase. V1 records the phase transition + a notes
/// artifact. Real implementation work belongs to the LLM milestone.
pub fn run_backend(opts: &RunOptions) -> io::Result<PhaseOutput> {
    let slug = opts.effective_slug();
    let output_dir = opts.project_root.join("output");
    fs::create_dir_all(&output_dir)?;
    let note = output_dir.join(format!("{slug}-backend-notes.md"));
    let body = format!(
        "# Backend notes — {slug}\n\n\
         > Instruction checklist for the interactive worker session.\n\
         > Open Claude Code / Codex in this workspace and follow each item.\n\n\
         ## Sources of truth\n\n\
         - `output/{slug}-architecture.md` (API surface + data model)\n\
         - `.super-dev/audit/frontend-api-calls.jsonl` (every URL the frontend wrote)\n\n\
         ## Build & verify checklist\n\n\
         - [ ] every route in the frontend audit log has a matching backend handler\n\
         - [ ] tests cover the acceptance criteria from the PRD\n\
         - [ ] secrets / env variables documented in `output/{slug}-architecture.md`\n",
    );
    fs::write(&note, body)?;
    audit(
        opts,
        "super-dev/agent.backend",
        &note,
        "SD-CODE-003",
        "backend notes recorded",
    );

    Ok(PhaseOutput {
        phase: Phase::Backend,
        artifacts: vec![note],
        gate: None,
    })
}

// =====================================================================
// quality (SD-EVID-003) — REAL scoring
// =====================================================================

/// One row in the quality report. Matches the shape required by
/// `SUPER_DEV_HOST_SPEC_V1` §6.3.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityCheck {
    /// Human-readable name.
    pub name: String,
    /// Grouping (artifact / evidence / code-rule / …).
    pub category: String,
    /// Detail line.
    pub description: String,
    /// `passed` | `warning` | `failed`.
    pub status: String,
    /// 0-100.
    pub score: i32,
    /// Relative weight.
    pub weight: f32,
    /// Free-form details.
    pub details: String,
}

/// The quality report document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityReport {
    /// Whether the run passed the gate.
    pub passed: bool,
    /// Plain mean of all check scores.
    pub total_score: i32,
    /// Weighted mean.
    pub weighted_score: f32,
    /// Optional scenario identifier.
    pub scenario: String,
    /// Names of checks that failed AND were marked critical.
    pub critical_failures: Vec<String>,
    /// Human-facing fixes.
    pub recommendations: Vec<String>,
    /// Summary roll-up.
    pub summary: QualitySummary,
    /// Per-check rows.
    pub checks: Vec<QualityCheck>,
}

/// Summary roll-up.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct QualitySummary {
    /// One-line headline.
    pub executive_summary: String,
    /// Free-form key/value context.
    pub summary_context: std::collections::BTreeMap<String, String>,
}

/// Run the `quality` phase (`SD-EVID-003`). Scans workspace artifacts +
/// audit logs and writes `output/<slug>-quality-gate.json`.
pub fn run_quality(opts: &RunOptions) -> io::Result<PhaseOutput> {
    let slug = opts.effective_slug();
    let output_dir = opts.project_root.join("output");
    fs::create_dir_all(&output_dir)?;
    let pass_threshold = 90;

    let mut checks: Vec<QualityCheck> = Vec::new();

    // SD-ART-001 — research artifact
    checks.push(file_present_check(
        "Research artifact",
        "artifact",
        "SD-ART-001 — output/<slug>-research.md present",
        &output_dir.join(format!("{slug}-research.md")),
        1.5,
    ));

    // Discovery section in research
    let research_content = fs::read_to_string(output_dir.join(format!("{slug}-research.md")))
        .unwrap_or_default()
        .to_ascii_lowercase();
    let has_discovery = research_content.contains("## discovery")
        || research_content.contains("target audience")
        || research_content.contains("design direction");
    checks.push(QualityCheck {
        name: "Discovery section".to_string(),
        category: "quality".to_string(),
        description: "Research brief includes Discovery questions (audience/tone/direction)"
            .to_string(),
        status: if has_discovery {
            "passed".to_string()
        } else {
            "warning".to_string()
        },
        score: if has_discovery { 100 } else { 60 },
        details: if has_discovery {
            "Discovery section found in research brief".to_string()
        } else {
            "Missing Discovery section — design direction may be inconsistent".to_string()
        },
        weight: 1.5,
    });

    // SD-ART-002 — three core docs
    for (label, file) in [
        ("PRD", output_dir.join(format!("{slug}-prd.md"))),
        (
            "Architecture",
            output_dir.join(format!("{slug}-architecture.md")),
        ),
        ("UI/UX", output_dir.join(format!("{slug}-uiux.md"))),
    ] {
        checks.push(file_present_check(
            &format!("Core document — {label}"),
            "artifact",
            "SD-ART-002 — core doc present",
            &file,
            2.0,
        ));
    }

    // SD-ART-003 — execution plan
    checks.push(file_present_check(
        "Execution plan",
        "artifact",
        "SD-ART-003 — output/<slug>-execution-plan.md present",
        &output_dir.join(format!("{slug}-execution-plan.md")),
        1.5,
    ));

    // SD-EVID-001 — API audit
    let api_log = opts
        .project_root
        .join(".super-dev/audit/frontend-api-calls.jsonl");
    checks.push(evidence_check(
        "API audit log",
        "SD-EVID-001 — frontend-api-calls.jsonl present and non-empty",
        &api_log,
        1.0,
    ));

    // SD-EVID-002 — tool-call audit
    let tool_log = opts.project_root.join(".super-dev/audit/tool-calls.jsonl");
    checks.push(evidence_check(
        "Tool-call audit log",
        "SD-EVID-002 — tool-calls.jsonl present and non-empty",
        &tool_log,
        1.0,
    ));

    // SD-CODE-001 / SD-CODE-002 — violations in audit log
    let (emoji_blocks, color_blocks) = count_code_violations(&tool_log);
    checks.push(violation_check(
        "Emoji block events",
        "SD-CODE-001 — no emoji-as-icon attempted in this run",
        emoji_blocks,
        2.0,
    ));
    checks.push(violation_check(
        "Hardcoded color block events",
        "SD-CODE-002 — no hardcoded colors attempted in this run",
        color_blocks,
        2.0,
    ));

    // SD-CODE-005 — anti-slop visual quality check on output artifacts
    let slop_issues = count_slop_violations(&output_dir);
    let slop_detail = if slop_issues == 0 {
        "No AI template patterns detected in output artifacts".to_string()
    } else {
        format!(
            "{slop_issues} AI-slop pattern(s) detected (Lorem ipsum, generic headings, purple gradients)"
        )
    };
    checks.push(QualityCheck {
        name: "Anti-AI-slop check".to_string(),
        category: "quality".to_string(),
        description: "SD-CODE-005 — no AI-template visual patterns in output".to_string(),
        status: if slop_issues == 0 {
            "passed".to_string()
        } else {
            "warning".to_string()
        },
        score: if slop_issues == 0 { 100 } else { 60 },
        details: slop_detail,
        weight: 1.5,
    });

    // SD-CODE-003 — API URL frontend↔backend consistency
    let api_consistency = check_api_url_consistency(opts, &slug);
    checks.push(QualityCheck {
        name: "API URL consistency".to_string(),
        category: "code-rule".to_string(),
        description: "SD-CODE-003 — frontend fetch URLs match architecture API surface".to_string(),
        status: api_consistency.0.clone(),
        score: api_consistency.1,
        details: api_consistency.2,
        weight: 2.0,
    });

    // Dark mode check — does the UIUX doc define dark mode tokens?
    let uiux_path = output_dir.join(format!("{slug}-uiux.md"));
    let dark_mode = check_dark_mode_support(&uiux_path);
    checks.push(QualityCheck {
        name: "Dark mode support".to_string(),
        category: "quality".to_string(),
        description: "UIUX doc includes dark mode / prefers-color-scheme tokens".to_string(),
        status: dark_mode.0.clone(),
        score: dark_mode.1,
        details: dark_mode.2,
        weight: 1.0,
    });

    let uiux_score = i32::try_from(score_uiux_completeness(&uiux_path)).unwrap_or(100);
    checks.push(QualityCheck {
        name: "Design system completeness".to_string(),
        category: "quality".to_string(),
        description: "UIUX doc includes color/typography/spacing/icon/component/a11y sections"
            .to_string(),
        status: if uiux_score >= 80 {
            "passed".to_string()
        } else if uiux_score >= 50 {
            "warning".to_string()
        } else {
            "failed".to_string()
        },
        score: uiux_score,
        details: format!("UIUX document completeness: {uiux_score}/100"),
        weight: 2.0,
    });

    let total_score = avg_score(&checks);
    let weighted_score = weighted_avg(&checks);
    let critical_failures: Vec<String> = checks
        .iter()
        .filter(|c| c.status == "failed" && c.category == "artifact")
        .map(|c| c.name.clone())
        .collect();
    let recommendations = checks
        .iter()
        .filter(|c| c.status != "passed")
        .map(|c| format!("Address `{}`: {}", c.name, c.details))
        .collect();
    let passed = total_score >= pass_threshold && critical_failures.is_empty();
    let mut summary_context = std::collections::BTreeMap::new();
    summary_context.insert("spec_version".into(), super_dev_spec::SPEC_VERSION.into());
    summary_context.insert("slug".into(), slug.clone());

    let executive_summary = if passed {
        format!("Quality gate PASSED with score {total_score}/100.")
    } else {
        format!(
            "Quality gate FAILED with score {}/100; {} critical issue(s).",
            total_score,
            critical_failures.len()
        )
    };

    let report = QualityReport {
        passed,
        total_score,
        weighted_score,
        scenario: "1-N+1".to_string(),
        critical_failures,
        recommendations,
        summary: QualitySummary {
            executive_summary,
            summary_context,
        },
        checks,
    };

    let json_path = output_dir.join(format!("{slug}-quality-gate.json"));
    let md_path = output_dir.join(format!("{slug}-quality-gate.md"));
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&report).unwrap_or_default(),
    )?;
    fs::write(&md_path, render_quality_md(&report))?;

    audit(
        opts,
        "super-dev/agent.quality",
        &json_path,
        "SD-EVID-003",
        "quality report written",
    );

    Ok(PhaseOutput {
        phase: Phase::Quality,
        artifacts: vec![json_path, md_path],
        gate: None,
    })
}

fn file_present_check(
    name: &str,
    category: &str,
    desc: &str,
    path: &Path,
    weight: f32,
) -> QualityCheck {
    let present = path.is_file() && fs::metadata(path).is_ok_and(|m| m.len() > 0);
    QualityCheck {
        name: name.to_string(),
        category: category.to_string(),
        description: desc.to_string(),
        status: if present { "passed" } else { "failed" }.to_string(),
        score: if present { 100 } else { 0 },
        weight,
        details: if present {
            format!("found {}", path.display())
        } else {
            format!("missing {}", path.display())
        },
    }
}

fn evidence_check(name: &str, desc: &str, path: &Path, weight: f32) -> QualityCheck {
    let lines = file_line_count(path);
    let status = if lines > 0 { "passed" } else { "warning" };
    let score = if lines > 0 { 100 } else { 60 };
    QualityCheck {
        name: name.to_string(),
        category: "evidence".to_string(),
        description: desc.to_string(),
        status: status.to_string(),
        score,
        weight,
        details: if lines > 0 {
            format!("{} rows recorded in {}", lines, path.display())
        } else {
            format!("no rows yet at {}", path.display())
        },
    }
}

fn violation_check(name: &str, desc: &str, blocks: usize, weight: f32) -> QualityCheck {
    let (status, score) = match blocks {
        0 => ("passed", 100),
        1..=2 => ("warning", 70),
        _ => ("failed", 30),
    };
    QualityCheck {
        name: name.to_string(),
        category: "code_rule".to_string(),
        description: desc.to_string(),
        status: status.to_string(),
        score,
        weight,
        details: format!("{blocks} block event(s) recorded in this run"),
    }
}

fn avg_score(checks: &[QualityCheck]) -> i32 {
    if checks.is_empty() {
        return 0;
    }
    let sum: i32 = checks.iter().map(|c| c.score).sum();
    sum / i32::try_from(checks.len()).unwrap_or(1)
}

fn weighted_avg(checks: &[QualityCheck]) -> f32 {
    if checks.is_empty() {
        return 0.0;
    }
    let total_weight: f32 = checks.iter().map(|c| c.weight).sum();
    if total_weight <= 0.0 {
        return 0.0;
    }
    let weighted: f32 = checks.iter().map(|c| c.score as f32 * c.weight).sum();
    weighted / total_weight
}

fn file_line_count(path: &Path) -> usize {
    fs::read_to_string(path).map_or(0, |t| t.lines().filter(|l| !l.trim().is_empty()).count())
}

fn count_code_violations(tool_log: &Path) -> (usize, usize) {
    let mut emoji = 0;
    let mut color = 0;
    if let Ok(text) = fs::read_to_string(tool_log) {
        for line in text.lines() {
            let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            if val.get("decision").and_then(serde_json::Value::as_str) != Some("block") {
                continue;
            }
            match val.get("clause").and_then(serde_json::Value::as_str) {
                Some("SD-CODE-001") => emoji += 1,
                Some("SD-CODE-002") => color += 1,
                _ => {}
            }
        }
    }
    (emoji, color)
}

fn render_quality_md(r: &QualityReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Quality gate — {}\n\n",
        if r.passed { "PASSED" } else { "FAILED" }
    ));
    out.push_str(&format!(
        "Total score: **{} / 100** (weighted {:.1})\n\n",
        r.total_score, r.weighted_score
    ));
    out.push_str(&format!("{}\n\n", r.summary.executive_summary));
    if !r.critical_failures.is_empty() {
        out.push_str("## Critical failures\n\n");
        for f in &r.critical_failures {
            out.push_str(&format!("- {f}\n"));
        }
        out.push('\n');
    }
    out.push_str(
        "## Checks\n\n| Check | Category | Status | Score | Details |\n|---|---|---|---|---|\n",
    );
    for c in &r.checks {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            c.name, c.category, c.status, c.score, c.details
        ));
    }
    if !r.recommendations.is_empty() {
        out.push_str("\n## Recommendations\n\n");
        for rec in &r.recommendations {
            out.push_str(&format!("- {rec}\n"));
        }
    }
    out
}

// =====================================================================
// delivery (SD-EVID-005) — proof pack
// =====================================================================

/// Run the `delivery` phase (`SD-EVID-005`). Emits compliance mapping
/// and a proof-pack zip in `release/`.
pub fn run_delivery(opts: &RunOptions) -> io::Result<PhaseOutput> {
    let slug = opts.effective_slug();

    // 1. Compliance mapping
    let mut artifacts = Vec::new();
    if let Some((path, _)) = write_compliance_mapping(&opts.project_root, &slug) {
        audit(
            opts,
            "super-dev/agent.delivery",
            &path,
            "SD-EVID-004",
            "compliance mapping written",
        );
        artifacts.push(path);
    }

    // 2. Frontend API audit summary — we re-extract from the latest frontend notes,
    //    if present, to refresh the audit (no-op if file absent).
    let fe_notes = opts
        .project_root
        .join("output")
        .join(format!("{slug}-frontend-notes.md"));
    if fe_notes.is_file() {
        if let Ok(text) = fs::read_to_string(&fe_notes) {
            let _ = extract_api_urls(fe_notes.to_string_lossy().as_ref(), &text);
        }
    }

    // 3. Proof pack zip
    let release_dir = opts.project_root.join("release");
    fs::create_dir_all(&release_dir)?;
    let run_id = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let zip_path = release_dir.join(format!("proof-pack-{slug}-{run_id}.zip"));
    let manifest = build_and_zip_proof_pack(&opts.project_root, &zip_path, &slug)?;

    let manifest_path = release_dir.join(format!("proof-pack-{slug}-{run_id}.manifest.txt"));
    fs::write(&manifest_path, manifest.join("\n"))?;

    audit(
        opts,
        "super-dev/agent.delivery",
        &zip_path,
        "SD-EVID-005",
        "proof pack assembled",
    );

    artifacts.push(zip_path);
    artifacts.push(manifest_path);

    Ok(PhaseOutput {
        phase: Phase::Delivery,
        artifacts,
        gate: None,
    })
}

fn build_and_zip_proof_pack(
    project_root: &Path,
    zip_path: &Path,
    slug: &str,
) -> io::Result<Vec<String>> {
    let file = File::create(zip_path)?;
    let mut zw = zip::ZipWriter::new(file);
    let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut manifest = Vec::new();

    // Glob targets
    let mut targets: Vec<PathBuf> = Vec::new();
    for name in [
        format!("output/{slug}-research.md"),
        format!("output/{slug}-prd.md"),
        format!("output/{slug}-architecture.md"),
        format!("output/{slug}-uiux.md"),
        format!("output/{slug}-execution-plan.md"),
        format!("output/{slug}-frontend-notes.md"),
        format!("output/{slug}-backend-notes.md"),
        format!("output/{slug}-quality-gate.json"),
        format!("output/{slug}-quality-gate.md"),
        format!("output/{slug}-compliance-mapping.json"),
        format!("output/knowledge-cache/{slug}-knowledge-bundle.json"),
        ".super-dev/audit/frontend-api-calls.jsonl".to_string(),
        ".super-dev/audit/tool-calls.jsonl".to_string(),
        ".super-dev/workflow-state.json".to_string(),
    ] {
        let p = project_root.join(&name);
        if p.is_file() {
            targets.push(p);
        }
    }
    // recursively include .super-dev/changes/ and .super-dev/decisions/
    for dir in [".super-dev/changes", ".super-dev/decisions"] {
        let d = project_root.join(dir);
        if d.is_dir() {
            walk_files(&d, &mut targets, 0);
        }
    }

    for t in &targets {
        let rel = t.strip_prefix(project_root).unwrap_or(t.as_path());
        let name = rel.to_string_lossy().to_string();
        if zw.start_file(&name, opts).is_err() {
            continue;
        }
        if let Ok(mut f) = File::open(t) {
            let mut buf = Vec::new();
            if f.read_to_end(&mut buf).is_ok() {
                let _ = zw.write_all(&buf);
            }
        }
        manifest.push(name);
    }
    zw.finish()?;
    Ok(manifest)
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    if depth > 6 {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk_files(&p, out, depth + 1);
        } else {
            out.push(p);
        }
    }
}

// =====================================================================
// helpers
// =====================================================================

/// Pick `override` text when non-empty, else compute the deterministic fallback.
fn write_preferring_richer(
    path: &Path,
    stdout_text: &Option<String>,
    fallback: impl FnOnce() -> String,
) -> io::Result<()> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    let candidate = pick(stdout_text, fallback);
    let body = prefer_richer(&candidate, &existing);
    fs::write(path, body)
}

fn pick(override_text: &Option<String>, fallback: impl FnOnce() -> String) -> String {
    match override_text {
        Some(text) if !text.trim().is_empty() => text.clone(),
        _ => fallback(),
    }
}

/// Check API URL consistency: every path in the architecture API surface
/// table should be referenced somewhere in the frontend code or notes.
fn check_api_url_consistency(opts: &RunOptions, slug: &str) -> (String, i32, String) {
    let arch_path = opts
        .project_root
        .join("output")
        .join(format!("{slug}-architecture.md"));
    let arch_content = fs::read_to_string(&arch_path).unwrap_or_default();

    let mut api_paths: Vec<String> = Vec::new();
    for line in arch_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.contains('/') {
            for part in trimmed.split('|') {
                let p = part.trim();
                if p.starts_with('/') && p.len() > 1 && !p.contains("---") {
                    let path = p.split_whitespace().next().unwrap_or(p);
                    if !api_paths.contains(&path.to_string()) {
                        api_paths.push(path.to_string());
                    }
                }
            }
        }
    }

    if api_paths.is_empty() {
        return (
            "warning".to_string(),
            70,
            "No API paths found in architecture doc — cannot verify consistency".to_string(),
        );
    }

    let fe_notes_path = opts
        .project_root
        .join("output")
        .join(format!("{slug}-frontend-notes.md"));
    let fe_content = fs::read_to_string(&fe_notes_path).unwrap_or_default();
    let api_log = opts
        .project_root
        .join(".super-dev/audit/frontend-api-calls.jsonl");
    let api_log_content = fs::read_to_string(&api_log).unwrap_or_default();
    let combined = format!("{fe_content}\n{api_log_content}");

    let mut missing: Vec<&str> = Vec::new();
    for path in &api_paths {
        if !combined.contains(path.as_str()) {
            missing.push(path);
        }
    }

    if missing.is_empty() {
        (
            "passed".to_string(),
            100,
            format!(
                "All {} API paths from architecture doc are referenced in frontend",
                api_paths.len()
            ),
        )
    } else {
        (
            "warning".to_string(),
            (100 - (i32::try_from(missing.len()).unwrap_or(5) * 15)).max(30),
            format!(
                "{}/{} API paths not found in frontend: {}",
                missing.len(),
                api_paths.len(),
                missing
                    .iter()
                    .take(5)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )
    }
}

/// Check if the UIUX document defines dark mode tokens.
fn check_dark_mode_support(uiux_path: &Path) -> (String, i32, String) {
    let content = fs::read_to_string(uiux_path).unwrap_or_default();
    let lower = content.to_ascii_lowercase();
    let has_dark = lower.contains("prefers-color-scheme")
        || lower.contains("dark mode")
        || lower.contains("dark-mode")
        || (lower.contains("@media") && lower.contains("dark"));

    if has_dark {
        (
            "passed".to_string(),
            100,
            "Dark mode tokens defined in UIUX document".to_string(),
        )
    } else if content.is_empty() {
        (
            "warning".to_string(),
            50,
            "UIUX document not yet created".to_string(),
        )
    } else {
        (
            "warning".to_string(),
            70,
            "No dark mode / prefers-color-scheme tokens found — consider adding for accessibility"
                .to_string(),
        )
    }
}

/// Count anti-slop violations across markdown artifacts in output/.
fn count_slop_violations(output_dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(rd) = fs::read_dir(output_dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&p) {
                let lower = content.to_ascii_lowercase();
                if lower.contains("lorem ipsum") || lower.contains("dolor sit amet") {
                    count += 1;
                }
                if lower.contains("welcome to") && lower.contains("# ") {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Score UIUX document completeness. Checks for key sections:
/// color palette, typography, spacing, icon library, components,
/// accessibility. Each section found = +16 points (max ~100).
fn score_uiux_completeness(path: &Path) -> u32 {
    let content = fs::read_to_string(path).unwrap_or_default();
    let lower = content.to_ascii_lowercase();
    if lower.is_empty() {
        return 0;
    }
    // Section presence (10 pts each, max 70)
    let sections = [
        "color",
        "typography",
        "spacing",
        "icon",
        "component",
        "accessibility",
        "dark",
    ];
    let section_score =
        (sections.iter().filter(|s| lower.contains(**s)).count() as u32 * 10).min(70);
    // Token count bonus (count --color / --font / --space var declarations)
    let token_count = content.matches("--").count() as u32;
    let token_bonus = if token_count >= 50 {
        20
    } else if token_count >= 20 {
        15
    } else if token_count >= 10 {
        10
    } else {
        0
    };
    // Length bonus
    let length_bonus = if content.len() > 2000 {
        10
    } else if content.len() > 500 {
        5
    } else {
        0
    };
    (section_score + token_bonus + length_bonus).min(100)
}

/// When the worker returns text via stdout AND already wrote a file to
/// disk, the disk version is often the richer one (full document) while
/// stdout may be just a summary. Pick whichever has more substance.
fn prefer_richer(stdout_text: &str, disk_text: &str) -> String {
    if disk_text.len() > stdout_text.len() * 2 && disk_text.len() > 200 {
        disk_text.to_string()
    } else {
        stdout_text.to_string()
    }
}

fn audit(opts: &RunOptions, tool_name: &str, target: &Path, clause: &str, reason: &str) {
    let _ = record_tool_call(
        &opts.project_root,
        tool_name,
        target.to_string_lossy().as_ref(),
        "audit",
        clause,
        reason,
        "",
        None,
    );
}

fn summarise_knowledge_dir(dir: &Path) -> String {
    if !dir.is_dir() {
        return "_no `knowledge/` directory in this workspace._".to_string();
    }
    let mut entries = Vec::new();
    walk_md(dir, &mut entries, 0);
    if entries.is_empty() {
        return "_knowledge directory is empty._".to_string();
    }
    let mut lines: Vec<String> = entries
        .iter()
        .take(40)
        .map(|p| format!("- `{p}`"))
        .collect();
    if entries.len() > 40 {
        lines.push(format!("- … and {} more", entries.len() - 40));
    }
    lines.join("\n")
}

fn walk_md(dir: &Path, out: &mut Vec<String>, depth: usize) {
    if depth > 4 || out.len() >= 200 {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_md(&p, out, depth + 1);
        } else if p.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(rel) = p.to_str() {
                let cleaned = rel.split("/knowledge/").nth(1).unwrap_or(rel);
                out.push(cleaned.to_string());
            }
        }
    }
}

fn render_prd(slug: &str, requirement: &str) -> String {
    format!(
        "# PRD — {slug}\n\n\
         > Offline scaffold — pass `--backend claude-code` or `--backend codex` to fill this in with real worker-generated content.\n\n\
         ## Goal\n\n{requirement}\n\n\
         ## Scope\n\n- _what's in_\n- _what's out_\n\n\
         ## User stories\n\n- As a user, I want …\n\n\
         ## Acceptance criteria\n\n- [ ] criterion 1\n- [ ] criterion 2\n\n\
         ## Risks & open questions\n\n- _list domain-specific risks_\n",
    )
}

fn render_architecture(slug: &str, requirement: &str) -> String {
    format!(
        "# Architecture — {slug}\n\n\
         > Offline scaffold — pass `--backend claude-code` or `--backend codex` to fill this in with real worker-generated content.\n\n\
         ## System overview\n\n_Diagram + prose describing the components implementing: {requirement}_\n\n\
         ## API surface\n\n| Method | Path | Purpose |\n|---|---|---|\n| GET | /api/example | _purpose_ |\n\n\
         ## Data model\n\n_Schemas / tables / message shapes._\n\n\
         ## Tech-stack rationale\n\n- Frontend: _choice + reason_\n- Backend: _choice + reason_\n- Storage: _choice + reason_\n",
    )
}

fn render_uiux(slug: &str, requirement: &str) -> String {
    format!(
        "# UI/UX — {slug}\n\n\
         > Offline scaffold — pass `--backend claude-code` or `--backend droid` to generate a real design system.\n\n\
         ## Visual direction\n\nModern Minimal — clean, precise, whitespace-first.\n\n\
         ## Color palette\n\n```css\n:root {{\n\
         \x20 --color-bg: #fafafa;\n\
         \x20 --color-surface: #ffffff;\n\
         \x20 --color-text: #111827;\n\
         \x20 --color-text-secondary: #6b7280;\n\
         \x20 --color-primary: #2563eb;\n\
         \x20 --color-primary-hover: #1d4ed8;\n\
         \x20 --color-accent: #f59e0b;\n\
         \x20 --color-border: #e5e7eb;\n\
         \x20 --color-error: #ef4444;\n\
         \x20 --color-success: #10b981;\n\
         }}\n\
         @media (prefers-color-scheme: dark) {{\n\
         \x20 :root {{\n\
         \x20\x20\x20 --color-bg: #0f172a;\n\
         \x20\x20\x20 --color-surface: #1e293b;\n\
         \x20\x20\x20 --color-text: #f1f5f9;\n\
         \x20\x20\x20 --color-text-secondary: #94a3b8;\n\
         \x20\x20\x20 --color-border: #334155;\n\
         \x20 }}\n\
         }}\n```\n\n\
         ## Typography system\n\n\
         - Headings: `Inter, system-ui, sans-serif` weight 600\n\
         - Body: `Inter, system-ui, sans-serif` weight 400\n\
         - `--text-xs: 0.75rem` / `--text-sm: 0.875rem` / `--text-base: 1rem` / `--text-lg: 1.125rem` / `--text-xl: 1.25rem` / `--text-2xl: 1.5rem` / `--text-3xl: 1.875rem`\n\n\
         ## Spacing scale\n\n\
         `--space-1: 4px` / `--space-2: 8px` / `--space-3: 12px` / `--space-4: 16px` / `--space-6: 24px` / `--space-8: 32px` / `--space-10: 40px` / `--space-12: 48px`\n\n\
         ## Icon library\n\n- Declared: Lucide\n\n\
         ## Page hierarchy\n\n- `/` Home\n  - `/detail/:id` Detail\n  - `/settings` Settings\n\n\
         ## Component inventory\n\n\
         _Components for: {requirement}_\n\n\
         | Component | States |\n\
         |---|---|\n\
         | Button | default / hover / active / disabled / loading |\n\
         | Input | default / focus / error / disabled |\n\
         | Card | default / hover / selected |\n\
         | Modal | open / closing (transition) |\n\n\
         ## Motion guidelines\n\n\
         - `--transition-fast: 150ms ease-out` (hover, focus)\n\
         - `--transition-normal: 250ms ease-in-out` (modals, drawers)\n\
         - `--transition-slow: 400ms ease-in-out` (page transitions)\n\n\
         ## Anti-patterns\n\n\
         1. No decorative hero gradients\n\
         2. No emoji as functional icons\n\
         3. No AI-chatbot shell layout\n\
         4. No cards with identical placeholder text\n\
         5. No cramped layouts without spacing tokens\n\n\
         ## Self-critique\n\n\
         | Dimension | Score |\n|---|---|\n\
         | Hierarchy clarity | 7/10 |\n\
         | Visual distinctiveness | 6/10 |\n\
         | Detail polish | 5/10 |\n\
         | Functional completeness | 7/10 |\n\
         | Innovation | 5/10 |\n\n\
         > Offline template scores low on distinctiveness + polish — use a real worker for production.\n\n\
         ## Accessibility notes\n\n\
         - Color contrast ≥ 4.5:1 (AA)\n\
         - Keyboard reachable for every interactive control\n\
         - Focus ring: 2px solid var(--color-primary), offset 2px\n\
         - `aria-label` on icon-only buttons\n",
    )
}

fn render_execution_plan(slug: &str, requirement: &str) -> String {
    format!(
        "# Execution plan — {slug}\n\n\
         > Skeleton execution plan — open in your worker session and flesh out per-task acceptance criteria.\n\n\
         ## Goal recap\n\n{requirement}\n\n\
         ## Sequence\n\n\
         1. Frontend skeleton + design tokens\n\
         2. Backend route stubs aligned with the architecture API surface\n\
         3. Integration smoke test\n\
         4. Quality gate + proof pack\n",
    )
}

fn render_tasks(slug: &str) -> String {
    format!(
        "# Tasks — {slug}\n\n\
         - [ ] frontend / scaffold pages per UIUX\n\
         - [ ] frontend / wire fetch calls to architecture API paths\n\
         - [ ] backend / implement architecture routes\n\
         - [ ] backend / write integration tests\n\
         - [ ] quality / run super-dev quality gate\n\
         - [ ] delivery / assemble proof pack\n",
    )
}

// =====================================================================
// tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn opts(root: &Path) -> RunOptions {
        RunOptions {
            project_root: root.to_path_buf(),
            requirement: "build a login system".to_string(),
            slug: "demo".to_string(),
            model: "stub".to_string(),
            backend: String::new(),
            design_system: String::new(),
            seed_template: String::new(),
        }
    }

    #[test]
    fn research_writes_artifact_and_bundle() {
        let tmp = TempDir::new().unwrap();
        let out = run_research(&opts(tmp.path()), None).unwrap();
        assert_eq!(out.phase, Phase::Research);
        assert!(out.artifacts[0].ends_with("output/demo-research.md"));
        let body = fs::read_to_string(&out.artifacts[0]).unwrap();
        assert!(body.contains("build a login system"));
    }

    #[test]
    fn docs_writes_three_files_and_stops_at_gate() {
        let tmp = TempDir::new().unwrap();
        let out = run_docs(&opts(tmp.path()), &DocsContent::default()).unwrap();
        assert_eq!(out.phase, Phase::Docs);
        assert_eq!(out.artifacts.len(), 3);
        assert_eq!(out.gate, Some(crate::gates::Gate::DocsConfirm));
    }

    #[test]
    fn spec_writes_plan_and_tasks() {
        let tmp = TempDir::new().unwrap();
        let out = run_spec(&opts(tmp.path())).unwrap();
        assert_eq!(out.phase, Phase::Spec);
        assert_eq!(out.artifacts.len(), 2);
        assert!(out.artifacts[0].ends_with("output/demo-execution-plan.md"));
        let body = fs::read_to_string(&out.artifacts[1]).unwrap();
        assert!(body.contains("Tasks"));
    }

    #[test]
    fn frontend_writes_notes_and_pauses_at_preview_gate() {
        let tmp = TempDir::new().unwrap();
        let out = run_frontend(&opts(tmp.path())).unwrap();
        assert_eq!(out.phase, Phase::Frontend);
        assert_eq!(out.gate, Some(crate::gates::Gate::PreviewConfirm));
    }

    #[test]
    fn backend_writes_notes_no_gate() {
        let tmp = TempDir::new().unwrap();
        let out = run_backend(&opts(tmp.path())).unwrap();
        assert_eq!(out.phase, Phase::Backend);
        assert!(out.gate.is_none());
    }

    #[test]
    fn quality_produces_real_score() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        // First run the prior phases so quality has something to grade
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();

        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        assert!(report.total_score > 0);
        // 5 artifacts present (research, prd, arch, uiux, execution-plan) + tool-call audit present
        // → expect score well above 0
        assert!(report
            .checks
            .iter()
            .any(|c| c.name == "PRD" || c.name.starts_with("Core document")));
    }

    #[test]
    fn quality_fails_on_missing_docs() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        assert!(!report.passed);
        assert!(!report.critical_failures.is_empty());
    }

    #[test]
    fn quality_counts_code_violations() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        // Seed two emoji-block events in the tool-call log
        let audit_dir = tmp.path().join(".super-dev/audit");
        fs::create_dir_all(&audit_dir).unwrap();
        let log = audit_dir.join("tool-calls.jsonl");
        fs::write(&log, r#"{"ts":1,"tool":"Write","file":"a.tsx","decision":"block","clause":"SD-CODE-001","reason":"emoji","session_id":""}
{"ts":2,"tool":"Write","file":"b.tsx","decision":"block","clause":"SD-CODE-001","reason":"emoji","session_id":""}
{"ts":3,"tool":"Write","file":"c.tsx","decision":"block","clause":"SD-CODE-002","reason":"color","session_id":""}
"#).unwrap();
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        let emoji_check = report
            .checks
            .iter()
            .find(|c| c.name == "Emoji block events")
            .unwrap();
        assert!(emoji_check.score < 100);
        assert!(emoji_check.details.contains('2'));
    }

    #[test]
    fn delivery_produces_proof_pack_zip() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        // Populate the workspace with a previous full run
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        run_frontend(&o).unwrap();
        run_backend(&o).unwrap();
        run_quality(&o).unwrap();

        let out = run_delivery(&o).unwrap();
        // expect at least the compliance mapping + zip + manifest
        assert!(out
            .artifacts
            .iter()
            .any(|p| p.extension().and_then(|s| s.to_str()) == Some("zip")));
        assert!(out
            .artifacts
            .iter()
            .any(|p| p.to_string_lossy().contains("compliance-mapping.json")));
        let zip = out
            .artifacts
            .iter()
            .find(|p| p.extension().and_then(|s| s.to_str()) == Some("zip"))
            .unwrap();
        assert!(zip.is_file());
        assert!(fs::metadata(zip).unwrap().len() > 0);
    }

    // ---- smart knowledge digest ----

    #[test]
    fn extract_keywords_filters_short_and_stopwords() {
        let kws = extract_keywords("Build a login system with OAuth 2 and MFA support");
        // 'build', 'with', 'and', 'system', 'support' are stopwords or short;
        // 'login', 'oauth', 'mfa' should survive.
        assert!(kws.iter().any(|k| k == "login"));
        assert!(kws.iter().any(|k| k == "oauth"));
        assert!(kws.iter().any(|k| k == "mfa"));
        assert!(!kws.iter().any(|k| k == "the"));
        assert!(!kws.iter().any(|k| k == "build"));
    }

    #[test]
    fn score_path_counts_keyword_hits() {
        let kws = vec!["login".to_string(), "oauth".to_string()];
        // Path hits are weighted 2x. Files that don't exist get 0
        // content hits, so score = path_hits * 2 + 0.
        assert_eq!(
            score_path("security/login-oauth-playbook.md", &kws),
            4 // 2 path hits * 2
        );
        assert_eq!(
            score_path("auth/login.md", &kws),
            2 // 1 path hit * 2
        );
        assert_eq!(score_path("docs/contributing.md", &kws), 0);
    }

    #[test]
    fn smart_digest_picks_keyword_matches_top() {
        let tmp = TempDir::new().unwrap();
        let kd = tmp.path().join("knowledge");
        fs::create_dir_all(kd.join("security")).unwrap();
        fs::create_dir_all(kd.join("infra")).unwrap();
        // Should rank these first (keyword "login" present)
        fs::write(
            kd.join("security/login-playbook.md"),
            "# Login Playbook\n\nUse OAuth2 with PKCE.\n",
        )
        .unwrap();
        fs::write(kd.join("security/oauth-complete.md"), "# OAuth Complete\n").unwrap();
        // Should NOT rank above (no keyword)
        fs::write(kd.join("infra/kubernetes-101.md"), "# Kubernetes 101\n").unwrap();
        fs::write(kd.join("infra/postgres-tuning.md"), "# Postgres Tuning\n").unwrap();

        let digest = smart_knowledge_digest(&kd, "build a login system with oauth");
        // The keyword-matched files appear before the unrelated ones.
        let login_idx = digest.find("login-playbook").unwrap();
        let kube_idx = digest.find("kubernetes").unwrap_or(usize::MAX);
        assert!(login_idx < kube_idx, "keyword-matched file must rank first");
        // Excerpt content is included.
        assert!(digest.contains("Use OAuth2 with PKCE."));
    }

    #[test]
    fn smart_digest_falls_back_to_lex_when_no_keyword_match() {
        let tmp = TempDir::new().unwrap();
        let kd = tmp.path().join("knowledge");
        fs::create_dir_all(&kd).unwrap();
        fs::write(kd.join("aaa-first.md"), "# A\n").unwrap();
        fs::write(kd.join("zzz-last.md"), "# Z\n").unwrap();
        // Requirement entirely in CJK → no keyword overlap with English file names.
        let digest = smart_knowledge_digest(&kd, "做一个登录系统");
        // Both files appear; lex-sorted: aaa- before zzz-.
        let a_idx = digest.find("aaa-first").unwrap();
        let z_idx = digest.find("zzz-last").unwrap();
        assert!(a_idx < z_idx);
    }

    #[test]
    fn smart_digest_handles_missing_dir() {
        let tmp = TempDir::new().unwrap();
        let kd = tmp.path().join("nonexistent");
        let digest = smart_knowledge_digest(&kd, "anything");
        assert!(digest.contains("no `knowledge/`"));
    }
}
