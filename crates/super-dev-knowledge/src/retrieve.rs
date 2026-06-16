//! Unified retrieval entry point — picks the configured engine and returns
//! ranked chunks ready for prompt formatting or TUI display.
//!
//! This is the single function the agent crate calls, replacing the old
//! `phase_knowledge_digest` / `knowledge_top_files` internals. It decides:
//! 1. Which `knowledge/` subdirs are relevant for the current phase.
//! 2. Whether to use BM25 (default) or hybrid BM25+vector (when
//!    `OPENAI_EMBED_KEY` is set).
//! 3. RRF (Reciprocal Rank Fusion) to merge the two rankings when hybrid.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super_dev_spec::Phase;

use crate::chunker::Chunk;
use crate::index::{load_or_build_index_multi, Bm25Index};
use crate::vector;

/// Cross-platform home directory: `HOME` then `USERPROFILE` (Windows).
/// Returns None when neither is set (fail-open). Previously only `HOME`
/// was checked, which is usually unset on Windows.
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

/// A retrieval hit: the chunk + a normalised 0..1 score.
#[derive(Debug, Clone)]
pub struct ScoredChunk {
    /// The matched chunk.
    pub chunk: Chunk,
    /// Normalised score in 0.0..=1.0 (1.0 = best match in this query).
    pub score: f32,
}

/// Which retrieval engine to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum RetrievalEngine {
    /// BM25 keyword retrieval only (default, offline, zero-dep).
    #[default]
    Bm25,
    /// BM25 + optional vector fusion. Vector results only contribute when
    /// `OPENAI_EMBED_KEY` is set; otherwise behaves identically to Bm25.
    Hybrid,
}

/// Per-project retrieval configuration (mirrors `[knowledge]` in .superdevrc).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct RetrievalConfig {
    /// Whether the knowledge base is enabled at all.
    pub enabled: bool,
    /// Which engine to use.
    pub engine: RetrievalEngine,
    /// How many chunks to return per query.
    pub top_k: usize,
    /// Extra directories (relative to project root) to include alongside
    /// the built-in `knowledge/` tree.
    pub custom_dirs: Vec<String>,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            engine: RetrievalEngine::default(),
            top_k: 6,
            custom_dirs: Vec::new(),
        }
    }
}

/// Map a pipeline phase to the knowledge subdirectories most relevant to
/// it. Mirrors the legacy `phase_knowledge_digest` mapping (phases.rs:64)
/// so phase-aware filtering behaviour is preserved.
///
/// **These are Super Dev's built-in business assumptions** about which
/// knowledge folders each pipeline phase should consult (e.g. Docs reads
/// `experts/product-manager` + `experts/architect` + `experts/uiux-designer`).
/// They encode the default knowledge-base layout shipped with Super Dev.
/// Teams whose `knowledge/` tree uses different directory names have two
/// non-fork escape hatches:
/// - per-phase override via `SUPER_DEV_KNOWLEDGE_PHASE_SUBDIRS` (full
///   replacement for a specific phase), and/or
/// - global extras via `SUPER_DEV_KNOWLEDGE_EXTRA_SUBDIRS` (appended to
///   every phase).
///
/// If a phase filter finds nothing, `filter_by_phase` warns + falls back to
/// unfiltered top-k so the prompt still gets context.
#[must_use]
pub fn phase_subdirs(phase: Phase) -> &'static [&'static str] {
    match phase {
        Phase::Research => &[], // research scans the whole tree
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
        Phase::DocsConfirm | Phase::PreviewConfirm => &[],
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
            "performance",
            "observability",
            "00-governance",
        ],
        Phase::Delivery => &[
            "experts/devops",
            "cicd",
            "operations",
            "release-engineering",
            "compliance",
            "00-governance",
            "security",
        ],
    }
}

/// Run a retrieval query against the project's knowledge base.
///
/// Builds (or loads the cached) BM25 index, optionally queries the vector
/// store, fuses the rankings via RRF, and returns the top-K chunks with
/// normalised scores.
///
/// `project_root` is the workspace root (where `.super-dev/` lives).
/// `knowledge_dir` is the `knowledge/` directory (usually `project_root/knowledge`).
///
/// Returns an empty vec when disabled, the index is empty, or the query
/// yields no matches — never errors (fail-open).
#[must_use]
pub fn retrieve(
    project_root: &Path,
    knowledge_dir: &Path,
    config: &RetrievalConfig,
    query: &str,
    phase: Phase,
) -> Vec<ScoredChunk> {
    retrieve_with_vector(project_root, knowledge_dir, config, query, phase, None)
}

/// Retrieval with a pre-embedded query vector. This is the **real hybrid
/// path**: when `query_vec` is `Some` AND the vector store is populated,
/// BM25 and vector rankings are fused via RRF (k=60). When `query_vec` is
/// `None` or the store is empty, this is identical to pure BM25.
///
/// The query vector must be obtained **asynchronously** (via
/// [`vector::embed_query`]) by the caller — typically the async runner
/// pre-embeds the requirement once, then passes `Some(&qvec)` into the
/// sync render functions. This keeps the network call isolated to the
/// runner seam and fail-open (a `None` vector just means BM25 only).
#[must_use]
pub fn retrieve_with_vector(
    project_root: &Path,
    knowledge_dir: &Path,
    config: &RetrievalConfig,
    query: &str,
    phase: Phase,
    query_vec: Option<&[f32]>,
) -> Vec<ScoredChunk> {
    if !config.enabled || query.trim().is_empty() {
        return Vec::new();
    }

    // Build / load the BM25 index over knowledge/ + any learned dirs.
    // Learned dirs (.super-dev/learned/ and ~/.super-dev/learned/) hold
    // auto-sedimented experience from prior runs.
    let mut dirs = vec![knowledge_dir.to_path_buf()];
    let project_learned = project_root.join(".super-dev/learned");
    if project_learned.is_dir() {
        dirs.push(project_learned);
    }
    if let Some(home) = home_dir() {
        let global_learned = home.join(".super-dev/learned");
        if global_learned.is_dir() {
            dirs.push(global_learned);
        }
    }
    let index = load_or_build_index_multi(project_root, &dirs);
    if index.chunks.is_empty() {
        return Vec::new();
    }

    // BM25 results over the full index (over-fetch so RRF has room).
    let bm25_raw = index.search(query, config.top_k * 3);
    let bm25_hits = filter_by_phase(&index, &bm25_raw, phase, config.top_k);

    // Vector fusion only when: hybrid engine, vector layer enabled, a query
    // vector was provided, AND the store actually has vectors.
    let use_vector =
        config.engine == RetrievalEngine::Hybrid && vector::is_enabled() && query_vec.is_some();
    if !use_vector {
        return normalise(&index, bm25_hits);
    }
    let query_vec = query_vec.unwrap_or(&[]);

    let store = vector::VectorStore::load(project_root);
    if store.is_empty() {
        return normalise(&index, bm25_hits);
    }
    let vec_hits = store.search(query_vec, config.top_k * 3);
    if vec_hits.is_empty() {
        return normalise(&index, bm25_hits);
    }

    // Real RRF fusion: merge the two ranked lists.
    let fused = rrf_fuse(&index, &bm25_hits, &vec_hits, RRF_K, config.top_k);
    if fused.is_empty() {
        return normalise(&index, bm25_hits);
    }
    normalise(&index, fused)
}

/// Standard RRF constant. `k=60` is the value used by Elasticsearch and the
/// original Cormack et al. paper; it balances rank vs score contribution.
const RRF_K: u32 = 60;

/// Phase-aware retrieval — the most common entry point. Picks subdirs for
/// the phase, runs the query, returns chunks.
#[must_use]
pub fn retrieve_for_phase(
    project_root: &Path,
    knowledge_dir: &Path,
    config: &RetrievalConfig,
    query: &str,
    phase: Phase,
) -> Vec<ScoredChunk> {
    retrieve_with_vector(project_root, knowledge_dir, config, query, phase, None)
}

/// Phase-aware retrieval with a pre-embedded query vector (the hybrid path).
/// The async runner pre-embeds the requirement once, then calls this so
/// every phase gets true BM25+vector RRF fusion without re-embedding.
#[must_use]
pub fn retrieve_for_phase_with_vector(
    project_root: &Path,
    knowledge_dir: &Path,
    config: &RetrievalConfig,
    query: &str,
    phase: Phase,
    query_vec: Option<&[f32]>,
) -> Vec<ScoredChunk> {
    retrieve_with_vector(project_root, knowledge_dir, config, query, phase, query_vec)
}

/// Filter raw BM25 `(chunk_idx, score)` results to only chunks whose path
/// falls under a phase-relevant subdir, then take top_k.
fn filter_by_phase(
    index: &Bm25Index,
    raw: &[(usize, f64)],
    phase: Phase,
    top_k: usize,
) -> Vec<(usize, f64)> {
    // Phase subdirs: a per-phase OVERRIDE (SUPER_DEV_KNOWLEDGE_PHASE_SUBDIRS)
    // replaces the static default when present; otherwise use the static map.
    // Either way, SUPER_DEV_KNOWLEDGE_EXTRA_SUBDIRS is appended so a team can
    // both override specific phases AND add global extras.
    let extras: &[String] = extra_phase_subdirs();
    let base: Vec<&str> = match phase_subdirs_override(phase) {
        Some(override_dirs) => override_dirs.iter().map(String::as_str).collect(),
        None => phase_subdirs(phase).to_vec(),
    };
    let subdirs: Vec<&str> = base
        .into_iter()
        .chain(extras.iter().map(String::as_str))
        .collect();
    let subdirs: &[&str] = &subdirs;
    // Research scans the whole tree (empty subdirs = no filter).
    if subdirs.is_empty() || matches!(phase, Phase::Research) {
        return raw.iter().take(top_k).copied().collect();
    }
    let filtered: Vec<(usize, f64)> = raw
        .iter()
        .filter(|(idx, _)| {
            let path = &index.chunks[*idx].meta.path;
            // Always allow sedimented lessons through (they're cross-cutting
            // experience from prior runs). Lessons are pathed `<domain>/lesson-*`
            // after the .super-dev/learned/ prefix is stripped, so we detect
            // them by the `lesson-` filename marker.
            let is_lesson = path.contains("lesson-");
            // Also accept the legacy `learned/` prefix (defensive).
            is_lesson || path.starts_with("learned") || subdirs.iter().any(|s| path.starts_with(s))
        })
        .copied()
        .collect();
    // If phase-filtering wipes out everything, fall back to unfiltered top_k
    // so the prompt still gets context (better irrelevant than empty).
    if filtered.is_empty() && !raw.is_empty() {
        // Surface that phase-filtering found nothing — previously this was
        // completely silent, so a user whose `knowledge/` layout doesn't
        // match the hardcoded phase_subdirs had no way to know filtering
        // failed and they were getting unfiltered fallback results.
        eprintln!(
            "warn: knowledge phase-filter for `{phase:?}` matched 0 chunks (expected subdirs: {subdirs:?}); \
             falling back to unfiltered top-{top_k}. If your knowledge/ layout uses different \
             directory names, results may be less phase-relevant."
        );
        raw.iter().take(top_k).copied().collect()
    } else {
        filtered.into_iter().take(top_k).collect()
    }
}

/// Normalise raw BM25/RRF scores to 0.0..=1.0 (best = 1.0) and attach chunks.
/// Extra knowledge subdirs to include in phase filtering, parsed from the
/// `SUPER_DEV_KNOWLEDGE_EXTRA_SUBDIRS` env var (comma-separated). Cached for
/// the process lifetime. These are ADDED to every phase's static subdir list
/// so a custom knowledge/ layout can opt into filtering.
fn extra_phase_subdirs() -> &'static [String] {
    static CACHE: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| {
        std::env::var("SUPER_DEV_KNOWLEDGE_EXTRA_SUBDIRS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

/// Per-phase subdir OVERRIDE map parsed from
/// `SUPER_DEV_KNOWLEDGE_PHASE_SUBDIRS`. Format: `phase:dir1,dir2;phase2:dir3`
/// (semicolon-separated `phase:dirs` entries; dirs comma-separated). When a
/// phase has an entry here, it FULLY REPLACES the static default subdirs for
/// that phase (the extras still apply). Lets a team whose knowledge/ layout
/// diverges from the built-in map override specific phases without forking.
/// Returns `Some(dirs)` when an override exists for `phase`, else `None`.
fn phase_subdirs_override(phase: Phase) -> Option<&'static [String]> {
    static CACHE: std::sync::OnceLock<std::collections::HashMap<String, Vec<String>>> =
        std::sync::OnceLock::new();
    let map = CACHE.get_or_init(|| {
        let raw = std::env::var("SUPER_DEV_KNOWLEDGE_PHASE_SUBDIRS").unwrap_or_default();
        let mut m: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for entry in raw.split(';') {
            let entry = entry.trim();
            let Some((phase_part, dirs_part)) = entry.split_once(':') else {
                continue;
            };
            let dirs: Vec<String> = dirs_part
                .split(',')
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty())
                .collect();
            if !dirs.is_empty() {
                m.insert(phase_part.trim().to_ascii_lowercase(), dirs);
            }
        }
        m
    });
    map.get(phase.id()).map(Vec::as_slice)
}

/// The minimum normalised score (fraction of the top hit's score) a chunk
/// must reach to be kept. Default 0.5 — chunks scoring below 50% of the
/// best hit are treated as noise and dropped. Override with
/// `SUPER_DEV_KNOWLEDGE_MIN_SCORE` (0.0 = keep everything, 1.0 = only exact
/// top-score ties). Useful for weak-match-heavy queries (CJK bigram queries
/// match many chunks loosely) where 0.5 drops everything.
fn min_score_filter() -> f32 {
    std::env::var("SUPER_DEV_KNOWLEDGE_MIN_SCORE")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(0.5)
}

/// Applies a weak `quality_score` boost: a chunk with `quality_score: 95`
/// gets ~1.24× its normalised score (clamped to 1.0), so curated docs rank
/// slightly above equally-matching un-scored ones. Missing quality_score is
/// treated as 50 (neutral).
fn normalise(index: &Bm25Index, hits: Vec<(usize, f64)>) -> Vec<ScoredChunk> {
    if hits.is_empty() {
        return Vec::new();
    }
    let max = hits
        .iter()
        .map(|(_, s)| *s)
        .fold(0.0_f64, f64::max)
        .max(1e-9);
    let min_score = min_score_filter();
    hits.into_iter()
        .map(|(idx, score)| {
            let base = (score / max) as f32;
            let qs = index.chunks[idx].quality_score.unwrap_or(50).clamp(0, 100);
            // Weak boost: score × (1 + quality/200). quality=50 → ×1.0 (neutral),
            // quality=100 → ×1.5, quality=0 → ×0.5. Clamped to 1.0.
            let boosted = (base * (1.0 + qs as f32 / 200.0)).min(1.0);
            ScoredChunk {
                chunk: index.chunks[idx].clone(),
                score: boosted,
            }
        })
        .filter(|sc| sc.score >= min_score) // drop noise below the (configurable) threshold
        .collect()
}

/// Reciprocal Rank Fusion — merge BM25 and vector ranked lists by
/// `1/(k + rank)`. `k=60` is the standard RRF constant (Elasticsearch,
/// original Cormack et al. paper).
///
/// BM25 hits are addressed by chunk index; vector hits by `(path, section)`.
/// We build a `(path, section) → chunk_idx` map from the index to unify the
/// two address spaces, then fuse. A chunk appearing in both lists gets a
/// higher fused score (the whole point of hybrid retrieval). Returns chunk
/// indices ranked by fused score, truncated to `top_k`.
fn rrf_fuse(
    index: &Bm25Index,
    bm25: &[(usize, f64)],
    vector_hits: &[(&str, &str, f32)],
    k: u32,
    top_k: usize,
) -> Vec<(usize, f64)> {
    // Map (path\0section) → chunk_idx so vector hits can be normalised to
    // the same address space as BM25 hits.
    let mut key_to_idx: HashMap<String, usize> = HashMap::new();
    for (i, chunk) in index.chunks.iter().enumerate() {
        key_to_idx.insert(format!("{}\0{}", chunk.meta.path, chunk.meta.section), i);
    }

    let mut scores: HashMap<usize, f64> = HashMap::new();
    let kf = f64::from(k);

    // BM25 contribution: rank 0 is the top hit.
    for (rank, (chunk_idx, _)) in bm25.iter().enumerate() {
        *scores.entry(*chunk_idx).or_insert(0.0) += 1.0 / (kf + rank as f64 + 1.0);
    }

    // Vector contribution: rank 0 is the top hit. Only count hits that map
    // to a known chunk (drops stale vectors whose source chunk was removed).
    for (rank, (path, section, _)) in vector_hits.iter().enumerate() {
        let key = format!("{path}\0{section}");
        if let Some(&chunk_idx) = key_to_idx.get(&key) {
            *scores.entry(chunk_idx).or_insert(0.0) += 1.0 / (kf + rank as f64 + 1.0);
        }
    }

    let mut fused: Vec<(usize, f64)> = scores.into_iter().collect();
    fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    fused.truncate(top_k);
    fused
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn seed_corpus(root: &Path) -> PathBuf {
        let kd = root.join("knowledge");
        fs::create_dir_all(kd.join("security")).unwrap();
        fs::write(
            kd.join("security/login.md"),
            "# Login Playbook\n\n## OAuth\n\nUse OAuth2 with PKCE for login authentication.\n\n## Risks\n\nToken theft.",
        )
        .unwrap();
        fs::create_dir_all(kd.join("database")).unwrap();
        fs::write(
            kd.join("database/postgres.md"),
            "# Postgres\n\n## Tuning\n\nshared_buffers and work_mem tuning for the database.",
        )
        .unwrap();
        kd
    }

    #[test]
    fn retrieve_returns_relevant_chunk() {
        let tmp = tempfile::TempDir::new().unwrap();
        let kd = seed_corpus(tmp.path());
        let cfg = RetrievalConfig::default();
        let hits = retrieve(tmp.path(), &kd, &cfg, "login oauth", Phase::Research);
        assert!(!hits.is_empty());
        assert!(hits[0].chunk.meta.path.contains("login"));
        assert!(hits[0].score > 0.0);
    }

    #[test]
    fn cjk_query_retrieves_relevant_content() {
        let tmp = tempfile::TempDir::new().unwrap();
        let kd = tmp.path().join("knowledge");
        fs::create_dir_all(kd.join("security")).unwrap();
        fs::write(
            kd.join("security/login.md"),
            "# 登录系统\n\n## 流程\n\n使用 OAuth2 做登录认证",
        )
        .unwrap();
        let cfg = RetrievalConfig::default();
        let hits = retrieve(tmp.path(), &kd, &cfg, "做一个登录系统", Phase::Research);
        assert!(
            !hits.is_empty(),
            "CJK requirement must retrieve CJK content"
        );
    }

    #[test]
    fn disabled_config_returns_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let kd = seed_corpus(tmp.path());
        let cfg = RetrievalConfig {
            enabled: false,
            ..RetrievalConfig::default()
        };
        assert!(retrieve(tmp.path(), &kd, &cfg, "login", Phase::Research).is_empty());
    }

    #[test]
    fn empty_query_returns_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let kd = seed_corpus(tmp.path());
        let cfg = RetrievalConfig::default();
        assert!(retrieve(tmp.path(), &kd, &cfg, "   ", Phase::Research).is_empty());
    }

    #[test]
    fn phase_filter_narrows_to_subdirs() {
        let tmp = tempfile::TempDir::new().unwrap();
        let kd = seed_corpus(tmp.path());
        let cfg = RetrievalConfig::default();
        // Quality phase maps to testing/security/00-governance — should still
        // find the security/login.md doc.
        let hits = retrieve(tmp.path(), &kd, &cfg, "login", Phase::Quality);
        assert!(hits.iter().any(|h| h.chunk.meta.path.contains("security")));
    }

    #[test]
    fn phase_filter_falls_back_when_no_match() {
        let tmp = tempfile::TempDir::new().unwrap();
        let kd = seed_corpus(tmp.path());
        let cfg = RetrievalConfig::default();
        // Backend phase subdirs are backend/api/database/security — a query
        // matching only the postgres doc should still hit.
        let hits = retrieve(
            tmp.path(),
            &kd,
            &cfg,
            "postgres database tuning",
            Phase::Backend,
        );
        assert!(!hits.is_empty());
    }

    #[test]
    fn top_k_limits_results() {
        let tmp = tempfile::TempDir::new().unwrap();
        let kd = seed_corpus(tmp.path());
        let cfg = RetrievalConfig {
            top_k: 1,
            ..RetrievalConfig::default()
        };
        let hits = retrieve(tmp.path(), &kd, &cfg, "auth login", Phase::Research);
        assert!(hits.len() <= 1);
    }

    #[test]
    fn scores_normalised_to_top_is_one() {
        let tmp = tempfile::TempDir::new().unwrap();
        let kd = seed_corpus(tmp.path());
        let cfg = RetrievalConfig::default();
        let hits = retrieve(tmp.path(), &kd, &cfg, "login", Phase::Research);
        if !hits.is_empty() {
            assert!((hits[0].score - 1.0).abs() < 1e-5, "top hit should be ~1.0");
        }
    }

    #[test]
    fn phase_subdirs_known_phases() {
        assert!(!phase_subdirs(Phase::Backend).is_empty());
        assert!(!phase_subdirs(Phase::Frontend).is_empty());
        assert!(phase_subdirs(Phase::Research).is_empty()); // whole-tree scan
        assert!(phase_subdirs(Phase::DocsConfirm).is_empty()); // gate, no retrieval
    }

    #[test]
    fn config_round_trips() {
        let cfg = RetrievalConfig {
            enabled: false,
            engine: RetrievalEngine::Hybrid,
            top_k: 12,
            custom_dirs: vec!["team/".into()],
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: RetrievalConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.engine, RetrievalEngine::Hybrid);
        assert_eq!(back.top_k, 12);
        assert!(!back.enabled);
    }

    #[test]
    fn rrf_fuse_merges_and_promotes_overlap() {
        // Build a small index so vector (path,section) hits can map back.
        let chunks = crate::chunker::chunk_text(
            "security/login.md",
            "# Login\n\n## OAuth\n\noauth pkce\n\n## Risks\ntoken theft",
        );
        let index = Bm25Index::from_chunks(chunks);
        // BM25 ranks chunk 0 (OAuth) first, chunk 1 (Risks) second.
        let bm25: Vec<(usize, f64)> = vec![(0, 5.0), (1, 1.0)];
        // Vector also ranks chunk 0's (path, section) first.
        let vec_hits: Vec<(&str, &str, f32)> = vec![
            ("security/login.md", "OAuth", 0.98),
            ("security/login.md", "Risks", 0.70),
        ];
        let fused = rrf_fuse(&index, &bm25, &vec_hits, 60, 5);
        // Chunk 0 appears at rank 0 in both lists → highest fused score.
        assert_eq!(fused[0].0, 0);
        assert!(fused[0].1 > fused[1].1, "overlap chunk must outrank solo");
    }

    #[test]
    fn rrf_fuse_drops_unknown_vector_hits() {
        let chunks = crate::chunker::chunk_text("a.md", "# A\n\n## S\n\nbody");
        let index = Bm25Index::from_chunks(chunks);
        let bm25: Vec<(usize, f64)> = vec![(0, 3.0)];
        // A vector hit whose (path, section) doesn't exist in the index.
        let vec_hits: Vec<(&str, &str, f32)> = vec![("gone.md", "X", 0.9)];
        let fused = rrf_fuse(&index, &bm25, &vec_hits, 60, 5);
        // Only the known BM25 chunk survives.
        assert_eq!(fused.len(), 1);
        assert_eq!(fused[0].0, 0);
    }

    #[test]
    fn rrf_fuse_respects_top_k() {
        let chunks = crate::chunker::chunk_text(
            "a.md",
            "# A\n\n## One\n\nx\n\n## Two\n\ny\n\n## Three\n\nz",
        );
        let index = Bm25Index::from_chunks(chunks);
        let bm25: Vec<(usize, f64)> = vec![(0, 3.0), (1, 2.0), (2, 1.0)];
        let vec_hits: Vec<(&str, &str, f32)> = vec![
            ("a.md", "One", 0.9),
            ("a.md", "Two", 0.7),
            ("a.md", "Three", 0.5),
        ];
        let fused = rrf_fuse(&index, &bm25, &vec_hits, 60, 2);
        assert_eq!(fused.len(), 2);
    }
}
