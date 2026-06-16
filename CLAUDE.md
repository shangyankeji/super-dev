# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working
with code in this repository.

## What this project is

Super Dev is a Rust workspace that ships a **single binary** (`super-dev`)
acting as a coach + orchestrator for AI coding hosts. It embeds
`SUPER_DEV_HOST_SPEC_V1` (see `spec/`).

As of 4.0 the **primary execution mode drives an already-logged-in host
CLI** (`claude --print`, `codex exec`) as a subprocess — no API key, the
user's existing host session is the brain. The binary has three modes:

- `--backend <id>` — drive a logged-in host CLI (no key); 23 backends supported
- default — offline deterministic templates

Just typing `super-dev` (no subcommand) launches a Claude Code-style chat
TUI over the same engine — first launch shows a backend picker that writes
`~/.super-dev/config.toml`; later launches drop straight into the
conversation. Slash commands (`/claude` `/codex` `/offline` `/continue`
`/revise` `/help` `/clear` `/quit`) live inside the chat.

3.0+ is a complete rebuild from a previous Python implementation; do not
look for `super_dev/` or `pyproject.toml` — they are intentionally gone.

## Build & test

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

## Workspace layout

| Crate | Purpose |
|---|---|
| `crates/super-dev` | The `super-dev` binary (clap CLI: `init` / `run` / `tui` / `continue` / `revise` / `spec` / `hook` / `verify` / `report` / `install` / `uninstall` / `doctor`) |
| `crates/super-dev-spec` | `SUPER_DEV_HOST_SPEC_V1` as Rust data — clauses, phases, gates, runtime kinds |
| `crates/super-dev-governance` | `rules` (block emoji / hardcoded colors), `audit` (API + tool-call JSONL), `context` (session injection), `compliance` (SD-EVID-004 mapping) |
| `crates/super-dev-agent` | 9-phase pipeline runner, gate semantics, workflow state, `events` stream, `manifest` (SD-META-001) |
| `crates/super-dev-runtime` | `Runtime` trait + Anthropic / OpenAI / Antigravity HTTP adapters + `OfflineRuntime` |
| `crates/super-dev-host` | `HostDriver` trait — drives a logged-in `claude` / `codex` CLI as a subprocess |
| `crates/super-dev-tui` | ratatui terminal app over the engine event stream |

## Conventions

- All `pub` items have docstrings.
- Every governance function is **fail-open**: an error path returns `Decision::pass()` or an empty record. The host MUST NEVER be blocked by a bug in the governor.
- Every clause in `super-dev-spec::CLAUSES` is tagged with its `SD-LAYER-NNN` id (e.g. `SD-CODE-001`). When you write or modify a governance rule, reference the clause id in the docstring.
- Tests live next to code (`mod tests { ... }` at the bottom of each `.rs`).

## Spec sync contract

`spec/SUPER_DEV_HOST_SPEC_V1.md` is the normative prose. Any change to
`super-dev-spec::CLAUSES` MUST be accompanied by a change to the
matching section of the markdown, and vice versa. The unit tests in
`crates/super-dev-spec/src/lib.rs` lock the data shape; add new clauses
there in `SD-LAYER-NNN` order.

## What lives outside the Rust workspace

- `knowledge/` — curated knowledge base (language-agnostic, used by the agent at runtime)
- `super-dev-website/` — Next.js marketing site (independent build)
- `output/`, `.super-dev/` — per-project user data (gitignored)
- `docs/assets/` — README images

## Anti-rules (do not undo these)

- Do not reintroduce Python packaging (`pyproject.toml`, `super_dev/`).
- Only add adapters for hosts that have a documented non-interactive CLI
  form (`binary [flags] "<prompt>"` → stdout). Currently 23 backends —
  see `super_dev_host::BACKEND_IDS` in `crates/super-dev-host/src/lib.rs`
  for the authoritative list (claude-code, codex, gemini, droid, opencode,
  qwen, copilot, trae, codebuddy, qoder, kimi, cursor-agent, continue,
  aider, plandex, cody, goose, amp, junie, grok-build, amazon-q, crush,
  gptme).
- Do not vendor any host SDK crate. Super Dev is pure-Rust by design.
  Driving the user's *installed* CLI as a subprocess — see
  `super-dev-host` — is the intended architecture.
