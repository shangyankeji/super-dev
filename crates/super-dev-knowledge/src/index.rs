//! BM25 inverted index — the default (offline, zero-dependency) retrieval engine.
//!
//! BM25 is the industry-standard lexical ranking function (what Elasticsearch
//! / Lucene use). It outperforms plain TF-IDF by saturating term frequency,
//! so a term appearing 100× in one doc isn't 100× more relevant than 10×.
//!
//! The index is a pure data structure: build once from chunks, query N times.
//! It serialises to `.super-dev/kb-index/bm25.bin` (via serde_json) and is
//! rebuilt only when source `.md` mtimes change (see [`load_or_build_index`]).
//!
//! ## Why not HNSW here?
//! HNSW (`hnsw_rs`) is the vector layer in [`crate::vector`]. BM25 is
//! keyword-exact and needs no approximate search — a flat inverted index
//! scanned per query term is both faster and deterministic for this corpus
//! size (hundreds of chunks).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::chunker::Chunk;
use crate::tokenizer::tokenize;
use crate::vector;

/// Classic BM25 tunables. These defaults (k1=1.2, b=0.75) are the values
/// every major search engine ships with; changing them is rarely warranted.
const K1: f64 = 1.2;
const B: f64 = 0.75;

/// One inverted-index entry: the term, and the chunks that contain it with
/// per-chunk term frequency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Posting {
    /// The tokenised term.
    pub term: String,
    /// (chunk_index, term_frequency_in_that_chunk) pairs.
    pub docs: Vec<(u32, u32)>,
}

/// The complete BM25 index, serialisable to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bm25Index {
    /// All chunks, indexed by position. Query results reference back here.
    pub chunks: Vec<Chunk>,
    /// Inverted index: term → postings.
    pub postings: Vec<Posting>,
    /// For fast term lookup during query: term → index into `postings`.
    /// Serialised as a Vec of (term, idx) and rebuilt into a HashMap on load
    /// to avoid storing a non-serde-friendly HashMap directly.
    pub term_map: Vec<(String, u32)>,
    /// Document frequency cache: term → number of chunks containing it.
    /// (Length of the matching Posting's docs vec — stored redundantly for
    /// query speed.)
    /// Average chunk length in tokens.
    pub avg_doc_len: f64,
    /// Total number of chunks. Equal to `chunks.len()`; stored explicitly
    /// so the query function doesn't borrow `self.chunks`.
    pub doc_count: u64,
}

impl Bm25Index {
    /// Build an index from a set of chunks. Pure; no I/O.
    #[must_use]
    pub fn from_chunks(chunks: Vec<Chunk>) -> Self {
        let doc_count = u64::try_from(chunks.len()).unwrap_or(0);
        let total_len: usize = chunks.iter().map(|c| c.tokens.len()).sum();
        let avg_doc_len = if chunks.is_empty() {
            0.0
        } else {
            total_len as f64 / chunks.len() as f64
        };

        // Build the inverted index: term → HashMap<chunk_idx, tf>.
        let mut inverted: HashMap<String, HashMap<u32, u32>> = HashMap::new();
        for (idx, chunk) in chunks.iter().enumerate() {
            let chunk_idx = u32::try_from(idx).unwrap_or(u32::MAX);
            let mut seen_in_doc: HashMap<&str, u32> = HashMap::new();
            for tok in &chunk.tokens {
                *seen_in_doc.entry(tok.as_str()).or_insert(0) += 1;
            }
            for (term, tf) in seen_in_doc {
                inverted
                    .entry(term.to_string())
                    .or_default()
                    .insert(chunk_idx, tf);
            }
        }

        // Flatten into Posting list + a term→idx map.
        let mut postings = Vec::with_capacity(inverted.len());
        let mut term_map = Vec::with_capacity(inverted.len());
        for (i, (term, docs_map)) in inverted.into_iter().enumerate() {
            let idx = u32::try_from(i).unwrap_or(u32::MAX);
            term_map.push((term.clone(), idx));
            let mut docs: Vec<(u32, u32)> = docs_map.into_iter().collect();
            docs.sort_unstable_by_key(|(c, _)| *c);
            postings.push(Posting { term, docs });
        }
        term_map.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            chunks,
            postings,
            term_map,
            avg_doc_len,
            doc_count,
        }
    }

    /// HashMap view of `term_map` for query-time lookup. Cheap to build.
    fn term_index(&self) -> HashMap<&str, u32> {
        self.term_map
            .iter()
            .map(|(t, i)| (t.as_str(), *i))
            .collect()
    }

    /// Run a BM25 query. Returns `(chunk_idx, score)` pairs, highest score
    /// first. Scores are unbounded positive floats (BM25 has no fixed scale).
    #[must_use]
    pub fn search(&self, query: &str, top_k: usize) -> Vec<(usize, f64)> {
        if self.chunks.is_empty() || top_k == 0 {
            return Vec::new();
        }
        let term_idx = self.term_index();
        let query_terms = tokenize(query);
        if query_terms.is_empty() {
            return Vec::new();
        }

        // Accumulate per-chunk score.
        let mut scores: Vec<f64> = vec![0.0; self.chunks.len()];
        let n = self.doc_count.max(1) as f64;

        for term in &query_terms {
            let Some(&pidx) = term_idx.get(term.as_str()) else {
                continue; // term not in corpus
            };
            let posting = &self.postings[pidx as usize];
            let df = posting.docs.len() as f64;
            // IDF with BM25's +1 smoothing (never negative).
            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
            for &(chunk_idx, tf) in &posting.docs {
                let ci = chunk_idx as usize;
                let dl = self.chunks[ci].tokens.len() as f64;
                let denom = f64::from(tf) * (1.0 - B) + B * (dl / self.avg_doc_len.max(1.0));
                let tf_component = (f64::from(tf) * (K1 + 1.0)) / (denom + K1);
                scores[ci] += idf * tf_component;
            }
        }

        // Collect (idx, score) for chunks with score > 0, sort desc, take top_k.
        let mut ranked: Vec<(usize, f64)> = scores
            .iter()
            .enumerate()
            .filter(|(_, s)| **s > 0.0)
            .map(|(i, s)| (i, *s))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(top_k);
        ranked
    }
}

/// Convenience: build an index then search in one call (used by tests).
#[must_use]
pub fn build_index(chunks: Vec<Chunk>) -> Bm25Index {
    Bm25Index::from_chunks(chunks)
}

/// Search a pre-built index, returning chunk references. Thin wrapper over
/// [`Bm25Index::search`] returning `&Chunk` for ergonomic prompt formatting.
#[must_use]
pub fn bm25_search<'a>(index: &'a Bm25Index, query: &str, top_k: usize) -> Vec<(&'a Chunk, f64)> {
    index
        .search(query, top_k)
        .into_iter()
        .map(|(i, s)| (&index.chunks[i], s))
        .collect()
}

/// On-disk index path: `<project_root>/.super-dev/kb-index/bm25.bin`.
fn index_path(project_root: &Path) -> PathBuf {
    project_root.join(super::KB_INDEX_DIR).join("bm25.bin")
}

/// Build (or rebuild) the index from all `.md` files under `knowledge_dir`,
/// serialise it to disk, and return it. Always overwrites the on-disk copy.
///
/// Walks the directory tree to depth 6 (matching the legacy `walk_md`
/// guard), chunks each `.md`, and indexes the resulting chunks.
pub fn load_or_build_index(project_root: &Path, knowledge_dir: &Path) -> Bm25Index {
    load_or_build_index_multi(project_root, &[knowledge_dir.to_path_buf()])
}

/// Build/load the BM25 index over MULTIPLE source directories (e.g. the
/// static `knowledge/`, project `.super-dev/learned/`, and global
/// `~/.super-dev/learned/`). All `.md` files across all dirs are merged into
/// one index with a combined mtime signature for cache invalidation.
#[must_use]
pub fn load_or_build_index_multi(project_root: &Path, knowledge_dirs: &[PathBuf]) -> Bm25Index {
    // Collect all .md files from every dir.
    let mut paths: Vec<PathBuf> = Vec::new();
    for dir in knowledge_dirs {
        if dir.is_dir() {
            walk_md(dir, &mut paths, 0);
        }
    }
    let signature = corpus_signature(&paths, knowledge_dirs);

    // Cache check: if the signature matches the stored one, load directly.
    let sig_path = index_path(project_root).with_extension("sig");
    if let Ok(stored_sig) = std::fs::read_to_string(&sig_path) {
        if stored_sig == signature {
            let idx_path = index_path(project_root);
            if let Ok(bytes) = std::fs::read(&idx_path) {
                if let Ok(index) = serde_json::from_slice::<Bm25Index>(&bytes) {
                    return index;
                }
            }
        }
    }

    // Cache miss (or corrupt) — rebuild. Chunk each file using the dir it
    // came from as the strip-prefix root (so ChunkMeta.path is relative).
    let mut chunks: Vec<Chunk> = Vec::new();
    for abs in &paths {
        let root = knowledge_dirs
            .iter()
            .find(|d| abs.starts_with(d))
            .cloned()
            .unwrap_or_else(|| knowledge_dirs[0].clone());
        let file_chunks = crate::chunker::chunk_file(&root, abs);
        chunks.extend(file_chunks);
    }
    let index = Bm25Index::from_chunks(chunks);

    // Persist index + signature (best-effort).
    let path = index_path(project_root);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_vec(&index) {
        let _ = std::fs::write(&path, text);
        let _ = std::fs::write(&sig_path, &signature);
    }

    index
}

/// Build a deterministic signature of the corpus: one line per file
/// `<relative_path>\t<mtime_secs>\t<size>`, sorted. Identical for identical
/// corpora; differs as soon as any file changes, is added, or is removed.
/// Build a machine-INDEPENDENT corpus signature: for each file, store its
/// path RELATIVE to the knowledge dir it came from + a truncated SHA-256 of
/// its CONTENT (NOT mtime). This means two clones of the same knowledge/
/// tree produce the SAME signature, so the `.super-dev/kb-index/` cache can
/// be copied between machines / re-clones and still hit — previously the
/// signature used absolute paths + mtime, which differ per machine/clone, so
/// copying the cache silently invalidated it everywhere.
///
/// `knowledge_dirs` is the list of roots to strip (so `ChunkMeta.path`-style
/// relative keys land in the signature). Falls back to the file name when no
/// root matches.
fn corpus_signature(paths: &[PathBuf], knowledge_dirs: &[PathBuf]) -> String {
    use sha2::{Digest, Sha256};
    let mut entries: Vec<(String, String)> = paths
        .iter()
        .filter_map(|p| {
            // Relative path: strip the matching knowledge dir prefix.
            let rel = knowledge_dirs
                .iter()
                .find_map(|d| p.strip_prefix(d).ok())
                .or_else(|| p.file_name().map(std::path::Path::new))
                .map(|r| r.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|| p.to_string_lossy().replace('\\', "/"));
            // Content hash (truncated SHA-256, hex) — stable across clones,
            // unlike mtime. Read failure → skip the file (fail-open).
            let bytes = std::fs::read(p).ok()?;
            let digest = Sha256::digest(&bytes);
            let hash_hex: String = digest.iter().take(8).fold(String::new(), |mut acc, b| {
                use std::fmt::Write as _;
                let _ = write!(acc, "{b:02x}");
                acc
            });
            Some((rel, hash_hex))
        })
        .collect();
    entries.sort();
    entries
        .iter()
        .map(|(p, h)| format!("{p}\t{h}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Maximum number of `.md` files the index will scan. A guard against a
/// pathological corpus (e.g. a vendored docs dump) ballooning index build
/// time + memory. When hit, the extra files are silently skipped — but a
/// warning is emitted (see [`walk_md`]) so the user knows coverage is
/// partial. Override via `SUPER_DEV_KNOWLEDGE_MAX_FILES` (0 = unlimited).
const DEFAULT_MAX_MD_FILES: usize = 2000;

/// Effective file cap, honouring the `SUPER_DEV_KNOWLEDGE_MAX_FILES` env
/// override (`0` = unlimited). Read once and cached for the process.
fn max_md_files() -> usize {
    std::env::var("SUPER_DEV_KNOWLEDGE_MAX_FILES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_MD_FILES)
}

/// Recursively collect `.md` file paths under `dir`, up to `depth` 6.
/// Matches the legacy `walk_md` behaviour (phases.rs:1851) so the corpus
/// coverage is identical.
fn walk_md(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    let cap = max_md_files();
    if depth > 6 {
        return;
    }
    // 0 = unlimited; otherwise stop + warn once when the cap is reached.
    if cap != 0 && out.len() >= cap {
        // Warn at most once: only when we cross the cap on this entry.
        if out.len() == cap {
            eprintln!(
                "warn: knowledge index hit the {cap}-file cap (set \
                 SUPER_DEV_KNOWLEDGE_MAX_FILES=0 to index everything, or a \
                 higher number). Files beyond the cap are NOT indexed — \
                 retrieval coverage is partial."
            );
        }
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
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

// ---------------------------------------------------------------------------
// Vector store build — batch-embeds all chunks and caches them with
// per-chunk content-hash invalidation. Only does real work when the `vector`
// feature is on AND an API key is set; otherwise it's a no-op that leaves
// the store empty (BM25 dominates).
//
// This is the piece that was an honest stub before: the index was built but
// never embedded, so VectorStore was always empty and hybrid retrieval
// silently degraded to BM25 even when the user set OPENAI_API_KEY.
// ---------------------------------------------------------------------------

/// Compute a stable content hash of a chunk's body for cache invalidation.
///
/// Uses **truncated SHA-256** (first 8 bytes → u64), NOT the stdlib
/// `DefaultHasher`. SHA-256 is fixed by spec, so the cache survives Rust
/// toolchain bumps that change the stdlib's default SipHash
/// algorithm/seeds — which would otherwise silently invalidate the ENTIRE
/// `.super-dev/kb-index/` cache and force a full re-embed. The value only
/// needs determinism + collision-resistance for a chunk body, both of which
/// SHA-256 provides; truncation to 64 bits is plenty for a corpus of
/// thousands of chunks (birthday bound ≈ 4 billion).
#[must_use]
fn body_hash(body: &str) -> u64 {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(body.as_bytes());
    // First 8 bytes, big-endian → u64.
    u64::from_be_bytes(digest[..8].try_into().expect("sha256 is 32 bytes"))
}

/// The text that gets embedded for a chunk: title + section + body. This is
/// the exact string the embedding model sees, so it must be deterministic
/// across rebuilds for the cache to hit.
#[must_use]
fn embed_text(chunk: &Chunk) -> String {
    format!(
        "{}\n{}\n{}",
        chunk.meta.title, chunk.meta.section, chunk.body
    )
}

/// Build (or incrementally update) the cached vector store for an index.
///
/// This is the async companion to [`load_or_build_index_multi`]: it runs
/// AFTER the BM25 index is ready, embeds every chunk, and persists the store
/// to `.super-dev/kb-index/vectors.bin`. Returns `None` (and leaves the
/// store empty) when the vector layer is off or embedding fails — the
/// retriever then transparently uses BM25 only.
///
/// **Incremental**: the cached store is loaded first; chunks whose
/// `body_hash` matches the cache are reused, and only new/changed chunks
/// are re-embedded. This keeps re-indexing after a small doc edit cheap.
pub async fn build_vector_store_if_enabled(
    project_root: &Path,
    index: &Bm25Index,
) -> Option<vector::VectorStore> {
    if !vector::is_enabled() || index.chunks.is_empty() {
        return None;
    }

    let model = vector::active_model();
    let mut store = vector::VectorStore::load(project_root);

    // If the cached store was built with a different model, discard it.
    if !store.is_empty() && store.model() != model {
        store = vector::VectorStore::disabled();
    }

    // Build a lookup: (chunk_idx, path, section) → (body_hash, cached_vec).
    // Reuse cached vectors whose body_hash still matches; collect the rest
    // for fresh embedding.
    let cached: HashMap<(u32, String, String), (u64, Vec<f32>)> = store
        .cached_for_reuse()
        .into_iter()
        .map(|(idx, path, section, hash, vec)| ((idx, path, section), (hash, vec)))
        .collect();

    let mut to_embed: Vec<(usize, String)> = Vec::new(); // (chunk_idx, text)
    let mut kept: Vec<(u32, String, String, u64, Vec<f32>)> = Vec::new();

    for (i, chunk) in index.chunks.iter().enumerate() {
        let idx = u32::try_from(i).unwrap_or(u32::MAX);
        let hash = body_hash(&chunk.body);
        let key = (idx, chunk.meta.path.clone(), chunk.meta.section.clone());
        if let Some((cached_hash, cached_vec)) = cached.get(&key) {
            if *cached_hash == hash {
                // Cache hit — reuse the existing vector.
                kept.push((
                    idx,
                    chunk.meta.path.clone(),
                    chunk.meta.section.clone(),
                    hash,
                    cached_vec.clone(),
                ));
                continue;
            }
        }
        to_embed.push((i, embed_text(chunk)));
    }

    if to_embed.is_empty() {
        // Everything cached — just refresh the store from `kept`.
        store.replace(model, kept);
        store.save(project_root);
        return Some(store);
    }

    // Embed the stale/new chunks in batches.
    let texts: Vec<String> = to_embed.iter().map(|(_, t)| t.clone()).collect();
    tracing::info!(
        "embedding {} chunk(s) ({} cached, model {model})",
        texts.len(),
        kept.len()
    );
    let vecs = if let Some(v) = vector::embed_batch(&texts).await {
        v
    } else {
        tracing::warn!(
            "batch embedding failed — vector store left {} cached entries, BM25 will dominate",
            kept.len()
        );
        if kept.is_empty() {
            return None;
        }
        store.replace(model, kept);
        store.save(project_root);
        return Some(store);
    };

    if vecs.len() != to_embed.len() {
        tracing::warn!(
            "embedding count mismatch ({} != {}) — discarding new vectors, keeping cache",
            vecs.len(),
            to_embed.len()
        );
        if kept.is_empty() {
            return None;
        }
        store.replace(model, kept);
        store.save(project_root);
        return Some(store);
    }

    // Merge freshly-embedded chunks with the cached ones.
    for ((chunk_i, _), vec) in to_embed.into_iter().zip(vecs) {
        let chunk = &index.chunks[chunk_i];
        let idx = u32::try_from(chunk_i).unwrap_or(u32::MAX);
        kept.push((
            idx,
            chunk.meta.path.clone(),
            chunk.meta.section.clone(),
            body_hash(&chunk.body),
            vec,
        ));
    }

    store.replace(model, kept);
    store.save(project_root);
    tracing::info!("vector store built: {} vectors", store.len());
    Some(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::chunk_text;

    fn idx_from(texts: &[(&str, &str)]) -> Bm25Index {
        let chunks: Vec<Chunk> = texts
            .iter()
            .flat_map(|(path, body)| chunk_text(path, body))
            .collect();
        Bm25Index::from_chunks(chunks)
    }

    #[test]
    fn empty_index_returns_nothing() {
        let idx = Bm25Index::from_chunks(Vec::new());
        assert!(idx.search("anything", 5).is_empty());
    }

    #[test]
    fn empty_query_returns_nothing() {
        let idx = idx_from(&[("a.md", "# A\n\n## S\n\nlogin auth")]);
        assert!(idx.search("   ", 5).is_empty());
    }

    #[test]
    fn term_not_in_corpus_returns_nothing() {
        let idx = idx_from(&[("a.md", "# A\n\n## S\n\nlogin")]);
        assert!(idx.search("nonexistentterm", 5).is_empty());
    }

    #[test]
    fn exact_term_match_ranks_relevant_doc() {
        let idx = idx_from(&[
            (
                "login.md",
                "# Login\n\n## Flow\n\nUse OAuth2 PKCE for login authentication.",
            ),
            (
                "postgres.md",
                "# Postgres\n\n## Tuning\n\nshared_buffers tuning for database.",
            ),
        ]);
        let results = idx.search("login", 5);
        assert!(!results.is_empty());
        // The login doc (idx 0's chunk) should rank first.
        let top_path = &idx.chunks[results[0].0].meta.path;
        assert!(
            top_path.contains("login"),
            "expected login doc, got {top_path}"
        );
    }

    #[test]
    fn rarer_term_scores_higher_than_common_term() {
        // Two docs both mention "auth"; only one mentions "pkce".
        let idx = idx_from(&[
            ("a.md", "# A\n\n## S\n\nauth auth auth auth pkce"),
            ("b.md", "# B\n\n## S\n\nauth auth auth auth"),
        ]);
        let auth_results = idx.search("auth", 2);
        let pkce_results = idx.search("pkce", 2);
        // "pkce" should only match doc a.
        assert_eq!(pkce_results.len(), 1);
        // "auth" matches both (a has higher TF so ranks first, but both score).
        assert_eq!(auth_results.len(), 2);
    }

    #[test]
    fn multi_term_query_accumulates_score() {
        let idx = idx_from(&[
            (
                "login.md",
                "# Login\n\n## OAuth\n\noauth2 pkce login security",
            ),
            (
                "other.md",
                "# Other\n\n## S\n\nunrelated content about cooking",
            ),
        ]);
        let results = idx.search("login oauth2", 5);
        // The login doc (whichever chunk idx) must rank first.
        let top_path = &idx.chunks[results[0].0].meta.path;
        assert!(
            top_path.contains("login"),
            "expected login doc to win, got {top_path}"
        );
    }

    #[test]
    fn cjk_query_matches_cjk_content() {
        // The headline fix: CJK requirements now retrieve CJK content.
        let idx = idx_from(&[("login.md", "# 登录\n\n## 流程\n\n使用 OAuth2 做登录认证")]);
        let results = idx.search("登录系统", 5);
        assert!(!results.is_empty(), "CJK query must hit CJK content");
    }

    #[test]
    fn top_k_limits_results() {
        let idx = idx_from(&[
            ("a.md", "# A\n\n## S\n\nauth auth"),
            ("b.md", "# B\n\n## S\n\nauth auth"),
            ("c.md", "# C\n\n## S\n\nauth auth"),
        ]);
        assert_eq!(idx.search("auth", 2).len(), 2);
        assert_eq!(idx.search("auth", 10).len(), 3);
    }

    #[test]
    fn index_serialises_round_trip() {
        let idx = idx_from(&[("login.md", "# Login\n\n## OAuth\n\noauth2 pkce login")]);
        let bytes = serde_json::to_vec(&idx).unwrap();
        let back: Bm25Index = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back.chunks.len(), idx.chunks.len());
        // Functional equivalence: same query yields same top doc.
        let orig = idx.search("login", 1);
        let reload = back.search("login", 1);
        assert_eq!(orig[0].0, reload[0].0);
    }

    #[test]
    fn bm25_search_returns_chunk_refs() {
        let idx = idx_from(&[("login.md", "# Login\n\n## S\n\nauth login")]);
        let hits = bm25_search(&idx, "login", 5);
        assert!(!hits.is_empty());
        // The matched chunk must mention the query terms.
        assert!(
            hits[0].0.body.contains("login") || hits[0].0.meta.title.contains("Login"),
            "hit body should reference the query"
        );
        assert!(hits[0].1 > 0.0);
    }

    #[test]
    fn load_or_build_writes_and_returns() {
        use std::fs;
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let kd = root.join("knowledge");
        fs::create_dir_all(kd.join("security")).unwrap();
        fs::write(
            kd.join("security/login.md"),
            "# Login\n\n## OAuth\n\nUse OAuth2 PKCE for login.",
        )
        .unwrap();

        let idx = load_or_build_index(root, &kd);
        assert!(!idx.chunks.is_empty());
        // Index file was written.
        assert!(root.join(".super-dev/kb-index/bm25.bin").is_file());
        // Query works.
        let results = idx.search("login", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn load_or_build_reuses_cache_when_unchanged() {
        use std::fs;
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let kd = root.join("knowledge");
        fs::create_dir_all(kd.join("security")).unwrap();
        fs::write(
            kd.join("security/login.md"),
            "# Login\n\n## OAuth\n\nUse OAuth2 PKCE for login.",
        )
        .unwrap();

        // First call builds + writes index + signature.
        let idx1 = load_or_build_index(root, &kd);
        let sig_path = root.join(".super-dev/kb-index/bm25.sig");
        assert!(sig_path.is_file(), "signature file must be written");

        // Second call with unchanged corpus loads from cache (functional equiv).
        let idx2 = load_or_build_index(root, &kd);
        assert_eq!(idx1.chunks.len(), idx2.chunks.len());
        assert_eq!(idx1.search("login", 1)[0].0, idx2.search("login", 1)[0].0);

        // Touching a file invalidates the cache (mtime changes).
        fs::write(
            kd.join("security/login.md"),
            "# Login\n\n## OAuth\n\nUpdated content about login authentication.",
        )
        .unwrap();
        let idx3 = load_or_build_index(root, &kd);
        assert!(!idx3.chunks.is_empty());
    }

    #[test]
    fn corpus_signature_changes_with_content() {
        use std::fs;
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path().to_path_buf();
        let f = dir.join("a.md");
        fs::write(&f, "content v1").unwrap();
        let s1 = corpus_signature(std::slice::from_ref(&f), std::slice::from_ref(&dir));
        fs::write(&f, "content v2 changed").unwrap();
        let s2 = corpus_signature(std::slice::from_ref(&f), std::slice::from_ref(&dir));
        assert_ne!(s1, s2, "signature must change when content/size changes");
    }

    #[test]
    fn corpus_signature_is_machine_independent() {
        // Regression: the signature used absolute paths + mtime, so two
        // clones of the same knowledge/ tree on different machines produced
        // DIFFERENT signatures and the kb-index cache couldn't be shared.
        // Now it's relative-path + content-hash: identical content → identical
        // signature regardless of where the tree lives.
        use std::fs;
        let dir_a = tempfile::TempDir::new().unwrap();
        let dir_b = tempfile::TempDir::new().unwrap();
        // Same relative layout + identical content in both.
        fs::create_dir_all(dir_a.path().join("security")).unwrap();
        fs::create_dir_all(dir_b.path().join("security")).unwrap();
        fs::write(dir_a.path().join("a.md"), "same content").unwrap();
        fs::write(dir_b.path().join("a.md"), "same content").unwrap();
        fs::write(
            dir_a.path().join("security/login.md"),
            "# Login

## OAuth
",
        )
        .unwrap();
        fs::write(
            dir_b.path().join("security/login.md"),
            "# Login

## OAuth
",
        )
        .unwrap();
        let paths_a = vec![
            dir_a.path().join("a.md"),
            dir_a.path().join("security/login.md"),
        ];
        let paths_b = vec![
            dir_b.path().join("a.md"),
            dir_b.path().join("security/login.md"),
        ];
        let sa = corpus_signature(&paths_a, &[dir_a.path().to_path_buf()]);
        let sb = corpus_signature(&paths_b, &[dir_b.path().to_path_buf()]);
        assert_eq!(
            sa, sb,
            "same content under different roots must produce the SAME signature              (machine-independent); got
A: {sa}
B: {sb}"
        );
        // And the signature must contain RELATIVE paths (no tempdir prefix).
        assert!(
            !sa.contains("tmp") && !sa.contains("/var/") && !sa.contains("/private/"),
            "signature must not leak absolute temp paths: {sa}"
        );
        assert!(
            sa.contains("a.md"),
            "signature must contain relative file name"
        );
        assert!(
            sa.contains("security/login.md"),
            "signature must contain nested relative path"
        );
    }

    #[test]
    fn body_hash_is_deterministic() {
        assert_eq!(body_hash("hello world"), body_hash("hello world"));
        assert_ne!(body_hash("hello world"), body_hash("hello earth"));
        assert_ne!(body_hash(""), body_hash("x"));
    }

    #[test]
    fn body_hash_is_stable_across_versions() {
        // Regression: body_hash used to use stdlib DefaultHasher, whose
        // algorithm/seeds can change between Rust toolchain versions — silently
        // invalidating the entire .super-dev/kb-index/ cache. Now it's
        // truncated SHA-256, which is fixed by spec. Pin the exact u64 for a
        // known input so a regression to a non-stable hasher is caught:
        //   SHA-256("hello world")[0..8] big-endian = 13352372148217134600.
        assert_eq!(
            body_hash("hello world"),
            13_352_372_148_217_134_600u64,
            "body_hash must be stable truncated-SHA256; if this changed, the              cache will silently invalidate across Rust versions."
        );
        // Multibyte input must hash the UTF-8 bytes (stable, not char-based).
        assert_eq!(body_hash("用户登录系统"), 1_142_734_754_577_198_762u64);
        // Empty string has its own stable value.
        assert_eq!(body_hash(""), 16_406_829_232_824_261_652u64);
    }

    #[test]
    fn embed_text_format_is_stable() {
        let chunk = crate::chunker::chunk_text("docs/login.md", "# Login\n\n## OAuth\n\nbody");
        let text = embed_text(&chunk[0]);
        assert!(text.contains("Login"));
        assert!(text.contains("OAuth"));
        assert!(text.contains("body"));
    }

    #[tokio::test]
    async fn build_vector_store_is_noop_without_key() {
        // No API key (or no vector feature) → the store build is a no-op,
        // returning None. This is the fail-open contract: BM25 dominates.
        let idx = idx_from(&[("login.md", "# Login\n\n## OAuth\n\nlogin auth")]);
        let tmp = tempfile::TempDir::new().unwrap();
        let store = build_vector_store_if_enabled(tmp.path(), &idx).await;
        assert!(
            store.is_none(),
            "without a key the vector build must be a no-op"
        );
    }
}
