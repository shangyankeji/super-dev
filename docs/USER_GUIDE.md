# Super Dev User Guide

## What is Super Dev?

Super Dev is the **project manager for AI coding**. It drives your already-installed AI coding CLI (Claude Code, Codex, Gemini, Droid, etc.) through a 9-phase commercial delivery pipeline, ensuring the AI produces code that meets real company standards.

Super Dev itself does NOT write code. It tells your AI coding tool WHAT to produce, checks the quality, and ensures nothing is missed.

## Quick Start

```bash
# 1. Install
npm install -g super-dev

# 2. Initialize a project
cd my-project
super-dev init

# 3. Launch the TUI
super-dev
```

On first launch, pick your worker (the AI coding tool you've already logged into). Then type your requirement and press Enter.

## The 9-Phase Pipeline

```
research → docs → ⏸ docs_confirm → spec → frontend → ⏸ preview_confirm → backend → quality → delivery
```

| Phase | What happens | Expert role |
|---|---|---|
| research | Competitive analysis, user discovery, design direction | Product Researcher |
| docs | PRD + Architecture + UI/UX design system | PM + Architect + Designer |
| docs_confirm | **GATE** — you review the 3 docs before coding starts | You |
| spec | Sprint breakdown, coding standards, task list | Engineering Manager |
| frontend | Worker implements frontend with approved design tokens | Frontend Lead |
| preview_confirm | **GATE** — you review the frontend before backend | You |
| backend | Worker implements API routes, database, auth, tests | Backend Lead |
| quality | 17 automated checks + 5-dimension visual review | QA Lead |
| delivery | Proof-pack zip with README + compliance mapping | Release Engineer |

## TUI Commands

### Worker
| Command | Description |
|---|---|
| `/claude` | Switch to Claude Code CLI |
| `/codex` | Switch to Codex CLI |
| `/gemini` | Switch to Gemini CLI |
| `/droid` | Switch to Droid CLI |
| `/opencode` | Switch to OpenCode CLI |
| `/offline` | Offline templates (no AI) |

### Design
| Command | Description |
|---|---|
| `/design` | Browse available design systems |
| `/design <name>` | Select a design system |
| `/template <name>` | Select a seed template |
| `/model <id>` | Set the AI model |

### Pipeline
| Command | Description |
|---|---|
| `/continue` or `c` | Approve the current gate |
| `/revise <text>` | Request changes at a gate |
| `/run [slug] <req>` | Start a new run |
| `/redo` | Re-run current requirement |
| `/diff <artifact>` | View an artifact (prd/architecture/uiux) |

### Inspect
| Command | Description |
|---|---|
| `/status` | Pipeline progress + quality score |
| `/export` | Export proof-pack |
| `/config` | View all settings |
| `/knowledge` | Browse knowledge files |
| `/doctor` | Self-test |
| `/verify` | Workspace conformance |

### General
| Command | Description |
|---|---|
| `/help` | All commands |
| `/clear` | Clear chat history |
| `/quit` | Exit |

## Design Systems

Super Dev ships 5 design systems. Select one before running to get deterministic visual output:

| Name | Best for |
|---|---|
| `modern-minimal` | SaaS, dev tools, dashboards |
| `editorial-clean` | Blogs, content sites, portfolios |
| `tech-utility` | CLI companions, monitoring, data tools |
| `soft-warm` | Consumer apps, education, wellness |
| `bold-geometric` | Brand launches, creative agencies |

## Seed Templates

| Name | Structure |
|---|---|
| `saas-landing` | Nav → Hero → Trust → Features → Pricing → FAQ → Footer |
| `dashboard` | Sidebar + KPI cards + Charts + Data table |
| `blog-content` | Featured article + Grid + Newsletter |
| `e-commerce` | Gallery + Product info + Variants + Reviews + Related |
| `auth-system` | Login + Signup + Forgot + MFA + Reset |
| `settings-page` | Sidebar tabs + Profile + Security + Billing |
| `docs-site` | Sidebar nav + Content + Code blocks + Search |

## Configuration

### `.superdevrc` (project-level)

```toml
[quality]
threshold = 85              # quality gate pass threshold (default: 90)
skip_checks = ["dark_mode"] # skip specific checks

[pipeline]
skip_phases = ["research"]  # skip phases you don't need
max_review_rounds = 2       # limit auto-fix cycles (default: 3)

[experts]
custom_knowledge = "team-standards/"  # additional knowledge directory
```

### `~/.super-dev/config.toml` (user-level)

```toml
backend = "claude-code"
model = "claude-sonnet-4-6"
design_system = "modern-minimal"
seed_template = "dashboard"
```

## Quality Gate

Super Dev runs 17 automated checks:

| Category | Checks |
|---|---|
| Artifacts | Research, PRD, Architecture, UIUX — content structure validation |
| Cross-reference | PRD↔Architecture route alignment, API URL consistency |
| Code quality | Emoji check, hardcoded colors, anti-AI-slop patterns |
| Design | UIUX token count, dark mode presence, design system completeness |
| Evidence | Audit log, tool-call log, discovery section |
| Depth | Acceptance criteria count, API route count |

## Expert Knowledge

Each pipeline phase is backed by a specialist's methodology:

| Expert | Knowledge | Used in |
|---|---|---|
| Product Manager | RICE scoring, AC format, edge cases, HEART metrics | Research, PRD |
| Architect | API design standards, security checklist (OWASP), auth patterns | Architecture |
| UI/UX Designer | Token architecture, interaction principles, WCAG 2.1, responsive | UIUX, Frontend |
| Frontend Lead | Component architecture, state management, error handling, performance | Frontend |
| Backend Lead | API handler pattern, database practices, JWT flow, logging standards | Backend |
| QA Lead | Test pyramid, AC→test conversion, pre-release checklist | Quality |
| DevOps | CI/CD pipeline, Docker, monitoring, rollback strategy | Delivery |

## FAQ

**Q: Do I need an API key?**
No. Super Dev drives your already-logged-in AI coding CLI. It uses your existing subscription.

**Q: What if the worker times out?**
Super Dev retries once. If it still fails, it falls back to an offline template with TODO markers. You can `/redo` to try again.

**Q: Can I customize the quality checks?**
Yes, via `.superdevrc`. Set `skip_checks` to disable specific checks, or `threshold` to change the pass score.

**Q: Does it work offline?**
Yes. Without a worker, it generates structured templates with TODO markers — useful for planning without AI.
