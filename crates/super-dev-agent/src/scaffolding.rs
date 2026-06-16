//! Engineering scaffolding generator — turns the OpenAPI contract into real
//! ops artifacts (Dockerfile, CI workflow, DB migrations, docker-compose,
//! .env.example) that a team can actually deploy.
//!
//! Until 4.6 Super Dev produced only Markdown checklists ("[ ] add a
//! Dockerfile") — pure vibecoding. This module generates the artifacts
//! themselves, grounded in the contract + detected tech stack, so the
//! quality gate can verify presence AND the worker has real starting points
//! instead of inventing everything from scratch.
//!
//! ## What it generates
//! All artifacts land under the project root (not `.super-dev/`) so they're
//! where a developer expects them, and the proof-pack bundles them.
//!
//! - `Dockerfile` — multi-stage build for the detected stack.
//! - `docker-compose.yml` — app + DB service.
//! - `.github/workflows/ci.yml` — lint → test → build, gated on the quality
//!   gate JSON.
//! - `migrations/0001_init.sql` — schema skeleton derived from the contract.
//! - `.env.example` — every env var the app + DB need.
//!
//! ## Detection
//! Tech stack is inferred from the architecture doc's prose + any manifest
//! files present (`package.json` → Node, `Cargo.toml` → Rust, etc.). The
//! generated templates use the detected primary language; unknown stacks get
//! a generic Node template (the most common case).
//!
//! Fail-open: any write failure is swallowed (returns what succeeded).

use std::path::Path;

use super_dev_contract::ApiSpec;

/// Detected primary tech stack, used to pick scaffolding templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]

pub enum TechStack {
    #[default]
    /// Node / TypeScript / JavaScript (Next, Express, Nest, etc.).
    Node,
    /// Rust (Axum, Actix, etc.).
    Rust,
    /// Python (FastAPI, Django, etc.).
    Python,
    /// Go.
    Go,
    /// Could not detect — default to Node (most common web stack).
    Unknown,
}

impl TechStack {
    /// Stable lower-case label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Go => "go",
            Self::Unknown => "unknown",
        }
    }
}

/// Detect the tech stack from workspace manifests + architecture doc prose.
#[must_use]
pub fn detect_stack(project_root: &Path, arch_text: &str) -> TechStack {
    // Manifest files take priority (authoritative).
    if project_root.join("Cargo.toml").is_file() {
        return TechStack::Rust;
    }
    if project_root.join("go.mod").is_file() {
        return TechStack::Go;
    }
    if project_root.join("pyproject.toml").is_file()
        || project_root.join("requirements.txt").is_file()
    {
        return TechStack::Python;
    }
    if project_root.join("package.json").is_file() {
        return TechStack::Node;
    }
    // Fall back to architecture doc prose.
    let lower = arch_text.to_ascii_lowercase();
    if lower.contains("rust") || lower.contains("axum") || lower.contains("actix") {
        return TechStack::Rust;
    }
    if lower.contains("python")
        || lower.contains("fastapi")
        || lower.contains("django")
        || lower.contains("flask")
    {
        return TechStack::Python;
    }
    if lower.contains("go ") || lower.contains("golang") || lower.contains("gin ") {
        return TechStack::Go;
    }
    if lower.contains("node")
        || lower.contains("typescript")
        || lower.contains("express")
        || lower.contains("next")
        || lower.contains("react")
    {
        return TechStack::Node;
    }
    TechStack::Unknown
}

/// Result of running the scaffolding generator.
#[derive(Debug, Clone, Default)]
pub struct ScaffoldingOutput {
    /// Every artifact path written, workspace-relative.
    pub artifacts: Vec<String>,
    /// The detected stack.
    pub stack: TechStack,
}

/// Generate all ops artifacts for `project_root` based on the OpenAPI contract
/// and detected tech stack. Writes files to disk (best-effort). Returns the
/// list of paths actually written.
#[must_use]
pub fn generate_scaffolding(
    project_root: &Path,
    spec: &ApiSpec,
    arch_text: &str,
) -> ScaffoldingOutput {
    let stack = detect_stack(project_root, arch_text);
    let mut artifacts = Vec::new();
    let endpoint_count = spec.len();

    for (rel, body) in [
        ("Dockerfile", render_dockerfile(stack, project_root)),
        ("docker-compose.yml", render_compose(stack)),
        (".env.example", render_env_example(stack, endpoint_count)),
        ("migrations/0001_init.sql", render_migration(spec, stack)),
        (".github/workflows/ci.yml", render_ci_workflow(stack)),
    ] {
        let path = project_root.join(rel);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::write(&path, body).is_ok() {
            artifacts.push(rel.to_string());
        }
    }

    ScaffoldingOutput { artifacts, stack }
}

// =====================================================================
// Template renderers — pure functions, no external templating dep.
// =====================================================================

fn render_dockerfile(stack: TechStack, project_root: &Path) -> String {
    match stack {
        TechStack::Node => NODE_DOCKERFILE.to_string(),
        TechStack::Rust => {
            let bin = rust_binary_name(project_root);
            RUST_DOCKERFILE
                .replace("target/release/app", &format!("target/release/{bin}"))
                .replace("/usr/local/bin/app", &format!("/usr/local/bin/{bin}"))
                .replace("CMD [\"app\"]", &format!("CMD [\"{bin}\"]"))
        }
        TechStack::Python => PYTHON_DOCKERFILE.to_string(),
        TechStack::Go => GO_DOCKERFILE.to_string(),
        TechStack::Unknown => NODE_DOCKERFILE.to_string(), // sensible default
    }
}

fn render_compose(stack: TechStack) -> String {
    let app_build = match stack {
        TechStack::Rust => "context: .\n      dockerfile: Dockerfile",
        _ => "context: .\n      dockerfile: Dockerfile",
    };
    // Note on indentation: this is a single raw-format string, so the
    // `\n` + trailing-space continuations in the format string are the
    // YAML indentation. Structure:
    //   services: {app, db, redis}   ← all three are siblings
    //   volumes: {pgdata}            ← top-level named-volume declaration
    // The earlier version accidentally appended `redis:` AFTER the
    // top-level `volumes:` key, which nested it under volumes and
    // produced invalid Compose YAML.
    format!(
        "# Generated by Super Dev — app + postgres + redis for local dev.\n\
         # Override POSTGRES_* / REDIS_* in .env for your environment.\n\
         services:\n  \
         app:\n    \
         build:\n      {app_build}\n    \
         ports:\n      - \"3000:3000\"\n    \
         environment:\n      - DATABASE_URL=postgresql://app:app@db:5432/app\n      - REDIS_URL=redis://redis:6379\n      - NODE_ENV=production\n    \
         depends_on:\n      db:\n        condition: service_healthy\n      redis:\n        condition: service_started\n  \
         db:\n    \
         image: postgres:16-alpine\n    \
         environment:\n      POSTGRES_USER: app\n      POSTGRES_PASSWORD: app\n      POSTGRES_DB: app\n    \
         ports:\n      - \"5432:5432\"\n    \
         volumes:\n      - pgdata:/var/lib/postgresql/data\n    \
         healthcheck:\n      test: [\"CMD-SHELL\", \"pg_isready -U app\"]\n      interval: 5s\n      timeout: 3s\n      retries: 5\n  \
         redis:\n    \
         image: redis:7-alpine\n    \
         ports:\n      - \"6379:6379\"\n    \
         command: redis-server --maxmemory 256mb --maxmemory-policy allkeys-lru\n\n\
         volumes:\n  pgdata:\n"
    )
}

fn render_env_example(stack: TechStack, endpoint_count: usize) -> String {
    let mut s = String::new();
    s.push_str("# Generated by Super Dev — copy to .env and fill in real values.\n\n");
    s.push_str("# Database\n");
    s.push_str("DATABASE_URL=postgresql://app:app@localhost:5432/app\n\n");
    s.push_str("# Auth\n");
    s.push_str("JWT_SECRET=change-me-to-a-real-secret\n");
    s.push_str("JWT_EXPIRES_IN=900\n\n");
    s.push_str("# Server\n");
    s.push_str("PORT=3000\n");
    s.push_str("NODE_ENV=development\n\n");
    match stack {
        TechStack::Node => {
            s.push_str("# Node\nnpm_config_registry=https://registry.npmjs.org\n");
            s.push_str("# Redis\nREDIS_URL=redis://localhost:6379\n");
            s.push_str("# SMTP (email)\nSMTP_URL=smtp://localhost:587\n");
        }
        TechStack::Rust => s.push_str("# Rust\nRUST_LOG=info\n"),
        TechStack::Python => s.push_str("# Python\nPYTHONUNBUFFERED=1\n"),
        TechStack::Go => s.push_str("# Go\nGIN_MODE=debug\n"),
        TechStack::Unknown => {}
    }
    if endpoint_count > 0 {
        // The contract is written by super-dev-contract::write_contract as
        // both openapi.json (machine-readable, the quality gate's source of
        // truth) and openapi.yaml (human-reviewable). Point at the YAML for
        // the reviewer, the JSON for codegen.
        s.push_str(&format!(
            "\n# {endpoint_count} API endpoints declared in              .super-dev/contracts/openapi.yaml (review) / openapi.json (codegen)\n"
        ));
    }
    s
}

fn sql_type_for(logical: &str) -> &'static str {
    match logical {
        "text" => "TEXT",
        "integer" => "INTEGER",
        "uuid" => "UUID",
        "date" => "DATE",
        "timestamptz" => "TIMESTAMPTZ",
        "jsonb" => "JSONB",
        "boolean" => "BOOLEAN",
        "numeric" => "NUMERIC(19,4)",
        _ => "TEXT",
    }
}

fn render_migration(spec: &ApiSpec, stack: TechStack) -> String {
    let mut s = String::new();
    s.push_str("-- Generated by Super Dev from the OpenAPI contract.\n");
    s.push_str("-- Initial schema. Adjust types/constraints to match your ORM.\n\n");
    // Derive entities from path segments: /api/products/:id → products table.
    // Skip namespace segments (api, v1, auth, health, meta, admin) and params.
    let non_entity_segments = [
        // Namespaces.
        "api", "v1", "v2", "auth", "health", "meta", "admin", "internal", "public",
        // Verbs / actions (never entities).
        "login", "logout", "register", "signup", "signin", "me", "verify", "reset", "refresh",
        "callback", "webhook", "confirm", "activate", "search", "list", "create", "update",
        "delete", "get",
    ];
    let mut tables: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for endpoint in &spec.endpoints {
        let entity = endpoint.path.trim_matches('/').split('/').find(|seg| {
            !seg.is_empty() && !seg.starts_with(':') && !non_entity_segments.contains(seg)
        });
        if let Some(e) = entity {
            tables.insert(e.to_string());
        }
    }
    // Always include a users table (auth is near-universal).
    tables.insert("users".to_string());

    for table in &tables {
        s.push_str(&format!(
            "CREATE TABLE IF NOT EXISTS {table} (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),"
        ));
        for (fname, ftype, _desc) in super_dev_contract::fields_for_entity(table) {
            let sql_type = sql_type_for(ftype);
            if fname.ends_with("_id") && fname != "parent_id" {
                let base = fname.trim_end_matches("_id");
                let ref_table = match base {
                    "owner" | "author" | "assignee" | "sender" | "recipient" | "user" => {
                        "users".to_string()
                    }
                    "parent" => table.clone(),
                    other if other.ends_with('s') => other.to_string(),
                    other if other.ends_with("ory") => format!("{}ies", &other[..other.len() - 2]),
                    other => format!("{other}s"),
                };
                s.push_str(&format!(
                    "
  {fname} UUID REFERENCES {ref_table}(id) ON DELETE SET NULL,"
                ));
            } else {
                let nullable = if ftype == "text" || ftype.ends_with("id") {
                    "NOT NULL"
                } else {
                    ""
                };
                s.push_str(&format!(
                    "
  {fname} {sql_type} {nullable},"
                ));
            }
        }
        s.push_str(
            "
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

",
        );
    }
    // Add indexes for all FK columns and common query patterns.
    for table in &tables {
        let fields = super_dev_contract::fields_for_entity(table);
        for (fname, _ftype, _desc) in &fields {
            if fname.ends_with("_id") && *fname != "parent_id" {
                s.push_str(&format!(
                    "\nCREATE INDEX IF NOT EXISTS idx_{table}_{fname} ON {table}({fname});\n"
                ));
            }
        }
        // Unique email for users
        if table.as_str() == "users" {
            s.push_str("\nCREATE UNIQUE INDEX IF NOT EXISTS uq_users_email ON users(email);\n");
        }
        // Status index for filtering
        s.push_str(&format!(
            "\nCREATE INDEX IF NOT EXISTS idx_{table}_status ON {table}(status);\n"
        ));
    }
    s.push('\n');

    // Users table gets auth columns.
    s.push_str(
        "-- Auth columns (merge into the users table above or run as a follow-up):\n\
         ALTER TABLE users ADD COLUMN IF NOT EXISTS email TEXT UNIQUE NOT NULL;\n\
         ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash TEXT;\n\
         ALTER TABLE users ADD COLUMN IF NOT EXISTS role TEXT NOT NULL DEFAULT 'user';\n",
    );
    let _ = stack; // migration is SQL regardless of stack
    s
}

fn render_ci_workflow(stack: TechStack) -> String {
    let (setup, test_cmd) = match stack {
        TechStack::Node => (
            "      - uses: actions/setup-node@v4\n        with:\n          node-version: '20'\n          cache: npm\n      - run: npm ci",
            "      - run: npm run lint --if-present\n      - run: npm test",
        ),
        TechStack::Rust => (
            "      - uses: dtolnay/rust-toolchain@stable\n      - uses: Swatinem/rust-cache@v2",
            "      - run: cargo fmt --all -- --check\n      - run: cargo clippy --all-targets -- -D warnings\n      - run: cargo test --all",
        ),
        TechStack::Python => (
            "      - uses: actions/setup-python@v5\n        with:\n          python-version: '3.12'\n      - run: pip install -r requirements.txt",
            "      - run: ruff check .\n      - run: pytest",
        ),
        TechStack::Go => (
            "      - uses: actions/setup-go@v5\n        with:\n          go-version: '1.22'",
            "      - run: go vet ./...\n      - run: go test ./...",
        ),
        TechStack::Unknown => (
            "      - uses: actions/setup-node@v4\n        with:\n          node-version: '20'\n          cache: npm\n      - run: npm ci || true",
            "      - run: npm run lint --if-present\n      - run: npm test --if-present",
        ),
    };
    format!(
        "# Generated by Super Dev — CI pipeline.\n\
         name: CI\n\n\
         on:\n  push:\n    branches: [main]\n  pull_request:\n\n\
         jobs:\n  ci:\n    runs-on: ubuntu-latest\n    steps:\n      \
         - uses: actions/checkout@v4\n{setup}\n{test_cmd}\n      \
         - name: Super Dev quality gate\n        \
         run: |\n          \
         if [ -f output/*-quality-gate.json ]; then\n            \
         # Fail the build if the quality gate didn't pass.\n            \
         python3 -c \"import json,glob; d=json.load(open(glob.glob('output/*-quality-gate.json')[0])); exit(0 if d['passed'] else 1)\"\n          fi\n"
    )
}

/// Read the binary name from a Rust project's Cargo.toml [package].name.
/// Falls back to "app" when unreadable or absent (the template default).
fn rust_binary_name(project_root: &Path) -> String {
    let cargo_toml = project_root.join("Cargo.toml");
    let Ok(content) = std::fs::read_to_string(&cargo_toml) else {
        return "app".to_string();
    };
    // Naive parse: find the [package] section and extract `name = "..."`.
    let mut in_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package {
            if let Some(rest) = trimmed.strip_prefix("name") {
                let rest = rest.trim_start_matches(['=', ' ']);
                let name = rest.trim_matches(|c| c == 34 as char || c == 39 as char);
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    "app".to_string()
}

const NODE_DOCKERFILE: &str = "# Generated by Super Dev — multi-stage Node build.\n\
# Production hardening:\n# - Pin the base image to a digest for reproducibility:\n#     docker build --build-arg NODE_BASE=node:20-alpine@sha256:<digest> .\n# - Or override the base entirely (e.g. a hardened internal image):\n#     docker build --build-arg NODE_BASE=registry.internal/node:20-hardened .\n# Super Dev emits the mutable tag as the default so the scaffold builds\n# out-of-the-box; set NODE_BASE at build time to pin/override.\nARG NODE_BASE=node:20-alpine\nFROM ${NODE_BASE} AS deps\nWORKDIR /app\nCOPY package*.json ./\nRUN npm ci --omit=dev || npm install --omit=dev\n\n\
FROM ${NODE_BASE} AS builder\nWORKDIR /app\nCOPY --from=deps /app/node_modules ./node_modules\nCOPY . .\nRUN npm run build --if-present\n\n\
FROM ${NODE_BASE} AS runner\nWORKDIR /app\nENV NODE_ENV=production\nRUN addgroup -S app && adduser -S app -G app\n\
COPY --from=builder /app/package*.json ./\nCOPY --from=builder /app/node_modules ./node_modules\n\
COPY --from=builder /app/dist ./dist\nCOPY --from=builder /app/public ./public\nCOPY --from=builder /app/src ./src\n\
RUN chown -R app:app /app\nUSER app\nHEALTHCHECK --interval=30s --timeout=3s --retries=3 \\
  CMD wget -qO- http://localhost:3000/api/health || exit 1
EXPOSE 3000\nCMD [\"npm\", \"start\"]\n";

const RUST_DOCKERFILE: &str = "# Generated by Super Dev — multi-stage Rust build.\n\
# NOTE: replace \"app\" below with your binary name (from Cargo.toml [package].name).\n\
FROM rust:1.87-slim AS builder\nWORKDIR /app\nCOPY Cargo.toml Cargo.lock ./\nRUN mkdir src && echo \"fn main() {}\" > src/main.rs && cargo build --release || true\nCOPY . .\nRUN cargo build --release\n\n\
FROM debian:bookworm-slim AS runner\nWORKDIR /app\nRUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*\n\
RUN groupadd -r app && useradd -r -g app app\nCOPY --from=builder /app/target/release/app /usr/local/bin/app\nUSER app\nHEALTHCHECK --interval=30s --timeout=3s --retries=3 \\
  CMD wget -qO- http://localhost:3000/api/health || exit 1
EXPOSE 3000\nCMD [\"app\"]\n";

const PYTHON_DOCKERFILE: &str = "# Generated by Super Dev — Python app (FastAPI/uvicorn).\n\
FROM python:3.12-slim AS builder\nWORKDIR /app\nCOPY requirements.txt .\nRUN pip install --no-cache-dir --user -r requirements.txt\n\n\
FROM python:3.12-slim AS runner\nWORKDIR /app\nRUN groupadd -r app && useradd -r -g app app\nCOPY --from=builder /root/.local /home/app/.local\nCOPY . .\nENV PATH=/home/app/.local/bin:$PATH\nUSER app\nHEALTHCHECK --interval=30s --timeout=3s --retries=3 \\
  CMD wget -qO- http://localhost:3000/api/health || exit 1
EXPOSE 3000\nCMD [\"uvicorn\", \"main:app\", \"--host\", \"0.0.0.0\", \"--port\", \"3000\"]\n";

const GO_DOCKERFILE: &str = "# Generated by Super Dev — multi-stage Go build.\n\
FROM golang:1.22-alpine AS builder\nWORKDIR /app\nCOPY go.* ./\nRUN go mod download\nCOPY . .\nRUN CGO_ENABLED=0 go build -o /app/server .\n\n\
FROM alpine:latest AS runner\nWORKDIR /app\nRUN addgroup -S app && adduser -S app -G app\nCOPY --from=builder /app/server /app/server\nUSER app\nHEALTHCHECK --interval=30s --timeout=3s --retries=3 \\
  CMD wget -qO- http://localhost:3000/api/health || exit 1
EXPOSE 3000\nCMD [\"/app/server\"]\n";

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn spec_with(endpoints: &[(&str, &str)]) -> ApiSpec {
        // Reuse the contract crate's parser via a markdown table.
        let rows: String = endpoints
            .iter()
            .map(|(m, p)| format!("| {m} | {p} | - | - | none | desc |\n"))
            .collect();
        let md = format!("| Method | Path | Request | Response | Auth | Description |\n|---|---|---|---|---|---|\n{rows}");
        super_dev_contract::parse_architecture(&md, "test")
    }

    #[test]
    fn detect_stack_from_manifests() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[package]").unwrap();
        assert_eq!(detect_stack(tmp.path(), ""), TechStack::Rust);
        let tmp2 = TempDir::new().unwrap();
        fs::write(tmp2.path().join("package.json"), "{}").unwrap();
        assert_eq!(detect_stack(tmp2.path(), ""), TechStack::Node);
    }

    #[test]
    fn detect_stack_from_prose_when_no_manifest() {
        assert_eq!(
            detect_stack(Path::new("/nonexistent"), "built with FastAPI and Python"),
            TechStack::Python
        );
        assert_eq!(
            detect_stack(Path::new("/nonexistent"), "uses Go and Gin framework"),
            TechStack::Go
        );
        assert_eq!(
            detect_stack(Path::new("/nonexistent"), "Next.js TypeScript app"),
            TechStack::Node
        );
    }

    #[test]
    fn detect_stack_unknown_defaults_node() {
        assert_eq!(
            detect_stack(Path::new("/nonexistent"), "some random text"),
            TechStack::Unknown
        );
    }

    #[test]
    fn generate_writes_all_artifacts() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), "{}").unwrap();
        let spec = spec_with(&[
            ("GET", "/api/users"),
            ("POST", "/api/users"),
            ("GET", "/api/orders/:id"),
        ]);
        let out = generate_scaffolding(tmp.path(), &spec, "Node app");
        // All 5 artifacts written.
        assert!(out.artifacts.contains(&"Dockerfile".to_string()));
        assert!(out.artifacts.contains(&"docker-compose.yml".to_string()));
        assert!(out.artifacts.contains(&".env.example".to_string()));
        assert!(out
            .artifacts
            .contains(&"migrations/0001_init.sql".to_string()));
        assert!(out
            .artifacts
            .contains(&".github/workflows/ci.yml".to_string()));
        assert_eq!(out.stack, TechStack::Node);
    }

    #[test]
    fn migration_derives_tables_from_paths() {
        let tmp = TempDir::new().unwrap();
        let spec = spec_with(&[("GET", "/api/users"), ("GET", "/api/orders/:id")]);
        let _out = generate_scaffolding(tmp.path(), &spec, "");
        let sql = fs::read_to_string(tmp.path().join("migrations/0001_init.sql")).unwrap();
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS users"));
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS orders"));
        // Namespace segments must NOT become tables.
        assert!(!sql.contains("CREATE TABLE IF NOT EXISTS auth"));
        assert!(!sql.contains("CREATE TABLE IF NOT EXISTS health"));
        // Auth columns on users.
        assert!(sql.contains("password_hash"));
        assert!(sql.contains("email TEXT UNIQUE"));
    }

    #[test]
    fn ci_workflow_includes_quality_gate_step() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), "{}").unwrap();
        let spec = ApiSpec::default();
        let _out = generate_scaffolding(tmp.path(), &spec, "");
        let ci = fs::read_to_string(tmp.path().join(".github/workflows/ci.yml")).unwrap();
        assert!(ci.contains("Super Dev quality gate"));
        assert!(ci.contains("quality-gate.json"));
        assert!(ci.contains("npm test"));
    }

    #[test]
    fn dockerfile_matches_stack() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[package]").unwrap();
        let out = generate_scaffolding(tmp.path(), &ApiSpec::default(), "");
        let df = fs::read_to_string(tmp.path().join("Dockerfile")).unwrap();
        assert!(
            df.contains("cargo build"),
            "rust dockerfile expected, got stack {:?}",
            out.stack
        );
    }

    #[test]
    fn node_dockerfile_has_arg_base_override() {
        // The node Dockerfile must expose a build ARG so the base image can
        // be pinned to a digest or overridden without editing the file.
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), "{}").unwrap();
        let _out = generate_scaffolding(tmp.path(), &ApiSpec::default(), "");
        let df = fs::read_to_string(tmp.path().join("Dockerfile")).unwrap();
        assert!(
            df.contains("ARG NODE_BASE="),
            "node Dockerfile must have an ARG NODE_BASE override, got:\n{df}"
        );
        assert!(
            df.contains("${NODE_BASE}"),
            "node Dockerfile FROM lines must reference ${{NODE_BASE}}, got:\n{df}"
        );
    }

    #[test]
    fn compose_has_postgres_and_healthcheck() {
        let tmp = TempDir::new().unwrap();
        let _out = generate_scaffolding(tmp.path(), &ApiSpec::default(), "");
        let compose = fs::read_to_string(tmp.path().join("docker-compose.yml")).unwrap();
        assert!(compose.contains("postgres:16"));
        assert!(compose.contains("pg_isready"));
        assert!(compose.contains("DATABASE_URL"));
        // redis, app and db must be top-level SIBLINGS under `services:`,
        // not accidentally nested under the top-level `volumes:` block.
        let svcs = compose
            .split("services:\n")
            .nth(1)
            .and_then(|rest| rest.split("\nvolumes:\n").next())
            .unwrap_or("");
        assert!(svcs.contains("redis:"), "redis missing from services block");
        assert!(svcs.contains("  app:"), "app not a top-level service");
        assert!(svcs.contains("  db:"), "db not a top-level service");
        // The only `pgdata:` after the named-volume declaration is the
        // volume itself; the service must NOT contain a bare `pgdata:`
        // sibling (that was the pre-fix malformed-YAML shape).
        assert!(
            !svcs.contains("\n  pgdata:"),
            "pgdata leaked into services block"
        );
    }

    #[test]
    fn env_example_documents_all_vars() {
        let tmp = TempDir::new().unwrap();
        let spec = spec_with(&[("GET", "/api/users"), ("POST", "/api/users")]);
        let _out = generate_scaffolding(tmp.path(), &spec, "Node");
        let env = fs::read_to_string(tmp.path().join(".env.example")).unwrap();
        assert!(env.contains("DATABASE_URL"));
        assert!(env.contains("JWT_SECRET"));
        assert!(env.contains("PORT"));
        assert!(env.contains("2 API endpoints"));
    }

    #[test]
    fn rust_ci_uses_cargo_commands() {
        let ci = render_ci_workflow(TechStack::Rust);
        assert!(ci.contains("cargo fmt"));
        assert!(ci.contains("cargo clippy"));
        assert!(ci.contains("cargo test"));
    }
}
