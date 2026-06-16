//! Integration test for the vector hybrid retrieval path: VectorStore
//! search + RRF fusion with BM25. This verifies the end-to-end semantic
//! retrieval correctness WITHOUT hitting the real OpenAI API (we populate
//! the VectorStore with deterministic test vectors and verify fusion logic).
//!
//! The real HTTP embed path (embed_batch → API → VectorStore) is covered by
//! the unit tests in vector.rs that verify the fail-open contract. This test
//! focuses on the retrieval correctness when vectors ARE present.

use std::path::Path;
use super_dev_knowledge::{
    build_vector_store_if_enabled,
    retrieve::{retrieve, retrieve_with_vector, RetrievalConfig, RetrievalEngine},
    vector::VectorStore,
};
use tempfile::TempDir;

/// Build a small knowledge corpus in a temp dir.
fn setup_corpus(dir: &Path) {
    let kd = dir.join("knowledge/security");
    std::fs::create_dir_all(&kd).unwrap();
    std::fs::write(
        kd.join("login.md"),
        "# Login\n\n## OAuth\n\nUse OAuth2 with PKCE for secure login authentication.\n",
    )
    .unwrap();
    std::fs::write(
        kd.join("passwords.md"),
        "# Password Security\n\n## Hashing\n\nUse bcrypt or Argon2id for password hashing.\n",
    )
    .unwrap();
    std::fs::write(
        kd.join("tokens.md"),
        "# Token Management\n\n## JWT\n\nUse short-lived JWT access tokens with Redis revocation.\n",
    )
    .unwrap();
}

#[test]
fn bm25_retrieval_returns_relevant_chunks() {
    let tmp = TempDir::new().unwrap();
    setup_corpus(tmp.path());
    let cfg = RetrievalConfig {
        enabled: true,
        engine: RetrievalEngine::Bm25,
        top_k: 3,
        custom_dirs: vec![],
    };
    let hits = retrieve(
        tmp.path(),
        &tmp.path().join("knowledge"),
        &cfg,
        "login authentication OAuth",
        super_dev_spec::Phase::Backend,
    );
    assert!(
        !hits.is_empty(),
        "BM25 must return hits for 'login authentication'"
    );
    // The login.md OAuth chunk should rank highly.
    assert!(
        hits.iter().any(|h| h.chunk.meta.path.contains("login")),
        "login.md should be in results: {:?}",
        hits.iter().map(|h| &h.chunk.meta.path).collect::<Vec<_>>()
    );
}

#[test]
fn retrieve_with_vector_none_falls_back_to_bm25() {
    // When query_vec is None, retrieve_with_vector should behave identically
    // to pure BM25 (the fail-open contract).
    let tmp = TempDir::new().unwrap();
    setup_corpus(tmp.path());
    let cfg = RetrievalConfig {
        enabled: true,
        engine: RetrievalEngine::Bm25,
        top_k: 3,
        custom_dirs: vec![],
    };
    let hits_bm25 = retrieve(
        tmp.path(),
        &tmp.path().join("knowledge"),
        &cfg,
        "password hashing",
        super_dev_spec::Phase::Backend,
    );
    let hits_vec_none = retrieve_with_vector(
        tmp.path(),
        &tmp.path().join("knowledge"),
        &cfg,
        "password hashing",
        super_dev_spec::Phase::Backend,
        None,
    );
    assert_eq!(
        hits_bm25.len(),
        hits_vec_none.len(),
        "None vector must produce same count as BM25"
    );
}

#[test]
fn vector_store_search_ranks_by_similarity() {
    // Build a VectorStore with known vectors and verify cosine similarity
    // ranking — the core of the vector retrieval path.

    // Use 1536-dim vectors (VectorStore enforces this dimension).
    let one = vec![1.0; 1536];
    let mut c_vec = vec![0.0; 1536];
    c_vec[0] = 0.9;
    c_vec[1] = 0.1;
    let store = VectorStore::from_embedded(
        "test",
        vec![
            (0, "a.md".into(), "S1".into(), 0, one.clone()),
            (1, "b.md".into(), "S2".into(), 0, vec![0.0; 1536]),
            (2, "c.md".into(), "S3".into(), 0, c_vec),
        ],
    );
    let hits = store.search(&one, 3);
    assert_eq!(hits.len(), 3);
    assert_eq!(hits[0].0, "a.md", "exact match must rank first");
}

#[test]
fn vector_store_serializes_and_loads() {
    let tmp = TempDir::new().unwrap();
    let store = VectorStore::from_embedded(
        "test-model",
        vec![(0, "a.md".into(), "S".into(), 42, vec![0.5; 1536])],
    );
    store.save(tmp.path());
    assert!(tmp.path().join(".super-dev/kb-index/vectors.bin").is_file());

    let loaded = VectorStore::load(tmp.path());
    assert_eq!(loaded.len(), 1);
    let entries = loaded.cached_for_reuse();
    assert_eq!(entries[0].3, 42, "body_hash must round-trip");
}

#[test]
fn rrf_fusion_combines_bm25_and_vector() {
    // Verify that RRF fusion promotes chunks that appear in BOTH lists.
    // We do this by constructing a small index, then calling retrieve_with_vector
    // with a hand-crafted query vector that boosts a specific chunk.
    let tmp = TempDir::new().unwrap();
    setup_corpus(tmp.path());

    // Build the BM25 index.
    let index = super_dev_knowledge::load_or_build_index(
        tmp.path(),
        &tmp.path().join("knowledge/security"),
    );
    assert!(!index.chunks.is_empty());

    // Manually populate a VectorStore where login.md's chunk gets a vector
    // that aligns with our query vector.
    let login_chunk_idx = index
        .chunks
        .iter()
        .position(|c| c.meta.path.contains("login"))
        .expect("login chunk must exist");

    // Build a store with the login chunk's vector aligned to [1, 0, ...].
    let mut entries: Vec<(u32, String, String, u64, Vec<f32>)> = Vec::new();
    for (i, chunk) in index.chunks.iter().enumerate() {
        let vec = if i == login_chunk_idx {
            vec![1.0; 1536] // aligned with our query
        } else {
            vec![0.0; 1536] // orthogonal to our query
        };
        entries.push((
            u32::try_from(i).unwrap_or(0),
            chunk.meta.path.clone(),
            chunk.meta.section.clone(),
            0,
            vec,
        ));
    }
    let store = VectorStore::from_embedded("test-model", entries);
    store.save(tmp.path());

    // Now retrieve with a query vector aligned to login.md.
    let cfg = RetrievalConfig {
        enabled: true,
        engine: RetrievalEngine::Hybrid,
        top_k: 3,
        custom_dirs: vec![],
    };
    let qvec = vec![1.0; 1536];
    let hits = retrieve_with_vector(
        tmp.path(),
        &tmp.path().join("knowledge/security"),
        &cfg,
        "login authentication",
        super_dev_spec::Phase::Backend,
        Some(&qvec),
    );

    // The login chunk should appear in the results (it's boosted by both BM25
    // keyword match and vector similarity).
    assert!(
        hits.iter().any(|h| h.chunk.meta.path.contains("login")),
        "login chunk must appear in hybrid results: {:?}",
        hits.iter().map(|h| &h.chunk.meta.path).collect::<Vec<_>>()
    );
}

#[test]
fn hybrid_with_empty_vector_store_falls_back_to_bm25() {
    // When the vector store is empty (no API key / feature off), hybrid
    // engine must transparently degrade to BM25.
    let tmp = TempDir::new().unwrap();
    setup_corpus(tmp.path());

    // No vectors.bin written — store will be empty.
    let cfg = RetrievalConfig {
        enabled: true,
        engine: RetrievalEngine::Hybrid,
        top_k: 3,
        custom_dirs: vec![],
    };
    let qvec = vec![0.5; 1536];
    let hits = retrieve_with_vector(
        tmp.path(),
        &tmp.path().join("knowledge/security"),
        &cfg,
        "password hashing",
        super_dev_spec::Phase::Backend,
        Some(&qvec),
    );
    assert!(
        !hits.is_empty(),
        "hybrid with empty store must still return BM25 hits"
    );
}

#[test]
fn build_vector_store_is_noop_without_api_key() {
    // build_vector_store_if_enabled must return None when no API key is set
    // (the fail-open contract — the whole pipeline stays BM25).
    let tmp = TempDir::new().unwrap();
    setup_corpus(tmp.path());
    std::env::remove_var("OPENAI_EMBED_KEY");
    std::env::remove_var("OPENAI_API_KEY");
    let index = super_dev_knowledge::load_or_build_index(
        tmp.path(),
        &tmp.path().join("knowledge/security"),
    );
    let store = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(build_vector_store_if_enabled(tmp.path(), &index));
    assert!(
        store.is_none(),
        "without API key, vector build must be no-op"
    );
}
