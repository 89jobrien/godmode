#!/usr/bin/env sh
# session-start.sh — run at the top of every Claude Code session in a godmode repo.
# Equivalent to the session ritual in using-godmode/SKILL.md.

set -e

echo "=== godmode session start ==="

# 1. Confirm CLI is installed
if ! command -v godmode >/dev/null 2>&1; then
    echo "ERROR: godmode not found. Install with:"
    echo "  cargo install --path crates/godmode-cli"
    exit 1
fi

# 2. Triage
godmode handon

echo ""
echo "Ready. Next:"
echo "  godmode task next      — see what to work on"
echo "  godmode task start <id> — begin a task"
echo "  godmode status          — mid-session check"
