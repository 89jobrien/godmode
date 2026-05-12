#!/usr/bin/env bash
# hooklib.sh — sourceable shell library for pre-commit/pre-push hooks.
#
# Source this from your hook scripts to get tool-optional format/lint
# checking with graceful degradation and failure collection.
#
# Usage:
#   source "$(dirname "$0")/hooklib.sh"
#   check_format "Rust"   "rustfmt" "cargo fmt --all" cargo fmt --all --check
#   check_format "Shell"  "shfmt"   "shfmt -w ."      shfmt -d .
#   check_lint   "Clippy" "cargo"   "cargo clippy --fix" cargo clippy -- -D warnings
#   hooklib_exit

HOOKLIB_FAILURES=()

# check_format <label> <tool> <fix_hint> <cmd...>
#
# Run a format check. If <tool> is not on PATH, skip with a warning.
# If the command fails, record the failure but continue.
check_format() {
    local label="$1" tool="$2" fix_hint="$3"
    shift 3

    if ! command -v "$tool" &>/dev/null; then
        echo "[hook] SKIP $label — $tool not found"
        return 0
    fi

    echo "[hook] Checking $label formatting..."
    if ! "$@" 2>&1; then
        HOOKLIB_FAILURES+=("$label format: run '$fix_hint' to fix")
    fi
}

# check_lint <label> <tool> <fix_hint> <cmd...>
#
# Run a lint check. Same semantics as check_format.
check_lint() {
    local label="$1" tool="$2" fix_hint="$3"
    shift 3

    if ! command -v "$tool" &>/dev/null; then
        echo "[hook] SKIP $label — $tool not found"
        return 0
    fi

    echo "[hook] Running $label lint..."
    if ! "$@" 2>&1; then
        HOOKLIB_FAILURES+=("$label lint: run '$fix_hint' to fix")
    fi
}

# check_coverage <pkg> [threshold]
#
# Run cargo llvm-cov for a package and enforce a coverage floor.
# Requires cargo-llvm-cov. Degrades gracefully if absent.
check_coverage() {
    local pkg="$1" threshold="${2:-80}"

    if ! command -v cargo-llvm-cov &>/dev/null; then
        echo "[hook] SKIP coverage — cargo-llvm-cov not found"
        return 0
    fi

    echo "[hook] Checking $pkg coverage (>= ${threshold}%)..."
    local output
    output=$(cargo llvm-cov -p "$pkg" --lib 2>&1)
    local pct
    pct=$(echo "$output" | awk '/TOTAL/ { gsub(/%/, "", $NF); print $NF }')

    if [ -n "$pct" ] && [ "$(echo "$pct < $threshold" | bc -l)" -eq 1 ]; then
        HOOKLIB_FAILURES+=("$pkg coverage ${pct}% < ${threshold}%")
    fi
}

# hooklib_exit
#
# Call at the end of your hook. Prints all failures and exits non-zero
# if any were recorded.
hooklib_exit() {
    if [ ${#HOOKLIB_FAILURES[@]} -eq 0 ]; then
        echo "[hook] All checks passed."
        exit 0
    fi

    echo ""
    echo "[hook] FAILURES:"
    for f in "${HOOKLIB_FAILURES[@]}"; do
        echo "  - $f"
    done
    echo ""
    exit 1
}
