#!/usr/bin/env bash
# Find all HANDOFF.*.yaml files across the workspace and show their git repo.
# Usage: find-handoffs.sh [root-dir]   (defaults to ~/dev)
set -euo pipefail

root="${1:-$HOME/dev}"

find "$root" -maxdepth 4 -type f -name 'HANDOFF.*.yaml' -not -path '*/target/*' -not -path '*/node_modules/*' 2>/dev/null \
    | while read -r f; do
        repo_dir=$(git -C "$(dirname "$f")" rev-parse --show-toplevel 2>/dev/null || echo "(not a git repo)")
        printf '%s\n  repo: %s\n' "$f" "$repo_dir"
    done
