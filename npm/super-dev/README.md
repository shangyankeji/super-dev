# Super Dev

> **AI 编码的项目经理** — drives the Claude Code / Codex you already
> logged into through a 9-phase commercial delivery pipeline.
> **No API key needed.**

## Install

```bash
npm install -g super-dev
```

## Use

```bash
super-dev                            # launch the interactive TUI
                                     # (auto-detects logged-in claude / codex)

super-dev init                       # write super-dev.yaml spec manifest

super-dev run "做一个登录系统" \
            --backend claude-code    # scripted form, no TUI
super-dev continue                   # approve the active gate
super-dev revise "去掉 OAuth"        # request a revision

super-dev verify                     # workspace conformance report
super-dev doctor                     # self-test
super-dev spec [--clauses]           # print SUPER_DEV_HOST_SPEC_V1
super-dev report                     # emit SD-EVID-004 compliance map
```

## Why this exists

Super Dev is **not** an LLM client. It does not call any AI API.
Instead it **drives** the host CLI you already use (`claude`, `codex`)
through a deterministic 9-phase pipeline:

```
research → docs → ⏸ docs_confirm → spec → frontend → ⏸ preview_confirm → backend → quality → delivery
```

At each `⏸ gate`, Super Dev pauses and surfaces the artifacts (PRD,
architecture, UIUX, …) for you to review. After every code-producing
phase it runs the project's build / test command (e.g. `cargo check`,
`npm install`) and records the outcome in `.super-dev/audit/verify.jsonl`
so a non-technical user can ship stable code without writing any.

The result is a `release/proof-pack-*.zip` containing every artifact,
every gate decision, and every audit row.

## Documentation

Full docs, design rationale, and the SUPER_DEV_HOST_SPEC_V1 spec:
<https://github.com/shangyankeji/super-dev>

## License

MIT
