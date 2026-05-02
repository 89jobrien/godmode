#!/usr/bin/env bash
# check-blocked.sh — After any Agent tool call, scan for BLOCKED.md files
# and surface them so the orchestrator session sees them immediately.

ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

# Check repo root and any active worktrees under .worktrees/
BLOCKED_FILES=()
[ -f "$ROOT/BLOCKED.md" ] && BLOCKED_FILES+=("$ROOT/BLOCKED.md")
for f in "$ROOT"/.worktrees/*/BLOCKED.md; do
    [ -f "$f" ] && BLOCKED_FILES+=("$f")
done

if [ ${#BLOCKED_FILES[@]} -gt 0 ]; then
    echo ""
    echo "BLOCKED.md detected:"
    for blocked in "${BLOCKED_FILES[@]}"; do
        echo "  $blocked"
        echo "--- content ---"
        head -30 "$blocked"
        echo "---"
    done
    echo "Surface to user before dispatching further agents."
fi
