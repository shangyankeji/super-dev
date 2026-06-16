//! Tech-debt ledger — tracks placeholder/TODO markers as persistent debt
//! items across runs, not just as a transient quality-gate count.
//!
//! Until 4.6 `count_placeholder_markers` returned a single `usize` that fed
//! one quality-check row and was then discarded. There was no memory of
//! *which* file had *what* TODO, no way to see if debt was growing or
//! shrinking across runs, and no "resolved" tracking when a worker filled
//! in a placeholder.
//!
//! This module turns placeholder detection into a first-class ledger:
//! - [`scan_debt`] walks `output/*.md` and returns structured [`DebtItem`]
//!   records (file, line, marker kind, severity).
//! - [`write_ledger`] persists them to `.super-dev/tech-debt.jsonl` —
//!   append-only, one row per marker, with a `first_seen` timestamp so the
//!   age of each debt item is queryable.
//! - [`read_ledger`] loads the full history for reporting.
//!
//! The quality gate still calls `count_placeholder_markers` for its score,
//! but now ALSO writes the structured ledger, so a reviewer can answer
//! "show me every unresolved TODO introduced in the last 3 runs".

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// On-disk ledger location, relative to project root.
pub const DEBT_LEDGER: &str = ".super-dev/tech-debt.jsonl";

/// The kind of placeholder marker found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebtKind {
    /// A literal `TODO` token.
    Todo,
    /// A `| TODO |` table cell.
    TodoCell,
    /// A `PLACEHOLDER` token.
    Placeholder,
    /// Lorem ipsum / filler text.
    FillerText,
    /// An unfilled Given/When/Then acceptance criterion.
    UnfilledAcceptance,
}

/// Resolution status of a debt item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebtStatus {
    /// Still present in the latest scan.
    Open,
    /// Was present in a prior run but absent now — resolved.
    Resolved,
}

impl DebtKind {
    /// Numeric severity (1=minor … 5=critical). Filler text and unfilled
    /// acceptance criteria are weighted higher because they signal a doc that
    /// can't be acted on, not just an incomplete note.
    #[must_use]
    pub fn severity(self) -> u8 {
        match self {
            Self::Todo => 2,
            Self::TodoCell => 3,
            Self::Placeholder => 3,
            Self::FillerText => 5,
            Self::UnfilledAcceptance => 4,
        }
    }
}

/// One debt item: a placeholder marker at a specific location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebtItem {
    /// Workspace-relative file path (e.g. `output/demo-prd.md`).
    pub file: String,
    /// 1-based line number.
    pub line: u32,
    /// What kind of marker.
    pub kind: DebtKind,
    /// The matching text (trimmed, capped at 80 chars for readability).
    pub snippet: String,
    /// ISO-8601 UTC timestamp when first seen.
    pub first_seen: String,
    /// Resolution status. Defaults to `Open` for new items; old JSONL rows
    /// without this field deserialize as `Open` (backwards-compatible).
    #[serde(default = "default_status")]
    pub status: DebtStatus,
    /// ISO-8601 UTC timestamp when the item was resolved. Empty when Open.
    #[serde(default)]
    pub resolved_at: String,
}

/// Default status for deserialising old ledger rows that predate the field.
fn default_status() -> DebtStatus {
    DebtStatus::Open
}

/// Scan `output/*.md` for placeholder markers, returning structured debt items.
///
/// Reads the markdown files under `output_dir` (this IS I/O, despite an
/// earlier doc comment claiming "pure / no I/O" — that was inaccurate and
/// contradicted the implementation). Classification is pure once a file's
/// text is in hand; persistence is left to the caller via [`write_ledger`].
#[must_use]
pub fn scan_debt(output_dir: &Path) -> Vec<DebtItem> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut items = Vec::new();
    let Ok(rd) = fs::read_dir(output_dir) else {
        return items;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let rel = p
            .strip_prefix(output_dir.parent().unwrap_or(Path::new("")))
            .unwrap_or(&p)
            .to_string_lossy()
            .to_string();
        let Ok(content) = fs::read_to_string(&p) else {
            continue;
        };
        for (i, line) in content.lines().enumerate() {
            let lineno = u32::try_from(i + 1).unwrap_or(0);
            let lower = line.to_ascii_lowercase();
            // Classify the strongest marker on this line (one item per line).
            if let Some(kind) = classify_line(line, &lower) {
                items.push(DebtItem {
                    file: rel.clone(),
                    line: lineno,
                    kind,
                    snippet: line.trim().chars().take(80).collect(),
                    first_seen: now.clone(),
                    status: DebtStatus::Open,
                    resolved_at: String::new(),
                });
            }
        }
    }
    items
}

/// Classify a single line's strongest placeholder marker. Returns `None`
/// when the line has none. Priority: filler > unfilled-acceptance > cell > todo.
fn classify_line(line: &str, lower: &str) -> Option<DebtKind> {
    if lower.contains("lorem ipsum") || lower.contains("dolor sit amet") {
        return Some(DebtKind::FillerText);
    }
    if lower.contains("given todo") {
        return Some(DebtKind::UnfilledAcceptance);
    }
    if line.contains("| TODO |") {
        return Some(DebtKind::TodoCell);
    }
    if line.contains("PLACEHOLDER") {
        return Some(DebtKind::Placeholder);
    }
    if line.contains("TODO") {
        return Some(DebtKind::Todo);
    }
    None
}

/// Append `items` to `.super-dev/tech-debt.jsonl`. One JSON object per line.
/// Returns the path written. Best-effort: a write failure returns the path
/// anyway (the quality gate already has the in-memory items).
pub fn write_ledger(project_root: &Path, items: &[DebtItem]) -> PathBuf {
    let path = project_root.join(DEBT_LEDGER);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        for item in items {
            if let Ok(line) = serde_json::to_string(item) {
                let _ = writeln!(f, "{line}");
            }
        }
    }
    path
}

/// Read the full debt ledger history. Returns an empty vec when missing or
/// malformed (fail-open).
#[must_use]
pub fn read_ledger(project_root: &Path) -> Vec<DebtItem> {
    let path = project_root.join(DEBT_LEDGER);
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<DebtItem>(l).ok())
        .collect()
}

/// Summarise the ledger: counts by kind + total severity weight. Used by
/// the quality gate and `super-dev report`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DebtSummary {
    /// Total items.
    pub total: usize,
    /// Sum of all item severities (a rough "debt score").
    pub severity_total: u64,
    /// Count per kind.
    pub by_kind: std::collections::BTreeMap<String, usize>,
}

#[must_use]
/// Summarise the ledger into counts + severity weight.
pub fn summarise(items: &[DebtItem]) -> DebtSummary {
    let mut by_kind = std::collections::BTreeMap::new();
    for item in items {
        *by_kind
            .entry(
                serde_json::to_string(&item.kind)
                    .unwrap_or_else(|_| "unknown".into())
                    .trim_matches('"')
                    .to_string(),
            )
            .or_insert(0) += 1;
        let _ = item;
    }
    let severity_total: u64 = items.iter().map(|i| u64::from(i.kind.severity())).sum();
    DebtSummary {
        total: items.len(),
        severity_total,
        by_kind,
    }
}

/// A trend snapshot: how debt changed between the prior ledger run and the
/// current scan. Used by the quality gate's "Tech debt trend" check and the
/// `super-dev report` command.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DebtTrend {
    /// Items present in the current scan (all Open).
    pub current_count: usize,
    /// Items that were in the prior ledger but NOT in the current scan → resolved.
    pub resolved_count: usize,
    /// Items in the current scan that were NOT in the prior ledger → newly introduced.
    pub new_count: usize,
    /// Net change: `new_count - resolved_count`. Negative = debt shrinking.
    pub net_change: i64,
}

/// Compare the current scan against the prior ledger to compute a trend.
/// Items are matched by `(file, line, kind)`. An item in the ledger but not
/// the scan is "resolved"; an item in the scan but not the ledger is "new".
#[must_use]
pub fn diff_against_ledger(current: &[DebtItem], ledger: &[DebtItem]) -> DebtTrend {
    let current_keys: std::collections::HashSet<(&str, u32, DebtKind)> = current
        .iter()
        .map(|d| (d.file.as_str(), d.line, d.kind))
        .collect();
    let ledger_keys: std::collections::HashSet<(&str, u32, DebtKind)> = ledger
        .iter()
        .map(|d| (d.file.as_str(), d.line, d.kind))
        .collect();

    let resolved_count = ledger_keys.difference(&current_keys).count();
    let new_count = current_keys.difference(&ledger_keys).count();
    let net_change = i64::try_from(new_count).unwrap_or(i64::MAX)
        - i64::try_from(resolved_count).unwrap_or(i64::MAX);

    DebtTrend {
        current_count: current.len(),
        resolved_count,
        new_count,
        net_change,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn seed(root: &Path, name: &str, body: &str) {
        let dir = root.join("output");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(name), body).unwrap();
    }

    #[test]
    fn scan_finds_all_marker_kinds() {
        let tmp = TempDir::new().unwrap();
        seed(
            tmp.path(),
            "prd.md",
            "# PRD\n\n| F1 | TODO | P0 | TODO |\n\nTODO: define scope\n\nLorem ipsum dolor sit amet\n\n- [ ] Given TODO, when TODO, then TODO\n",
        );
        let items = scan_debt(&tmp.path().join("output"));
        // todo_cell (line 3), todo (line 5), filler (line 7), unfilled (line 9)
        let kinds: Vec<DebtKind> = items.iter().map(|i| i.kind).collect();
        assert!(kinds.contains(&DebtKind::TodoCell));
        assert!(kinds.contains(&DebtKind::Todo));
        assert!(kinds.contains(&DebtKind::FillerText));
        assert!(kinds.contains(&DebtKind::UnfilledAcceptance));
    }

    #[test]
    fn scan_skips_clean_files() {
        let tmp = TempDir::new().unwrap();
        seed(
            tmp.path(),
            "good.md",
            "# Real PRD\n\nReal content with no placeholders.\n",
        );
        assert!(scan_debt(&tmp.path().join("output")).is_empty());
    }

    #[test]
    fn scan_skips_non_markdown() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("output");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("data.json"), "{\"x\":\"TODO\"}").unwrap();
        assert!(scan_debt(&dir).is_empty());
    }

    #[test]
    fn classify_priority_filler_over_todo() {
        // A line with both lorem ipsum AND TODO → filler wins.
        assert_eq!(
            classify_line("TODO lorem ipsum", "todo lorem ipsum"),
            Some(DebtKind::FillerText)
        );
    }

    #[test]
    fn severity_weighting() {
        assert!(DebtKind::FillerText.severity() > DebtKind::Todo.severity());
        assert!(DebtKind::UnfilledAcceptance.severity() > DebtKind::TodoCell.severity());
    }

    #[test]
    fn write_then_read_round_trip() {
        let tmp = TempDir::new().unwrap();
        let items = vec![
            DebtItem {
                file: "output/a.md".into(),
                line: 3,
                kind: DebtKind::Todo,
                snippet: "TODO: fix".into(),
                first_seen: "2026-06-14T00:00:00Z".into(),
                status: DebtStatus::Open,
                resolved_at: String::new(),
            },
            DebtItem {
                file: "output/b.md".into(),
                line: 1,
                kind: DebtKind::FillerText,
                snippet: "lorem ipsum".into(),
                first_seen: "2026-06-14T00:00:00Z".into(),
                status: DebtStatus::Open,
                resolved_at: String::new(),
            },
        ];
        let path = write_ledger(tmp.path(), &items);
        assert!(path.is_file());
        let back = read_ledger(tmp.path());
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].kind, DebtKind::Todo);
        assert_eq!(back[1].file, "output/b.md");
    }

    #[test]
    fn read_missing_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(read_ledger(tmp.path()).is_empty());
    }

    #[test]
    fn write_appends_across_calls() {
        let tmp = TempDir::new().unwrap();
        write_ledger(
            tmp.path(),
            &[DebtItem {
                file: "a".into(),
                line: 1,
                kind: DebtKind::Todo,
                snippet: "x".into(),
                first_seen: "t".into(),
                status: DebtStatus::Open,
                resolved_at: String::new(),
            }],
        );
        write_ledger(
            tmp.path(),
            &[DebtItem {
                file: "b".into(),
                line: 2,
                kind: DebtKind::Todo,
                snippet: "y".into(),
                first_seen: "t".into(),
                status: DebtStatus::Open,
                resolved_at: String::new(),
            }],
        );
        assert_eq!(read_ledger(tmp.path()).len(), 2);
    }

    #[test]
    fn summarise_counts_by_kind() {
        let items = vec![
            DebtItem {
                file: "a".into(),
                line: 1,
                kind: DebtKind::Todo,
                snippet: "x".into(),
                first_seen: "t".into(),
                status: DebtStatus::Open,
                resolved_at: String::new(),
            },
            DebtItem {
                file: "b".into(),
                line: 2,
                kind: DebtKind::Todo,
                snippet: "y".into(),
                first_seen: "t".into(),
                status: DebtStatus::Open,
                resolved_at: String::new(),
            },
            DebtItem {
                file: "c".into(),
                line: 3,
                kind: DebtKind::FillerText,
                snippet: "z".into(),
                first_seen: "t".into(),
                status: DebtStatus::Open,
                resolved_at: String::new(),
            },
        ];
        let s = summarise(&items);
        assert_eq!(s.total, 3);
        assert_eq!(s.by_kind.get("todo"), Some(&2));
        assert_eq!(s.by_kind.get("filler_text"), Some(&1));
        // filler (5) + todo (2) + todo (2) = 9
        assert_eq!(s.severity_total, 9);
    }

    #[test]
    fn diff_detects_new_and_resolved() {
        // Prior ledger had 2 items; current scan has 2 items — one overlaps,
        // one is new, one was resolved.
        let ledger = vec![
            DebtItem {
                file: "output/a.md".into(),
                line: 3,
                kind: DebtKind::Todo,
                snippet: "x".into(),
                first_seen: "t".into(),
                status: DebtStatus::Open,
                resolved_at: String::new(),
            },
            DebtItem {
                file: "output/b.md".into(),
                line: 1,
                kind: DebtKind::FillerText,
                snippet: "y".into(),
                first_seen: "t".into(),
                status: DebtStatus::Open,
                resolved_at: String::new(),
            },
        ];
        let current = vec![DebtItem {
            file: "output/a.md".into(),
            line: 3,
            kind: DebtKind::Todo,
            snippet: "x".into(),
            first_seen: "t".into(),
            status: DebtStatus::Open,
            resolved_at: String::new(),
            // b.md is gone (resolved), c.md is new.
        }];
        let current = {
            let mut c = current;
            c.push(DebtItem {
                file: "output/c.md".into(),
                line: 5,
                kind: DebtKind::Placeholder,
                snippet: "z".into(),
                first_seen: "t".into(),
                status: DebtStatus::Open,
                resolved_at: String::new(),
            });
            c
        };
        let trend = diff_against_ledger(&current, &ledger);
        assert_eq!(trend.current_count, 2);
        assert_eq!(trend.resolved_count, 1); // b.md gone
        assert_eq!(trend.new_count, 1); // c.md added
        assert_eq!(trend.net_change, 0); // 1 new - 1 resolved
    }

    #[test]
    fn diff_all_new_when_no_prior_ledger() {
        let current = vec![DebtItem {
            file: "a".into(),
            line: 1,
            kind: DebtKind::Todo,
            snippet: "x".into(),
            first_seen: "t".into(),
            status: DebtStatus::Open,
            resolved_at: String::new(),
        }];
        let trend = diff_against_ledger(&current, &[]);
        assert_eq!(trend.current_count, 1);
        assert_eq!(trend.new_count, 1);
        assert_eq!(trend.resolved_count, 0);
    }
}
