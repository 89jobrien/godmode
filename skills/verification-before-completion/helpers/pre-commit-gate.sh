#!/usr/bin/env sh
# pre-commit-gate.sh — run before claiming any task is done or creating a PR.
# Usage: sh skills/verification-before-completion/helpers/pre-commit-gate.sh [crate]

set -e

CRATE=${1:-}

if [ -n "$CRATE" ]; then
    echo "=== nextest: $CRATE ==="
    cargo nextest run -p "$CRATE"
    echo "=== clippy: $CRATE ==="
    cargo clippy -p "$CRATE" -- -D warnings
    echo "=== fmt check: $CRATE ==="
    cargo fmt -p "$CRATE" -- --check
else
    echo "=== nextest: workspace ==="
    cargo nextest run --workspace
    echo "=== clippy: workspace ==="
    cargo clippy --workspace -- -D warnings
    echo "=== fmt check: all ==="
    cargo fmt --all -- --check
fi

echo "=== recent commits ==="
git log --oneline -5

echo ""
echo "All gates passed."
