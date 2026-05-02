---
name: "godmode:cap"
description: >
  Commit and push workflow with pre-flight validation and hook recovery. Use when the user
  says "cap", "commit and push", or "ship it". Runs cargo test, fmt, clippy, stages all
  changes, writes a conventional commit, handles pre-commit hook failures, and pushes.
---

# Cap — Commit and Push

Run the full validation gate, commit, and push in one pass.

## Workflow

### Step 1: Validate

> Run via `nu skills/_lib/quality-gate.nu run-quality-gate` or use the commands below:

```bash
cargo check --workspace
cargo nextest run --workspace   # preferred; fallback: cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
```

If `cargo check` or tests fail — **stop**. Report failures. Do not commit broken code.

If `cargo fmt --check` fails — run `cargo fmt --all` to fix, then continue.

If clippy warnings exist — fix them before committing.

### Step 2: Stage

```bash
git add -A
git diff --cached --stat   # show what's staged
```

Review staged files. If unrelated changes are staged, report them and ask before proceeding.

### Step 3: Write commit message

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <summary>

[optional body]
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `ci`

Scope: crate name or module (e.g. `godmode-core`, `graph`, `cli`)

Derive the message from the staged diff — do not ask the user to write it.

### Step 4: Verify branch and commit

> (`guardrails.nu check-branch <expected>`)

```bash
git branch --show-current
```

If the output is `main` and the user did not explicitly ask to commit to main — stop and report.
Do not proceed.

```bash
git commit -m "<message>"
```

#### Hook failure recovery

If the pre-commit hook fails:

1. Read the hook output carefully — identify the exact rule and match.
2. **False positive** (test fixture, doc URL, variable name): add a minimum exclusion to the
   allowlist config (`.gitleaksignore`, obfsck allowlist, etc.) and retry once.
3. **Real issue**: fix the flagged content, re-stage, retry once.
4. If still failing after one retry — stop and report. Do not use `--no-verify`.

### Step 5: Push

```bash
git push
```

If push is rejected (non-fast-forward): report to user. Do not force-push without explicit
instruction.

### Step 6: Sync task state (if applicable)

If a godmode task is currently `running`, mark it done with the commit SHA:

```bash
godmode task done <task-id> --commit <sha>
```

If no running task exists, skip this step.

## Guardrails

- Never use `--no-verify`.
- Never force-push without explicit user instruction.
- Never commit to `main` directly — verify with `git branch --show-current` first.
- If branch is `main` and no branch was specified — ask before pushing.
