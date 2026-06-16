# Contributing to Super Dev | 贡献指南

Thank you for contributing to Super Dev! 感谢贡献。

Super Dev 3.x is a **Rust workspace** that ships a single static binary
plus per-host plugin bundles. Contributions can target any layer:
governance kernel, agent runner, runtime adapters, CLI, plugin
manifests, or the SUPER_DEV_HOST_SPEC_V1 spec itself.

## How to contribute | 如何贡献

1. Fork [super-dev](https://github.com/shangyankeji/super-dev).
2. Create a feature branch: `git checkout -b feat/your-feature`.
3. Make your changes; run the local checks below.
4. Open a Pull Request against `main`.

## Development setup | 开发环境

Required:

- Rust **1.75+** (stable channel; check with `rustc --version`).
- A working `cargo` (`rustup` is the easiest installer).
- No Python, no Node, no Docker — Super Dev 3.x is pure Rust.

```bash
git clone https://github.com/shangyankeji/super-dev.git
cd super-dev
cargo build --workspace
```

## Workspace layout | 工作区结构

```
crates/
├── super-dev/             # main binary (clap CLI)
├── super-dev-spec/        # SUPER_DEV_HOST_SPEC_V1 as Rust data
├── super-dev-governance/  # rules / audit / context / compliance kernel
├── super-dev-agent/       # 9-phase runner + gates + state + experts + coach
├── super-dev-host/        # subprocess drivers for 23 host CLIs (claude/codex/simple)
└── super-dev-runtime/     # Runtime trait + OfflineRuntime (deterministic fallback)

spec/
└── SUPER_DEV_HOST_SPEC_V1.md   # normative specification
```

## Local checks | 本地校验

Every PR must pass these three commands clean:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Convenient aliases (no installation required):

```bash
cargo fmt --all                       # apply formatting
cargo clippy --workspace --fix        # auto-apply safe lint fixes
cargo test --workspace --all-targets  # unit + integration + doc tests
```

## Adding a new spec clause | 新增规范条款

Spec changes touch four places that **must stay in sync**:

1. **Markdown** — add the new clause section to `spec/SUPER_DEV_HOST_SPEC_V1.md`.
2. **Rust data** — append a `Clause { id, layer, title, level, section }`
   entry to `crates/super-dev-spec/src/lib.rs#CLAUSES`. IDs are
   permanent — never renumber.
3. **Implementation** — if the clause is enforceable, add the
   judgment / audit logic to `crates/super-dev-governance/src/{rules,audit,...}.rs`.
4. **Compliance mapping** — if the clause maps to external frameworks,
   extend `framework_for()` in
   `crates/super-dev-governance/src/compliance.rs`.

Tests in `crates/super-dev-spec/src/lib.rs` pin the clause-table
structure; they will fail if you add a malformed ID. Add a unit test
for the new rule alongside the implementation.

## Adding a new host | 新增宿主

A "host" is a non-interactive AI coding CLI driven as a subprocess —
**no plugin bundles, no install.rs, no Agent SDK required**. The driver
lives entirely in `crates/super-dev-host/`.

1. Add a factory to `crates/super-dev-host/src/simple.rs`
   (`SimpleHostDriver::<name>()`) with the program name, base args, the
   non-interactive prompt channel, and a `version_args` for probing.
   Honour the `SUPER_DEV_<NAME>_BIN` env override like the other factories.
2. Register the id in **two places** (a test keeps them in sync):
   - `driver_for()` and `BACKEND_IDS` in
     `crates/super-dev-host/src/lib.rs`, and
   - a new `BackendArg` variant + `id()`/`from_id()`/`BACKEND_ARG_IDS`
     entry in `crates/super-dev/src/main.rs`.
3. Add the executable name to the doctor probe table in
   `crates/super-dev/src/doctor.rs#check_ai_backends`.
4. The TUI needs **no change** — slash verbs, palette, and did-you-mean
   are derived dynamically from `BACKEND_IDS`.

Only hosts with a documented **non-interactive CLI** form are in scope
(`binary [flags] "<prompt>"` → stdout). The `backend_arg_ids_match_host`
and `every_backend_arg_has_a_driver` tests in `main.rs` will fail if the
selector and driver registry drift apart.

## Commit conventions | 提交规范

Follow [Conventional Commits](https://www.conventionalcommits.org):

```
feat(scope): description    # new functionality
fix(scope): description     # bug fix
docs: description           # documentation only
test: description           # tests only
refactor(scope): description
chore: description          # tooling, deps
ci: description             # GitHub Actions / release workflow
```

Common scopes: `spec`, `governance`, `agent`, `runtime`, `cli`,
`install`, `plugin`, `coach`.

## PR checklist | PR 自检清单

Before requesting review:

- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy -D warnings` clean
- [ ] `cargo test --workspace` green
- [ ] New code has unit tests in the same file (`mod tests { ... }`)
- [ ] If you changed `spec/SUPER_DEV_HOST_SPEC_V1.md`, you also
      changed `crates/super-dev-spec/src/lib.rs#CLAUSES` (or vice versa)
- [ ] PR description explains the *why*, not just the *what*
- [ ] CHANGELOG.md updated under `[Unreleased]` for user-visible changes

## Reporting issues | 报告问题

Open issues at https://github.com/shangyankeji/super-dev/issues with:

- `super-dev verify` output (paste verbatim)
- Reproduction steps
- Expected vs actual behavior
- OS + `rustc --version` for build issues

## License | 许可

By contributing you agree your code is licensed under the project's
[MIT License](LICENSE).
