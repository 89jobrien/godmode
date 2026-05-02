#!/usr/bin/env bash
# wave-integrate — sequential rebase + test + merge loop for parallel agent branches
# Usage: wave-integrate [--branches "feat/a feat/b feat/c"] [--base main] [--dry-run]
#
# Reads branches from --branches (space-separated) or from stdin (one per line).
# Rebases each onto --base, runs cargo test --workspace, then merges to base.
# Writes conflict-resolution-log.md to repo root on completion.

set -euo pipefail

# ── color helpers ─────────────────────────────────────────────────────────────
GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
RESET='\033[0m'
BOLD='\033[1m'
ok() { echo -e "  ${GREEN}✓${RESET}  $*"; }
fail() { echo -e "  ${RED}✗${RESET}  $*"; }
step() { echo -e "\n  ${CYAN}${BOLD}▸${RESET}  ${BOLD}$*${RESET}"; }
warn() { echo -e "  ${YELLOW}!${RESET}  $*"; }

# ── argument parsing ──────────────────────────────────────────────────────────
BRANCHES=""
BASE="main"
DRY_RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
    --branches)
        BRANCHES="$2"
        shift 2
        ;;
    --base)
        BASE="$2"
        shift 2
        ;;
    --dry-run)
        DRY_RUN=1
        shift
        ;;
    *)
        echo "Unknown argument: $1"
        exit 1
        ;;
    esac
done

# ── resolve branch list ───────────────────────────────────────────────────────
declare -a BRANCH_LIST=()
if [[ -z "$BRANCHES" ]]; then
    # Read from stdin if no --branches flag
    if [[ -t 0 ]]; then
        echo "No branches provided. Use --branches 'feat/a feat/b' or pipe branch names."
        exit 1
    fi
    while IFS= read -r line; do
        line="${line//[$'\t\r\n ']/}"
        [[ -n "$line" ]] && BRANCH_LIST+=("$line")
    done
else
    read -ra BRANCH_LIST <<<"$BRANCHES"
fi

if [[ ${#BRANCH_LIST[@]} -eq 0 ]]; then
    echo "No branches provided. Use --branches 'feat/a feat/b' or pipe branch names."
    exit 1
fi

# ── repo root ─────────────────────────────────────────────────────────────────
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

echo -e "\nWave integration: ${#BRANCH_LIST[@]} branches onto ${BASE}"
echo "Branches: $(
    IFS=', '
    echo "${BRANCH_LIST[*]}"
)"

# ── stash any dirty worktree ─────────────────────────────────────────────────
STASHED=0
if [[ -n "$(git status --porcelain)" ]]; then
    step "Stashing uncommitted changes"
    git stash push -m "wave-integrate: auto-stash"
    STASHED=1
    ok "Stashed"
fi

# ── update base ───────────────────────────────────────────────────────────────
step "Updating ${BASE}"
if ! git checkout "$BASE"; then
    fail "Failed to checkout ${BASE}"
    exit 1
fi
if ! git pull; then
    fail "Failed to pull ${BASE}"
    exit 1
fi
ok "${BASE} up to date"

# ── tracking arrays ───────────────────────────────────────────────────────────
declare -a INTEGRATED=()
declare -a FAILED=()
declare -A INTEGRATED_SHAS=()

# ── per-branch loop ───────────────────────────────────────────────────────────
for BRANCH in "${BRANCH_LIST[@]}"; do
    echo -e "\n━━━ ${BRANCH} ━━━"

    # Checkout
    if ! git checkout "$BRANCH" 2>/dev/null; then
        fail "Cannot checkout ${BRANCH} — skipping"
        FAILED+=("$BRANCH")
        continue
    fi

    # Fetch latest
    git fetch origin "$BRANCH" 2>/dev/null || true

    # Rebase onto base
    step "Rebasing ${BRANCH} onto ${BASE}"
    if ! git rebase "$BASE"; then
        warn "Rebase conflicts detected — inspect and resolve manually"

        # List conflicted files
        CONFLICTS=$(git diff --name-only --diff-filter=U 2>/dev/null || true)
        if [[ -z "$CONFLICTS" ]]; then
            fail "Rebase failed with no detectable conflict files — aborting rebase"
            git rebase --abort 2>/dev/null || true
            FAILED+=("$BRANCH")
            git checkout "$BASE" 2>/dev/null || true
            continue
        fi

        echo -e "\nConflicted files:"
        while IFS= read -r f; do echo "  - $f"; done <<<"$CONFLICTS"
        echo -e "\nResolve conflicts, then run: git add <files> && git rebase --continue"
        echo "Then re-run wave-integrate with remaining branches."
        git rebase --abort 2>/dev/null || true
        FAILED+=("$BRANCH")
        git checkout "$BASE" 2>/dev/null || true
        continue
    fi
    ok "Rebase clean"

    # Run tests
    step "Running cargo nextest run --workspace"
    if ! cargo nextest run --workspace; then
        fail "Tests failed after rebase"
        warn "Fix tests on this branch before continuing"
        FAILED+=("$BRANCH")
        git checkout "$BASE" 2>/dev/null || true
        continue
    fi
    ok "Tests pass"

    # Capture SHA
    SHA="$(git rev-parse --short HEAD)"

    if [[ $DRY_RUN -eq 0 ]]; then
        # Merge to base
        step "Merging ${BRANCH} into ${BASE}"
        git checkout "$BASE"
        if ! git merge --no-ff "$BRANCH" -m "integrate: merge ${BRANCH}"; then
            fail "Merge failed for ${BRANCH}"
            FAILED+=("$BRANCH")
            continue
        fi
        ok "Merged ${BRANCH} -> ${BASE} at ${SHA}"
    else
        warn "[dry-run] Would merge ${BRANCH} at ${SHA} into ${BASE}"
        git checkout "$BASE" 2>/dev/null || true
    fi

    INTEGRATED+=("$BRANCH")
    INTEGRATED_SHAS["$BRANCH"]="$SHA"
done

# ── final test run on base ────────────────────────────────────────────────────
if [[ $DRY_RUN -eq 0 && ${#INTEGRATED[@]} -gt 0 ]]; then
    step "Final test run on ${BASE}"
    if ! cargo nextest run --workspace; then
        fail "Final tests failed on integration branch — do not proceed"
        exit 1
    fi
    ok "All tests pass on integrated branch"
fi

# ── write conflict log template ───────────────────────────────────────────────
LOG_PATH="${REPO_ROOT}/conflict-resolution-log.md"
TIMESTAMP="$(date '+%Y-%m-%d %H:%M')"

BRANCH_SUMMARY=""
for b in "${INTEGRATED[@]}"; do
    BRANCH_SUMMARY+="- ${b} (${INTEGRATED_SHAS[$b]:-unknown})"$'\n'
done

FAILED_SUMMARY="none"
if [[ ${#FAILED[@]} -gt 0 ]]; then
    FAILED_SUMMARY="$(
        IFS=', '
        echo "${FAILED[*]}"
    )"
fi

cat >"$LOG_PATH" <<EOF
# Wave Integration Log — ${TIMESTAMP}

## Branches Integrated

${BRANCH_SUMMARY}
## Failed / Skipped

${FAILED_SUMMARY}

## Conflict Resolution Log

| File | Branch | Main-side intent | Branch-side intent | Resolution |
|------|--------|-----------------|-------------------|------------|
| _fill in_ | | | | |

## Notes

_Add any manual resolution notes here._
EOF

ok "Conflict log template written to ${LOG_PATH}"

# ── restore stash ─────────────────────────────────────────────────────────────
if [[ $STASHED -eq 1 ]]; then
    step "Restoring stashed changes"
    git checkout "$BASE" 2>/dev/null || true
    git stash pop || warn "Could not restore stash — run 'git stash pop' manually"
    ok "Stash restored"
fi

# ── summary ───────────────────────────────────────────────────────────────────
echo -e "\n━━━ Summary ━━━"
echo "  Integrated : ${#INTEGRATED[@]} branches"
echo "  Failed     : ${#FAILED[@]} branches"
if [[ ${#FAILED[@]} -gt 0 ]]; then
    echo "  Failed list: $(
        IFS=', '
        echo "${FAILED[*]}"
    )"
fi
