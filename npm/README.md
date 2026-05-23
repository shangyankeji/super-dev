# Super Dev — npm distribution

This directory packages Super Dev for `npm install -g super-dev`. End
users only ever see the `super-dev` name; the multi-package layout is
the standard pattern used by `esbuild`, `biome`, `swc`, `@tailwindcss/oxide`,
etc. to ship a prebuilt Rust binary via npm without forcing every user
to download every platform's binary.

## Layout

```
npm/
├── super-dev/                        ← the user-facing package (`npm i -g super-dev`)
│   ├── package.json                  ← optionalDependencies for every platform
│   ├── bin/cli.js                    ← thin JS shim, resolves + execs the binary
│   └── README.md                     ← end-user README rendered on the npm page
├── cli-darwin-arm64/                 ← per-platform sub-package (one binary each)
│   └── package.json                  ←   os: ["darwin"], cpu: ["arm64"]
├── cli-darwin-x64/                   ← Intel Mac
├── cli-linux-x64/                    ← Linux x86_64
├── cli-linux-arm64/                  ← Linux ARM (Pi, Graviton, ARM cloud)
├── cli-win32-x64/                    ← Windows x86_64
└── scripts/
    ├── stage.sh                      ← copy a built binary into its sub-package
    ├── smoke.sh                      ← local end-to-end smoke test
    └── publish.sh                    ← publish all 6 packages to npm
```

## Why this works (the npm magic)

Every platform sub-package declares `os` / `cpu` in its `package.json`:

```json
{ "os": ["darwin"], "cpu": ["arm64"] }
```

The main `super-dev` package lists all five platform packages under
`optionalDependencies`. When a user runs `npm i -g super-dev`, npm:

1. Tries to install every `optionalDependency`.
2. Silently skips any whose `os` / `cpu` does not match the current host.
3. Ends up installing only the matching `@super-dev/cli-<platform>`.

The JS shim then uses `require.resolve('@super-dev/cli-<platform>/bin/super-dev')`
to locate the binary and `child_process.spawnSync` to exec it with the
user's argv. `stdio: 'inherit'` preserves the TTY so the ratatui UI
works.

## How a release flows

1. CI builds `super-dev` for each target (see `.github/workflows/release.yml`).
2. For each target the CI calls `npm/scripts/stage.sh <platform> <binary>`.
3. `npm/scripts/publish.sh` publishes every platform package (`@super-dev/cli-*`)
   first, then the main `super-dev` package last so its
   `optionalDependencies` resolve cleanly.

## Local smoke test (M8 verification)

```bash
./npm/scripts/smoke.sh
```

Builds super-dev release-mode for the host platform, stages it into
the matching `cli-<platform>/bin/`, then invokes
`node npm/super-dev/bin/cli.js --version` and asserts the binary's
real version string came through.

## Maintenance

The version in **every** `package.json` (main + 5 platforms) must
match the workspace `Cargo.toml#workspace.package.version`. The
`smoke.sh` test fails if the JS shim's resolved binary reports a
different version than `cargo pkgid` does.

When bumping versions, update:
- `Cargo.toml` (workspace + 6 internal-dep refs)
- `npm/super-dev/package.json`
- `npm/cli-*/package.json` (×5)
- `npm/super-dev/package.json#optionalDependencies` versions

A future `scripts/bump-version.sh` could do this in one shot.
