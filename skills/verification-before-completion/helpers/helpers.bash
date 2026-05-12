#!/usr/bin/env bash
# helpers.bash — bats helper library for testing git hooks.
#
# Source from a .bats file:
#   load helpers
#
# Provides: setup_test_repo, teardown_test_repo, stub_all_tools,
#           stub_only_path, hooktest_emit_json

# --- Test repo lifecycle ---

# Creates a temporary git repo with an initial commit.
# Sets TEST_REPO to the path; cd's into it.
setup_test_repo() {
    TEST_REPO="$(mktemp -d)"
    export TEST_REPO
    git -C "$TEST_REPO" init -q
    git -C "$TEST_REPO" config user.name "test"
    git -C "$TEST_REPO" config user.email "test@test"
    touch "$TEST_REPO/.gitkeep"
    git -C "$TEST_REPO" add .
    git -C "$TEST_REPO" commit -q -m "initial"
    cd "$TEST_REPO" || return 1
}

# Removes the temporary git repo created by setup_test_repo.
teardown_test_repo() {
    if [ -n "${TEST_REPO:-}" ] && [ -d "$TEST_REPO" ]; then
        rm -rf "$TEST_REPO"
    fi
}

# --- Tool stubbing ---

# Creates stub scripts for common Rust toolchain commands in a temp dir
# and prepends that dir to PATH. The stubs exit 0 silently.
# Sets STUB_BIN to the stub directory path.
stub_all_tools() {
    STUB_BIN="$(mktemp -d)"
    export STUB_BIN

    for tool in cargo rustfmt clippy-driver cargo-nextest cargo-fmt shfmt; do
        printf '#!/usr/bin/env sh\nexit 0\n' > "$STUB_BIN/$tool"
        chmod +x "$STUB_BIN/$tool"
    done

    export PATH="$STUB_BIN:$PATH"
}

# Replaces PATH with only the stub dir + /usr/bin + /bin (minimal system).
# Call stub_all_tools first to populate STUB_BIN.
stub_only_path() {
    if [ -z "${STUB_BIN:-}" ]; then
        echo "stub_only_path: call stub_all_tools first" >&2
        return 1
    fi
    export PATH="$STUB_BIN:/usr/bin:/bin"
}

# --- JSONL result logging ---

# Appends a JSONL record to $HOOKTEST_LOG.
#
# Usage: hooktest_emit_json <test_name> <status> <duration_ms>
#
# If HOOKTEST_LOG is unset, defaults to $TEST_REPO/hooktest.jsonl
# (or /tmp/hooktest.jsonl if TEST_REPO is also unset).
hooktest_emit_json() {
    local name="${1:?test name required}"
    local status="${2:?status required}"    # pass | fail | skip
    local duration="${3:-0}"
    local log="${HOOKTEST_LOG:-${TEST_REPO:-/tmp}/hooktest.jsonl}"
    local ts
    ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    printf '{"test":"%s","status":"%s","duration_ms":%s,"ts":"%s"}\n' \
        "$name" "$status" "$duration" "$ts" >> "$log"
}
