# Agent Prompt Template

Copy and fill in for each dispatched agent.

---

You are implementing tasks for crate: **`<CRATE>`**

## Tasks

<TASK_LIST — exact descriptions, one per bullet>

- [ ] `<id>`: <title>
- [ ] `<id>`: <title>

## Workflow (follow exactly)

1. Run `git branch --show-current`. If output is `main`, STOP and report — do not proceed.
2. Read `crates/<CRATE>/src/` files relevant to the tasks.
3. For each task:
   a. Write a FAILING test. Run: `cargo nextest run -p <CRATE> -- <test>`. Confirm FAIL.
   b. Implement minimum code to pass. Run: `cargo nextest run -p <CRATE>`. All green.
   c. Run: `cargo clippy -p <CRATE> -- -D warnings`. Fix all warnings.
   d. Run `git branch --show-current` — must NOT be `main`.
   e. Commit: `git commit -m "feat(<CRATE>): <summary>"`
4. If stuck after 3 attempts: write `BLOCKED.md` at repo root with crate, task, three
   attempts tried, exact error. STOP. Continue with remaining independent tasks if any.
5. Final check:
   ```bash
   cargo nextest run -p <CRATE>
   cargo clippy -p <CRATE> -- -D warnings
   git log --oneline -3
   ```

## Constraints

- Do NOT modify files outside `crates/<CRATE>/`
- Do NOT commit to `main`
- Do NOT use `--no-verify`

## Wave State Update

On completion, update `.ctx/wave-status.json` for your entry:

```bash
# Read current state, then write your entry:
# status: "done" or "blocked"
# branch: $(git branch --show-current)
# commits: [$(git log --oneline -3 | awk '{print $1}')]
```

## Report Back

- Tasks completed (with commit SHAs)
- Tasks blocked (BLOCKED.md path and reason)
- Any unexpected scope encountered

## Allowed Tools

Read, Write, Edit, Bash, Grep, Glob
