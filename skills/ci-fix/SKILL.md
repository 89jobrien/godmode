---
name: ci-fix
description: >
  Self-healing CI diagnosis and fix loop. Use when CI is failing, when asked to "fix CI",
  or after a push that broke the pipeline. Fetches the latest failed run, classifies the
  root cause, applies a targeted fix, verifies locally, and re-pushes.
---

# CI Fix — Diagnose and Repair

Fetch the failure, classify it, fix it, verify locally, push.

## Workflow

### Step 1: Fetch failed runs

```bash
gh run list --limit 5 --status failure
```

Pick the most recent failure for the current branch.

### Step 2: Download logs

```bash
gh run view <run-id> --log-failed
```

Read the full failure output. Identify the first error — that's usually the root cause.

### Step 3: Classify

| Class                      | Symptoms                                          | Fix                                   |
| -------------------------- | ------------------------------------------------- | ------------------------------------- |
| `compile_error`            | `error[E...]`, missing match arm, wrong import    | Fix source, `cargo check`             |
| `test_failure`             | `FAILED`, assertion mismatch, panic               | Fix impl or test, `cargo nextest run` |
| `clippy_warning`           | `error: ...`, `-D warnings` gate                  | Fix all warnings, `cargo clippy`      |
| `fmt_check`                | `Diff in ...`                                     | Run `cargo fmt --all`                 |
| `pre_commit_hook`          | gitleaks, obfsck, coursers block                  | Add allowlist entry                   |
| `runner_environment`       | missing tool, wrong Xcode, wrong target           | Update workflow YAML                  |
| `false_positive_detection` | secret scanner flags test/doc content             | Add `.gitleaksignore` entry           |
| `dependency_issue`         | lockfile conflict, yanked crate, version mismatch | Update `Cargo.toml`/`Cargo.lock`      |

If the class is ambiguous — report it to the user before fixing.

### Step 4: Fix

Apply the minimum targeted fix for the classified root cause. Do not refactor unrelated code.

Verify locally before pushing:

```bash
# compile_error / test_failure / clippy_warning / fmt_check
cargo check --workspace
cargo nextest run --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all --check

# runner_environment — read the workflow file first (use Read tool, not cat)
# Read: .github/workflows/<file>.yml
# then edit only the failing step
```

### Step 5: Commit and push

```bash
git add -A
git commit -m "fix(ci): <short description of root cause>"
git push
```

Then verify the new run passes:

```bash
gh run list --limit 3   # polling fallback — gh run watch requires interactive TTY
```

## Guardrails

- Fix one root cause per pass. If there are multiple independent failures, fix them
  sequentially — most CI failures cascade from a single root.
- Do NOT switch self-hosted runners to GitHub-hosted without asking.
- Do NOT change model names, API keys, or secrets in workflow files.
- Do NOT use `--no-verify`.
- If the same failure class recurs across 3+ sessions, add a note to `CLAUDE.md` to
  prevent it at the source.
