//! Spec test-vector runner — implements SUPER_DEV_HOST_SPEC_V1 §1.4.
//!
//! Reads `tests/spec_vectors/<clause>.json` and asserts that the governance
//! rule for that clause produces the `expected_decision` for every vector.
//! This is the spec's normative "conformance test": any implementation of
//! SD-CODE-001/002 must pass these vectors.
//!
//! The vectors live at the workspace root (`tests/spec_vectors/`) so they're
//! shared across crates; this test reads them via a relative path.

use serde::Deserialize;
use super_dev_governance::{check_ai_slop, check_color_tokens, check_emoji, Decision};

/// One `(file_path, content) → expected_decision` vector.
#[derive(Debug, Deserialize)]
struct Vector {
    file_path: String,
    content: String,
    expected_decision: String,
}

/// The vector file shape (see tests/spec_vectors/SD-CODE-001.json).
#[derive(Debug, Deserialize)]
struct VectorFile {
    clause: String,
    vectors: Vec<Vector>,
}

/// Resolve the workspace-root spec_vectors dir. The test binary runs from
/// the crate dir, so we walk up to find `tests/spec_vectors`.
fn spec_vectors_dir() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    for _ in 0..6 {
        let candidate = dir.join("tests/spec_vectors");
        if candidate.is_dir() {
            return candidate;
        }
        dir = match dir.parent() {
            Some(p) => p.to_path_buf(),
            None => break,
        };
    }
    // Fall back to the workspace layout (crate is at crates/<name>, vectors at top).
    std::path::PathBuf::from("../../tests/spec_vectors")
}

fn load_vectors(clause: &str) -> Vec<Vector> {
    let path = spec_vectors_dir().join(format!("{clause}.json"));
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let parsed: VectorFile =
        serde_json::from_str(&body).unwrap_or_else(|e| panic!("malformed {}: {e}", path.display()));
    assert_eq!(
        parsed.clause, clause,
        "clause field mismatch in {clause}.json"
    );
    parsed.vectors
}

fn assert_decision(actual: &Decision, expected: &str, clause: &str, file_path: &str) {
    let actual_blocked = actual.block;
    let want_block = expected == "block";
    assert!(
        actual_blocked == want_block,
        "{} vector for `{}`: expected decision `{expected}` but got `{}`\n  reason: {}",
        clause,
        file_path,
        if actual_blocked { "block" } else { "pass" },
        actual.reason,
    );
}

#[test]
fn sd_code_001_emoji_vectors_pass() {
    for v in load_vectors("SD-CODE-001") {
        let d = check_emoji(&v.file_path, &v.content);
        assert_decision(&d, &v.expected_decision, "SD-CODE-001", &v.file_path);
    }
}

#[test]
fn sd_code_002_color_vectors_pass() {
    for v in load_vectors("SD-CODE-002") {
        let d = check_color_tokens(&v.file_path, &v.content);
        assert_decision(&d, &v.expected_decision, "SD-CODE-002", &v.file_path);
    }
}

#[test]
fn sd_code_005_slop_vectors_pass() {
    for v in load_vectors("SD-CODE-005") {
        let d = check_ai_slop(&v.file_path, &v.content);
        assert_decision(&d, &v.expected_decision, "SD-CODE-005", &v.file_path);
    }
}
