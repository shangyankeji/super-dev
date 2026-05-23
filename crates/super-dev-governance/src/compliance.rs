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
    let quality_path = project_root
        .join("output")
        .join(format!("{slug}-quality-gate.json"));
    let quality_raw = fs::read_to_string(&quality_path).ok();
    let quality_value: Option<serde_json::Value> = quality_raw
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());

    let tool_calls = read_jsonl::<ToolCallRecord>(
        &project_root
            .join(".super-dev")
            .join("audit")
            .join("tool-calls.jsonl"),
    );
    let api_calls = read_jsonl::<ApiCallRecord>(
        &project_root
            .join(".super-dev")
            .join("audit")
            .join("frontend-api-calls.jsonl"),
    );

    if quality_value.is_none() && tool_calls.is_empty() && api_calls.is_empty() {
        return None;
    }

    let doc = build_compliance_mapping(&ComplianceInputs {
        slug,
        quality_report: quality_value.as_ref(),
        tool_calls: &tool_calls,
        api_calls: &api_calls,
        generated_at: None,
        declared_by: None,
    });

    let out_dir = project_root.join("output");
    let _ = fs::create_dir_all(&out_dir);
    let out_path = out_dir.join(format!("{slug}-compliance-mapping.json"));
    if let Ok(text) = serde_json::to_string_pretty(&doc) {
        let _ = fs::write(&out_path, text);
    }
    Some((out_path, doc))
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
        });
        assert!(doc.clauses.is_empty());
    }
}
