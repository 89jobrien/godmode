#!/usr/bin/env bash
# Sync a HANDOFF.yaml into doob's DB, then print DB state for that project
# side-by-side so drift is visible at a glance. DB is authoritative on
# conflict (see SKILL.md "Sync conflict order") -- this script does not
# resolve conflicts, it only surfaces them.
#
# Usage: reconcile-handoff.sh <path/to/HANDOFF.project.project.yaml> [project-name]
set -euo pipefail

file="${1:?usage: reconcile-handoff.sh <handoff-file> [project-name]}"
project="${2:-$(basename "$file" | sed -E 's/^HANDOFF\.([^.]+)\..*/\1/')}"

if ! command -v doob >/dev/null 2>&1; then
    echo "doob CLI not found on PATH" >&2
    exit 1
fi

echo "== syncing $file =="
doob handoff sync --file "$file"

echo
echo "== doob DB state for project '$project' =="
doob handoff list -p "$project" --json
