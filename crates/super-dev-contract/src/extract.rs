//! Extract API calls from the worker's produced frontend source.
//!
//! Upgrades `extract_api_urls` (which returned bare `Vec<String>` path
//! strings) into typed [`FrontendCall`] records carrying the HTTP method
//! when inferable. Used by [`crate::validate`] to cross-check against the
//! contract.
//!
//! ## Method inference
//! Frontend HTTP libraries encode the method differently:
//! - `fetch('/api/x', { method: 'POST' })` — method in an options object.
//! - `axios.post('/api/x')`, `axios.get(...)` — method is the function name.
//! - `useSWR('/api/x', fetcher)` — method is always GET (SWR is read-only).
//!
//! We handle the common shapes; unknown call patterns default to GET (the
//! most common case) so a call is never silently dropped.

use std::path::{Path, PathBuf};

use regex::Regex;
use std::sync::OnceLock;

use crate::parse::HttpVerb;

/// Frontend file extensions worth scanning for API calls.
const FRONTEND_EXTS: &[&str] = &["tsx", "ts", "jsx", "js", "vue", "svelte", "astro"];

/// Directories that never contain hand-written source worth scanning.
/// Kept conservative: a missing entry here means a wasted walk over a
/// potentially huge generated/vendored tree.
const SKIP_DIRS: &[&str] = &[
    // JS/TS toolchains
    "node_modules",
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".turbo",
    ".vercel",
    ".astro",
    "dist",
    "build",
    "out",
    ".output",
    "coverage",
    ".nyc_output",
    ".pnpm-store",
    ".parcel-cache",
    // Python
    "__pycache__",
    ".venv",
    "venv",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    // Rust / Go / others
    "target",
    "vendor",
    ".gradle",
    // VCS / meta
    ".git",
    ".hg",
    ".svn",
    // Super Dev's own output
    ".super-dev",
    "output",
    "release",
    "knowledge",
    // Generic caches
    ".cache",
];

/// One API call found in frontend source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendCall {
    /// Workspace-relative file path.
    pub file: String,
    /// HTTP method, inferred from the call shape. Defaults to GET.
    pub method: HttpVerb,
    /// The path the call targets, e.g. `/api/users/123`.
    pub path: String,
}

/// Regex for `fetch('/api/...')` and `fetch('/api/...', {...})`.
/// Captures the path in the `path` group, and an optional `method: 'POST'`
/// in the `method` group. Query strings are stripped after capture (not in
/// the regex — the `?` in a char class is fragile across regex versions).
fn fetch_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Path stops at quote / `#` / whitespace / `$` (template-literal
        // interpolation), so fetch(`/api/${id}`) captures `/api/` (the static
        // prefix) rather than `/api/${id`.
        Regex::new(
            r#"fetch\s*\(\s*['"`](?P<path>/[^'"`\#\s$]+)(?:['"`]|\$)(?:[^)]*?method\s*:\s*['"`](?P<method>GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)['"`])?"#,
        )
        .expect("fetch regex well-formed")
    })
}

/// Regex for `axios.get('/api/...')` / `axios.post(...)` etc.
fn axios_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"axios\s*\.\s*(?P<method>get|post|put|delete|patch|head|options)\s*\(\s*['"`](?P<path>/[^'"`\#\s]+)['"`]"#,
        )
        .expect("axios regex well-formed")
    })
}

/// Regex for `ky.get(...)` / `http.get(...)` — same shape as axios.
fn method_client_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?:ky|http)\s*\.\s*(?P<method>get|post|put|delete|patch)\s*\(\s*['"`](?P<path>/[^'"`\#\s]+)['"`]"#,
        )
        .expect("method-client regex well-formed")
    })
}

/// Regex for `useSWR('/api/...')` / `useQuery('/api/...')` — always GET.
fn swr_query_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // useMutation is conventionally POST, but we can't tell from the call
        // shape alone, so default to GET and let the caller (or a second pass)
        // refine. Stops path at `$` for template literals.
        Regex::new(
            r#"(?:useSWR|useSWRInfinite|useQuery|useMutation|useFetch)\s*\(\s*['"`](?P<path>/[^'"`\#\s$]+)['"`]"#,
        )
        .expect("swr/query regex well-formed")
    })
}

/// Regex for a DIRECT `axios('/api/x', {...})` call (no `.method`).
/// Method defaults to GET unless a `method:` option is present.
fn axios_direct_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"axios\s*\(\s*['"`](?P<path>/[^'"`\#\s$]+)['"`](?:[^)]*?method\s*:\s*['"`](?P<method>GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)['"`])?"#,
        )
        .expect("axios-direct regex well-formed")
    })
}

/// Regex for wrapped/SDK client calls: `api.get('/api/x')`,
/// `httpClient.post(...)`, `client.delete(...)`, `service.put(...)`,
/// `request('/api/x')`, `fetcher('/api/x')`. These are the common names
/// projects give a typed wrapper around fetch/axios; without this, a whole
/// app's API surface would be invisible to SD-CODE-003 alignment.
fn wrapped_client_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?:api|httpClient|client|request|fetcher|service)\s*(?:\.\s*(?P<method>get|post|put|delete|patch))?\s*\(\s*['"`](?P<path>/[^'"`\#\s$]+)['"`]"#,
        )
        .expect("wrapped-client regex well-formed")
    })
}

/// Strip a query string / fragment from a captured path:
/// `/api/search?q=test#sec` → `/api/search`.
fn strip_query(path: &str) -> &str {
    path.split(['?', '#']).next().unwrap_or(path)
}

/// Scan frontend source under `project_root` and return every API call
/// found, deduped by `(file, method, path)`. Walks the tree (skipping
/// vendored / generated dirs) to depth 8.
///
/// Returns an empty vec when no frontend source is present (fail-open —
/// the quality gate reports "no frontend calls to validate").
#[must_use]
pub fn extract_frontend_calls(project_root: &Path) -> Vec<FrontendCall> {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_frontend_sources(project_root, &mut files, 0);
    let mut calls: Vec<FrontendCall> = Vec::new();
    for file in &files {
        let Ok(content) = std::fs::read_to_string(file) else {
            continue;
        };
        let rel = file
            .strip_prefix(project_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file.to_string_lossy().to_string());
        calls.extend(extract_from_file(&rel, &content));
    }
    // Dedupe across ALL files by (method, path). `Vec::dedup` only removes
    // *consecutive* duplicates, so the old `calls.dedup()` was a no-op for
    // the same call made in two different files (they're never adjacent).
    // We keep the first-seen `file` so audit can still point at a source.
    let mut seen: std::collections::HashSet<(HttpVerb, String)> = std::collections::HashSet::new();
    calls.retain(|c| seen.insert((c.method, c.path.clone())));
    calls
}

/// Extract calls from one file's content.
fn extract_from_file(file: &str, content: &str) -> Vec<FrontendCall> {
    let mut calls: Vec<FrontendCall> = Vec::new();
    let push = |calls: &mut Vec<FrontendCall>, method: HttpVerb, path: &str| {
        let path = path.to_string();
        if !calls.iter().any(|c| c.method == method && c.path == path) {
            calls.push(FrontendCall {
                file: file.to_string(),
                method,
                path,
            });
        }
    };

    // fetch('/api/x') or fetch('/api/x', { method: 'POST' })
    for cap in fetch_regex().captures_iter(content) {
        let path = cap.name("path").map(|m| m.as_str()).unwrap_or("");
        if path.is_empty() {
            continue;
        }
        let method = cap
            .name("method")
            .and_then(|m| HttpVerb::parse(m.as_str()))
            .unwrap_or(HttpVerb::Get);
        push(&mut calls, method, strip_query(path));
    }
    // axios.get / axios.post / ...
    for cap in axios_regex().captures_iter(content) {
        let path = cap.name("path").map(|m| m.as_str()).unwrap_or("");
        let method = cap
            .name("method")
            .and_then(|m| HttpVerb::parse(m.as_str()))
            .unwrap_or(HttpVerb::Get);
        if !path.is_empty() {
            push(&mut calls, method, strip_query(path));
        }
    }
    // ky.get / http.get / ...
    for cap in method_client_regex().captures_iter(content) {
        let path = cap.name("path").map(|m| m.as_str()).unwrap_or("");
        let method = cap
            .name("method")
            .and_then(|m| HttpVerb::parse(m.as_str()))
            .unwrap_or(HttpVerb::Get);
        if !path.is_empty() {
            push(&mut calls, method, strip_query(path));
        }
    }
    // useSWR / useQuery / useMutation — default GET (call shape can't tell)
    for cap in swr_query_regex().captures_iter(content) {
        let path = cap.name("path").map(|m| m.as_str()).unwrap_or("");
        if !path.is_empty() {
            push(&mut calls, HttpVerb::Get, strip_query(path));
        }
    }
    // Direct axios('/api/x') (no .method) — GET unless a method: option is set.
    for cap in axios_direct_regex().captures_iter(content) {
        let path = cap.name("path").map(|m| m.as_str()).unwrap_or("");
        if path.is_empty() {
            continue;
        }
        let method = cap
            .name("method")
            .and_then(|m| HttpVerb::parse(m.as_str()))
            .unwrap_or(HttpVerb::Get);
        push(&mut calls, method, strip_query(path));
    }
    // Wrapped clients: api.get / httpClient.post / client.delete / request(...) etc.
    for cap in wrapped_client_regex().captures_iter(content) {
        let path = cap.name("path").map(|m| m.as_str()).unwrap_or("");
        if path.is_empty() {
            continue;
        }
        let method = cap
            .name("method")
            .and_then(|m| HttpVerb::parse(m.as_str()))
            .unwrap_or(HttpVerb::Get);
        push(&mut calls, method, strip_query(path));
    }

    calls
}

/// Maximum directory depth for the frontend-source walk. Guards against a
/// pathological nesting (rare, but a symlink-resolved tree could be deep).
const MAX_FRONTEND_DEPTH: usize = 8;

fn collect_frontend_sources(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    if depth > MAX_FRONTEND_DEPTH {
        // Warn once at the boundary so a project with genuinely-deep source
        // trees knows coverage is partial (previously this was silent).
        if depth == MAX_FRONTEND_DEPTH + 1 {
            eprintln!(
                "warn: frontend source walk hit the depth-{MAX_FRONTEND_DEPTH} cap at {};                  files deeper than this are NOT scanned for API calls.                  If your source lives deeper, consider flattening or raise                  the cap.",
                dir.display()
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
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.starts_with('.') || SKIP_DIRS.contains(&name) {
                continue;
            }
            collect_frontend_sources(&p, out, depth + 1);
        } else {
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            if FRONTEND_EXTS.contains(&ext) {
                out.push(p);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_fetch_get() {
        let calls = extract_from_file("src/api.ts", "fetch('/api/users')");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method, HttpVerb::Get);
        assert_eq!(calls[0].path, "/api/users");
    }

    #[test]
    fn extract_fetch_with_method() {
        let calls = extract_from_file(
            "src/api.ts",
            "fetch('/api/orders', { method: 'POST', body: JSON.stringify(data) })",
        );
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method, HttpVerb::Post);
        assert_eq!(calls[0].path, "/api/orders");
    }

    #[test]
    fn extract_axios_methods() {
        let calls = extract_from_file(
            "src/api.ts",
            "axios.get('/api/users'); axios.post('/api/orders', body); axios.delete('/api/x')",
        );
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0].method, HttpVerb::Get);
        assert_eq!(calls[1].method, HttpVerb::Post);
        assert_eq!(calls[2].method, HttpVerb::Delete);
    }

    #[test]
    fn extract_ky_and_http() {
        let calls = extract_from_file(
            "src/api.ts",
            "ky.put('/api/items'); http.patch('/api/items/1')",
        );
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].method, HttpVerb::Put);
        assert_eq!(calls[1].method, HttpVerb::Patch);
    }

    #[test]
    fn extract_swr_always_get() {
        let calls = extract_from_file("src/api.ts", "useSWR('/api/profile', fetcher)");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method, HttpVerb::Get);
    }

    #[test]
    fn dedupes_within_file() {
        let calls = extract_from_file(
            "src/api.ts",
            "fetch('/api/users'); fetch('/api/users'); fetch('/api/users')",
        );
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn ignores_external_urls() {
        // Only paths starting with `/` are captured (the regex requires it).
        let calls = extract_from_file("src/api.ts", "fetch('https://cdn.example.com/img.png')");
        assert!(calls.is_empty());
    }

    #[test]
    fn ignores_non_frontend_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("server.py"), "fetch('/api/x')").unwrap();
        assert!(extract_frontend_calls(tmp.path()).is_empty());
    }

    #[test]
    fn skips_vendored_dirs() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("node_modules/lib")).unwrap();
        std::fs::write(
            tmp.path().join("node_modules/lib/x.ts"),
            "fetch('/api/evil')",
        )
        .unwrap();
        assert!(extract_frontend_calls(tmp.path()).is_empty());
    }

    #[test]
    fn extracts_from_real_project_layout() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src/api");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("client.ts"),
            "fetch('/api/users'); axios.post('/api/auth/login', creds)",
        )
        .unwrap();
        let calls = extract_frontend_calls(tmp.path());
        assert_eq!(calls.len(), 2);
        assert!(calls.iter().any(|c| c.path == "/api/users"));
        assert!(calls.iter().any(|c| c.path == "/api/auth/login"));
    }

    #[test]
    fn strips_query_strings_from_path() {
        let calls = extract_from_file("src/api.ts", "fetch('/api/search?q=test')");
        assert_eq!(calls[0].path, "/api/search");
    }

    #[test]
    fn extracts_direct_axios_call() {
        // axios('/api/x') with no .method — previously missed.
        let calls = extract_from_file("src/a.ts", "axios('/api/upload', { onUploadProgress })");
        assert!(
            calls.iter().any(|c| c.path == "/api/upload"),
            "direct axios() must be captured"
        );
    }

    #[test]
    fn extracts_template_literal_fetch_prefix() {
        // fetch(`/api/users/${id}`) — capture the static prefix /api/users/
        // (stop at `${`), not the broken `/api/users/${id`.
        let calls = extract_from_file("src/a.ts", "fetch(`/api/users/${id}`)");
        let matched: Vec<&str> = calls.iter().map(|c| c.path.as_str()).collect();
        assert!(
            matched
                .iter()
                .any(|p| *p == "/api/users/" || *p == "/api/users"),
            "template-literal fetch must capture static prefix, got {matched:?}"
        );
        assert!(
            !matched.iter().any(|p| p.contains("${")),
            "must not capture the interpolation, got {matched:?}"
        );
    }

    #[test]
    fn extracts_react_query_use_mutation() {
        let calls = extract_from_file("src/a.ts", "useMutation('/api/posts')");
        assert!(
            calls.iter().any(|c| c.path == "/api/posts"),
            "useMutation must be captured"
        );
    }

    #[test]
    fn extracts_wrapped_client_calls() {
        // api.get / httpClient.post / request(...) — common SDK wrappers.
        let calls = extract_from_file(
            "src/a.ts",
            "api.get('/api/products'); httpClient.post('/api/orders'); request('/api/health')",
        );
        let paths: Vec<&str> = calls.iter().map(|c| c.path.as_str()).collect();
        assert!(
            paths.contains(&"/api/products"),
            "api.get must be captured: {paths:?}"
        );
        assert!(
            paths.contains(&"/api/orders"),
            "httpClient.post must be captured: {paths:?}"
        );
        assert!(
            paths.contains(&"/api/health"),
            "request() must be captured: {paths:?}"
        );
    }
}
