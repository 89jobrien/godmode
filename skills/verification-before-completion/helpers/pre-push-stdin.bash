#!/usr/bin/env bash
# pre-push-stdin.bash — synthetic stdin generator for testing pre-push hooks.
#
# Git pre-push hooks receive lines on stdin in the format:
#   <local ref> <local sha> <remote ref> <remote sha>
#
# Source this file to get helper functions that produce valid stdin
# without needing a real remote.

ZERO_SHA="0000000000000000000000000000000000000000"

# fake_remote_stdin [local_ref] [remote_ref]
#
# Outputs a single stdin line suitable for piping into a pre-push hook.
# Uses the current HEAD as local SHA and the zero SHA as remote SHA
# (simulating a new branch push).
#
# Usage:
#   fake_remote_stdin | .git/hooks/pre-push origin https://example.com
fake_remote_stdin() {
    local local_ref="${1:-refs/heads/main}"
    local remote_ref="${2:-refs/heads/main}"
    local local_sha
    local_sha="$(git rev-parse HEAD 2>/dev/null || echo "$ZERO_SHA")"

    printf '%s %s %s %s\n' "$local_ref" "$local_sha" "$remote_ref" "$ZERO_SHA"
}

# fake_delete_stdin [ref]
#
# Outputs stdin simulating a branch deletion push (local SHA is zero).
fake_delete_stdin() {
    local ref="${1:-refs/heads/feature}"
    printf '%s %s %s %s\n' "(delete)" "$ZERO_SHA" "$ref" "$ZERO_SHA"
}

# fake_update_stdin [local_ref] [remote_ref] [remote_sha]
#
# Outputs stdin simulating an update to an existing remote branch.
# Uses HEAD as local SHA; caller provides the old remote SHA.
fake_update_stdin() {
    local local_ref="${1:-refs/heads/main}"
    local remote_ref="${2:-refs/heads/main}"
    local remote_sha="${3:-$(git rev-parse HEAD~1 2>/dev/null || echo "$ZERO_SHA")}"
    local local_sha
    local_sha="$(git rev-parse HEAD 2>/dev/null || echo "$ZERO_SHA")"

    printf '%s %s %s %s\n' "$local_ref" "$local_sha" "$remote_ref" "$remote_sha"
}
