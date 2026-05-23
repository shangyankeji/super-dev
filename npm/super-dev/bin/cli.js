#!/usr/bin/env node
// SPDX-License-Identifier: MIT
//
// super-dev — thin JS shim. npm picks the matching `@super-dev/cli-*`
// platform sub-package (via optionalDependencies + the `os` / `cpu`
// fields in each sub-package). This shim resolves that sub-package's
// prebuilt Rust binary and exec's it with the user's argv.
//
// The shim is deliberately minimal:
//   - no dependencies (zero install-time cost beyond node itself)
//   - no parsing of argv (every flag goes straight to the binary)
//   - stdio is inherited so the ratatui TUI gets a real TTY

'use strict';

const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

// Node platform/arch → our sub-package name.
const PLATFORM_PACKAGES = {
  'darwin-arm64': '@super-dev/cli-darwin-arm64',
  'darwin-x64': '@super-dev/cli-darwin-x64',
  'linux-x64': '@super-dev/cli-linux-x64',
  'linux-arm64': '@super-dev/cli-linux-arm64',
  'win32-x64': '@super-dev/cli-win32-x64',
};

function platformKey() {
  return `${process.platform}-${process.arch}`;
}

function binaryName() {
  return process.platform === 'win32' ? 'super-dev.exe' : 'super-dev';
}

function findBinary() {
  const key = platformKey();
  const pkg = PLATFORM_PACKAGES[key];
  if (!pkg) {
    const supported = Object.keys(PLATFORM_PACKAGES).join(', ');
    console.error(
      `super-dev: unsupported platform ${key}. Supported: ${supported}.`,
    );
    console.error(
      'Open an issue at https://github.com/shangyankeji/super-dev/issues',
    );
    process.exit(1);
  }
  const bin = binaryName();

  // 1) Published case — npm installed the sibling platform package.
  try {
    return require.resolve(`${pkg}/bin/${bin}`);
  } catch (_) {
    /* fall through */
  }

  // 2) Local dev — both packages live as siblings under npm/.
  const sibling = path.resolve(
    __dirname,
    '..',
    '..',
    `cli-${process.platform}-${process.arch}`,
    'bin',
    bin,
  );
  if (fs.existsSync(sibling)) return sibling;

  console.error(
    `super-dev: ${pkg} not installed.\n` +
      'Try: npm install -g super-dev --force\n' +
      "(npm 'optionalDependencies' should normally pick the right one.)",
  );
  process.exit(1);
}

const binary = findBinary();
const result = spawnSync(binary, process.argv.slice(2), {
  stdio: 'inherit',
});

if (result.error) {
  console.error(`super-dev: failed to exec binary: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status === null ? 1 : result.status);
