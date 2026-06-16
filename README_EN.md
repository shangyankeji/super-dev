# Super Dev

<div align="center">

<img src="docs/assets/super-dev-logo.png" alt="Super Dev — A coach for AI coding hosts" width="600">

### A coach + orchestrator for AI coding hosts · One spec, one binary, one TUI

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![Spec](https://img.shields.io/badge/spec-SUPER__DEV__HOST__SPEC__V1-blue)](spec/SUPER_DEV_HOST_SPEC_V1.md)
[![Version](https://img.shields.io/badge/version-4.6.0-success)](CHANGELOG.md)

[简体中文](README.md) | English

</div>

---

## What is Super Dev

`Super Dev` is a **coach + orchestrator for AI coding hosts**. It does not write code and it is not an IDE. It lands a complete commercial-project delivery specification ([SUPER_DEV_HOST_SPEC_V1](spec/SUPER_DEV_HOST_SPEC_V1.md)) as a 9-phase pipeline — what to research first, what artifacts to produce, when to pause for sign-off, what files to refuse, what evidence to leave behind.

**Headline mode: drive the host CLI you already logged into.** Super Dev treats your already-installed `claude` / `codex` commands as on-demand execution backends — **no API key needed**, it runs on your existing host subscription. Type `super-dev` in your terminal to enter a Claude Code-style chat where you converse with the project manager.

One binary, zero runtime dependencies, single-file distribution.

## Two facts that anchor everything

- **We do not call any LLM API** — Super Dev itself is a deterministic Rust state machine
- **The actual coding is done by the Claude Code / Codex you already logged into on this machine** — we just schedule them

| Mode | Command | Needs an API key | Notes |
|---|---|---|---|
| **Host CLI** (recommended) | `--backend claude-code` / `--backend codex` | **No** | Drives your logged-in host CLI on its existing subscription |
| **Offline** | (default, no flag) | No | Deterministic templates, no network (demo / CI only) |

> Earlier 1.x experimented with shipping built-in HTTP clients for Anthropic / OpenAI / Antigravity. 4.0 **removed them** — Super Dev is the "project manager", not an LLM client.

## 23 supported AI coding CLIs

Super Dev drives every mainstream AI coding tool that ships a non-interactive CLI — whichever you have installed works:

| Backend ID | Binary | Non-interactive form | TUI |
|---|---|---|---|
| `claude-code` | `claude` | `claude --print --output-format text "<p>"` | `/claude` |
| `codex` | `codex` | `codex exec --sandbox workspace-write "<p>"` | `/codex` |
| `gemini` | `gemini` | `gemini -p "<p>"` | `/gemini` |
| `droid` | `droid` | `droid exec --auto low -o text "<p>"` | `/droid` |
| `opencode` | `opencode` | `opencode run "<p>"` | `/opencode` |
| `qwen` | `qwen` | `qwen -p "<p>"` | `/qwen` |
| `copilot` | `copilot` | `copilot -p --allow-all-tools "<p>"` | `/copilot` |
| `trae` | `trae-cli` | `trae-cli run "<p>"` | `/trae` |
| `codebuddy` | `codebuddy` | `codebuddy -p -y "<p>"` | `/codebuddy` |
| `qoder` | `qodercli` | `qodercli -p "<p>"` | `/qoder` |
| `kimi` | `kimi` | `kimi --print --output-format text "<p>"` | `/kimi` |
| `cursor-agent` | `cursor-agent` | `cursor-agent -p --output-format text "<p>"` | `/cursor-agent` |
| `continue` | `cn` | `cn -p "<p>"` | `/continue` |
| `aider` | `aider` | `aider --yes --no-stream --message "<p>"` | `/aider` |
| `plandex` | `plandex` | `plandex tell --skip-menu --stop "<p>"` | `/plandex` |
| `cody` | `cody` | `cody chat --message "<p>"` | `/cody` |
| `goose` | `goose` | `goose run -t "<p>"` | `/goose` |
| `amp` | `amp` | `amp -x "<p>"` | `/amp` |
| `junie` | `junie` | `junie "<p>"` | `/junie` |
| `grok-build` | `grok-build` | `grok-build -p "<p>"` | `/grok-build` |
| `amazon-q` | `q` | `q chat --no-interactive --trust-all-tools "<p>"` | `/amazon-q` |
| `crush` | `crush` | `crush run "<p>"` | `/crush` |
| `gptme` | `gptme` | `gptme -n "<p>"` | `/gptme` |

`/antigravity` is registered as an alias for `/gemini` (Google released the Antigravity CLI on 5/19 to supersede Gemini CLI — the alias is forward-compatible). The authoritative list is `super_dev_host::BACKEND_IDS`; run `super-dev doctor` to see which are installed locally.

## Install

```bash
# ★ Recommended — one line, npm auto-picks the prebuilt binary for your OS/CPU
npm install -g super-dev
```

Or build from source:

```bash
git clone https://github.com/shangyankeji/super-dev.git
cd super-dev && cargo build --release
cp target/release/super-dev /usr/local/bin/
```

Supported platforms: macOS (Intel + Apple Silicon), Linux (x86_64 + ARM64), Windows x86_64. npm only installs the platform sub-package that matches your machine.

## Usage

```bash
# Initialise the workspace (writes the super-dev.yaml spec manifest)
# ★ Simplest invocation — auto-detects your logged-in claude-code / codex
super-dev

# ★ Simplest invocation — drops you into a Claude Code-style chat.
#   First launch picks a worker (claude-code / codex / offline) and
#   saves it to ~/.super-dev/config.toml. Switch later with /claude /codex.
super-dev

# Initialise a workspace (writes the super-dev.yaml spec manifest;
# run once per project)
super-dev init

# === Command-line pipeline (scripts / CI) ===
super-dev run "Build a login system"                      # offline deterministic templates
super-dev run "..." --backend claude-code                 # drive a host CLI, no key
super-dev run "..." --backend codex
super-dev continue                                        # approve the active gate
super-dev revise "make the hero dark"                     # stay at the gate, request a revision

# === Help / discovery ===
super-dev examples                                        # cheat-sheet
super-dev guide                                           # 60-second walkthrough
super-dev doctor                                          # self-test

# === Status & spec ===
super-dev verify                                          # verify workspace conformance
super-dev spec [--clauses]                                # print the spec / clause table
super-dev report                                          # emit the SD-EVID-004 compliance mapping
```

### Slash commands inside the TUI

| Command | Action |
|---|---|
| `/claude` / `/codex` / `/offline` | switch the worker (saves to ~/.super-dev/config.toml) |
| `/continue` | approve the active gate |
| `/revise <text>` | stay at the gate, request changes |
| `/help` or `F1` | help overlay |
| `/clear` | clear chat history |
| `/quit` or `Esc` | exit |
| (plain text) | new requirement, or revision if a gate is open |

## Spec coverage (25 clauses × 4 layers)

See [SUPER_DEV_HOST_SPEC_V1.md](spec/SUPER_DEV_HOST_SPEC_V1.md).

- **Layer 1 — Code constraints** (4): no emoji icons, design tokens for colors, frontend/backend API alignment, tech-stack pre-research
- **Layer 2 — Flow contract** (6): phase chain, `docs_confirm` / `preview_confirm` gates, gate-local revisions, session continuity
- **Layer 3 — Delivery artifacts** (6): PRD / architecture / UIUX / spec / tasks / ADRs
- **Layer 4 — Evidence chain** (5): API audit, tool-call audit, quality report, SOC 2 / ISO 27001 / EU AI Act compliance mapping, proof pack

## Project layout

```
super-dev/
├── Cargo.toml                 # workspace manifest
├── crates/
│   ├── super-dev/             # main binary (CLI + TUI entry)
│   ├── super-dev-spec/        # spec as Rust data
│   ├── super-dev-governance/  # rules / audit / context / compliance kernel
│   ├── super-dev-agent/       # 9-phase runner + gates + state + event stream + manifest
│   ├── super-dev-runtime/     # Runtime trait + OfflineRuntime (deterministic fallback)
│   ├── super-dev-host/        # HostDriver — drives 23 logged-in host CLIs
│   ├── super-dev-contract/    # OpenAPI 3.1 contract layer (derive / parse / validate)
│   ├── super-dev-knowledge/   # BM25 + optional vector (hybrid) knowledge retrieval
│   └── super-dev-tui/         # ratatui terminal app
├── spec/
│   └── SUPER_DEV_HOST_SPEC_V1.md   # normative specification
├── knowledge/                 # governance knowledge base
├── super-dev-website/         # Next.js marketing site (separate project)
└── docs/assets/               # images used by README + spec
```

## Development

```bash
# Build everything
cargo build --workspace

# Run all tests
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets

# Format
cargo fmt --all
```

## License

MIT — see [LICENSE](LICENSE).
