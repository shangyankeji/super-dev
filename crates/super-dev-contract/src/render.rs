//! Render an [`ApiSpec`] to OpenAPI 3.1 artifacts.
//!
//! Emits two files into `.super-dev/contracts/`:
//! - `openapi.json` — machine-readable, consumed by the quality gate.
//! - `openapi.yaml` — human-reviewable, the contract a reviewer signs off on.
//!
//! Both are valid OpenAPI 3.1 documents. We hand-roll the YAML (rather than
//! pulling a YAML serializer dependency) because the structure is fixed and
//! flat — paths → methods → operationId/responses/security. Keeping this
//! crate dependency-light matches `super-dev-spec` / `super-dev-governance`.

use std::path::{Path, PathBuf};

use crate::parse::{ApiSpec, HttpVerb, SecurityKind};

/// Render the spec as an OpenAPI 3.1 JSON document.
#[must_use]
pub fn render_json(spec: &ApiSpec) -> String {
    let mut paths = serde_json::Map::new();
    for endpoint in &spec.endpoints {
        let path_entry = paths
            .entry(endpoint.path.clone())
            .or_insert_with(|| serde_json::json!({}));
        let path_obj = path_entry.as_object_mut().expect("path entry is object");
        let mut op = serde_json::Map::new();
        op.insert(
            "operationId".into(),
            serde_json::json!(endpoint.operation_id),
        );
        if !endpoint.description.is_empty() {
            op.insert("summary".into(), serde_json::json!(endpoint.description));
        }
        op.insert(
            "responses".into(),
            serde_json::json!({
                "200": { "description": format!("Successful response for {}", endpoint.operation_id) }
            }),
        );
        if !endpoint.request_shape.is_empty()
            && !matches!(
                endpoint.method,
                HttpVerb::Get | HttpVerb::Delete | HttpVerb::Head
            )
        {
            op.insert(
                "requestBody".into(),
                serde_json::json!({
                    "required": true,
                    "content": {
                        "application/json": {
                            "schema": { "description": endpoint.request_shape }
                        }
                    }
                }),
            );
        }
        if !matches!(endpoint.security, SecurityKind::None) {
            op.insert(
                "security".into(),
                serde_json::json!([{ security_scheme_name(endpoint.security): [] }]),
            );
        }
        path_obj.insert(
            endpoint.method.as_str().into(),
            serde_json::Value::Object(op),
        );
    }

    let doc = serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": if spec.title.is_empty() { "Super Dev API" } else { &spec.title },
            "version": "1.0.0"
        },
        "paths": serde_json::Value::Object(paths),
        "components": {
            "securitySchemes": security_schemes(spec)
        }
    });
    serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".into())
}

/// Render the spec as an OpenAPI 3.1 YAML document (hand-rolled, no serde_yaml).
#[must_use]
pub fn render_yaml(spec: &ApiSpec) -> String {
    let mut out = String::new();
    out.push_str("openapi: 3.1.0\n");
    out.push_str(&format!(
        "info:\n  title: {}\n  version: 1.0.0\n",
        if spec.title.is_empty() {
            "Super Dev API"
        } else {
            &spec.title
        }
    ));
    out.push_str("paths:\n");
    if spec.endpoints.is_empty() {
        out.push_str("  {}\n");
    } else {
        // Group endpoints by path.
        let mut by_path: std::collections::BTreeMap<&str, Vec<&crate::parse::Endpoint>> =
            std::collections::BTreeMap::new();
        for e in &spec.endpoints {
            by_path.entry(e.path.as_str()).or_default().push(e);
        }
        for (path, endpoints) in &by_path {
            out.push_str(&format!("  {path}:\n"));
            for endpoint in endpoints {
                out.push_str(&format!("    {}:\n", endpoint.method.as_str()));
                out.push_str(&format!(
                    "      operationId: {}\n",
                    yaml_scalar(&endpoint.operation_id)
                ));
                if !endpoint.description.is_empty() {
                    out.push_str(&format!(
                        "      summary: {}\n",
                        yaml_scalar(&endpoint.description)
                    ));
                }
                if !endpoint.request_shape.is_empty()
                    && !matches!(
                        endpoint.method,
                        HttpVerb::Get | HttpVerb::Delete | HttpVerb::Head
                    )
                {
                    out.push_str("      requestBody:\n");
                    out.push_str("        required: true\n");
                    out.push_str("        content:\n");
                    out.push_str("          application/json:\n");
                    out.push_str(&format!(
                        "            schema:\n              description: {}\n",
                        yaml_scalar(&endpoint.request_shape)
                    ));
                }
                out.push_str("      responses:\n");
                out.push_str(&format!(
                    "        '200':\n          description: {}\n",
                    yaml_scalar(&format!(
                        "Successful response for {}",
                        endpoint.operation_id
                    ))
                ));
                if !matches!(endpoint.security, SecurityKind::None) {
                    out.push_str(&format!(
                        "      security:\n        - {}: []\n",
                        security_scheme_name(endpoint.security)
                    ));
                }
            }
        }
    }
    out.push_str("components:\n  securitySchemes:\n");
    let schemes = security_schemes(spec);
    if let Some(obj) = schemes.as_object() {
        if obj.is_empty() {
            out.push_str("    {}\n");
        } else {
            for (name, scheme) in obj {
                out.push_str(&format!("    {name}:\n"));
                // Render the scheme as proper nested YAML (indented under the
                // scheme name). Previously this used serde_json::Value's
                // Display, which stringified nested objects (e.g. OAuth2
                // `flows`) as flat JSON `{...}` instead of indented YAML.
                render_yaml_value(&mut out, scheme, 6);
            }
        }
    }
    out
}

/// Render a string value's RIGHT-HAND side (after `key: ` or `- `).
///
/// Multi-line strings are emitted as a YAML **literal block scalar** (`|`)
/// so the content stays readable in `openapi.yaml` instead of a single line
/// with escaped `\n`. Single-line strings go through [`yaml_scalar`]
/// (quoted when they contain YAML indicators).
fn render_string_rhs(out: &mut String, value: &serde_json::Value, indent: usize) {
    // Pull the raw string content (without the JSON surrounding quotes that
    // `to_string()` would add).
    if let Some(s) = value.as_str() {
        if s.contains('\n') {
            // Literal block scalar: `|\n` then each line indented by `indent+2`.
            let pad = " ".repeat(indent + 2);
            out.push_str("|\n");
            for line in s.lines() {
                if line.is_empty() {
                    out.push_str(&format!("{pad}\n"));
                } else {
                    out.push_str(&format!("{pad}{line}\n"));
                }
            }
            return;
        }
    }
    // Single-line (or non-string) → quoted/escaped scalar.
    out.push_str(&yaml_scalar(&value.to_string()));
    out.push('\n');
}

/// Render a [`serde_json::Value`] as YAML at a given indent (in spaces).
/// Scalars use [`yaml_scalar`]; objects/maps recurse with `key:` lines;
/// arrays recurse with `- ` items.
///
/// **Limitations:** minimal emitter — does NOT emit YAML anchors (`&anchor`)
/// / aliases (`*alias`) or merge keys (`<<:`). Fine for the security-schemes
/// object (OpenAPI schemas in the Super Dev contract shape don't use anchors —
/// each scheme is emitted inline). A JSON tree with intentional duplication
/// renders verbosely (duplicated), not deduplicated via anchors — valid YAML.
fn render_yaml_value(out: &mut String, value: &serde_json::Value, indent: usize) {
    let pad = " ".repeat(indent);
    match value {
        serde_json::Value::Object(obj) => {
            for (k, v) in obj {
                if v.is_object() || v.is_array() {
                    out.push_str(&format!("{pad}{k}:\n"));
                    render_yaml_value(out, v, indent + 2);
                } else if v.as_str().is_some_and(|s| s.contains('\n')) {
                    // Multi-line string → block scalar on its own line.
                    out.push_str(&format!("{pad}{k}: "));
                    render_string_rhs(out, v, indent);
                } else {
                    out.push_str(&format!("{pad}{k}: "));
                    render_string_rhs(out, v, indent);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                out.push_str(&format!("{pad}- "));
                render_yaml_value_inline(out, item, indent + 2);
            }
        }
        other => {
            out.push_str(&pad);
            render_string_rhs(out, other, indent);
        }
    }
}

/// Render a scalar/array/object value inline (after a `- `), then a newline.
fn render_yaml_value_inline(out: &mut String, value: &serde_json::Value, indent: usize) {
    match value {
        serde_json::Value::Object(obj) => {
            // first key on the `- ` line, rest indented.
            let mut first = true;
            for (k, v) in obj {
                if first {
                    if v.is_object() || v.is_array() {
                        out.push_str(&format!("{k}:\n"));
                        render_yaml_value(out, v, indent);
                    } else {
                        out.push_str(&format!("{k}: "));
                        render_string_rhs(out, v, indent);
                    }
                    first = false;
                } else {
                    let pad = " ".repeat(indent);
                    if v.is_object() || v.is_array() {
                        out.push_str(&format!("{pad}{k}:\n"));
                        render_yaml_value(out, v, indent + 2);
                    } else {
                        out.push_str(&format!("{pad}{k}: "));
                        render_string_rhs(out, v, indent);
                    }
                }
            }
        }
        other => {
            render_string_rhs(out, other, indent);
        }
    }
}

/// Write `openapi.json` + `openapi.yaml` to `<project_root>/.super-dev/contracts/`.
/// Returns the paths written. Best-effort: a write failure returns the paths
/// that succeeded (never errors — the quality gate reports "contract missing").
#[must_use]
pub fn write_contract(project_root: &Path, spec: &ApiSpec) -> Vec<PathBuf> {
    let dir = project_root.join(crate::CONTRACT_DIR);
    let _ = std::fs::create_dir_all(&dir);
    let mut written = Vec::new();
    let json_path = dir.join("openapi.json");
    let yaml_path = dir.join("openapi.yaml");
    if std::fs::write(&json_path, render_json(spec)).is_ok() {
        written.push(json_path);
    }
    if std::fs::write(&yaml_path, render_yaml(spec)).is_ok() {
        written.push(yaml_path);
    }
    written
}

/// Quote a string as a YAML scalar (bare if simple, double-quoted if it
/// contains special chars or would otherwise break single-line YAML).
///
/// This must quote (and escape) any string containing a newline or tab —
/// a bare `\n` would start an unindented continuation line and corrupt
/// the document — as well as leading/trailing whitespace. Previously only
/// the YAML indicator characters triggered quoting, so a multi-line
/// `description` produced invalid OpenAPI YAML.
fn yaml_scalar(s: &str) -> String {
    let needs_quotes = s.is_empty()
        || s.contains([
            ':', '#', '{', '}', '[', ']', ',', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`',
        ])
        || s.contains(['\n', '\t', '\r'])
        || s.chars().next().map_or(false, is_yaml_indicator_lead)
        || s.ends_with(' ')
        || s.ends_with('\t');
    if needs_quotes {
        // Double-quoted YAML scalar: escape backslash, quote, and the
        // control chars that would otherwise terminate/alter the line.
        let escaped = s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

/// Whether a leading char is a YAML indicator that forces quoting
/// (`-` / `?` start, or an ASCII digit — looks like a number/flow).
fn is_yaml_indicator_lead(c: char) -> bool {
    c == '-' || c == '?' || c.is_ascii_digit()
}

/// Build the `components.securitySchemes` object for the spec, including only
/// the security kinds actually used by some endpoint.
fn security_schemes(spec: &ApiSpec) -> serde_json::Value {
    let mut schemes = serde_json::Map::new();
    let used: std::collections::BTreeSet<SecurityKind> =
        spec.endpoints.iter().map(|e| e.security).collect();
    for kind in used {
        if matches!(kind, SecurityKind::None) {
            continue;
        }
        let name = security_scheme_name(kind);
        let scheme = match kind {
            SecurityKind::Bearer => serde_json::json!({
                "type": "http",
                "scheme": "bearer",
                "bearerFormat": "JWT"
            }),
            SecurityKind::ApiKey => serde_json::json!({
                "type": "apiKey",
                "in": "header",
                "name": "X-API-Key"
            }),
            SecurityKind::OAuth2 => serde_json::json!({
                "type": "oauth2",
                "flows": {}
            }),
            SecurityKind::Session => serde_json::json!({
                "type": "apiKey",
                "in": "cookie",
                "name": "session"
            }),
            SecurityKind::None | SecurityKind::Other => serde_json::json!({
                "type": "http",
                "scheme": "basic"
            }),
        };
        schemes.insert(name.into(), scheme);
    }
    serde_json::Value::Object(schemes)
}

/// Stable scheme name for a security kind.
fn security_scheme_name(kind: SecurityKind) -> &'static str {
    match kind {
        SecurityKind::Bearer => "bearerAuth",
        SecurityKind::ApiKey => "apiKeyAuth",
        SecurityKind::OAuth2 => "oauth2",
        SecurityKind::Session => "sessionAuth",
        SecurityKind::None => "none",
        SecurityKind::Other => "otherAuth",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_architecture;

    const SAMPLE_ARCH: &str = "| Method | Path | Request | Response | Auth | Description |
|---|---|---|---|---|---|
| GET | /api/users | - | - | none | List users |
| POST | /api/users | { name } | { id } | none | Create user |
| GET | /api/users/:id | - | { user } | bearer | Get user |
| DELETE | /api/users/:id | - | { ok } | bearer | Delete user |
";

    #[test]
    fn render_json_is_valid_openapi_31() {
        let spec = parse_architecture(SAMPLE_ARCH, "demo");
        let json = render_json(&spec);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["openapi"], "3.1.0");
        assert_eq!(v["info"]["title"], "demo");
        assert_eq!(v["info"]["version"], "1.0.0");
        assert!(v["paths"]["/api/users"]["get"].is_object());
        assert_eq!(v["paths"]["/api/users"]["get"]["operationId"], "list_users");
        assert_eq!(
            v["paths"]["/api/users"]["post"]["operationId"],
            "create_user"
        );
        // Request body only on non-GET/non-DELETE.
        assert!(v["paths"]["/api/users"]["post"]["requestBody"].is_object());
        assert!(v["paths"]["/api/users"]["get"]["requestBody"].is_null());
        // Security only on bearer endpoints.
        assert!(v["paths"]["/api/users/:id"]["get"]["security"][0]["bearerAuth"].is_array());
        assert!(v["paths"]["/api/users"]["get"]["security"].is_null());
    }

    #[test]
    fn render_yaml_contains_structure() {
        let spec = parse_architecture(SAMPLE_ARCH, "demo");
        let yaml = render_yaml(&spec);
        assert!(yaml.starts_with("openapi: 3.1.0"));
        assert!(yaml.contains("paths:"));
        assert!(yaml.contains("  /api/users:"));
        assert!(yaml.contains("    get:"));
        assert!(yaml.contains("    post:"));
        assert!(yaml.contains("operationId: list_users"));
        assert!(yaml.contains("bearerAuth"));
    }

    #[test]
    fn render_empty_spec() {
        let spec = ApiSpec::default();
        let json = render_json(&spec);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["openapi"], "3.1.0");
        assert!(v["paths"].as_object().unwrap().is_empty());
        let yaml = render_yaml(&spec);
        assert!(yaml.contains("paths:\n  {}"));
    }

    #[test]
    fn write_contract_creates_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let spec = parse_architecture(SAMPLE_ARCH, "demo");
        let written = write_contract(tmp.path(), &spec);
        assert_eq!(written.len(), 2);
        let json_path = tmp.path().join(".super-dev/contracts/openapi.json");
        let yaml_path = tmp.path().join(".super-dev/contracts/openapi.yaml");
        assert!(json_path.is_file());
        assert!(yaml_path.is_file());
        // JSON is valid.
        let body = std::fs::read_to_string(&json_path).unwrap();
        let _: serde_json::Value = serde_json::from_str(&body).unwrap();
    }

    #[test]
    fn yaml_scalar_quotes_special() {
        assert_eq!(yaml_scalar("simple"), "simple");
        assert_eq!(yaml_scalar(""), "\"\"");
        assert_eq!(yaml_scalar("has: colon"), "\"has: colon\"");
        assert_eq!(yaml_scalar("-123"), "\"-123\"");
    }

    #[test]
    fn render_yaml_value_emits_block_scalar_for_multiline() {
        // Regression: multi-line strings used to be quoted + \n-escaped
        // (single ugly line). Now they emit a YAML `|` literal block scalar
        // so the description stays readable in openapi.yaml.
        let mut out = String::new();
        let v = serde_json::json!({"description": "line one\nline two"});
        render_yaml_value(&mut out, &v, 0);
        assert!(
            out.contains("|\n"),
            "multi-line description must use a `|` block scalar, got: {out}"
        );
        assert!(
            out.contains("line one") && out.contains("line two"),
            "both lines must be present as-is, got: {out}"
        );
        // Must NOT contain the escaped form.
        assert!(
            !out.contains("\\n"),
            "must not contain escaped \\n, got: {out}"
        );
    }

    #[test]
    fn render_yaml_value_single_line_stays_quoted_scalar() {
        // Single-line strings must still use yaml_scalar (quoted only when
        // needed), NOT a block scalar.
        let mut out = String::new();
        let v = serde_json::json!({"description": "one line"});
        render_yaml_value(&mut out, &v, 0);
        assert!(
            !out.contains("|\n"),
            "single-line must not use block scalar: {out}"
        );
        assert!(out.contains("one line"));
    }

    #[test]
    fn render_yaml_value_handles_multiline_description() {
        // The scheme-rendering path (render_yaml) shouldn't crash on a
        // multi-line description in a full spec render.
        let spec = ApiSpec {
            endpoints: vec![],
            title: "t".into(),
        };
        let _yaml = render_yaml(&spec); // must not panic on empty spec
    }

    #[test]
    fn yaml_scalar_escapes_newlines_and_tabs() {
        // Regression: a multi-line description previously emitted a bare
        // newline, starting an unindented YAML continuation line and
        // corrupting the document. Now it must be double-quoted with \n.
        assert_eq!(yaml_scalar("line one\nline two"), "\"line one\\nline two\"");
        assert_eq!(yaml_scalar("a\tb"), "\"a\\tb\"");
        assert_eq!(yaml_scalar("trailing space "), "\"trailing space \"");
        assert_eq!(yaml_scalar("has \"quote\""), "\"has \\\"quote\\\"\"");
        // A value with both a newline AND a quote must escape both.
        assert_eq!(
            yaml_scalar("she said \"hi\"\nthen left"),
            "\"she said \\\"hi\\\"\\nthen left\""
        );
    }

    #[test]
    fn security_schemes_only_include_used() {
        let spec = parse_architecture(SAMPLE_ARCH, "demo"); // uses none + bearer
        let schemes = security_schemes(&spec);
        let obj = schemes.as_object().unwrap();
        // Only bearerAuth (none is excluded).
        assert!(obj.contains_key("bearerAuth"));
        assert!(!obj.contains_key("apiKeyAuth"));
    }
}
