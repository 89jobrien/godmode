---
name: "godmode:verification-before-completion"
description: >
  Use before claiming any work is complete, fixed, or passing. Triggers before committing,
  before creating a PR, before reporting "done", or before saying tests pass.
---

# Verification Before Completion

**Rule**: Evidence before claims, always. No completion claim without fresh verification
output.

"NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE."

## The Five-Step Gate

Before saying any task is done:

1. **Identify** the verification command
2. **Execute** it fully — not a partial check, not a cached result
3. **Read** the complete output and exit code
4. **Verify** the output matches your claim
5. **Then and only then** make the assertion

## Rust Verification Sequence

> Run via `godmode verify [--crate-name <crate>]` or use the commands below:

```bash
cargo nextest run -p <crate>              # all tests green
cargo clippy -p <crate> -- -D warnings   # zero warnings
cargo fmt --check                         # no formatting diff
git log --oneline -3                      # commits present
```

For workspace-level completion:

```bash
cargo nextest run --workspace
cargo clippy --workspace -- -D warnings
```

## Red Flags — You Are About to Lie

- "Should work" → run it and check
- "Probably passes" → run it and check
- "Tests were passing earlier" → run them now
- "I fixed the issue" → verify the fix is actually in and tests pass
- "Looks good" → evidence or it didn't happen

## Subagent Reports

Never trust a subagent's self-report without independent verification. Run the verification
commands yourself after a subagent claims completion.

Check that commits actually exist:

```bash
git log --oneline -5
```

An empty commit list means the subagent did not finish.

## Additional Resources

- **`references/verification-commands.md`** — per-crate and workspace gates, false "done" states, exit code gotchas
- **`helpers/pre-commit-gate.sh`** — run before any completion claim: `sh helpers/pre-commit-gate.sh [crate]`
