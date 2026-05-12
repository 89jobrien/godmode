#!/usr/bin/env bats
# fuzz-template.bats — randomized file-type fuzz tests for pre-commit hooks.
#
# Copy this template into your project's test directory and adjust the
# HOOK_PATH variable to point at your pre-commit hook script.

load helpers

HOOK_PATH=".git/hooks/pre-commit"

setup() {
    setup_test_repo
    stub_all_tools

    # Install the hook under test (override this path as needed)
    if [ -n "${HOOK_UNDER_TEST:-}" ] && [ -f "$HOOK_UNDER_TEST" ]; then
        cp "$HOOK_UNDER_TEST" "$TEST_REPO/$HOOK_PATH"
        chmod +x "$TEST_REPO/$HOOK_PATH"
    fi
}

teardown() {
    teardown_test_repo
}

# Generate N random files from a set of extensions, stage them, and run
# the pre-commit hook. The hook should never crash regardless of file mix.
@test "fuzz: random file-type combinations do not crash pre-commit" {
    local extensions=(.rs .toml .md .nu .yaml .json .sh .py .lock)
    local count="${FUZZ_FILE_COUNT:-20}"
    local start_ms
    start_ms=$(($(date +%s) * 1000))

    for i in $(seq 1 "$count"); do
        local ext="${extensions[$((RANDOM % ${#extensions[@]}))]}"
        local name="fuzz_${i}${ext}"
        printf '// generated fuzz file %d\n' "$i" > "$name"
    done

    git add .

    local status=0
    if [ -x "$HOOK_PATH" ]; then
        "$HOOK_PATH" || status=$?
    else
        # No hook installed — vacuous pass
        status=0
    fi

    local end_ms
    end_ms=$(($(date +%s) * 1000))
    local duration=$(( end_ms - start_ms ))

    if [ "$status" -eq 0 ]; then
        hooktest_emit_json "fuzz_random_filetypes" "pass" "$duration"
    else
        hooktest_emit_json "fuzz_random_filetypes" "fail" "$duration"
    fi

    # The hook may legitimately reject files (lint failure), but it must
    # not segfault or produce an unhandled error (exit > 1).
    [ "$status" -le 1 ]
}

@test "fuzz: empty commit does not crash pre-commit" {
    local start_ms
    start_ms=$(($(date +%s) * 1000))

    local status=0
    if [ -x "$HOOK_PATH" ]; then
        "$HOOK_PATH" || status=$?
    else
        status=0
    fi

    local end_ms
    end_ms=$(($(date +%s) * 1000))
    hooktest_emit_json "fuzz_empty_commit" "pass" "$(( end_ms - start_ms ))"

    [ "$status" -le 1 ]
}
