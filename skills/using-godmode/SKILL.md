---
name: "godmode:using-godmode"
description: >
  Use at the start of every conversation to orient to available skills and workspace rules.
  Replaces superpowers entirely. Triggers on session start, "/godmode", or "what skills do
  you have".
requires: []
next: []
---

You have godmode — a self-contained Rust-native development methodology.

## Instruction Priority

1. User's explicit instructions (CLAUDE.md, direct requests) — highest
2. Godmode skills — override defaults
3. Default system prompt — lowest

## Phased Workflow

Every non-trivial task progresses through five phases. Short
confirmations ("do it", "act", "go") advance to the next phase.

<godmode-phase name="ORIENT" mode="read-only" response-header="# Phase: ORIENT" skills="godmode handon">
Default phase. Read files, run `godmode handon`, summarize state. No modifications.
</godmode-phase>

<godmode-phase name="PLAN" mode="read-only" response-header="# Phase: PLAN" skills="brainstorm, writing-plans">
Produce a written plan. Still read-only. End with "Type ACT to proceed."
</godmode-phase>

<godmode-phase name="ACT" mode="read-write" response-header="# Phase: ACT" skills="task-driven-development, parallel-agents">
Edit files, run commands, dispatch subagents. Enter on user approval.
</godmode-phase>

<godmode-phase name="VERIFY" mode="read + test" response-header="# Phase: VERIFY" skills="verification-before-completion">
Run the project's canonical gate, not a generic approximation. Prefer the repo's
own task runner (`cargo xtask verify`, `make check`, `npm test`) over assembling
bare tool invocations by hand: the project gate usually covers steps the bare
commands miss (feature/arch matrices, lint configs, fixtures). Never prescribe a
generic gate the repo does not use. Return to ACT if failures. Ask to SHIP when green.
</godmode-phase>

<godmode-phase name="SHIP" mode="commit/push" response-header="# Phase: SHIP" skills="cap, session-wrap-commit-push">
Commit, push, handoff. Only on explicit approval. Return to ORIENT after.
</godmode-phase>

**Phase transitions**: Users can skip phases ("just fix it" = all phases in one pass).
After each ACT turn, default to VERIFY. Multiple ACT turns are fine.

## Skill Invocation Rule

Before responding in any phase, check if a skill applies.
1% chance it's relevant = invoke it.

```text
Message received → skill applies? → YES: invoke Skill tool first → NO: respond
```

**Priority**: Process skills first (`brainstorm`, `systematic-debugging`),
then implementation skills (`task-driven-development`, `parallel-agents`),
then quality gates (`verification-before-completion`, `code-review`).

## Red Flags (you are rationalizing — stop)

- "This is simple" → check anyway
- "I need context first" → skill check comes before exploration
- "The skill is overkill" → if it exists, use it
- "Just this one thing first" → check before acting

## Available Skills

The skill index is in three files, split by area. Read the file whose area matches the task, then choose a skill from its rows.

| File                                | Read before                                                                                                                      |
| ----------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `references/skills-index-1-of-3.md` | choosing a skill for planning, implementation, review, debugging, or docs work                                                   |
| `references/skills-index-2-of-3.md` | choosing a skill for release work, Rust quality, crux, agent improvement, coursers rules, or devloop analysis                    |
| `references/skills-index-3-of-3.md` | choosing a skill for devloop daily or session work, doob, minibox, Maestro, Pieces, secrets, environment, or the using-* entries |

## Always-Active Rules

- **Never `--no-verify`** on commits. Pre-commit hooks always run.
- **Commit signing**: SSH key via 1Password agent. Unlock 1Password and retry on failure.
- **Shell**: Nushell (`nu`) primary. No bash-isms (`&&`, `$()`, `export VAR=`) in `.nu` files.
- **Git in other repos**: `git -C <path>` not `cd <path> && git`.
- **Rust**: `taskit check compile` + `taskit check lint` after every change, `taskit test run` for tests.
  Fix all clippy warnings before committing.
- **Secrets**: Never pass raw `op://` URIs. Use `op read` or `op run`.
- **Scope**: Touch only files within the explicitly requested scope.
- **Destructive ops**: Show diff, get explicit confirmation before dropping stashes or
  deleting files.

## Session Ritual

`handon` and `handoff` are **status readouts, not state mutations**. Run the command, then report
what it printed. Do not re-derive state, do not infer state the command did not print, and do not
describe work as finished while a task is still `running`.

**Start — where things stand:**

```bash
godmode handon      # graph counts, active pipeline + position, next runnable task, uncommitted files
```

Report as a status update:

- Current graph counts (done / running / pending / blocked).
- The active pipeline and the step it is waiting on.
- The single next runnable task, by ID and title.
- Whether the tree is dirty. If it is, name the files.

**End — where things ended:**

```bash
godmode handoff     # session totals, warnings on still-running tasks, uncommitted files
```

Report as a status update:

- What completed this session.
- Anything still `running` — always surface it, never silently drop it.
- Tree state, by filename when dirty.
- The single most useful next action.

Where the handoff snapshot and task files live, and how to run a reflection, is in `references/session-artifacts.md`. Read it before writing or reading those files.

## Additional Resources

- **`helpers/session-start.sh`** — run at session start to verify CLI and triage

## CLI Quick Reference

The full subcommand list is in `references/cli-quick-reference.md`. Read it before running, adding, or changing a godmode subcommand.
