#!/usr/bin/env bash
# Stage a prebuilt super-dev binary into the matching platform package.
#
# Used by:
#   - Local smoke testing (no need to publish first)
#   - The release pipeline (CI builds N platforms, calls this N times)
#
# Idempotent: re-running with the same args is a no-op apart from the
# file copy.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NPM_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage: stage.sh <platform> <binary-path>

  platform     darwin-arm64 | darwin-x64 | linux-x64 | linux-arm64 | win32-x64
  binary-path  path to the prebuilt super-dev[.exe] binary

Examples:
  stage.sh darwin-arm64 target/release/super-dev
  stage.sh win32-x64    target/x86_64-pc-windows-msvc/release/super-dev.exe
USAGE
  exit 1
}

[[ $# -eq 2 ]] || usage
PLATFORM="$1"
BINARY="$2"

[[ -f "$BINARY" ]] || { echo "stage.sh: binary not found: $BINARY" >&2; exit 1; }

case "$PLATFORM" in
  darwin-arm64|darwin-x64|linux-x64|linux-arm64)
    BIN_NAME="super-dev"
    ;;
  win32-x64)
    BIN_NAME="super-dev.exe"
    ;;
  *)
    echo "stage.sh: unsupported platform: $PLATFORM" >&2
    exit 1
    ;;
esac

DEST_DIR="$NPM_ROOT/cli-$PLATFORM/bin"
[[ -d "$NPM_ROOT/cli-$PLATFORM" ]] || {
  echo "stage.sh: no sub-package npm/cli-$PLATFORM/ (typo?)" >&2
  exit 1
}
mkdir -p "$DEST_DIR"
cp "$BINARY" "$DEST_DIR/$BIN_NAME"
chmod +x "$DEST_DIR/$BIN_NAME"

echo "stage.sh: $BINARY → $DEST_DIR/$BIN_NAME"
