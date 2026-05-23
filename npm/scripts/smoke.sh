#!/usr/bin/env bash
# Local smoke test for the npm distribution layer.
#
# Builds super-dev for the current platform, stages it into the right
# `cli-*` sub-package, then invokes the JS shim and verifies it execs
# the real binary by checking `--version` output.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NPM_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$NPM_ROOT/.." && pwd)"

# Map uname → our platform key.
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64)   PLATFORM="darwin-arm64" ;;
  Darwin-x86_64)  PLATFORM="darwin-x64" ;;
  Linux-x86_64)   PLATFORM="linux-x64" ;;
  Linux-aarch64)  PLATFORM="linux-arm64" ;;
  *)
    echo "smoke.sh: unsupported uname: $(uname -s)-$(uname -m)" >&2
    exit 1
    ;;
esac

echo "▶ smoke.sh: building super-dev (release) for $PLATFORM..."
(cd "$REPO_ROOT" && cargo build --release --bin super-dev --quiet)

echo "▶ smoke.sh: staging into npm/cli-$PLATFORM/"
"$SCRIPT_DIR/stage.sh" "$PLATFORM" "$REPO_ROOT/target/release/super-dev"

# Cargo pkgid output varies across cargo versions:
#   `super-dev@4.2.0`                       (newer cargo)
#   `path+file://.../super-dev#4.2.0`       (older cargo)
# Both forms put the version after the last `#` or `@` — strip everything before it.
EXPECTED_VERSION="$(cd "$REPO_ROOT" && cargo pkgid -p super-dev | sed -E 's/.*[#@]//' | tail -n1)"

echo "▶ smoke.sh: invoking JS shim → $(node --version)"
OUTPUT="$(node "$NPM_ROOT/super-dev/bin/cli.js" --version 2>&1)"

if [[ "$OUTPUT" == *"$EXPECTED_VERSION"* ]]; then
  echo "✓ smoke.sh: shim resolved + ran binary OK"
  echo "  expected: super-dev $EXPECTED_VERSION"
  echo "  got:      $OUTPUT"
else
  echo "✗ smoke.sh: version mismatch" >&2
  echo "  expected to contain: $EXPECTED_VERSION" >&2
  echo "  got:                 $OUTPUT" >&2
  exit 1
fi
