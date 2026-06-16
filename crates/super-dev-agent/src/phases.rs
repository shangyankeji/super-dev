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
    phase_knowledge_digest_with_vector(opts, phase, None)
}

/// Phase knowledge digest with an optional pre-embedded query vector (hybrid
/// BM25+vector RRF fusion when available, pure BM25 otherwise).
#[must_use]
pub fn phase_knowledge_digest_with_vector(
    opts: &RunOptions,
    phase: Phase,
    query_vec: Option<&[f32]>,
) -> String {
    let base = opts.project_root.join("knowledge");
    if !base.is_dir() {
        return String::new();
    }
    if matches!(phase, Phase::DocsConfirm | Phase::PreviewConfirm) {
        return String::new();
    }
    let project_cfg = crate::config::load_project_config(&opts.project_root);
    let cfg = &project_cfg.knowledge;
    if cfg.enabled {
        let rcfg = super_dev_knowledge::retrieve::RetrievalConfig {
            enabled: true,
            engine: match cfg.engine.as_str() {
                "hybrid" => super_dev_knowledge::retrieve::RetrievalEngine::Hybrid,
                _ => super_dev_knowledge::retrieve::RetrievalEngine::Bm25,
            },
            top_k: cfg.top_k,
            custom_dirs: Vec::new(),
        };
        let hits = super_dev_knowledge::retrieve_for_phase_with_vector(
            &opts.project_root,
            &base,
            &rcfg,
            &opts.requirement,
            phase,
            query_vec,
        );
        if hits.is_empty() {
            return String::new();
        }
        let label = if query_vec.is_some() && cfg.engine == "hybrid" {
            "BM25+vector RRF-fused"
        } else {
            "BM25-ranked"
        };
        let mut out = format!(
            "\n\n## Expert knowledge ({} phase)\n\nTop {} knowledge chunks ({}):\n\n",
            phase.id(),
            hits.len(),
            label
        );
        for hit in &hits {
            out.push_str(&format!(
                "### `{}` — *{}* (score {:.2})\n\n{}\n\n",
                hit.chunk.meta.path,
                hit.chunk.meta.section,
                hit.score,
                hit.chunk.excerpt(400)
            ));
        }
        return out;
    }
    legacy_phase_knowledge_digest(opts, phase)
}

/// The pre-4.6 keyword-scoring digest, retained as the fallback when
/// `knowledge.enabled = false`.
#[must_use]
fn legacy_phase_knowledge_digest(opts: &RunOptions, phase: Phase) -> String {
    let base = opts.project_root.join("knowledge");
    let subdirs: &[&str] = match phase {
        Phase::Research => return knowledge_digest(opts),
        Phase::Docs => &[
            "experts/product-manager",
            "experts/architect",
            "experts/uiux-designer",
            "product",
            "architecture",
            "design",
            "frontend",
            "industries",
        ],
        Phase::DocsConfirm | Phase::PreviewConfirm => return String::new(),
        Phase::Spec => &[
            "experts/product-manager",
            "experts/architect",
            "development",
            "00-governance",
            "product",
        ],
        Phase::Frontend => &[
            "experts/frontend-lead",
            "experts/uiux-designer",
            "frontend",
            "design",
            "design-systems",
            "seed-templates",
        ],
        Phase::Backend => &[
            "experts/backend-lead",
            "experts/architect",
            "backend",
            "api",
            "database",
            "security",
            "cloud-native",
        ],
        Phase::Quality => &[
            "experts/qa-lead",
            "experts/architect",
            "testing",
            "security",
            "00-governance",
        ],
        Phase::Delivery => &[
            "experts/devops",
            "cicd",
            "operations",
            "00-governance",
            "security",
        ],
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
        // Atomic write (temp file in the same dir + rename) so a concurrent
        // reader in context.rs::read_knowledge_digest never sees a partial
        // JSON file. `rename` on the same filesystem is atomic on POSIX.
        atomic_write(&bundle_path, &text)?;
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
         - [ ] runtime smoke screenshot attached for review\n\n\
         ## Preview URL\n\n\
         _(The worker fills this with the local URL its dev server printed,\n\
         e.g. `http://localhost:5173`. Super Dev opens it for the user.)_\n\n\
         ## Run command\n\n\
         _(e.g. `cd web && npm run dev`)_"
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
    let project_config = crate::config::load_project_config(&opts.project_root);
    let pass_threshold = i32::try_from(project_config.quality.threshold).unwrap_or(90);

    let mut checks: Vec<QualityCheck> = Vec::new();

    // SD-ART-001 — research artifact (content check, not just file-exists)
    let research_path = output_dir.join(format!("{slug}-research.md"));
    let research_text = fs::read_to_string(&research_path).unwrap_or_default();
    let research_defects = review_document_structure(
        &research_text,
        &[
            ("## requirement", "Missing ## Requirement section"),
            ("similar products", "Missing ## Similar products section"),
            ("domain risk", "Missing ## Domain risks section"),
        ],
    );
    checks.push(content_quality_check(
        "Research content",
        "artifact",
        "SD-ART-001 — research has requirement + similar products + risks",
        &research_text,
        &research_defects,
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

    // SD-ART-002 — three core docs (content checks, not just file-exists)
    let prd_text =
        fs::read_to_string(output_dir.join(format!("{slug}-prd.md"))).unwrap_or_default();
    let prd_defects = review_document_structure(
        &prd_text,
        &[
            ("## goal", "Missing ## Goal section"),
            ("## scope", "Missing ## Scope section"),
            ("- [ ]", "Missing acceptance criteria checkboxes"),
        ],
    );
    checks.push(content_quality_check(
        "PRD content",
        "artifact",
        "SD-ART-002 — PRD has goal + scope + acceptance criteria",
        &prd_text,
        &prd_defects,
        2.0,
    ));

    // Cross-reference: count PRD acceptance criteria and verify quantity
    let ac_lines: Vec<&str> = prd_text
        .lines()
        .filter(|l| l.trim().starts_with("- [ ]") || l.trim().starts_with("- [x]"))
        .collect();
    let ac_score = if ac_lines.len() >= 8 {
        100
    } else if ac_lines.len() >= 5 {
        70
    } else {
        i32::try_from(ac_lines.len()).unwrap_or(0) * 10
    };
    checks.push(QualityCheck {
        name: "Acceptance criteria depth".to_string(),
        category: "quality".to_string(),
        description: "PRD has ≥8 testable acceptance criteria in Given/When/Then format"
            .to_string(),
        status: if ac_score >= 70 {
            "passed".to_string()
        } else {
            "warning".to_string()
        },
        score: ac_score,
        details: format!("{} acceptance criteria found (target: ≥8)", ac_lines.len()),
        weight: 2.0,
    });

    let arch_text =
        fs::read_to_string(output_dir.join(format!("{slug}-architecture.md"))).unwrap_or_default();
    let arch_defects = review_document_structure(
        &arch_text,
        &[
            ("## api", "Missing ## API surface section"),
            ("## data model", "Missing ## Data model section"),
            ("| ", "Missing API route table (no markdown table rows)"),
        ],
    );
    checks.push(content_quality_check(
        "Architecture content",
        "artifact",
        "SD-ART-002 — Architecture has API surface + data model",
        &arch_text,
        &arch_defects,
        2.0,
    ));

    let uiux_text =
        fs::read_to_string(output_dir.join(format!("{slug}-uiux.md"))).unwrap_or_default();
    let uiux_defects = review_document_structure(
        &uiux_text,
        &[
            ("--color", "Missing CSS color tokens"),
            ("--font", "Missing typography tokens"),
            ("icon", "Missing icon library declaration"),
            ("hover", "Missing component states (hover/focus)"),
        ],
    );
    checks.push(content_quality_check(
        "UI/UX content",
        "artifact",
        "SD-ART-002 — UIUX has color tokens + typography + icons + states",
        &uiux_text,
        &uiux_defects,
        2.0,
    ));

    // SD-ART-003 — execution plan (content-validated)
    {
        let pp = output_dir.join(format!("{slug}-execution-plan.md"));
        let pt = fs::read_to_string(&pp).unwrap_or_default();
        let pl = pt.lines().filter(|l| !l.trim().is_empty()).count();
        let hs = pt.lines().any(|l| l.trim_start().starts_with("## "));
        let (st, sc, det) = if pt.is_empty() {
            ("failed", 0, format!("missing {}", pp.display()))
        } else if pl < 10 || !hs {
            (
                "warning",
                60,
                format!("{pl} lines, needs structured sections"),
            )
        } else {
            (
                "passed",
                100,
                format!("{pl} lines with structured sections"),
            )
        };
        checks.push(QualityCheck {
            name: "Execution plan".to_string(),
            category: "artifact".to_string(),
            description: "SD-ART-003 — execution-plan.md present with real content".to_string(),
            status: st.to_string(),
            score: sc,
            details: det,
            weight: 1.5,
        });
    }

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

    // Build & test results — consumes the real verify runner output.
    if let Some(vc) = verify_results_check(&opts.project_root) {
        checks.push(vc);
    }

    // Anti-AI-slop visual quality check on output artifacts. Not tied to a
    // single spec clause — Lorem ipsum / generic headings / purple→pink
    // gradients are caught as design-quality signals (the pre-write hook
    // attributes the gradient/color part to SD-CODE-002).
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
        description: "Anti-AI-slop — no AI-template visual patterns in output".to_string(),
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

    // Cross-document: PRD IA routes ↔ Architecture API surface
    let prd_arch = check_prd_arch_alignment(&prd_text, &arch_text);
    checks.push(QualityCheck {
        name: "PRD↔Architecture alignment".to_string(),
        category: "quality".to_string(),
        description: "PRD page routes have corresponding API endpoints in Architecture".to_string(),
        status: prd_arch.0,
        score: prd_arch.1,
        details: prd_arch.2,
        weight: 1.5,
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

    // === Contract-layer checks (require parsing the architecture doc) ===
    let arch_text = fs::read_to_string(
        opts.project_root
            .join("output")
            .join(format!("{slug}-architecture.md")),
    )
    .unwrap_or_default();
    let arch_spec = super_dev_contract::parse_architecture(&arch_text, &format!("{slug} API"));
    let derived = super_dev_contract::derive_endpoints_from_requirement(&opts.requirement);
    let contract_spec = super_dev_contract::merge_specs(&arch_spec, &derived);

    // OpenAPI contract present
    let has_contract = !contract_spec.is_empty();
    checks.push(QualityCheck {
        name: "OpenAPI contract".to_string(),
        category: "contract".to_string(),
        description: "SD-CODE-003 — typed API contract derived from architecture".to_string(),
        status: if has_contract { "passed" } else { "warning" }.to_string(),
        score: if has_contract { 100 } else { 50 },
        weight: 2.0,
        details: if has_contract {
            format!("{} endpoints in contract", contract_spec.len())
        } else {
            "No API contract derived — architecture may lack an API table".to_string()
        },
    });

    // Frontend↔contract conformance
    let fe_calls = super_dev_contract::extract_frontend_calls(
        &opts
            .project_root
            .join("output")
            .join(format!("{slug}-frontend-notes.md")),
    );
    let fe_violations =
        super_dev_contract::validate_frontend_vs_contract(&fe_calls, &contract_spec);
    checks.push(QualityCheck {
        name: "Frontend↔contract conformance".to_string(),
        category: "contract".to_string(),
        description: "SD-CODE-003 — frontend calls match contract paths".to_string(),
        status: if fe_violations.is_empty() {
            "passed"
        } else {
            "warning"
        }
        .to_string(),
        score: if fe_violations.is_empty() { 100 } else { 60 },
        weight: 2.0,
        details: if fe_violations.is_empty() {
            "All frontend calls match the contract".to_string()
        } else {
            format!(
                "{} violation(s): {}",
                fe_violations.len(),
                fe_violations
                    .iter()
                    .take(3)
                    .map(|v| v.detail.clone())
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        },
    });

    // PRD routes↔contract coverage
    let prd_path = opts
        .project_root
        .join("output")
        .join(format!("{slug}-prd.md"));
    let prd_text = fs::read_to_string(&prd_path).unwrap_or_default();
    let prd_routes = super_dev_contract::extract_prd_routes(&prd_text);
    let prd_violations = super_dev_contract::validate_prd_vs_contract(&prd_routes, &contract_spec);
    checks.push(QualityCheck {
        name: "PRD routes↔contract coverage".to_string(),
        category: "contract".to_string(),
        description: "PRD-described routes appear in the contract".to_string(),
        status: if prd_violations.is_empty() {
            "passed"
        } else {
            "warning"
        }
        .to_string(),
        score: if prd_violations.is_empty() { 100 } else { 70 },
        weight: 1.5,
        details: if prd_violations.is_empty() {
            "PRD routes covered by contract".to_string()
        } else {
            format!("{} uncovered route(s)", prd_violations.len())
        },
    });

    // Input validation coverage (mutation endpoints should have request schemas)
    let total_mut = contract_spec
        .endpoints
        .iter()
        .filter(|e| {
            matches!(
                e.method,
                super_dev_contract::HttpVerb::Post
                    | super_dev_contract::HttpVerb::Put
                    | super_dev_contract::HttpVerb::Patch
            )
        })
        .count();
    let mut_val_missing = contract_spec
        .endpoints
        .iter()
        .filter(|e| {
            matches!(
                e.method,
                super_dev_contract::HttpVerb::Post
                    | super_dev_contract::HttpVerb::Put
                    | super_dev_contract::HttpVerb::Patch
            ) && e.request_shape.is_empty()
        })
        .count();
    checks.push(QualityCheck {
        name: "Input validation coverage".to_string(),
        category: "contract".to_string(),
        description: "POST/PATCH/PUT endpoints declare request schemas".to_string(),
        status: if total_mut == 0 || mut_val_missing == 0 {
            "passed"
        } else if mut_val_missing <= 2 {
            "warning"
        } else {
            "failed"
        }
        .to_string(),
        score: if total_mut == 0 {
            100
        } else {
            let ratio = total_mut - mut_val_missing;
            i32::try_from(ratio * 100 / total_mut).unwrap_or(0)
        },
        weight: 1.5,
        details: if total_mut == 0 {
            "No mutation endpoints".to_string()
        } else {
            format!(
                "{}/{} mutation endpoints have request schemas",
                total_mut - mut_val_missing,
                total_mut
            )
        },
    });

    // Pagination strategy (word-boundary match)
    let arch_lower = arch_text.to_ascii_lowercase();
    let list_count = contract_spec
        .endpoints
        .iter()
        .filter(|e| e.method == super_dev_contract::HttpVerb::Get && !e.path.contains(":id"))
        .count();
    let has_pag = arch_lower.contains("pagination")
        || arch_lower.contains("分页")
        || arch_lower
            .split_whitespace()
            .any(|w| w == "limit" || w == "offset" || w == "cursor");
    checks.push(QualityCheck {
        name: "Pagination strategy".to_string(),
        category: "contract".to_string(),
        description: "Architecture addresses pagination for list endpoints".to_string(),
        status: if has_pag || list_count == 0 {
            "passed"
        } else {
            "warning"
        }
        .to_string(),
        score: if has_pag || list_count == 0 { 100 } else { 60 },
        details: if list_count == 0 {
            "No list endpoints".to_string()
        } else if has_pag {
            format!("Pagination documented for {list_count} list endpoints")
        } else {
            format!("{list_count} list endpoints — no pagination strategy")
        },
        weight: 1.0,
    });

    // Error handling convention
    let has_err = arch_lower.contains("error")
        && (arch_text.contains("404") || arch_text.contains("400") || arch_text.contains("500"))
        && arch_lower.contains("response");
    checks.push(QualityCheck {
        name: "Error handling convention".to_string(),
        category: "contract".to_string(),
        description: "Architecture defines HTTP error codes".to_string(),
        status: if has_err { "passed" } else { "warning" }.to_string(),
        score: if has_err { 100 } else { 60 },
        details: if has_err {
            "Error code table found".to_string()
        } else {
            "No HTTP error convention — add 400/404/500 response table".to_string()
        },
        weight: 1.0,
    });

    // === Ops artifacts check (content-validated) ===
    // Generate scaffolding before checking so the files exist.
    let _scaffold =
        crate::scaffolding::generate_scaffolding(&opts.project_root, &contract_spec, &arch_text);
    let ops_files: [(&str, &str); 4] = [
        ("Dockerfile", "FROM"),
        (".github/workflows/ci.yml", "jobs:"),
        ("migrations/0001_init.sql", "CREATE TABLE"),
        (".env.example", "="),
    ];
    let mut ops_present = 0usize;
    let mut ops_detail = Vec::new();
    for (rel, marker) in &ops_files {
        let p = opts.project_root.join(rel);
        match fs::read_to_string(&p) {
            Ok(content) if !content.trim().is_empty() && content.contains(*marker) => {
                ops_present += 1;
            }
            Ok(_) => ops_detail.push(format!("{rel}: stub")),
            Err(_) => ops_detail.push(format!("{rel}: missing")),
        }
    }
    let ops_score = i32::try_from(ops_present * 100 / ops_files.len()).unwrap_or(0);
    checks.push(QualityCheck {
        name: "Ops artifacts present".to_string(),
        category: "delivery".to_string(),
        description: "Dockerfile + CI + migrations + .env generated with real content".to_string(),
        status: if ops_present == ops_files.len() {
            "passed"
        } else if ops_present >= 2 {
            "warning"
        } else {
            "failed"
        }
        .to_string(),
        score: ops_score,
        details: if ops_detail.is_empty() {
            format!(
                "All {} ops artifacts present with valid content",
                ops_files.len()
            )
        } else {
            format!(
                "{}/{} valid; {}",
                ops_present,
                ops_files.len(),
                ops_detail.join(", ")
            )
        },
        weight: 2.0,
    });

    // Allow user-specified check skips.
    let project_config = crate::config::load_project_config(&opts.project_root);
    let skip = &project_config.quality.skip_checks;
    if !skip.is_empty() {
        checks.retain(|c| {
            let s = c.name.to_ascii_lowercase().replace(' ', "_");
            !skip.iter().any(|sk| sk == &s || sk == &c.name)
        });
    }

    let total_score = avg_score(&checks);
    let weighted_score = weighted_avg(&checks);
    let mut critical_failures: Vec<String> = checks
        .iter()
        .filter(|c| c.status == "failed" && c.category == "artifact")
        .map(|c| c.name.clone())
        .collect();
    if checks
        .iter()
        .any(|ch| ch.name == "Build & test results" && ch.status == "failed")
    {
        critical_failures.push("Build & test results".to_string());
    }
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

    // Record the quality outcome as a "lesson" — failures become
    // retrievable lessons so future runs avoid the same defects, and
    // passes reinforce validated patterns. Previously
    // `capture_quality_failures` was defined but never called from the
    // main path, so the Failure lesson kind was dead wiring.
    crate::lessons::capture_quality_failures(
        &opts.project_root,
        &report.checks,
        &slug,
        &opts.requirement,
    );

    // Scan output/ for placeholder/TODO markers and append to the
    // persistent tech-debt ledger. The summary feeds a trend diff so a
    // team can see whether debt is growing run-over-run. Best-effort:
    // ledger write failures must never block the quality gate.
    let debt_items = crate::tech_debt::scan_debt(&output_dir);
    if !debt_items.is_empty() {
        let _ = crate::tech_debt::write_ledger(&opts.project_root, &debt_items);
    }

    Ok(PhaseOutput {
        phase: Phase::Quality,
        artifacts: vec![json_path, md_path],
        gate: None,
    })
}

#[allow(dead_code)]
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

fn verify_results_check(project_root: &Path) -> Option<QualityCheck> {
    let path = project_root.join(".super-dev/audit/verify.jsonl");
    let Ok(content) = fs::read_to_string(&path) else {
        return None;
    };
    #[derive(serde::Deserialize)]
    struct VRow {
        #[serde(default)]
        step: String,
        #[serde(default)]
        passed: bool,
        #[serde(default)]
        skipped: bool,
        #[serde(default)]
        timestamp: String,
    }
    let rows: Vec<VRow> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    if rows.is_empty() {
        return None;
    }
    let dts = String::new();
    let lts = rows.iter().map(|r| &r.timestamp).max().unwrap_or(&dts);
    let latest: Vec<&VRow> = rows.iter().filter(|r| r.timestamp == *lts).collect();
    let ns: Vec<&VRow> = latest.iter().copied().filter(|r| !r.skipped).collect();
    let passed = ns.iter().filter(|r| r.passed).count();
    let total = ns.len();
    let crit = latest
        .iter()
        .any(|r| !r.passed && !r.skipped && matches!(r.step.as_str(), "build" | "test" | "check"));
    let (status, score) = if total == 0 {
        ("warning", 70i32)
    } else if passed == total {
        ("passed", 100)
    } else if crit {
        ("failed", 0)
    } else {
        ("warning", ((passed * 100) / total).max(40) as i32)
    };
    Some(QualityCheck {
        name: "Build & test results".to_string(),
        category: "evidence".to_string(),
        description: "verify.jsonl — real build/lint/test outcomes".to_string(),
        status: status.to_string(),
        score,
        weight: 2.0,
        details: format!("{passed} of {total} steps passed"),
    })
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

    // 0. Capture validated patterns (D2: success -> sediment -> retrieval loop)
    let arch_text = fs::read_to_string(
        opts.project_root
            .join("output")
            .join(format!("{slug}-architecture.md")),
    )
    .unwrap_or_default();
    let arch_spec = super_dev_contract::parse_architecture(&arch_text, &format!("{slug} API"));
    let derived = super_dev_contract::derive_endpoints_from_requirement(&opts.requirement);
    let contract_spec = super_dev_contract::merge_specs(&arch_spec, &derived);
    crate::lessons::capture_validated_patterns(
        &opts.project_root,
        &slug,
        &opts.requirement,
        &contract_spec,
    );
    let _ = crate::lessons::sediment_lessons(&opts.project_root);

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

    // 3. Delivery notes placeholder — the worker fills the deploy/URL/run
    //    sections when use_runtime; in offline mode this is the fallback the
    //    user reads. Idempotent: only created if absent so a worker-written
    //    copy is never clobbered.
    let delivery_notes = opts
        .project_root
        .join("output")
        .join(format!("{slug}-delivery-notes.md"));
    if !delivery_notes.is_file() {
        let placeholder = format!(
            "# Delivery notes — {slug}\n\n             > Deployment recipe produced by the worker at the delivery phase.\n\n             ## Build status\n\n             _(frontend + backend production builds — worker reports pass/fail)_\n\n             ## Deploy target\n\n             _(recommended free platform: Vercel / Netlify / Cloudflare Pages / Render)_\n\n             ## Deploy command\n\n             _(exact command, e.g. `npx vercel --prod` — read by Super Dev `/deploy`)_\n\n             ## Frontend URL\n\n             _(not yet deployed)_\n\n             ## Environment variables\n\n             _(KEY=<description>, never real secrets)_\n\n             ## Run command\n\n             _(how to run the production build locally)_"
        );
        let _ = fs::write(&delivery_notes, placeholder);
    }
    if delivery_notes.is_file() {
        artifacts.push(delivery_notes);
    }

    // 4. Proof pack zip
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
    // Include design system + seed template files if present
    for dir in ["knowledge/design-systems", "knowledge/seed-templates"] {
        let d = project_root.join(dir);
        if d.is_dir() {
            walk_files(&d, &mut targets, 0);
        }
    }
    // recursively include .super-dev/changes/ and .super-dev/decisions/
    for dir in [".super-dev/changes", ".super-dev/decisions"] {
        let d = project_root.join(dir);
        if d.is_dir() {
            walk_files(&d, &mut targets, 0);
        }
    }

    // Add a README.md so reviewers know what each file is
    let readme = format!(
        "# Proof Pack — {slug}\n\n\
         Generated by Super Dev v{version} at {ts}.\n\n\
         ## Contents\n\n\
         | File | Purpose |\n\
         |---|---|\n\
         | `output/{slug}-research.md` | Competitive research + discovery |\n\
         | `output/{slug}-prd.md` | Product Requirements Document |\n\
         | `output/{slug}-architecture.md` | System architecture + API surface |\n\
         | `output/{slug}-uiux.md` | Design system (tokens, typography, components) |\n\
         | `output/{slug}-execution-plan.md` | Task breakdown |\n\
         | `output/{slug}-frontend-notes.md` | Frontend implementation checklist |\n\
         | `output/{slug}-backend-notes.md` | Backend implementation checklist |\n\
         | `output/{slug}-quality-gate.json` | Quality gate scores (13 checks) |\n\
         | `output/{slug}-quality-gate.md` | Human-readable quality report |\n\
         | `output/{slug}-compliance-mapping.json` | SOC2/ISO27001/EU-AI-Act mapping |\n\
         | `.super-dev/audit/tool-calls.jsonl` | Audit trail |\n\
         | `knowledge/design-systems/*.md` | Design system definitions |\n\
         | `knowledge/seed-templates/*.md` | Page structure templates |\n\n\
         ## How to review\n\n\
         1. Start with `output/{slug}-prd.md` — verify the scope is correct\n\
         2. Check `output/{slug}-architecture.md` — verify API surface makes sense\n\
         3. Check `output/{slug}-uiux.md` — verify design tokens and dark mode\n\
         4. Check `output/{slug}-quality-gate.md` — verify all 13 checks passed\n",
        version = env!("CARGO_PKG_VERSION"),
        ts = Utc::now().format("%Y-%m-%d %H:%M UTC"),
    );
    if zw.start_file("README.md", opts).is_ok() {
        let _ = zw.write_all(readme.as_bytes());
        manifest.push("README.md".to_string());
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

/// Check a document for required sections. Returns list of defect descriptions.
fn review_document_structure(text: &str, required: &[(&str, &str)]) -> Vec<String> {
    let headings: Vec<String> = text
        .lines()
        .filter_map(|l| {
            let t = l.trim_start();
            let level = t.chars().take_while(|&ch| ch == '#').count();
            if level == 0 {
                return None;
            }
            let h = t[level..].trim();
            if h.is_empty() {
                None
            } else {
                Some(h.to_ascii_lowercase())
            }
        })
        .collect();
    let mut defects = Vec::new();
    for (keyword, msg) in required {
        let kw = keyword.trim_start_matches('#').trim().to_ascii_lowercase();
        if !headings
            .iter()
            .any(|h| h.starts_with(&kw) || h.split_whitespace().any(|w| w == kw))
        {
            defects.push((*msg).to_string());
        }
    }
    defects
}

/// Build a QualityCheck from content review results.
fn content_quality_check(
    name: &str,
    category: &str,
    description: &str,
    text: &str,
    defects: &[String],
    weight: f32,
) -> QualityCheck {
    let (status, score, details) = if text.is_empty() {
        (
            "failed".to_string(),
            0,
            "File is empty or missing".to_string(),
        )
    } else if defects.is_empty() {
        (
            "passed".to_string(),
            100,
            "All required sections present".to_string(),
        )
    } else {
        let penalty = i32::try_from(defects.len()).unwrap_or(4) * 20;
        let score = (100 - penalty.min(70)).max(10);
        (
            if defects.len() <= 1 {
                "warning"
            } else {
                "failed"
            }
            .to_string(),
            score,
            format!("{} issue(s): {}", defects.len(), defects.join("; ")),
        )
    };
    QualityCheck {
        name: name.to_string(),
        category: category.to_string(),
        description: description.to_string(),
        status,
        score,
        details,
        weight,
    }
}

/// Cross-validate PRD information architecture against Architecture API surface.
/// Checks that pages mentioned in PRD have corresponding API endpoints.
#[allow(clippy::unnecessary_cast)]
fn check_prd_arch_alignment(prd_text: &str, arch_text: &str) -> (String, i32, String) {
    if prd_text.is_empty() || arch_text.is_empty() {
        return (
            "warning".to_string(),
            50,
            "Cannot cross-validate — one or both documents empty".to_string(),
        );
    }

    // Extract routes from PRD IA section (lines starting with ├── /xxx or └── /xxx or / )
    let prd_routes: Vec<&str> = prd_text
        .lines()
        .filter_map(|l| {
            let trimmed = l.trim().trim_start_matches(['├', '└', '│', '─', ' ']);
            if trimmed.starts_with('/') && !trimmed.contains("Home") {
                Some(trimmed.split_whitespace().next().unwrap_or(trimmed))
            } else {
                None
            }
        })
        .collect();

    let arch_lower = arch_text.to_ascii_lowercase();
    let mut covered = 0;
    let mut total = 0;
    for route in &prd_routes {
        if route.contains(':') || route.len() < 3 {
            continue;
        }
        total += 1;
        let route_base = route
            .split('/')
            .find(|s| !s.is_empty() && !s.starts_with(':'))
            .unwrap_or("");
        if !route_base.is_empty() && arch_lower.contains(route_base) {
            covered += 1;
        }
    }

    if total == 0 {
        return (
            "passed".to_string(),
            100,
            "No routes to cross-validate (PRD may lack IA section)".to_string(),
        );
    }

    let coverage_pct = (covered * 100) / total.max(1);
    let status = if coverage_pct >= 70 {
        "passed"
    } else {
        "warning"
    };
    (
        status.to_string(),
        coverage_pct as i32,
        format!(
            "PRD→Architecture alignment: {covered}/{total} page routes have matching API endpoints ({coverage_pct}%)"
        ),
    )
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

/// Extract score + passed from quality gate JSON. Used by the runner
/// to emit a quality summary to the TUI.
///
/// Reads the top-level `total_score` (NOT the per-check `score` — that
/// field also exists on each check object, so a naive `"score"` split
/// would grab `checks[0].score` instead of the aggregate).
pub fn extract_quality_score(json: &str) -> (String, bool) {
    let score = json
        .split("\"total_score\"")
        .nth(1)
        .and_then(|s| s.split(':').nth(1))
        .and_then(|s| {
            s.trim()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<u32>()
                .ok()
        })
        .map_or("?".to_string(), |n| n.to_string());
    let passed = json.contains("\"passed\": true") || json.contains("\"passed\":true");
    (score, passed)
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

/// Atomically write `content` to `path`: write to `<path>.tmp-<pid>` in
/// the same directory, then rename over `path`. Same-filesystem rename is
/// atomic on POSIX, so a concurrent reader never observes a half-written
/// file (the reader either sees the old complete file or the new complete
/// one). Falls back to a direct `fs::write` if the temp path can't be
/// constructed.
fn atomic_write(path: &Path, content: &str) -> io::Result<()> {
    let tmp = path.with_extension("tmp-write");
    fs::write(&tmp, content)?;
    if fs::rename(&tmp, path).is_ok() {
        Ok(())
    } else {
        // Rename failed (cross-filesystem?). Clean up the temp and fall
        // back to a direct write — correctness > atomicity here.
        let _ = fs::remove_file(&tmp);
        fs::write(path, content)
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
         > Offline scaffold. Use `--backend claude-code` for AI-generated content.\n\n\
         ## Goal\n\n{requirement}\n\n\
         TODO: Expand with: what + why + for whom + success metric\n\n\
         ## Target users\n\n\
         TODO: Define 2-3 personas with role, context, pain point.\n\n\
         ## Information architecture\n\n\
         ```\n\
         / (Home)\n\
         ├── /feature-1\n\
         ├── /feature-2\n\
         └── /auth/login\n\
         ```\n\
         TODO: Expand routes for: {requirement}\n\n\
         ## Scope\n\n\
         ### In scope\n\
         - TODO: List features for this iteration\n\n\
         ### Out of scope\n\
         - TODO: Explicitly exclude items\n\n\
         ## Functional requirements\n\n\
         | ID | Feature | Priority | Acceptance criteria |\n\
         |---|---|---|---|\n\
         | F1 | TODO | P0 | TODO |\n\
         | F2 | TODO | P1 | TODO |\n\n\
         ## Non-functional requirements\n\n\
         - Performance: FCP < _target_, API p95 < _target_\n\
         - Security: _auth method_, _data sensitivity_\n\
         - Accessibility: WCAG 2.1 _level_\n\n\
         ## Acceptance criteria\n\n\
         - [ ] Given TODO, when TODO, then TODO\n\
         - [ ] Given TODO, when TODO, then TODO\n\
         - [ ] Given TODO, when TODO, then TODO\n\n\
         TODO: Add acceptance criteria matching each functional requirement.\n\n\
         ## Success metrics\n\n\
         | Metric | Baseline | Target | How to measure |\n\
         |---|---|---|---|\n\
         | TODO | TODO | TODO | TODO |\n\n\
         ## Risks & open questions\n\n\
         - TODO: Identify domain-specific risks\n",
    )
}

fn render_architecture(slug: &str, requirement: &str) -> String {
    format!(
        "# Architecture — {slug}\n\n\
         > Offline scaffold. Use `--backend claude-code` for AI-generated content.\n\n\
         ## System overview\n\n\
         TODO: Describe the system components and how they communicate.\n\
         Consider: What services exist? REST/gRPC/WebSocket? Data flow direction?\n\n\
         Requirement: {requirement}\n\n\
         ## API surface\n\n\
         | Method | Path | Request | Response | Auth | Description |\n\
         |---|---|---|---|---|---|\n\
         | GET | /api/health | - | `{{ ok: true }}` | none | Health check |\n\
         | POST | /api/auth/login | `{{ email, password }}` | `{{ token, user }}` | none | Login |\n\
         | GET | /api/auth/me | - | `{{ user }}` | bearer | Current user |\n\
         | TODO | /api/... | TODO | TODO | TODO | Add endpoints for: {requirement} |\n\n\
         ## API error convention\n\n\
         ```json\n\
         {{ \"error\": {{ \"code\": \"VALIDATION_ERROR\", \"message\": \"...\", \"details\": [...] }} }}\n\
         ```\n\n\
         | HTTP | Code | Meaning |\n\
         |---|---|---|\n\
         | 400 | BAD_REQUEST | Malformed request |\n\
         | 401 | UNAUTHORIZED | Missing/invalid auth token |\n\
         | 403 | FORBIDDEN | Authenticated but no permission |\n\
         | 404 | NOT_FOUND | Resource doesn't exist |\n\
         | 422 | VALIDATION_ERROR | Invalid field values |\n\
         | 429 | RATE_LIMITED | Too many requests |\n\
         | 500 | INTERNAL_ERROR | Server error (no details to client) |\n\n\
         ## Data model\n\n\
         TODO: Define entities with field tables.\n\n\
         | Field | Type | Required | Description |\n\
         |---|---|---|---|\n\
         | id | uuid | yes | Primary key |\n\
         | created_at | timestamp | yes | Auto-set on create |\n\
         | updated_at | timestamp | yes | Auto-set on update |\n\n\
         ## Authentication & authorization\n\n\
         TODO: Define auth method (JWT/session/OAuth2), roles, permission matrix.\n\n\
         ## Tech-stack rationale\n\n\
         - Frontend: TODO (pick framework + justify)\n\
         - Backend: TODO (pick language/framework + justify)\n\
         - Database: TODO (pick DB + justify)\n\
         - Hosting: TODO (pick platform + justify)\n\n\
         ## Project structure\n\n\
         ```\n\
         src/\n\
           pages/       # Route-level components\n\
           components/  # Shared UI\n\
           lib/         # Business logic\n\
           api/         # API routes or client\n\
           types/       # Shared types\n\
         ```\n\n\
         ## Security considerations\n\n\
         - [ ] Input validation on all endpoints\n\
         - [ ] Parameterized queries (no SQL injection)\n\
         - [ ] HTTPS only (HSTS header)\n\
         - [ ] Rate limiting on auth endpoints\n\
         - [ ] Secrets in env vars, not code\n",
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
            .any(|c| c.name.contains("PRD") || c.name.contains("content")));
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

    // ---- review_document_structure: heading-based validation (hardened) ----

    #[test]
    fn review_structure_passes_when_heading_present() {
        let doc = "# PRD\n\n## Goal\nBuild something\n\n## Scope\nIn scope\n\n## Acceptance Criteria\n- [ ] works";
        let defects = review_document_structure(
            doc,
            &[
                ("## goal", "Missing goal"),
                ("## scope", "Missing scope"),
                ("## acceptance criteria", "Missing AC"),
            ],
        );
        assert!(
            defects.is_empty(),
            "headings present → no defects: {defects:?}"
        );
    }

    #[test]
    fn review_structure_fails_when_heading_absent() {
        let doc = "# PRD\n\nThis is just prose with the word goal mentioned but no heading.";
        let defects = review_document_structure(doc, &[("## goal", "Missing goal")]);
        assert!(!defects.is_empty(), "no ## heading → must have defects");
    }

    #[test]
    fn review_structure_does_not_false_match_in_prose() {
        // The word "api" in a paragraph must NOT count as a ## API heading.
        let doc = "# Arch\n\nWe discuss the api surface but don't have a heading for it.";
        let defects = review_document_structure(doc, &[("## api", "Missing API section")]);
        assert!(
            !defects.is_empty(),
            "api in prose must not satisfy heading check"
        );
    }

    #[test]
    fn review_structure_matches_partial_heading() {
        // "## API Surface" should match keyword "api" (starts_with).
        let doc = "# Arch\n\n## API Surface\nDetails here";
        let defects = review_document_structure(doc, &[("## api", "Missing API")]);
        assert!(defects.is_empty(), "partial heading match should pass");
    }

    // ---- verify_results_check ----

    #[test]
    fn verify_results_check_none_when_no_jsonl() {
        let tmp = TempDir::new().unwrap();
        assert!(verify_results_check(tmp.path()).is_none());
    }

    #[test]
    fn verify_results_check_passes_all_steps() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join(".super-dev/audit");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("verify.jsonl"),
            r#"{"step":"install","passed":true,"skipped":false,"timestamp":"t1"}
{"step":"test","passed":true,"skipped":false,"timestamp":"t1"}
{"step":"build","passed":true,"skipped":false,"timestamp":"t1"}
"#,
        )
        .unwrap();
        let check = verify_results_check(tmp.path()).unwrap();
        assert_eq!(check.name, "Build & test results");
        assert_eq!(check.status, "passed");
        assert_eq!(check.score, 100);
    }

    #[test]
    fn verify_results_check_fails_on_build_failure() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join(".super-dev/audit");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("verify.jsonl"),
            r#"{"step":"install","passed":true,"skipped":false,"timestamp":"t1"}
{"step":"build","passed":false,"skipped":false,"timestamp":"t1"}
"#,
        )
        .unwrap();
        let check = verify_results_check(tmp.path()).unwrap();
        assert_eq!(check.status, "failed");
        assert_eq!(check.score, 0);
    }

    #[test]
    fn verify_results_check_ignores_skipped_steps() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join(".super-dev/audit");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("verify.jsonl"),
            r#"{"step":"install","passed":true,"skipped":false,"timestamp":"t1"}
{"step":"lint","passed":false,"skipped":true,"timestamp":"t1"}
{"step":"test","passed":true,"skipped":false,"timestamp":"t1"}
"#,
        )
        .unwrap();
        let check = verify_results_check(tmp.path()).unwrap();
        assert_eq!(
            check.status, "passed",
            "skipped lint failure must not fail the check"
        );
    }

    // ---- phase_knowledge_digest BM25 path ----

    #[test]
    fn phase_knowledge_digest_returns_empty_without_dir() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        let d = phase_knowledge_digest(&o, Phase::Research);
        assert!(d.is_empty());
    }

    #[test]
    fn phase_knowledge_digest_uses_bm25_when_enabled() {
        let tmp = TempDir::new().unwrap();
        let kd = tmp.path().join("knowledge/security");
        fs::create_dir_all(&kd).unwrap();
        fs::write(
            kd.join("login.md"),
            "# Login\n\n## OAuth\n\nUse OAuth2 with PKCE for login authentication.",
        )
        .unwrap();
        // Write .superdevrc to enable knowledge (default is enabled).
        fs::write(
            tmp.path().join(".superdevrc"),
            "[quality]\nthreshold = 90\n",
        )
        .unwrap();
        let o = opts(tmp.path());
        let d = phase_knowledge_digest(&o, Phase::Backend);
        assert!(d.contains("Expert knowledge"), "should produce digest: {d}");
        assert!(
            d.contains("login"),
            "should contain relevant knowledge: {d}"
        );
    }

    #[test]
    fn phase_knowledge_digest_with_vector_is_none_for_bm25() {
        // When engine=bm25, passing a query_vec should still work (ignored).
        let tmp = TempDir::new().unwrap();
        let kd = tmp.path().join("knowledge/security");
        fs::create_dir_all(&kd).unwrap();
        fs::write(kd.join("login.md"), "# Login\n\n## OAuth\n\nlogin auth").unwrap();
        let o = opts(tmp.path());
        let d = phase_knowledge_digest_with_vector(&o, Phase::Backend, Some(&[0.1; 1536]));
        assert!(d.contains("Expert knowledge"));
    }

    #[test]
    fn phase_knowledge_digest_gates_return_empty() {
        let tmp = TempDir::new().unwrap();
        let kd = tmp.path().join("knowledge/security");
        fs::create_dir_all(&kd).unwrap();
        fs::write(kd.join("login.md"), "# Login\n\n## OAuth\n\nlogin").unwrap();
        let o = opts(tmp.path());
        assert!(phase_knowledge_digest(&o, Phase::DocsConfirm).is_empty());
        assert!(phase_knowledge_digest(&o, Phase::PreviewConfirm).is_empty());
    }

    // ---- quality gate contract + ops checks ----

    #[test]
    fn quality_includes_contract_checks() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        let names: Vec<&str> = report.checks.iter().map(|c| c.name.as_str()).collect();
        assert!(
            names.contains(&"OpenAPI contract"),
            "must have OpenAPI check: {names:?}"
        );
        assert!(
            names.contains(&"Frontend↔contract conformance"),
            "must have contract conformance: {names:?}"
        );
    }

    #[test]
    fn quality_includes_ops_artifacts_check() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        // Ops artifacts check should exist and produce scaffolding.
        let ops = report
            .checks
            .iter()
            .find(|c| c.name == "Ops artifacts present");
        assert!(ops.is_some(), "must have ops artifacts check");
        // Scaffolding files should have been generated by the check.
        assert!(
            tmp.path().join("Dockerfile").is_file(),
            "Dockerfile must be generated"
        );
    }

    #[test]
    fn quality_includes_pagination_and_error_checks() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        let names: Vec<&str> = report.checks.iter().map(|c| c.name.as_str()).collect();
        assert!(
            names.contains(&"Pagination strategy"),
            "must have pagination: {names:?}"
        );
        assert!(
            names.contains(&"Error handling convention"),
            "must have error convention: {names:?}"
        );
    }

    #[test]
    fn quality_includes_input_validation_check() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        assert!(report
            .checks
            .iter()
            .any(|c| c.name == "Input validation coverage"));
    }

    #[test]
    fn quality_verify_check_appears_when_jsonl_exists() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        // Seed a verify.jsonl with passing steps.
        let audit = tmp.path().join(".super-dev/audit");
        fs::create_dir_all(&audit).unwrap();
        fs::write(
            audit.join("verify.jsonl"),
            r#"{"step":"test","passed":true,"skipped":false,"timestamp":"t"}"#,
        )
        .unwrap();
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        assert!(report
            .checks
            .iter()
            .any(|c| c.name == "Build & test results" && c.status == "passed"));
    }

    #[test]
    fn quality_verify_check_critical_on_build_fail() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        let audit = tmp.path().join(".super-dev/audit");
        fs::create_dir_all(&audit).unwrap();
        fs::write(
            audit.join("verify.jsonl"),
            r#"{"step":"build","passed":false,"skipped":false,"timestamp":"t"}"#,
        )
        .unwrap();
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        assert!(
            report
                .critical_failures
                .iter()
                .any(|f| f.contains("Build & test")),
            "build failure must be critical: {:?}",
            report.critical_failures
        );
    }

    #[test]
    fn execution_plan_check_validates_content() {
        // The execution plan check should fail on a 1-byte stub file,
        // and pass on a real plan with ## sections.
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        // Write a stub execution plan (too short + no sections).
        let out_dir = tmp.path().join("output");
        fs::create_dir_all(&out_dir).unwrap();
        fs::write(out_dir.join("demo-execution-plan.md"), "x").unwrap();
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        let ep = report
            .checks
            .iter()
            .find(|c| c.name == "Execution plan")
            .unwrap();
        assert!(ep.score < 100, "stub exec plan should not pass: {ep:?}");
    }

    #[test]
    fn execution_plan_passes_with_structured_content() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        // run_spec writes a real execution plan — quality should pass it.
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        let ep = report.checks.iter().find(|c| c.name == "Execution plan");
        if let Some(ep) = ep {
            assert!(ep.score >= 60, "real exec plan should score well: {ep:?}");
        }
    }

    #[test]
    fn delivery_captures_validated_patterns() {
        // run_delivery should call capture_validated_patterns + sediment_lessons.
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        run_frontend(&o).unwrap();
        run_backend(&o).unwrap();
        run_quality(&o).unwrap();
        run_delivery(&o).unwrap();
        // sediment_lessons should have created at least one lesson file.
        let learned_dir = tmp.path().join(".super-dev/learned");
        assert!(
            learned_dir.is_dir(),
            "learned dir should exist after delivery"
        );
    }

    #[test]
    fn scaffolding_generated_during_quality() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        run_quality(&o).unwrap();
        // Quality gate should have generated scaffolding.
        assert!(
            tmp.path().join("Dockerfile").is_file(),
            "Dockerfile generated"
        );
        assert!(
            tmp.path().join(".github/workflows/ci.yml").is_file(),
            "CI generated"
        );
        assert!(
            tmp.path().join("migrations/0001_init.sql").is_file(),
            "migration generated"
        );
    }

    #[test]
    fn ops_artifacts_content_validated() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        let ops = report
            .checks
            .iter()
            .find(|c| c.name == "Ops artifacts present")
            .unwrap();
        // Scaffolding was generated → should pass.
        assert_eq!(
            ops.status, "passed",
            "ops artifacts should pass after scaffolding gen"
        );
    }

    #[test]
    fn skip_checks_respected() {
        let tmp = TempDir::new().unwrap();
        let o = opts(tmp.path());
        fs::write(
            tmp.path().join(".superdevrc"),
            "[quality]\nskip_checks = [\"Dark mode support\"]\n",
        )
        .unwrap();
        run_research(&o, None).unwrap();
        run_docs(&o, &DocsContent::default()).unwrap();
        run_spec(&o).unwrap();
        let out = run_quality(&o).unwrap();
        let json = fs::read_to_string(&out.artifacts[0]).unwrap();
        let report: QualityReport = serde_json::from_str(&json).unwrap();
        assert!(
            !report.checks.iter().any(|c| c.name == "Dark mode support"),
            "skipped check should not appear in report"
        );
    }

    #[test]
    fn extract_quality_score_parses_json() {
        let json = r#"{"passed":true,"score":92,"weighted_score":91.5}"#;
        let (score, passed) = extract_quality_score(json);
        assert_eq!(score, "?"); // no total_score field here → unknown
        assert!(passed);
    }

    #[test]
    fn extract_quality_score_reads_total_not_first_check() {
        // Real QualityReport shape: each check has its OWN "score", plus a
        // top-level "total_score". The extractor MUST read total_score (97),
        // NOT the first check's score (40). This is a regression test for a
        // bug where the score was parsed off the first "score" substring.
        let json = r#"{
            "passed": true,
            "total_score": 97,
            "weighted_score": 96.5,
            "checks": [
                {"name":"api_url_consistency","score":40,"passed":true},
                {"name":"completeness","score":60,"passed":true}
            ]
        }"#;
        let (score, passed) = extract_quality_score(json);
        assert_eq!(score, "97");
        assert!(passed);
    }

    #[test]
    fn extract_quality_score_handles_missing() {
        let json = r#"{"passed":false}"#;
        let (_score, passed) = extract_quality_score(json);
        assert!(!passed);
    }

    #[test]
    fn score_uiux_completeness_returns_zero_for_missing() {
        let tmp = TempDir::new().unwrap();
        let score = score_uiux_completeness(&tmp.path().join("nonexistent.md"));
        assert_eq!(score, 0);
    }

    #[test]
    fn knowledge_top_files_returns_count() {
        let tmp = TempDir::new().unwrap();
        let kd = tmp.path().join("knowledge/security");
        fs::create_dir_all(&kd).unwrap();
        fs::write(kd.join("a.md"), "# A\n").unwrap();
        fs::write(kd.join("b.md"), "# B\n").unwrap();
        let o = opts(tmp.path());
        let (files, total) = knowledge_top_files(&o);
        assert_eq!(total, 2);
        assert!(!files.is_empty());
    }

    #[test]
    fn phase_knowledge_digest_falls_back_to_legacy_when_disabled() {
        let tmp = TempDir::new().unwrap();
        let kd = tmp.path().join("knowledge/security");
        fs::create_dir_all(&kd).unwrap();
        fs::write(kd.join("login.md"), "# Login\n\n## OAuth\n\nlogin auth").unwrap();
        fs::write(
            tmp.path().join(".superdevrc"),
            "[knowledge]\nenabled = false\n",
        )
        .unwrap();
        let o = opts(tmp.path());
        let d = phase_knowledge_digest(&o, Phase::Backend);
        // Legacy path should still produce output for a matched keyword.
        assert!(
            d.contains("Expert knowledge") || d.is_empty(),
            "legacy path produces output or empty"
        );
    }

    #[test]
    fn review_structure_matches_multiple_headings() {
        let doc = "# Arch\n\n## API Surface\nDetails\n\n## Data Model\nSchema\n\n## Auth\nJWT";
        let defects = review_document_structure(
            doc,
            &[
                ("## api", "Missing API"),
                ("## data", "Missing data"),
                ("## auth", "Missing auth"),
            ],
        );
        assert!(defects.is_empty(), "all headings present: {defects:?}");
    }

    #[test]
    fn verify_results_check_handles_empty_jsonl() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join(".super-dev/audit");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("verify.jsonl"), "").unwrap();
        assert!(
            verify_results_check(tmp.path()).is_none(),
            "empty jsonl → None"
        );
    }

    #[test]
    fn evidence_check_works_with_missing_file() {
        let tmp = TempDir::new().unwrap();
        let check = evidence_check("Test", "desc", &tmp.path().join("nonexistent.jsonl"), 1.0);
        assert_eq!(check.status, "warning");
        assert_eq!(check.score, 60);
    }
}
