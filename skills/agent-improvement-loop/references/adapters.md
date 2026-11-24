# Adapters

Read this before running Phase 1 or Phase 3 against a trace source other than the default, or before passing `--profile` to coursers/crs commands.

## Coursers / crs adapter (default)

| Loop phase       | Command                                            |
| ---------------- | -------------------------------------------------- |
| Phase 1 traces   | `crs discover --since <N> --format json`           |
| Phase 1 insights | `crs insights --format json`                       |
| Phase 1 stats    | `crs stats`                                        |
| Phase 3 evals    | `echo '<payload>' \| coursers pre [--profile <p>]` |
| Phase 7 validate | `crs validate [--profile <p>]`                     |
| Phase 7 test     | `taskit test run`                                  |

Profile isolation: pass `--profile <name>` to all coursers/crs commands to keep
test-profile state separate from production.

## Generic adapter

Replace Phase 1 with any trace source that emits structured JSON records with fields:

- `command` or `input` — what the agent did
- `exit_code` or `outcome` — success/failure
- `timestamp` — when it happened
- `session_id` (optional) — for grouping

Replace Phase 3 evals with any eval harness (promptfoo, pytest, custom shell scripts) that
can assert on agent output given a fixed input.
