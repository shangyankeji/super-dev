//! `super-dev-knowledge` — the structured knowledge base that replaces the
//! old "one folder + keyword sort + dump into prompt" approach.
//!
//! Super Dev ships a curated `knowledge/` directory (domain standards,
//! playbooks, expert methodology, design systems). Until 4.5 this corpus
//! was consumed by a naive keyword-scoring path matcher
//! (`score_path` in `super-dev-agent::phases`) that:
//! - only tokenised ASCII words ≥ 3 chars,
//! - matched against file paths (×2) + first 500 chars of content,
//! - fell back to dictionary order when no keyword overlapped
//!   (the common case for CJK requirements against English filenames).
//!
//! That produced irrelevant retrievals and, for Chinese requirements,
//! effectively a random sample. This crate does retrieval properly:
//!
//! - **`chunker`** — markdown-aware segmentation that strips front-matter
//!   and splits on `## H2` sections, preserving per-chunk metadata
//!   (path, title, tags, domain). One knowledge file becomes N chunks.
//! - **`index`** — a pure-Rust BM25 inverted index over the chunk corpus,
//!   with mixed ASCII + CJK-bigram tokenisation. Serialised to
//!   `.super-dev/kb-index/bm25.bin` and rebuilt only when source files
//!   change (mtime-checked).
//! - **`vector`** — an OPTIONAL semantic layer. When `OPENAI_EMBED_KEY` is
//!   present it embeds each chunk via the `/v1/embeddings` endpoint the
//!   user's existing subscription already covers (no new dependency) and
//!   stores the vectors. No key → falls back to BM25 silently.
//! - **`retrieve`** — one entry point that picks the configured engine and
//!   returns ranked [`Chunk`] hits with scores, ready for the agent to
//!   format into a prompt or a TUI panel.
//!
//! ## Safety contract
//! - Pure functions over on-disk data. Retrieval NEVER blocks the
//!   pipeline: a corrupt/missing index returns an empty result, not an
//!   error (fail-open, same as the governance kernel).
//! - The optional vector layer touches the network only when explicitly
//!   enabled by an env var; the default engine is fully offline.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::too_many_lines,
    clippy::format_push_string,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::unnecessary_map_or,
    clippy::unused_async,
    clippy::needless_continue,
    clippy::manual_let_else,
    clippy::nonminimal_bool
)]

pub mod chunker;
pub mod index;
pub mod retrieve;
pub mod tokenizer;
pub mod vector;

pub use chunker::{chunk_file, chunk_text, Chunk, ChunkMeta};
pub use index::{
    bm25_search, build_index, build_vector_store_if_enabled, load_or_build_index, Bm25Index,
    Posting,
};
pub use retrieve::{
    retrieve, retrieve_for_phase, retrieve_for_phase_with_vector, retrieve_with_vector,
    RetrievalConfig, RetrievalEngine, ScoredChunk,
};
pub use tokenizer::tokenize;
pub use vector::VectorStore;

/// Knowledge base index storage location, relative to the project root.
/// The BM25 index (`bm25.bin`) and optional vector store (`vectors.bin`)
/// live here; both are created on demand by their writers.
pub const KB_INDEX_DIR: &str = ".super-dev/kb-index";
