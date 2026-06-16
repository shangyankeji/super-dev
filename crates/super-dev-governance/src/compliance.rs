//! Compliance mapping — turn Super Dev evidence into auditor-ready output.
//!
//! Implements `SD-EVID-004`. Takes the in-workspace evidence files (the
//! quality report + the two audit JSONL trails) and produces a
//! structured mapping document that links every clause that fired to
//! its corresponding controls in SOC 2 (2017 TSC), ISO/IEC 27001:2022
//! Annex A, and EU AI Act articles.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super_dev_spec::SPEC_VERSION;

use crate::audit::{ApiCallRecord, ToolCallRecord};

/// External compliance-framework references attached to one clause.
///
/// Owned strings so the type is fully Serialize+Deserialize across IPC
/// / file boundaries.
#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct ComplianceFrameworks {
    /// SOC 2 (2017 Trust Services Criteria) control identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub soc2_cc: Vec<String>,
    /// ISO/IEC 27001:2022 Annex A control identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub iso27001_annex_a: Vec<String>,
    /// EU AI Act article references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub eu_ai_act_article: Vec<String>,
}

fn s(slice: &[&str]) -> Vec<String> {
    slice.iter().map(|s| (*s).to_string()).collect()
}

/// The canonical clause → external-framework table.
///
/// Reviewed quarterly. Frameworks pinned to: SOC 2 2017 TSC, ISO/IEC
/// 27001:2022, EU AI Act 2024/1689.
#[must_use]
pub fn framework_for(clause_id: &str) -> ComplianceFrameworks {
    match clause_id {
        // Layer 1 — code-weight
        "SD-CODE-001" => ComplianceFrameworks {
            soc2_cc: s(&["CC8.1"]),
            iso27001_annex_a: s(&["A.5.34", "A.8.28"]),
            eu_ai_act_article: s(&["Article 15"]),
        },
        "SD-CODE-002" => ComplianceFrameworks {
            soc2_cc: s(&["CC8.1"]),
            iso27001_annex_a: s(&["A.8.28"]),
            eu_ai_act_article: s(&["Article 15"]),
        },
        "SD-CODE-003" => ComplianceFrameworks {
            soc2_cc: s(&["CC7.1", "CC8.1"]),
            iso27001_annex_a: s(&["A.8.28", "A.8.30"]),
            eu_ai_act_article: s(&["Article 15"]),
        },
        "SD-CODE-004" => ComplianceFrameworks {
            soc2_cc: s(&["CC8.1"]),
            iso27001_annex_a: s(&["A.5.37"]),
            eu_ai_act_article: s(&["Article 13"]),
        },
        // Layer 2 — flow contract
        "SD-FLOW-001" => ComplianceFrameworks {
            soc2_cc: s(&["CC8.1"]),
            iso27001_annex_a: s(&["A.5.37"]),
            eu_ai_act_article: s(&["Article 17"]),
        },
        "SD-FLOW-002" | "SD-FLOW-003" => ComplianceFrameworks {
            soc2_cc: s(&["CC1.4", "CC8.1"]),
            iso27001_annex_a: s(&["A.5.31"]),
            eu_ai_act_article: s(&["Article 14"]),
        },
        "SD-FLOW-004" | "SD-FLOW-005" => ComplianceFrameworks {
            soc2_cc: s(&["CC8.1"]),
            iso27001_annex_a: s(&["A.5.37"]),
            eu_ai_act_article: s(&["Article 14"]),
        },
        "SD-FLOW-006" => ComplianceFrameworks {
            soc2_cc: s(&["CC7.2"]),
            iso27001_annex_a: s(&["A.8.15"]),
            eu_ai_act_article: s(&["Article 12"]),
        },
        // Layer 3 — artifacts
        "SD-ART-001" => ComplianceFrameworks {
            soc2_cc: s(&["CC2.2"]),
            iso27001_annex_a: s(&["A.5.37"]),
            eu_ai_act_article: s(&["Article 13"]),
        },
        "SD-ART-002" | "SD-ART-003" | "SD-ART-004" => ComplianceFrameworks {
            soc2_cc: s(&["CC8.1"]),
            iso27001_annex_a: s(&["A.5.37"]),
            eu_ai_act_article: s(&["Article 11"]),
        },
        "SD-ART-005" => ComplianceFrameworks {
            soc2_cc: s(&["CC8.1"]),
            iso27001_annex_a: s(&["A.5.37"]),
            eu_ai_act_article: Vec::new(),
        },
        "SD-ART-006" => ComplianceFrameworks {
            soc2_cc: s(&["CC8.1"]),
            iso27001_annex_a: s(&["A.5.37"]),
            eu_ai_act_article: s(&["Article 17"]),
        },
        // Layer 4 — evidence
        "SD-EVID-001" | "SD-EVID-002" => ComplianceFrameworks {
            soc2_cc: s(&["CC7.2"]),
            iso27001_annex_a: s(&["A.8.15"]),
            eu_ai_act_article: s(&["Article 12"]),
        },
        "SD-EVID-003" => ComplianceFrameworks {
            soc2_cc: s(&["CC4.1"]),
            iso27001_annex_a: s(&["A.5.36"]),
            eu_ai_act_article: s(&["Article 17"]),
        },
        "SD-EVID-004" | "SD-EVID-005" => ComplianceFrameworks {
            soc2_cc: s(&["CC2.2"]),
            iso27001_annex_a: s(&["A.5.36"]),
            eu_ai_act_article: s(&["Article 11"]),
        },
        "SD-META-001" => ComplianceFrameworks {
            soc2_cc: s(&["CC1.1"]),
            iso27001_annex_a: s(&["A.5.36"]),
            eu_ai_act_article: s(&["Article 11"]),
        },
        _ => ComplianceFrameworks::default(),
    }
}

/// Static lookup symbol kept for callers that just want the table.
///
/// (Returns the function pointer; consumers call it per clause id.)
pub const CLAUSE_COMPLIANCE: fn(&str) -> ComplianceFrameworks = framework_for;

/// Per-clause aggregate in the final mapping JSON.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClauseEvidence {
    /// Clause id, e.g. `SD-CODE-001`.
    pub id: String,
    /// Number of times this clause fired during the run.
    pub fired_count: u64,
    /// Decision → count map (`block`/`allow`/`audit`/`warn`).
    pub decisions: BTreeMap<String, u64>,
    /// SOC 2 control identifiers.
    pub soc2_cc: Vec<String>,
    /// ISO/IEC 27001:2022 Annex A controls.
    pub iso27001_annex_a: Vec<String>,
    /// EU AI Act article references.
    pub eu_ai_act_article: Vec<String>,
    /// Evidence file paths (workspace-relative).
    pub evidence: Vec<String>,
    /// SHA-256 hashes of key artifact contents, for tamper-evident audit
    /// (`"<workspace-relative-path>:<sha256>"`). Empty for clauses that
    /// don't attach content hashes. Backwards-compatible: old JSON loads fine.
    #[serde(default)]
    pub content_hashes: Vec<String>,
}

impl ClauseEvidence {
    fn new(clause_id: &str) -> Self {
        let fw = framework_for(clause_id);
        Self {
            id: clause_id.to_string(),
            fired_count: 0,
            decisions: BTreeMap::new(),
            soc2_cc: fw.soc2_cc,
            iso27001_annex_a: fw.iso27001_annex_a,
            eu_ai_act_article: fw.eu_ai_act_article,
            evidence: Vec::new(),
            content_hashes: Vec::new(),
        }
    }

    fn bump(&mut self, decision: &str) {
        self.fired_count += 1;
        *self.decisions.entry(decision.to_string()).or_insert(0) += 1;
    }

    fn ensure_evidence(&mut self, path: &str) {
        if !self.evidence.iter().any(|p| p == path) {
            self.evidence.push(path.to_string());
        }
    }

    /// Attach a content SHA-256 for a key artifact, for tamper-evident audit.
    /// Format: `"<path>:<hex-sha256>"`.
    fn add_content_hash(&mut self, path: &str, sha: &str) {
        let entry = format!("{path}:{sha}");
        if !self.content_hashes.iter().any(|e| e == &entry) {
            self.content_hashes.push(entry);
        }
    }
}

/// Compute the SHA-256 hex digest of a file's bytes. Returns `None` when the
/// file is unreadable (fail-open — the hash is evidence enrichment, not a gate).
#[must_use]
pub fn file_sha256(path: &std::path::Path) -> Option<String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Some(format!("{:x}", hasher.finalize()))
}

/// Compute the SHA-256 hex digest of an in-memory string.
#[must_use]
pub fn content_sha256(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Inputs to `build_compliance_mapping` — pre-parsed evidence.
#[derive(Debug, Default)]
pub struct ComplianceInputs<'a> {
    /// Project slug used in the output filename and clauses' evidence paths.
    pub slug: &'a str,
    /// Parsed quality-gate JSON if present. `None` when missing.
    pub quality_report: Option<&'a serde_json::Value>,
    /// Tool-call audit rows.
    pub tool_calls: &'a [ToolCallRecord],
    /// API-call audit rows.
    pub api_calls: &'a [ApiCallRecord],
    /// Optional pinned generation timestamp (ISO-8601 UTC). Use for tests.
    pub generated_at: Option<String>,
    /// Optional `declared_by` string, e.g. `super-dev@4.4.0`.
    pub declared_by: Option<String>,
    /// Optional project root — when set, content SHA-256 hashes are computed
    /// for the key artifacts (quality gate, architecture, contract) and
    /// attached to the relevant clauses for tamper-evident audit. `None`
    /// skips hashing (tests / offline).
    pub project_root: Option<&'a std::path::Path>,
}

/// The final mapping document.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ComplianceMapping {
    /// Always `SUPER_DEV_HOST_SPEC_V1` in V1.
    pub spec_version: String,
    /// Project slug.
    pub slug: String,
    /// ISO-8601 UTC timestamp.
    pub generated_at: String,
    /// E.g. `super-dev@4.4.0`.
    pub declared_by: String,
    /// `Some(bool)` when a quality report was present; `None` otherwise.
    pub quality_gate_passed: Option<bool>,
    /// Per-clause aggregates, ordered by clause id.
    pub clauses: Vec<ClauseEvidence>,
    /// Roll-up counts.
    pub summary: ComplianceSummary,
}

/// Roll-up counts attached to the mapping document.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ComplianceSummary {
    /// How many unique clauses fired.
    pub total_clauses_fired: usize,
    /// How many tool-call rows we ingested.
    pub total_tool_calls: usize,
    /// How many api-call rows we ingested.
    pub total_api_audit_rows: usize,
    /// Human-readable list of frameworks covered.
    pub frameworks: Vec<String>,
}

/// Build the compliance-mapping document from pre-parsed inputs.
///
/// Pure function — no I/O. Callers (CLI / agent / CI) handle file
/// loading and writing.
#[must_use]
pub fn build_compliance_mapping(inputs: &ComplianceInputs<'_>) -> ComplianceMapping {
    let mut evidence: BTreeMap<String, ClauseEvidence> = BTreeMap::new();

    // Tool-call trail
    for row in inputs.tool_calls {
        if row.clause.is_empty() {
            continue;
        }
        // Defensive: skip unrecognised clause IDs
        if framework_for(&row.clause) == ComplianceFrameworks::default() {
            continue;
        }
        let entry = evidence
            .entry(row.clause.clone())
            .or_insert_with(|| ClauseEvidence::new(&row.clause));
        entry.bump(&row.decision);
        entry.ensure_evidence(".super-dev/audit/tool-calls.jsonl");
    }

    // API-call trail counts toward SD-CODE-003 + SD-EVID-001
    if !inputs.api_calls.is_empty() {
        let url_count: usize = inputs.api_calls.iter().map(|r| r.urls.len()).sum();
        let row_count = inputs.api_calls.len();
        let contribution = if url_count > 0 { url_count } else { row_count };
        for clause_id in ["SD-CODE-003", "SD-EVID-001"] {
            let entry = evidence
                .entry(clause_id.to_string())
                .or_insert_with(|| ClauseEvidence::new(clause_id));
            #[allow(clippy::cast_possible_truncation)]
            let inc = contribution as u64;
            entry.fired_count += inc;
            *entry.decisions.entry("audit".to_string()).or_insert(0) += inc;
            entry.ensure_evidence(".super-dev/audit/frontend-api-calls.jsonl");
        }
    }

    // Quality report counts toward SD-EVID-003
    let quality_passed = inputs
        .quality_report
        .and_then(|v| v.get("passed"))
        .and_then(serde_json::Value::as_bool);
    if inputs.quality_report.is_some() {
        let entry = evidence
            .entry("SD-EVID-003".to_string())
            .or_insert_with(|| ClauseEvidence::new("SD-EVID-003"));
        let outcome = if quality_passed.unwrap_or(false) {
            "passed"
        } else {
            "failed"
        };
        entry.bump(outcome);
        entry.ensure_evidence(&format!("output/{}-quality-gate.json", inputs.slug));
        entry.ensure_evidence(&format!("output/{}-quality-gate.md", inputs.slug));
    }

    // Content-hash enrichment: compute SHA-256 of key artifacts for
    // tamper-evident audit. Best-effort — unreadable files are skipped.
    if let Some(root) = inputs.project_root {
        // Quality gate → SD-EVID-003
        let qg_path = format!("output/{}-quality-gate.json", inputs.slug);
        if let Some(sha) = file_sha256(&root.join(&qg_path)) {
            if let Some(entry) = evidence.get_mut("SD-EVID-003") {
                entry.add_content_hash(&qg_path, &sha);
            }
        }
        // Architecture doc → SD-CODE-003 (API alignment source of truth)
        let arch_path = format!("output/{}-architecture.md", inputs.slug);
        if let Some(sha) = file_sha256(&root.join(&arch_path)) {
            if let Some(entry) = evidence.get_mut("SD-CODE-003") {
                entry.add_content_hash(&arch_path, &sha);
            }
        }
        // Tool-call audit → SD-EVID-002
        let tc = ".super-dev/audit/tool-calls.jsonl";
        if let Some(sha) = file_sha256(&root.join(tc)) {
            if let Some(entry) = evidence.get_mut("SD-EVID-002") {
                entry.add_content_hash(tc, &sha);
            }
        }
    }

    let clauses: Vec<ClauseEvidence> = evidence.into_values().collect();
    let total_clauses_fired = clauses.len();

    ComplianceMapping {
        spec_version: SPEC_VERSION.to_string(),
        slug: inputs.slug.to_string(),
        generated_at: inputs
            .generated_at
            .clone()
            .unwrap_or_else(|| Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        declared_by: inputs
            .declared_by
            .clone()
            .unwrap_or_else(|| concat!("super-dev@", env!("CARGO_PKG_VERSION")).to_string()),
        quality_gate_passed: quality_passed,
        clauses,
        summary: ComplianceSummary {
            total_clauses_fired,
            total_tool_calls: inputs.tool_calls.len(),
            total_api_audit_rows: inputs.api_calls.len(),
            frameworks: vec![
                "SOC 2 (2017 TSC)".to_string(),
                "ISO/IEC 27001:2022".to_string(),
                "EU AI Act".to_string(),
            ],
        },
    }
}

/// I/O wrapper: read evidence from disk, build the mapping, write it.
/// Returns `Some((output_path, document))` on success, `None` when
/// there is no evidence at all.
#[must_use]
pub fn write_compliance_mapping(
    project_root: &Path,
    slug: &str,
) -> Option<(PathBuf, ComplianceMapping)> {
    // Sanitize the slug before it's interpolated into a filename — a slug
    // containing `..` or path separators would otherwise write outside
    // `output/` (path traversal). The slug derives from the workspace dir
    // name or `--slug`, which we don't fully control.
    let safe_slug = sanitize_slug(slug);
    let quality_path = project_root
        .join("output")
        .join(format!("{safe_slug}-quality-gate.json"));
    let quality_raw = fs::read_to_string(&quality_path).ok();
    let quality_value: Option<serde_json::Value> = quality_raw
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());

    let mut tool_calls = read_jsonl::<ToolCallRecord>(
        &project_root
            .join(".super-dev")
            .join("audit")
            .join("tool-calls.jsonl"),
    );
    let mut api_calls = read_jsonl::<ApiCallRecord>(
        &project_root
            .join(".super-dev")
            .join("audit")
            .join("frontend-api-calls.jsonl"),
    );
    // Sort by ts_ms (then ts) so two calls sharing a second still order
    // deterministically by sub-second arrival. Old rows without ts_ms (0)
    // fall back to their second-granularity ts.
    tool_calls.sort_by_key(|r| (r.ts_ms, r.ts));
    api_calls.sort_by_key(|r| (r.ts_ms, r.ts));

    if quality_value.is_none() && tool_calls.is_empty() && api_calls.is_empty() {
        return None;
    }

    let doc = build_compliance_mapping(&ComplianceInputs {
        slug: &safe_slug,
        quality_report: quality_value.as_ref(),
        tool_calls: &tool_calls,
        api_calls: &api_calls,
        generated_at: None,
        declared_by: None,
        project_root: Some(project_root),
    });

    let out_dir = project_root.join("output");
    let _ = fs::create_dir_all(&out_dir);
    let out_path = out_dir.join(format!("{safe_slug}-compliance-mapping.json"));
    if let Ok(text) = serde_json::to_string_pretty(&doc) {
        atomic_write(&out_path, &text);
    }
    Some((out_path, doc))
}

/// Reduce a project slug to a filename-safe component: strip path
/// separators, `..` traversal segments, and other shell/path metacharacters.
/// Used wherever a slug is interpolated into a filesystem path so a hostile
/// oraccidental slug can't escape the output directory.
fn sanitize_slug(slug: &str) -> String {
    let cleaned: String = slug
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    // Collapse any `..` (even dot-underscore-separated) that could traverse.
    let no_traversal = cleaned.replace("..", "_");
    if no_traversal.is_empty() || no_traversal.chars().all(|c| c == '_' || c == '.') {
        "project".to_string()
    } else {
        no_traversal
    }
}

/// Atomically write `content` to `path` (write to a temp file in the same
/// dir, then rename). Same-filesystem rename is atomic on POSIX, so a
/// concurrent reader never sees a half-written compliance-mapping file.
/// Falls back to a direct write on cross-filesystem rename failure.
fn atomic_write(path: &Path, content: &str) {
    let tmp = path.with_extension("tmp-write");
    if fs::write(&tmp, content).is_err() {
        // Can't even write the temp — fall back to direct write.
        let _ = fs::write(path, content);
        return;
    }
    if fs::rename(&tmp, path).is_err() {
        let _ = fs::remove_file(&tmp);
        let _ = fs::write(path, content);
    }
}

fn read_jsonl<T>(path: &Path) -> Vec<T>
where
    T: serde::de::DeserializeOwned,
{
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<T>(line).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;

    fn fake_tool_call(clause: &str, decision: &str) -> ToolCallRecord {
        ToolCallRecord {
            ts: 1,
            ts_ms: 1000,
            tool: "Write".into(),
            file: "x.tsx".into(),
            decision: decision.into(),
            clause: clause.into(),
            reason: String::new(),
            session_id: String::new(),
        }
    }

    #[test]
    fn frameworks_for_known_clause() {
        let fw = framework_for("SD-CODE-001");
        assert_eq!(fw.soc2_cc, vec!["CC8.1"]);
        assert_eq!(fw.iso27001_annex_a, vec!["A.5.34", "A.8.28"]);
        assert_eq!(fw.eu_ai_act_article, vec!["Article 15"]);
    }

    #[test]
    fn frameworks_default_for_unknown_clause() {
        assert_eq!(
            framework_for("SD-CODE-999"),
            ComplianceFrameworks::default()
        );
    }

    #[test]
    fn build_aggregates_tool_calls() {
        let calls = vec![
            fake_tool_call("SD-CODE-001", "block"),
            fake_tool_call("SD-CODE-001", "block"),
            fake_tool_call("SD-CODE-002", "block"),
        ];
        let doc = build_compliance_mapping(&ComplianceInputs {
            slug: "demo",
            quality_report: None,
            tool_calls: &calls,
            api_calls: &[],
            generated_at: Some("2026-05-20T00:00:00Z".into()),
            declared_by: Some("super-dev@4.4.0".into()),
            project_root: None,
        });
        assert_eq!(doc.summary.total_clauses_fired, 2);
        let by_id: BTreeMap<_, _> = doc.clauses.iter().map(|c| (c.id.as_str(), c)).collect();
        assert_eq!(by_id["SD-CODE-001"].fired_count, 2);
        assert_eq!(by_id["SD-CODE-001"].decisions["block"], 2);
        assert_eq!(by_id["SD-CODE-002"].fired_count, 1);
    }

    #[test]
    fn build_includes_api_audit_against_two_clauses() {
        let api = vec![ApiCallRecord {
            ts: 1,
            ts_ms: 1000,
            file: "src/U.tsx".into(),
            tool: "Write".into(),
            urls: vec!["/api/users".into(), "/api/orders".into()],
            session_id: String::new(),
        }];
        let doc = build_compliance_mapping(&ComplianceInputs {
            slug: "demo",
            quality_report: None,
            tool_calls: &[],
            api_calls: &api,
            generated_at: Some("t".into()),
            declared_by: None,
            project_root: None,
        });
        let ids: Vec<_> = doc.clauses.iter().map(|c| c.id.clone()).collect();
        assert!(ids.contains(&"SD-CODE-003".to_string()));
        assert!(ids.contains(&"SD-EVID-001".to_string()));
        for c in &doc.clauses {
            if c.id == "SD-CODE-003" || c.id == "SD-EVID-001" {
                assert_eq!(c.fired_count, 2);
                assert!(c
                    .evidence
                    .iter()
                    .any(|p| p.ends_with("frontend-api-calls.jsonl")));
            }
        }
    }

    #[test]
    fn build_records_quality_gate_outcome() {
        let q = serde_json::json!({"passed": true, "total_score": 95});
        let doc = build_compliance_mapping(&ComplianceInputs {
            slug: "demo",
            quality_report: Some(&q),
            tool_calls: &[],
            api_calls: &[],
            generated_at: Some("t".into()),
            declared_by: None,
            project_root: None,
        });
        assert_eq!(doc.quality_gate_passed, Some(true));
        let evid3 = doc.clauses.iter().find(|c| c.id == "SD-EVID-003").unwrap();
        assert_eq!(evid3.fired_count, 1);
        assert!(evid3.decisions.contains_key("passed"));
    }

    #[test]
    fn build_skips_unrecognised_clauses() {
        let calls = vec![fake_tool_call("SD-CODE-999", "block")];
        let doc = build_compliance_mapping(&ComplianceInputs {
            slug: "demo",
            quality_report: None,
            tool_calls: &calls,
            api_calls: &[],
            generated_at: Some("t".into()),
            declared_by: None,
            project_root: None,
        });
        assert!(doc.clauses.is_empty());
    }

    #[test]
    fn content_sha256_is_deterministic() {
        let a = content_sha256("hello world");
        let b = content_sha256("hello world");
        let c = content_sha256("hello earth");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64); // 32 bytes hex = 64 chars
    }

    #[test]
    fn file_sha256_returns_none_for_missing() {
        assert!(file_sha256(std::path::Path::new("/nonexistent/x.json")).is_none());
    }

    #[test]
    fn build_attaches_content_hashes_when_root_set() {
        use std::fs;
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("output")).unwrap();
        fs::create_dir_all(root.join(".super-dev/audit")).unwrap();
        fs::write(
            root.join("output/demo-quality-gate.json"),
            r#"{"passed":true,"total_score":95}"#,
        )
        .unwrap();
        fs::write(
            root.join("output/demo-architecture.md"),
            "# Arch\n\n## API\n\n| GET | /api/users |",
        )
        .unwrap();
        fs::write(root.join(".super-dev/audit/tool-calls.jsonl"), "").unwrap();

        let q = serde_json::json!({"passed": true, "total_score": 95});
        let doc = build_compliance_mapping(&ComplianceInputs {
            slug: "demo",
            quality_report: Some(&q),
            tool_calls: &[],
            api_calls: &[],
            generated_at: Some("t".into()),
            declared_by: None,
            project_root: Some(root),
        });
        // SD-EVID-003 must carry a content hash for the quality gate file.
        let evid3 = doc.clauses.iter().find(|c| c.id == "SD-EVID-003").unwrap();
        assert!(
            evid3
                .content_hashes
                .iter()
                .any(|h| h.starts_with("output/demo-quality-gate.json:")),
            "expected quality-gate content hash, got {:?}",
            evid3.content_hashes
        );
    }

    #[test]
    fn sanitize_slug_strips_path_traversal() {
        // A slug with path separators / traversal must collapse to a safe
        // filename component so write_compliance_mapping can't escape output/.
        use super::sanitize_slug;
        assert_eq!(sanitize_slug("demo"), "demo");
        assert_eq!(sanitize_slug("my-app_2"), "my-app_2");
        // `..` traversal collapsed.
        assert!(!sanitize_slug("..").contains(".."));
        assert!(!sanitize_slug("../etc/passwd").contains(".."));
        assert!(!sanitize_slug("..").contains('/'));
        // Path separators replaced.
        assert!(!sanitize_slug("a/b\\c").contains('/') && !sanitize_slug("a/b\\c").contains('\\'));
        // Empty / all-junk → fallback.
        assert_eq!(sanitize_slug(""), "project");
        assert_eq!(sanitize_slug("   "), "project");
        assert_eq!(sanitize_slug("...."), "project");
    }

    #[test]
    fn write_compliance_mapping_cannot_escape_output_dir() {
        // A hostile slug must NOT let the file land outside output/.
        use std::fs;
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("output")).unwrap();
        fs::create_dir_all(root.join(".super-dev/audit")).unwrap();
        fs::write(
            root.join("output/demo-quality-gate.json"),
            r#"{"passed":true,"total_score":95}"#,
        )
        .unwrap();
        fs::write(
            root.join(".super-dev/audit/tool-calls.jsonl"),
            r#"{"ts":0,"tool":"x","target":"y","decision":"allow","clause":"SD-CODE-001","reason":"r","detail":""}"#,
        )
        .unwrap();
        let (path, _doc) = write_compliance_mapping(root, "../../etc/evil").unwrap();
        // The written path must be inside <root>/output/.
        let out_dir = root.join("output");
        assert!(
            path.starts_with(&out_dir),
            "compliance mapping escaped output dir: {}",
            path.display()
        );
    }
}
