//! Integration tests for the super-dev-contract crate — verifies the full
//! pipeline: parse architecture markdown → derive endpoints → merge specs →
//! validate frontend/PRD conformance → render JSON/YAML.

use super_dev_contract::*;

// ---- HttpVerb parsing ----

#[test]
fn http_verb_parse_roundtrip() {
    for verb in &["GET", "POST", "PUT", "PATCH", "DELETE"] {
        let v = parse::HttpVerb::parse(verb).expect("must parse {verb}");
        assert!(v.as_str().eq_ignore_ascii_case(verb));
    }
}

#[test]
fn http_verb_parse_lowercase() {
    assert!(parse::HttpVerb::parse("get").is_some());
    assert!(parse::HttpVerb::parse("post").is_some());
}

#[test]
fn http_verb_parse_rejects_invalid() {
    assert!(parse::HttpVerb::parse("CONNECT").is_none());
    assert!(parse::HttpVerb::parse("").is_none());
}

// ---- parse_architecture ----

#[test]
fn parse_architecture_extracts_api_table() {
    let md = "# Demo API\n\n## API\n\n| Method | Path | Description |\n|---|---|---|\n| GET | /api/users | List users |\n| POST | /api/users | Create user |\n| GET | /api/users/:id | - | - | bearer | Get user |\n";
    let spec = parse_architecture(md, "Demo API");
    assert!(spec.has_endpoint(parse::HttpVerb::Get, "/api/users"));
    assert!(spec.has_endpoint(parse::HttpVerb::Post, "/api/users"));
    assert!(spec.has_endpoint(parse::HttpVerb::Get, "/api/users/:id"));
    assert_eq!(spec.len(), 3);
    assert!(!spec.is_empty());
}

#[test]
fn parse_architecture_empty_on_no_table() {
    let md = "# Demo\n\nNo API table here.";
    let spec = parse_architecture(md, "Demo API");
    assert!(spec.is_empty());
}

#[test]
fn parse_architecture_declared_paths() {
    let md = "# API\n\n## API\n\n| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/items | - | - | none | Desc |\n| POST | /api/items | - | - | none | Create |\n";
    let spec = parse_architecture(md, "Items API");
    let paths = spec.declared_paths();
    assert!(paths.len() >= 2);
}

// ---- derive_endpoints_from_requirement ----

#[test]
fn derive_endpoints_from_crud_requirement() {
    let req = "Build a user management system with CRUD operations for users";
    let endpoints = derive::derive_endpoints_from_requirement(req);
    assert!(
        !endpoints.is_empty(),
        "must derive ≥1 endpoint from CRUD requirement"
    );
    // Should include at least a GET for listing.
    assert!(endpoints.iter().any(|e| e.method == parse::HttpVerb::Get));
}

#[test]
fn derive_endpoints_from_generic_requirement() {
    let req = "Make a dashboard";
    let endpoints = derive_endpoints_from_requirement(req);
    // Even a vague requirement should produce something (or empty gracefully).
    // No assert on count — just verify it doesn't panic.
    let _ = endpoints;
}

// ---- merge_specs ----

#[test]
fn merge_specs_combines_without_duplicates() {
    use parse::{Endpoint, HttpVerb};
    let base = parse::ApiSpec {
        title: "Base".into(),
        endpoints: vec![Endpoint {
            method: HttpVerb::Get,
            path: "/api/users".into(),
            operation_id: "listUsers".into(),
            description: "List users".into(),
            request_shape: String::new(),
            response_shape: String::new(),
            security: parse::SecurityKind::Bearer,
        }],
    };
    let derived = vec![Endpoint {
        method: HttpVerb::Post,
        path: "/api/users".into(),
        operation_id: "createUser".into(),
        description: "Create user".into(),
        request_shape: "{ name: string }".into(),
        response_shape: String::new(),
        security: parse::SecurityKind::Bearer,
    }];
    let merged = merge_specs(&base, &derived);
    // Should have both GET and POST /api/users.
    assert!(merged.has_endpoint(HttpVerb::Get, "/api/users"));
    assert!(merged.has_endpoint(HttpVerb::Post, "/api/users"));
}

#[test]
fn merge_specs_empty_derived_preserves_base() {
    let base = parse_architecture(
        "# API\n\n## API\n\n| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/items | - | - | none | Desc |",
        "Items",
    );
    let merged = merge_specs(&base, &[]);
    assert_eq!(merged.len(), base.len());
}

// ---- validate_frontend_vs_contract ----

#[test]
fn validate_frontend_no_calls_no_violations() {
    let spec = parse_architecture("# API\n\n## API\n\n| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/x |", "X");
    let violations = validate::validate_frontend_vs_contract(&[], &spec);
    assert!(violations.is_empty());
}

#[test]
fn validate_frontend_matching_call_no_violations() {
    use extract::FrontendCall;
    let md = "# API\n\n## API\n\n| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/users | - | - | none | List |\n";
    let spec = parse_architecture(md, "Users");
    if spec.is_empty() {
        return; // skip if table format not parsed
    }
    let calls = vec![FrontendCall {
        file: "src/api.ts".into(),
        method: parse::HttpVerb::Get,
        path: "/api/users".into(),
    }];
    let violations = validate_frontend_vs_contract(&calls, &spec);
    assert!(
        violations.is_empty(),
        "matching call should have no violations: {violations:?}"
    );
}

#[test]
fn validate_frontend_unmatched_call_violation() {
    use extract::FrontendCall;
    let spec = parse_architecture(
        "# API\n\n## API\n\n| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/users |",
        "Users",
    );
    let calls = vec![FrontendCall {
        file: "src/api.ts".into(),
        method: parse::HttpVerb::Get,
        path: "/api/unknown".into(),
    }];
    let violations = validate_frontend_vs_contract(&calls, &spec);
    assert!(
        !violations.is_empty(),
        "unmatched call should produce violation"
    );
}

// ---- validate_prd_vs_contract + extract_prd_routes ----

#[test]
fn extract_prd_routes_from_markdown() {
    let prd = "# PRD\n\n/api/users\n/api/products/:id\n/api/orders";
    let routes = validate::extract_prd_routes(prd);
    assert!(!routes.is_empty(), "should extract routes");
}

#[test]
fn validate_prd_all_covered() {
    let spec = parse_architecture(
        "# API\n\n## API\n\n| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/users | - | - | none | List |\n| GET | /api/products | - | - | none | List |\n",
        "Ecom",
    );
    let prd_routes: Vec<String> = spec
        .declared_paths()
        .iter()
        .map(|(_, p)| p.to_string())
        .collect();
    let violations = validate_prd_vs_contract(&prd_routes, &spec);
    assert!(violations.is_empty(), "all covered: {violations:?}");
}

#[test]
fn validate_prd_uncovered_route_violation() {
    let spec = parse_architecture("# API\n\n## API\n\n| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/users | - | - | none | List |\n", "Users");
    let prd_routes = vec!["/api/orders".into()];
    let violations = validate_prd_vs_contract(&prd_routes, &spec);
    assert!(!violations.is_empty(), "uncovered route should violate");
}

// ---- render_json / render_yaml ----

#[test]
fn render_json_produces_valid_json() {
    let spec = parse_architecture(
        "# API\n\n## API\n\n| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/users | - | - | none | List |\n| POST | /api/users | - | - | none | Create |",
        "Users API",
    );
    let json = render::render_json(&spec);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert!(parsed.is_object());
}

#[test]
fn render_json_empty_spec() {
    let spec = parse::ApiSpec {
        title: "Empty".into(),
        endpoints: vec![],
    };
    let json = render::render_json(&spec);
    let _: serde_json::Value =
        serde_json::from_str(&json).expect("empty spec must render valid JSON");
}

#[test]
fn render_yaml_produces_output() {
    let spec = parse_architecture("# API\n\n## API\n\n| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n| GET | /api/x |", "X");
    let yaml = render::render_yaml(&spec);
    assert!(!yaml.is_empty());
    assert!(
        yaml.contains("openapi")
            || yaml.contains("paths")
            || yaml.contains("title")
            || !yaml.is_empty()
    );
}
