---
name: "godmode:crs-hook-testing"
description: >
  Coursers rule lifecycle pipeline. Walk a course-correct rule (or candidate rule) through
  the full lifecycle: author → validate → probe → fire (pre/post) → observe → refine/retire.
  Use when adding a new crs/coursers rule, auditing an existing one, or diagnosing a rule
  that is misfiring or dead.
requires: []
next: []
argument-hint: "[<rule-name-or-pattern>]"
---

# CRS Hook Testing — Coursers Rule Lifecycle

A structured pipeline for managing the lifecycle of a `course-correct` rule in
`~/.config/coursers/course-correct-rules.json`.

Each stage maps to one or more `crs` / `coursers` subcommands. Walk through stages in
order for a new rule. Jump to the relevant stage when debugging or refining an existing
rule.

## Reference files

Read the file named in a row before doing the task in its right column. These files are not
loaded automatically.

| File                                       | Read before                                                                                                        |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------ |
| `references/stages-0-1-discover-author.md` | discovering candidate commands for a new rule                                                                      |
| `references/stage-1-author.md`             | adding or editing a rule in `course-correct-rules.json`                                                            |
| `references/stages-2-3-validate-probe.md`  | running `crs validate`, or testing a rule against a command with `crs probe` or `crs pre`                          |
| `references/stages-4-5-fire-observe.md`    | firing a rule through the real pre or post hook                                                                    |
| `references/stage-5-observe.md`            | checking a rule's activity with stats, heat, history, log, or export once it has been live                         |
| `references/stage-6-refine-retire.md`      | changing a rule based on observations, removing an obsolete rule, or checking rule health with insights and replay |

---

## Quick Reference

| Stage       | Command(s)                                          |
| ----------- | --------------------------------------------------- |
| Discover    | `crs discover`, `crs suggest`                       |
| Author      | Edit `~/.config/coursers/course-correct-rules.json` |
| Validate    | `crs validate`                                      |
| Probe       | `echo '<cmd>' \| crs probe`                         |
| Fire (pre)  | `echo '<json>' \| coursers pre`                     |
| Fire (post) | `echo '<json>' \| coursers post`                    |
| Observe     | `crs stats`, `crs heat`, `crs history`, `crs log`   |
| Export      | `crs export`                                        |
| Refine      | Edit rules → validate → probe → observe             |
| Replay      | `crs replay`, `crs insights`                        |

---

## Common Pitfalls

- `probe` vs `pre` divergence: `probe` matches the whole command string; `pre` splits on
  shell operators first. Always test with `crs pre` before trusting `crs probe` alone.
- After editing `crates/core` or `crates/coursers`, the live `crs`/`coursers` binaries
  are NOT updated automatically. Run `cargo install --path crates/coursers` to deploy.
- The `no-find-use-glob` rule matches `find .ctx` in commit messages. Exceptions for
  `git (commit|log|tag|stash)` (including `git -C` form) suppress this.
- State files: global at `~/.config/coursers/course-correct-state.json`; project-local
  at `.ctx/course-correct-state.json` (wins over global).
