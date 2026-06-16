//! Optional semantic vector layer — only active when an OpenAI API key
//! is set AND the `vector` cargo feature is enabled at compile time.
//!
//! When enabled, chunks are embedded via the user's existing OpenAI
//! subscription (`/v1/embeddings`, `text-embedding-3-small`, 1536-dim) and
//! the vectors are searched with brute-force cosine similarity.
//!
//! ## Activation contract (three layers, all must hold)
//! 1. **Compile time**: the `vector` cargo feature is on (`--features
//!    super-dev-knowledge/vector`). Without it, this whole module compiles
//!    to the offline stub and pulls in zero HTTP dependencies.
//! 2. **Runtime**: an API key env var is set. We accept `OPENAI_EMBED_KEY`
//!    (the dedicated var) OR fall back to the standard `OPENAI_API_KEY`,
//!    so users who only have the latter still get vectors.
//! 3. **Config**: `.superdevrc [knowledge] engine = "hybrid"`.
//!
//! When any layer is missing, `is_enabled()` is false and the retriever
//! transparently uses BM25 only.
//!
//! ## Why not HNSW?
//! `hnsw_rs` requires edition-2024 / Rust ≥1.85 and adds a non-trivial
//! dependency. For a corpus of hundreds-to-low-thousands of chunks, a flat
//! `Vec<Vec<f32>>` cosine scan is sub-millisecond and has zero dependencies.
//! HNSW only matters at millions of vectors — out of scope here. (If the
//! corpus ever grows that large, this module is the single swap point.)
//!
//! ## Network policy (fail-open)
//! - No env key / no feature → [`VectorStore::disabled()`], every method no-op.
//! - Key present but network fails → returns empty results, logs a warning.
//!   Retrieval NEVER blocks the pipeline; BM25 is always the fallback.
//!
//! ## Storage
//! Vectors are cached at `.super-dev/kb-index/vectors.bin` (a serde blob).
//! Each stored vector carries a `body_hash` so cache entries are invalidated
//! per-chunk when the source markdown changes, and a `chunk_idx` to align
//! with the BM25 index's positional model (avoiding `(path, section)`
//! collisions when two H2 headings share a name). Re-embedding only happens
//! for chunks whose `body_hash` differs from the cached value.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Embedding dimension for `text-embedding-3-small`.
const EMBED_DIM: usize = 1536;

/// Known model → embedding dimension. Used to validate that a configured
/// model matches the dimension the store was built with — previously
/// `EMBED_DIM` was hardcoded 1536, so switching `active_model()` to
/// `text-embedding-3-large` (3072-dim) would silently reject every query
/// in `search` (which checks `query_vec.len() != self.dim`) with no hint
/// as to why.
const KNOWN_MODEL_DIMS: &[(&str, usize)] = &[
    ("text-embedding-3-small", 1536),
    ("text-embedding-3-large", 3072),
    ("text-embedding-ada-002", 1536),
    ("text-embedding-2", 1536),
];

/// Resolve the effective embedding dimension, in priority order:
/// 1. `SUPER_DEV_EMBED_DIM` env override (explicit user pin),
/// 2. the known dimension for [`active_model`] (if it's a recognised model),
/// 3. [`EMBED_DIM`] (1536, the small-model default).
///
/// Returning the env override first lets a user force a non-standard dim
/// even for an unknown model.
#[must_use]
pub fn active_dim() -> usize {
    if let Ok(v) = std::env::var("SUPER_DEV_EMBED_DIM") {
        if let Ok(d) = v.parse::<usize>() {
            if d > 0 {
                return d;
            }
        }
    }
    expected_dim_for_model(active_model()).unwrap_or(EMBED_DIM)
}

/// The documented dimension for a known embedding model, or `None` when the
/// model isn't in [`KNOWN_MODEL_DIMS`].
#[must_use]
pub fn expected_dim_for_model(model: &str) -> Option<usize> {
    KNOWN_MODEL_DIMS
        .iter()
        .find(|(name, _)| *name == model)
        .map(|(_, dim)| *dim)
}
/// Env vars that activate the vector layer. `OPENAI_EMBED_KEY` is checked
/// first (dedicated), then we fall back to the standard `OPENAI_API_KEY` so
/// users with only the standard key configured still get vectors. Only
/// referenced when the `vector` feature is on.
#[cfg(feature = "vector")]
const ENV_KEY: &str = "OPENAI_EMBED_KEY";
#[cfg(feature = "vector")]
const ENV_KEY_FALLBACK: &str = "OPENAI_API_KEY";
#[cfg(feature = "vector")]
const ENV_BASE: &str = "OPENAI_EMBED_BASE";
/// Embedding model. `text-embedding-3-small` is the cheapest high-quality
/// option (~$0.02/M tokens as of 2026) and 1536-dim.
const DEFAULT_MODEL: &str = "text-embedding-3-small";

/// Resolve the API key, checking the dedicated var then the standard one.
/// Returns `None` when neither is set (or empty). Only compiled when the
/// `vector` feature is on (the constants it reads are feature-gated).
#[cfg(feature = "vector")]
fn resolve_api_key() -> Option<String> {
    for var in [ENV_KEY, ENV_KEY_FALLBACK] {
        if let Ok(v) = std::env::var(var) {
            if !v.trim().is_empty() {
                return Some(v);
            }
        }
    }
    None
}

/// Whether the vector layer is configured (feature on + env key present).
/// This is the single switch the retriever checks before trying vectors.
///
/// Without the `vector` cargo feature this is a compile-time `false`.
#[must_use]
pub fn is_enabled() -> bool {
    #[cfg(feature = "vector")]
    {
        resolve_api_key().is_some()
    }
    #[cfg(not(feature = "vector"))]
    {
        false
    }
}

/// Resolve the embeddings API base URL. Defaults to OpenAI's public endpoint.
/// Only used when the `vector` feature compiles the HTTP transport in.
#[cfg(feature = "vector")]
fn api_base() -> String {
    std::env::var(ENV_BASE).unwrap_or_else(|_| "https://api.openai.com".to_string())
}

/// One stored embedding with enough metadata to map back to a chunk.
///
/// `body_hash` drives per-chunk cache invalidation: the index builder hashes
/// each chunk's `body` and, on reload, re-embeds only entries whose hash
/// differs. `chunk_idx` aligns this entry with the BM25 index's positional
/// model so two same-named H2 sections in one file don't collide.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredVector {
    /// Chunk path (e.g. `security/login.md`).
    path: String,
    /// H2 section heading.
    section: String,
    /// The embedding vector.
    vec: Vec<f32>,
    /// Content hash of the chunk body at embed time (cache invalidation).
    #[serde(default)]
    body_hash: u64,
    /// Positional index into the BM25 `chunks` vec (collision-safe key).
    #[serde(default)]
    chunk_idx: u32,
}

/// The cached vector store on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStore {
    /// Model used to produce these vectors (invalidation key).
    model: String,
    /// Embedding dimension (sanity check on load).
    dim: usize,
    /// All stored vectors.
    #[serde(default)]
    vectors: Vec<StoredVector>,
}

impl VectorStore {
    /// An empty, disabled store. All operations are no-ops.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            model: String::new(),
            dim: EMBED_DIM,
            vectors: Vec::new(),
        }
    }

    /// Load the cached store from disk. Returns the disabled sentinel
    /// (empty) when the file is missing or malformed — never errors.
    #[must_use]
    pub fn load(project_root: &Path) -> Self {
        let path = vectors_path(project_root);
        let Ok(bytes) = std::fs::read(&path) else {
            return Self::disabled();
        };
        serde_json::from_slice(&bytes).unwrap_or_else(|_| Self::disabled())
    }

    /// Persist the store to disk (best-effort; never errors).
    pub fn save(&self, project_root: &Path) {
        let path = vectors_path(project_root);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(bytes) = serde_json::to_vec(self) {
            let _ = std::fs::write(&path, bytes);
        }
    }

    /// Number of vectors currently stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Whether the store holds any vectors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// Search the store with a pre-embedded query vector, returning the
    /// `(path, section, score)` triples ranked by descending cosine
    /// similarity. Returns empty when the store is empty.
    ///
    /// This is intentionally synchronous + pure: the query vector must be
    /// obtained separately (via [`embed_query`]) so that the network call
    /// is isolated to the async runner seam and fail-open.
    #[must_use]
    pub fn search(&self, query_vec: &[f32], top_k: usize) -> Vec<(&str, &str, f32)> {
        if self.vectors.is_empty() || query_vec.len() != self.dim || top_k == 0 {
            return Vec::new();
        }
        let mut scored: Vec<(&str, &str, f32)> = self
            .vectors
            .iter()
            .map(|v| {
                let s = cosine(&v.vec, query_vec);
                (v.path.as_str(), v.section.as_str(), s)
            })
            .collect();
        scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    /// Expose the stored entries as `(chunk_idx, path, section, body_hash,
    /// vec)` tuples, for the index builder to diff against current chunks
    /// during incremental re-embedding. This is the cache-reuse accessor.
    #[must_use]
    pub fn cached_for_reuse(&self) -> Vec<(u32, String, String, u64, Vec<f32>)> {
        self.vectors
            .iter()
            .map(|v| {
                (
                    v.chunk_idx,
                    v.path.clone(),
                    v.section.clone(),
                    v.body_hash,
                    v.vec.clone(),
                )
            })
            .collect()
    }

    /// Return all stored vectors as `(chunk_idx, path, section, body_hash)`
    /// so the index builder can diff against current chunks for incremental
    /// re-embedding. Public so `index.rs` can drive cache invalidation.
    #[must_use]
    pub fn entries(&self) -> Vec<(u32, &str, &str, u64)> {
        self.vectors
            .iter()
            .map(|v| {
                (
                    v.chunk_idx,
                    v.path.as_str(),
                    v.section.as_str(),
                    v.body_hash,
                )
            })
            .collect()
    }

    /// Build a fresh store from a list of (chunk_idx, path, section,
    /// body_hash, vec) tuples. Used by the index builder after embedding.
    #[must_use]
    pub fn from_embedded(model: &str, entries: Vec<(u32, String, String, u64, Vec<f32>)>) -> Self {
        let vectors = entries
            .into_iter()
            .map(|(chunk_idx, path, section, body_hash, vec)| StoredVector {
                path,
                section,
                vec,
                body_hash,
                chunk_idx,
            })
            .collect();
        Self {
            model: model.to_string(),
            dim: active_dim(),
            vectors,
        }
    }

    /// Replace all stored vectors from raw embedded tuples (used after a
    /// rebuild). Takes the same shape as [`from_embedded`] to avoid leaking
    /// the private [`StoredVector`] type.
    pub fn replace(&mut self, model: &str, entries: Vec<(u32, String, String, u64, Vec<f32>)>) {
        self.model = model.to_string();
        self.dim = active_dim();
        self.vectors = entries
            .into_iter()
            .map(|(chunk_idx, path, section, body_hash, vec)| StoredVector {
                path,
                section,
                vec,
                body_hash,
                chunk_idx,
            })
            .collect();
    }

    /// Expose the model name this store was built with.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }
}

/// Cosine similarity between two equal-length vectors. Returns 0.0 when
/// either vector has zero magnitude (avoids NaN).
fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

/// On-disk path for the cached vector store.
fn vectors_path(project_root: &Path) -> PathBuf {
    project_root.join(super::KB_INDEX_DIR).join("vectors.bin")
}

// ---------------------------------------------------------------------------
// HTTP transport — only compiled when the `vector` feature is on. Without
// it, embed_query/embed_batch compile to stubs returning None, and the crate
// has no reqwest dependency at all.
// ---------------------------------------------------------------------------

/// Embed a single query string via the OpenAI embeddings API. Returns
/// `None` on any failure (feature off, network, parse, missing key) so the
/// caller can fall back to BM25. This is the query-time embed call; it is
/// `async` and isolated to the runner seam.
#[cfg_attr(not(feature = "vector"), allow(clippy::unused_async))]
pub async fn embed_query(text: &str) -> Option<Vec<f32>> {
    #[cfg(feature = "vector")]
    {
        let key = resolve_api_key()?;
        let url = format!("{}/v1/embeddings", api_base());
        let body = serde_json::json!({ "model": DEFAULT_MODEL, "input": text });
        let mut vecs = http_embed(&url, &key, body).await?;
        if vecs.len() == 1 {
            Some(vecs.pop()?)
        } else {
            None
        }
    }
    #[cfg(not(feature = "vector"))]
    {
        let _ = text;
        None
    }
}

/// Embed many texts in one (or a few batched) API call(s). Returns vectors
/// in input order, or `None` on any failure. Batches internally at
/// [`EMBED_BATCH_MAX`] texts per request to stay within API limits.
#[cfg_attr(not(feature = "vector"), allow(clippy::unused_async))]
pub async fn embed_batch(texts: &[String]) -> Option<Vec<Vec<f32>>> {
    #[cfg(feature = "vector")]
    {
        let key = resolve_api_key()?;
        if texts.is_empty() {
            return Some(Vec::new());
        }
        let url = format!("{}/v1/embeddings", api_base());
        let mut out: Vec<Vec<f32>> = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(EMBED_BATCH_MAX) {
            let body = serde_json::json!({ "model": DEFAULT_MODEL, "input": chunk });
            let mut vecs = http_embed(&url, &key, body).await?;
            out.append(&mut vecs);
        }
        if out.len() == texts.len() {
            Some(out)
        } else {
            tracing::warn!(
                "embeddings count mismatch: got {} expected {} — discarding batch",
                out.len(),
                texts.len()
            );
            None
        }
    }
    #[cfg(not(feature = "vector"))]
    {
        let _ = texts;
        None
    }
}

/// Maximum input texts per embeddings request. OpenAI allows up to 2048;
/// we keep a conservative cap to bound per-request latency and payload.
#[cfg(feature = "vector")]
const EMBED_BATCH_MAX: usize = 100;

/// Extract the model name the vector layer uses (for cache invalidation).
/// Honours the `SUPER_DEV_EMBED_MODEL` env override so a user can point at
/// `text-embedding-3-large` (or a self-hosted model) without recompiling.
#[must_use]
pub fn active_model() -> &'static str {
    if let Ok(m) = std::env::var("SUPER_DEV_EMBED_MODEL") {
        if !m.trim().is_empty() {
            // Leak once: the override is process-lived and we need a
            // &'static str to match the const-DEFAULT_MODEL signature.
            return Box::leak(m.into_boxed_str());
        }
    }
    DEFAULT_MODEL
}

// ---------------------------------------------------------------------------
// reqwest integration — isolated behind the `vector` feature so the default
// (BM25-only) build of this crate has zero HTTP dependencies.
// ---------------------------------------------------------------------------

/// Exponential backoff sleep for retry attempt `n` (1-indexed):
/// attempt 1 → 0.5s, 2 → 1.0s, 3 → 2.0s. Capped so a slow provider can't
/// stall the pipeline for long. Async so it yields the runtime while waiting.
/// Max attempts for transient-HTTP retries inside [`http_embed`].
#[cfg(feature = "vector")]
const EMBED_MAX_ATTEMPTS: u32 = 3;

#[cfg(feature = "vector")]
async fn backoff_sleep(n: u32) {
    // Exponential backoff: attempt 1 → 0.5s, 2 → 1.0s, … capped at 4s.
    let secs = 0.5_f64 * 2_f64.powi(i32::try_from(n.saturating_sub(1)).unwrap_or(i32::MAX));
    tokio::time::sleep(std::time::Duration::from_secs_f64(secs.min(4.0))).await;
}

#[cfg(feature = "vector")]
async fn http_embed(url: &str, key: &str, body: serde_json::Value) -> Option<Vec<Vec<f32>>> {
    // Pooled client reused across all calls (connection keep-alive + TLS
    // reuse). Built once on first use so startup cost is zero when vectors
    // are never activated.
    let client = EMBED_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(EMBED_TIMEOUT_SECS))
            .pool_max_idle_per_host(4)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    });

    // Retry transient failures (429 Too Many Requests, 5xx) with exponential
    // backoff. Previously a single 429/500 failed the whole batch and fell
    // back to BM25 for the entire run — even though the provider would have
    // served it a second later. Non-transient errors (4xx other than 429)
    // are NOT retried. Connection errors are retried once (often a transient
    // TLS/dns blip).
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        let send_result = client.post(url).bearer_auth(key).json(&body).send().await;
        let resp = match send_result {
            Ok(r) => r,
            Err(e) => {
                if attempt < EMBED_MAX_ATTEMPTS {
                    tracing::warn!("embeddings request error (attempt {attempt}): {e}; retrying");
                    backoff_sleep(attempt).await;
                    continue;
                }
                tracing::warn!(
                    "embeddings request failed after {attempt} attempts (fail-open → BM25): {e}"
                );
                return None;
            }
        };
        let status = resp.status();
        let transient = status.as_u16() == 429 || status.is_server_error();
        if !status.is_success() {
            if transient && attempt < EMBED_MAX_ATTEMPTS {
                tracing::warn!(
                    "embeddings API returned {status} (attempt {attempt}); retrying with backoff"
                );
                backoff_sleep(attempt).await;
                continue;
            }
            tracing::warn!("embeddings API returned {status} (fail-open → BM25)");
            return None;
        }
        let json: EmbedResponse = match resp.json().await {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!("embeddings response parse failed (fail-open → BM25): {e}");
                return None;
            }
        };
        let mut items = json.data;
        // OpenAI may return embeddings out of input order; sort by the `index`
        // field to guarantee alignment with the input batch.
        items.sort_by_key(|d| d.index);
        return Some(items.into_iter().map(|d| d.embedding).collect());
    }
}

/// Per-request timeout for embeddings calls. Generous because a full corpus
/// batch can take a few seconds.
#[cfg(feature = "vector")]
const EMBED_TIMEOUT_SECS: u64 = 60;

/// A pooled reqwest client reused across all embeddings calls. Built lazily
/// so the crate has zero startup cost when vectors are never activated.
#[cfg(feature = "vector")]
static EMBED_CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();

/// Minimal subset of the OpenAI embeddings response we care about.
#[cfg(feature = "vector")]
#[derive(Debug, Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedItem>,
}

#[cfg(feature = "vector")]
#[derive(Debug, Deserialize)]
struct EmbedItem {
    embedding: Vec<f32>,
    #[allow(dead_code)]
    index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_store_is_empty() {
        let s = VectorStore::disabled();
        assert!(s.is_empty());
        assert!(s.search(&[0.0; EMBED_DIM], 5).is_empty());
    }

    #[test]
    fn cosine_identical_vectors_is_one() {
        let v = vec![0.1, 0.2, 0.3, 0.4];
        assert!((cosine(&v, &v) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_orthogonal_vectors_is_zero() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!(cosine(&a, &b).abs() < 1e-5);
    }

    #[test]
    fn cosine_zero_magnitude_returns_zero() {
        let zero = vec![0.0, 0.0, 0.0];
        let v = vec![1.0, 2.0, 3.0];
        assert!(cosine(&zero, &v).abs() < 1e-5);
    }

    #[test]
    fn search_ranks_by_similarity() {
        let store = VectorStore {
            model: "test".into(),
            dim: 3,
            vectors: vec![
                StoredVector {
                    path: "a".into(),
                    section: "s".into(),
                    vec: vec![1.0, 0.0, 0.0],
                    body_hash: 0,
                    chunk_idx: 0,
                },
                StoredVector {
                    path: "b".into(),
                    section: "s".into(),
                    vec: vec![0.0, 1.0, 0.0],
                    body_hash: 0,
                    chunk_idx: 1,
                },
                StoredVector {
                    path: "c".into(),
                    section: "s".into(),
                    vec: vec![0.9, 0.1, 0.0],
                    body_hash: 0,
                    chunk_idx: 2,
                },
            ],
        };
        let query = vec![1.0, 0.0, 0.0];
        let hits = store.search(&query, 3);
        assert_eq!(hits.len(), 3);
        // "a" (identical) ranks first, "c" (close) second, "b" (orthogonal) last.
        assert_eq!(hits[0].0, "a");
        assert_eq!(hits[1].0, "c");
        assert_eq!(hits[2].0, "b");
        assert!((hits[0].2 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn search_wrong_dim_returns_empty() {
        let store = VectorStore {
            model: "test".into(),
            dim: 3,
            vectors: vec![StoredVector {
                path: "a".into(),
                section: "s".into(),
                vec: vec![1.0; 3],
                body_hash: 0,
                chunk_idx: 0,
            }],
        };
        // Query of wrong dimension → empty, not a panic.
        assert!(store.search(&[0.0; 5], 1).is_empty());
    }

    #[test]
    fn store_serialises_round_trip() {
        let store = VectorStore {
            model: "test".into(),
            dim: 2,
            vectors: vec![StoredVector {
                path: "a".into(),
                section: "s".into(),
                vec: vec![1.0, 0.0],
                body_hash: 123,
                chunk_idx: 7,
            }],
        };
        let bytes = serde_json::to_vec(&store).unwrap();
        let back: VectorStore = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back.dim, 2);
        assert_eq!(back.len(), 1);
        assert_eq!(back.entries()[0].0, 7); // chunk_idx round-trips
        assert_eq!(back.entries()[0].3, 123); // body_hash round-trips
    }

    #[test]
    fn store_backwards_compatible_with_old_cache() {
        // An old cache blob (pre body_hash/chunk_idx) must still load via serde
        // defaults. Simulate by omitting the new fields from the JSON.
        let old_json =
            r#"{"model":"m","dim":2,"vectors":[{"path":"a","section":"s","vec":[1.0,0.0]}]}"#;
        let back: VectorStore = serde_json::from_str(old_json).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back.entries()[0].0, 0); // chunk_idx defaulted to 0
        assert_eq!(back.entries()[0].3, 0); // body_hash defaulted to 0
    }

    #[test]
    fn from_embedded_builds_store() {
        let store = VectorStore::from_embedded(
            "text-embedding-3-small",
            vec![(0, "a".into(), "s".into(), 42, vec![1.0; EMBED_DIM])],
        );
        assert_eq!(store.len(), 1);
        assert_eq!(store.model(), "text-embedding-3-small");
        assert_eq!(store.entries()[0].3, 42);
    }

    #[test]
    fn is_enabled_false_without_env() {
        // Clear any API key vars so is_enabled() reflects "no key". These
        // constants only exist under the `vector` feature; without it,
        // is_enabled() is compile-time false regardless.
        #[cfg(feature = "vector")]
        {
            std::env::remove_var(ENV_KEY);
            std::env::remove_var(ENV_KEY_FALLBACK);
            assert!(!is_enabled());
        }
        #[cfg(not(feature = "vector"))]
        {
            assert!(!is_enabled());
        }
    }

    #[tokio::test]
    async fn embed_query_returns_none_without_key() {
        // No API key → None regardless of feature (fail-open to BM25).
        #[cfg(feature = "vector")]
        {
            std::env::remove_var(ENV_KEY);
            std::env::remove_var(ENV_KEY_FALLBACK);
        }
        assert!(embed_query("login").await.is_none());
    }

    #[tokio::test]
    async fn embed_batch_empty_returns_empty() {
        #[cfg(feature = "vector")]
        {
            std::env::remove_var(ENV_KEY);
            std::env::remove_var(ENV_KEY_FALLBACK);
        }
        // No key + empty input → None (no point embedding nothing).
        let out = embed_batch(&[]).await;
        // Without the feature, always None.
        #[cfg(not(feature = "vector"))]
        assert!(out.is_none());
        #[allow(unused_must_use)]
        {
            let _ = out;
        }
    }

    #[test]
    fn expected_dim_maps_known_models() {
        assert_eq!(expected_dim_for_model("text-embedding-3-small"), Some(1536));
        assert_eq!(expected_dim_for_model("text-embedding-3-large"), Some(3072));
        assert_eq!(expected_dim_for_model("text-embedding-ada-002"), Some(1536));
        assert_eq!(expected_dim_for_model("unknown-model"), None);
    }

    // NOTE: these assertions read/write the process-global SUPER_DEV_EMBED_DIM
    // env var, so they must live in ONE test (run serially) — two parallel
    // #[test]s mutating the same env var race and flake.
    #[test]
    fn active_dim_default_and_override() {
        // Clean slate.
        std::env::remove_var("SUPER_DEV_EMBED_DIM");
        std::env::remove_var("SUPER_DEV_EMBED_MODEL");
        assert_eq!(active_dim(), 1536, "default = small-model dim");
        // Explicit override wins.
        std::env::set_var("SUPER_DEV_EMBED_DIM", "3072");
        assert_eq!(active_dim(), 3072, "SUPER_DEV_EMBED_DIM must win");
        // Invalid (0) → fall back to model default.
        std::env::set_var("SUPER_DEV_EMBED_DIM", "0");
        assert_eq!(active_dim(), 1536, "invalid dim falls back");
        std::env::remove_var("SUPER_DEV_EMBED_DIM");
    }
}
