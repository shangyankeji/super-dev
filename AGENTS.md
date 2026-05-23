# AGENTS.md

Guidance for any AI coding agent (Codex CLI / Codex Desktop / Antigravity
CLI / Claude Code / …) when operating inside this repository.

## What this repo is

A Rust workspace that ships a single binary `super-dev` — a coach for
AI coding hosts. The product is the specification
[`SUPER_DEV_HOST_SPEC_V1`](spec/SUPER_DEV_HOST_SPEC_V1.md); the binary
is one of its delivery surfaces.

## Build / test / lint

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

## Workspace layout

See [README.md](README.md). Short version:

- `crates/super-dev` — binary
- `crates/super-dev-spec` — spec as Rust data
- `crates/super-dev-governance` — rules + audit + compliance kernel
- `crates/super-dev-agent` — 9-phase runner + gates + workflow state
- `crates/super-dev-runtime` — Anthropic / OpenAI / Antigravity HTTP adapters

## Hard rules

- **Pure Rust.** No Python, no Node, no subprocess shims to vendor SDKs.
- **Fail-open governance.** Any governance function must return a pass
  decision (or empty record) on unexpected input — the host MUST NEVER
  be blocked by a bug in the governor.
- **Spec is the source of truth.** When data and prose diverge, fix
  both.
- **Three-runtime scope.** The list of supported host families is exactly
  Anthropic, OpenAI, Antigravity. Do not propose adding others.

## Recommended sequence for new contributors

1. `cargo test --workspace` — green baseline.
2. Read `spec/SUPER_DEV_HOST_SPEC_V1.md`.
3. Skim `crates/super-dev-spec/src/lib.rs` (clauses + phases as data).
4. Open `crates/super-dev-governance/src/rules.rs` to see how a clause
   is enforced end-to-end.
