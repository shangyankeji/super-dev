//! Parse the architecture Markdown's API surface table into a typed
//! [`ApiSpec`].
//!
//! The current architecture doc (`render_architecture` in super-dev-agent)
//! emits a Markdown table:
//!
//! ```text
//! | Method | Path | Request | Response | Auth | Description |
//! |---|---|---|---|---|---|
//! | POST | /api/auth/login | { email, password } | { token, user } | none | Login |
//! ```
//!
//! This module upgrades the fragile `line.starts_with('|')` + `split('|')`
//! parser (which mis-extracted any `/`-containing cell as a path) into a
//! column-aware parser that validates method verbs, dedupes endpoints, and
//! produces typed [`Endpoint`] records.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// An HTTP method, restricted to the verbs OpenAPI allows in a path item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpVerb {
    /// `GET`
    Get,
    /// `POST`
    Post,
    /// `PUT`
    Put,
    /// `DELETE`
    Delete,
    /// `PATCH`
    Patch,
    /// `OPTIONS`
    Options,
    /// `HEAD`
    Head,
}

impl HttpVerb {
    /// Parse a method string (case-insensitive). Returns `None` for anything
    /// that isn't a standard HTTP verb.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "DELETE" => Some(Self::Delete),
            "PATCH" => Some(Self::Patch),
            "OPTIONS" => Some(Self::Options),
            "HEAD" => Some(Self::Head),
            _ => None,
        }
    }

    /// Lowercase identifier used as the OpenAPI path-item key.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Delete => "delete",
            Self::Patch => "patch",
            Self::Options => "options",
            Self::Head => "head",
        }
    }
}

/// How an endpoint authenticates. Upgrades the free-text Auth column
/// (`bearer` / `none` / `jwt`) into a typed enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SecurityKind {
    /// No authentication required (public endpoint).
    None,
    /// Bearer token (JWT or opaque) in the Authorization header.
    Bearer,
    /// API key in a header / query / cookie.
    ApiKey,
    /// OAuth 2.0 with a named flow.
    OAuth2,
    /// Session cookie (server-rendered apps).
    Session,
    /// Unrecognised auth description — kept as a catch-all so parsing never
    /// drops an endpoint entirely.
    Other,
}

impl SecurityKind {
    /// Parse the Auth column text into a security kind. Tolerant: common
    /// synonyms (`jwt` → Bearer, `token` → Bearer) are mapped.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        let lower = s.trim().to_ascii_lowercase();
        if lower.is_empty() || lower == "none" || lower == "no" || lower == "public" {
            return Self::None;
        }
        if lower.contains("bearer") || lower.contains("jwt") || lower.contains("token") {
            return Self::Bearer;
        }
        if lower.contains("api key")
            || lower.contains("apikey")
            || lower.contains("api-key")
            || lower == "key"
        {
            return Self::ApiKey;
        }
        if lower.contains("oauth") {
            return Self::OAuth2;
        }
        if lower.contains("session") || lower.contains("cookie") {
            return Self::Session;
        }
        Self::Other
    }
}

/// One API endpoint: method + path + metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Endpoint {
    /// HTTP method.
    pub method: HttpVerb,
    /// Path template, e.g. `/api/users/:id`. Always starts with `/`.
    pub path: String,
    /// Stable operation identifier (e.g. `listUsers`). Derived from the
    /// description when the doc doesn't give one.
    pub operation_id: String,
    /// Human-readable description (the Description column).
    pub description: String,
    /// Request body shape as free text (the Request column). Empty for GET.
    pub request_shape: String,
    /// Response body shape as free text (the Response column).
    pub response_shape: String,
    /// Auth requirement.
    pub security: SecurityKind,
}

/// The full typed API contract extracted from the architecture doc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiSpec {
    /// All endpoints, deduped by `(method, path)`.
    pub endpoints: Vec<Endpoint>,
    /// API title from the architecture doc H1 (best-effort).
    pub title: String,
}

impl ApiSpec {
    /// Whether the contract declares an endpoint matching `method` + `path`.
    /// Path templates match literal segments: `/api/users/:id` matches a
    /// call to `/api/users/123` but not `/api/users`.
    #[must_use]
    pub fn has_endpoint(&self, method: HttpVerb, call_path: &str) -> bool {
        self.endpoints
            .iter()
            .any(|e| e.method == method && path_template_matches(&e.path, call_path))
    }

    /// All unique `(method, path)` pairs declared. Used by validators.
    #[must_use]
    pub fn declared_paths(&self) -> Vec<(HttpVerb, &str)> {
        self.endpoints
            .iter()
            .map(|e| (e.method, e.path.as_str()))
            .collect()
    }

    /// Number of endpoints.
    #[must_use]
    pub fn len(&self) -> usize {
        self.endpoints.len()
    }

    /// Whether the contract is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }
}

/// Does a declared path template match an actual call path?
///
/// - `/api/users` matches only `/api/users`.
/// - `/api/users/:id` matches `/api/users/123` and `/api/users/abc`.
/// - `/api/:org/:repo` matches `/api/foo/bar`.
/// - A literal segment does NOT match a different literal (`/api/users`
///   ≠ `/api/orders`).
fn path_template_matches(template: &str, call_path: &str) -> bool {
    // Strip query strings / fragments from the call path.
    let call_path = call_path.split(['?', '#']).next().unwrap_or(call_path);
    let template_segments: Vec<&str> = template.trim_end_matches('/').split('/').collect();
    let call_segments: Vec<&str> = call_path.trim_end_matches('/').split('/').collect();
    if template_segments.len() != call_segments.len() {
        return false;
    }
    template_segments
        .iter()
        .zip(call_segments.iter())
        .all(|(t, c)| is_template_param(t) || t == c)
}

/// Whether a template segment is a `:param` placeholder. Requires `:` to be
/// followed by at least one valid param-name char (letter / underscore /
/// digit), so a bare `:`, `::`, or `:#` is treated as a literal segment and
/// must match the call segment exactly (previously anything starting with
/// `:` matched anything, so `/api/:` would wrongly match `/api/foo`).
fn is_template_param(segment: &str) -> bool {
    let mut chars = segment.chars();
    matches!(chars.next(), Some(':'))
        && chars
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Parse the architecture Markdown into an [`ApiSpec`]. Looks for a Markdown
/// table whose header row contains `Method` and `Path` columns, then parses
/// each data row into a typed [`Endpoint`].
///
/// Tolerant: malformed rows are skipped, never cause failure. An architecture
/// doc with no API table yields an empty spec (the quality gate then reports
/// "no contract found").
#[must_use]
pub fn parse_architecture(arch_markdown: &str, title: &str) -> ApiSpec {
    let mut endpoints = extract_endpoints_from_table(arch_markdown);
    // Ensure operationIds are unique across the whole spec — OpenAPI requires
    // this, but `derive_operation_id` slugifies the description, so two
    // endpoints sharing a description (e.g. both "List") would collide.
    dedupe_operation_ids(&mut endpoints);
    ApiSpec {
        endpoints,
        title: title.to_string(),
    }
}

/// Disambiguate duplicate `operation_id`s in-place by appending a numeric
/// suffix (`_2`, `_3`, …) to every occurrence after the first. An endpoint
/// whose id is already unique is left untouched.
pub(crate) fn dedupe_operation_ids(endpoints: &mut [Endpoint]) {
    use std::collections::HashMap;
    let mut seen: HashMap<String, usize> = HashMap::new();
    for ep in endpoints.iter_mut() {
        let base = ep.operation_id.clone();
        let count = seen.entry(base.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            ep.operation_id = format!("{base}_{count}");
        }
    }
}

/// Walk the markdown, find the first table with Method+Path columns, and
/// parse its rows.
fn extract_endpoints_from_table(md: &str) -> Vec<Endpoint> {
    let lines: Vec<&str> = md.lines().collect();
    // Find the header row: a line starting with `|` that contains both
    // "method" and "path" (case-insensitive).
    let header_idx = lines.iter().position(|l| {
        let lower = l.to_ascii_lowercase();
        l.trim().starts_with('|') && lower.contains("method") && lower.contains("path")
    });

    let Some(header_idx) = header_idx else {
        return Vec::new();
    };

    let headers = split_table_row(lines[header_idx]);
    let col = |name: &str| -> Option<usize> {
        headers
            .iter()
            .position(|h| h.to_ascii_lowercase().trim() == name)
    };
    // `col` returns Option but this fn returns Vec, so we can't use `?`.
    // Early-return empty when required columns are absent.
    let (Some(method_col), Some(path_col)) = (col("method"), col("path")) else {
        return Vec::new();
    };
    let req_col = col("request");
    let resp_col = col("response");
    let auth_col = col("auth");
    let desc_col = col("description");

    let mut endpoints: Vec<Endpoint> = Vec::new();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();

    // Skip the separator row (`|---|---|`) immediately after the header.
    for line in lines.iter().skip(header_idx + 1) {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            // Tables end at the first non-pipe line.
            if !endpoints.is_empty() {
                break;
            }
            continue;
        }
        if trimmed
            .replace('|', "")
            .chars()
            .all(|c| c == '-' || c.is_whitespace())
        {
            continue; // separator row
        }
        let cells = split_table_row(trimmed);
        if cells.len() <= method_col.max(path_col) {
            continue;
        }
        let Some(method) = HttpVerb::parse(&cells[method_col]) else {
            continue; // skip rows whose method isn't a real verb (e.g. "TODO")
        };
        let path = cells[path_col].trim().to_string();
        if !path.starts_with('/') {
            continue; // not a real API path
        }
        // Dedupe by (method, path).
        let key = (method.as_str().to_string(), path.clone());
        if !seen.insert(key) {
            continue;
        }
        let description = desc_col
            .and_then(|i| cells.get(i))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let request_shape = req_col
            .and_then(|i| cells.get(i))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let response_shape = resp_col
            .and_then(|i| cells.get(i))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let security = auth_col
            .and_then(|i| cells.get(i))
            .map(|s| SecurityKind::parse(s))
            .unwrap_or(SecurityKind::None);
        let operation_id = derive_operation_id(method, &path, &description);
        endpoints.push(Endpoint {
            method,
            path,
            operation_id,
            description,
            request_shape,
            response_shape,
            security,
        });
    }

    endpoints
}

/// Split a markdown table row `| a | b | c |` into `["a", "b", "c"]`.
/// Handles the outer pipes and trims each cell.
/// Split a markdown table row `| a | b | c |` into `["a", "b", "c"]`.
///
/// Handles the outer pipes, trims each cell, and — unlike a naive `split('|')`
/// — does NOT split on an escaped pipe `\|` inside a cell (e.g. a description
/// containing `OR\|fallback`), un-escaping it back to `|` in the result.
fn split_table_row(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'|') {
            // Escaped pipe — literal '|' in the cell, not a delimiter.
            chars.next();
            current.push('|');
        } else if c == '|' {
            cells.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(c);
        }
    }
    cells.push(current.trim().to_string());
    cells
}

/// Derive a stable operationId from method + path when the doc doesn't
/// provide one. E.g. `POST /api/auth/login` → `postApiAuthLogin`.
fn derive_operation_id(method: HttpVerb, path: &str, description: &str) -> String {
    // Prefer a slugified description when present.
    if !description.trim().is_empty() {
        let slug: String = description
            .trim()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { ' ' })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("_");
        if !slug.is_empty() {
            return slug.to_lowercase();
        }
    }
    // Fall back to method + path segments.
    let path_slug: String = path
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty() && !s.starts_with(':'))
        .collect::<Vec<_>>()
        .join("_");
    format!("{}_{}", method.as_str(), path_slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ARCH: &str = "# Architecture — demo

## API surface

| Method | Path | Request | Response | Auth | Description |
|---|---|---|---|---|---|
| GET | /api/health | - | { ok: true } | none | Health check |
| POST | /api/auth/login | { email, password } | { token, user } | none | Login |
| GET | /api/auth/me | - | { user } | bearer | Current user |
| DELETE | /api/users/:id | - | { deleted: true } | bearer | Delete user |
| TODO | /api/... | TODO | TODO | TODO | Add endpoints |

## Data model
";

    #[test]
    fn parses_real_table() {
        let spec = parse_architecture(SAMPLE_ARCH, "demo");
        assert_eq!(spec.len(), 4); // the TODO row is dropped
        assert_eq!(spec.endpoints[0].method, HttpVerb::Get);
        assert_eq!(spec.endpoints[0].path, "/api/health");
        assert_eq!(spec.endpoints[1].method, HttpVerb::Post);
        assert_eq!(spec.endpoints[1].path, "/api/auth/login");
    }

    #[test]
    fn parses_security_kinds() {
        let spec = parse_architecture(SAMPLE_ARCH, "demo");
        assert_eq!(spec.endpoints[0].security, SecurityKind::None); // health
        assert_eq!(spec.endpoints[2].security, SecurityKind::Bearer); // me
        assert_eq!(spec.endpoints[3].security, SecurityKind::Bearer); // delete user
    }

    #[test]
    fn derives_operation_ids() {
        let spec = parse_architecture(SAMPLE_ARCH, "demo");
        assert_eq!(spec.endpoints[0].operation_id, "health_check");
        assert_eq!(spec.endpoints[1].operation_id, "login");
    }

    #[test]
    fn split_table_row_handles_escaped_pipe() {
        // Regression: a description cell containing an escaped pipe (common
        // when noting alternatives like "POST\|GET") used to be split into
        // two cells, corrupting the column alignment.
        let cells = split_table_row("| GET | /api/x | do thing OR\\|fallback | none |");
        assert_eq!(
            cells.len(),
            4,
            "escaped pipe must not add a cell: {cells:?}"
        );
        assert!(
            cells[2].contains("OR|fallback"),
            "escaped pipe must be un-escaped to literal | in the cell: {cells:?}"
        );
    }

    #[test]
    fn operation_ids_disambiguated_when_descriptions_collide() {
        // Regression: two endpoints with the SAME description ("List") used
        // to both get operation_id = "list", violating OpenAPI uniqueness.
        // The table uses distinct paths so both rows survive dedup.
        let md = "| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/users | - | - | none | List |\n| GET | /api/posts | - | - | none | List |\n";
        let spec = parse_architecture(md, "t");
        assert_eq!(spec.len(), 2);
        let ids: Vec<&str> = spec
            .endpoints
            .iter()
            .map(|e| e.operation_id.as_str())
            .collect();
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(
            ids.len(),
            unique.len(),
            "operationIds must be unique, got {ids:?}"
        );
        // First keeps "list", second becomes "list_2".
        assert_eq!(spec.endpoints[0].operation_id, "list");
        assert_eq!(spec.endpoints[1].operation_id, "list_2");
    }

    #[test]
    fn dedupes_repeated_endpoints() {
        let md = "| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/x | - | - | none | First |\n| GET | /api/x | - | - | none | Duplicate |\n";
        let spec = parse_architecture(md, "t");
        assert_eq!(spec.len(), 1);
    }

    #[test]
    fn no_table_yields_empty_spec() {
        let md = "# Architecture\n\n## API surface\n\nNo table here.";
        assert!(parse_architecture(md, "t").is_empty());
    }

    #[test]
    fn skips_non_path_rows() {
        // A row whose path column doesn't start with `/` is dropped.
        let md = "| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | health | - | - | none | No slash |\n";
        assert!(parse_architecture(md, "t").is_empty());
    }

    #[test]
    fn path_template_matches_literal() {
        assert!(path_template_matches("/api/users", "/api/users"));
        assert!(!path_template_matches("/api/users", "/api/orders"));
    }

    #[test]
    fn path_template_matches_param() {
        assert!(path_template_matches("/api/users/:id", "/api/users/123"));
        assert!(path_template_matches(
            "/api/users/:id",
            "/api/users/abc-xyz"
        ));
        assert!(!path_template_matches("/api/users/:id", "/api/users"));
        assert!(!path_template_matches(
            "/api/users/:id",
            "/api/users/123/posts"
        ));
    }

    #[test]
    fn path_template_matches_multi_param() {
        assert!(path_template_matches("/api/:org/:repo", "/api/foo/bar"));
        assert!(!path_template_matches("/api/:org/:repo", "/api/foo"));
    }

    #[test]
    fn path_template_strips_query_string() {
        assert!(path_template_matches(
            "/api/users",
            "/api/users?include=email"
        ));
        assert!(path_template_matches("/api/users", "/api/users#section"));
    }

    #[test]
    fn path_template_trailing_slash_normalised() {
        assert!(path_template_matches("/api/users/", "/api/users"));
        assert!(path_template_matches("/api/users", "/api/users/"));
    }

    #[test]
    fn has_endpoint_finds_declared() {
        let spec = parse_architecture(SAMPLE_ARCH, "demo");
        assert!(spec.has_endpoint(HttpVerb::Get, "/api/health"));
        assert!(spec.has_endpoint(HttpVerb::Post, "/api/auth/login"));
        assert!(spec.has_endpoint(HttpVerb::Delete, "/api/users/42"));
        assert!(!spec.has_endpoint(HttpVerb::Put, "/api/health"));
        assert!(!spec.has_endpoint(HttpVerb::Get, "/api/nonexistent"));
    }

    #[test]
    fn security_kind_synonyms() {
        assert_eq!(SecurityKind::parse("none"), SecurityKind::None);
        assert_eq!(SecurityKind::parse(""), SecurityKind::None);
        assert_eq!(SecurityKind::parse("bearer token"), SecurityKind::Bearer);
        assert_eq!(SecurityKind::parse("JWT"), SecurityKind::Bearer);
        assert_eq!(SecurityKind::parse("api-key"), SecurityKind::ApiKey);
        assert_eq!(SecurityKind::parse("OAuth2"), SecurityKind::OAuth2);
        assert_eq!(SecurityKind::parse("session cookie"), SecurityKind::Session);
    }

    #[test]
    fn httpverb_round_trips() {
        assert_eq!(HttpVerb::parse("post"), Some(HttpVerb::Post));
        assert_eq!(HttpVerb::parse("DELETE"), Some(HttpVerb::Delete));
        assert_eq!(HttpVerb::parse("bogus"), None);
        assert_eq!(HttpVerb::Post.as_str(), "post");
    }
}
