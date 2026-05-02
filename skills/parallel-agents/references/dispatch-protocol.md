# Parallel Dispatch Protocol

## Independence Test

Two task chains are independent when:

1. No task in chain A appears in the `depends_on` of any task in chain B
2. The crates they target do not import each other (within the same session)
3. They don't write to the same files

Use `godmode dispatch --json` to identify independent chains automatically.

## Agent Prompt Requirements

Each agent prompt must be:

| Requirement    | Detail                                              |
| -------------- | --------------------------------------------------- |
| Self-contained | All context inline — no "see the other agent"       |
| Scoped         | One crate or one problem domain only                |
| Explicit       | Exact test names, file paths, task descriptions     |
| Constrained    | "Do NOT modify files outside `crates/<crate>/`"     |
| Tool-listed    | `allowedTools: Read, Write, Edit, Bash, Grep, Glob` |

## Worktree vs. Single Repo

For agents working in the same repo:

- Each agent on its own branch (not `main`)
- Verify with `git branch --show-current` before every commit
- If output is `main` — STOP, do not commit

For heavy parallel work, use git worktrees:

```bash
git worktree add ../repo-agent-1 -b agent/t1
git worktree add ../repo-agent-2 -b agent/t3
```

## Rate Limit

Cap at **5 concurrent agents**. Queue additional chains and dispatch as slots free.

## Integration After Parallel Work

```bash
# 1. Verify all agents committed
git log --oneline -5   # for each agent branch

# 2. Merge sequentially with --no-ff (never octopus merge)
git checkout main
git merge --no-ff agent/t1 -m "merge: <description>"
git merge --no-ff agent/t2 -m "merge: <description>"

# 3. Full workspace gate
cargo nextest run --workspace
cargo clippy --workspace -- -D warnings
```

## BLOCKED.md Protocol

If an agent writes `BLOCKED.md`:

1. Read it before doing anything else
2. The agent stopped after 3 failed attempts — the architecture likely needs redesign
3. Do not spawn another agent with identical parameters
4. Surface to user with the BLOCKED.md content
