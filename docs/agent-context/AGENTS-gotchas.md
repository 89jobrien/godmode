# Gotchas

Read this before debugging a surprising failure in plan, dispatch, task, or hook code.

- `ls <dir> | wc -l` in the Nushell tool wrapper counts **table rows, not entries**. Use
  `rg --files <dir> | wc -l` or a Python one-liner when counting files.
- Bash heredocs (`cat << 'EOF'`) do not survive the Nushell tool wrapper — write to a file with
  the Write tool, then `cat <file> >> target`. Same for `cmd 2>&1`, `cmd && cmd`, and `||`.
- CLI Quick Reference (`## CLI subcommands`) can silently drift from `crates/godmode-cli/src/main.rs`
  — when adding/changing a `Cmd` variant, grep `enum.*Action` in `main.rs` and diff against the
  reference block. `skill`/`release`/`pipeline`/`policy` families were undocumented for a while.
- `skills/introspection/helpers/audit.nu` checks skill-index completeness and cross-references
  subcommand calls; run it after editing any `skills/*/SKILL.md` or `agents/*.md`. Report lands in
  `.ctx/godmode/reports/introspection/` (gitignored).
- `godmode plan ingest` safely coexists with earlier plans. Colliding IDs are namespaced by the plan
  filename, internal dependencies are rewritten, and JSON output reports actual added/skipped IDs.
- `godmode task add <title> --id <id> --depends-on ""` registers an empty string as a dep,
  causing "unmet dependencies" on start. Omit `--depends-on` entirely for root tasks.
- `dispatch --critical-path` shows the critical path tasks; `godmode status` also surfaces it.
- Pre-commit hook runs `taskit check fmt --check` and fails the commit on drift — it checks but
  does not fix. A PostToolUse hook runs fmt on edited `.rs` files but does NOT auto-stage; run
  `taskit check fmt` then `git add` the reformatted files, or the pre-commit check still fails.
- `plan::parse` returns `Result<Vec<Task>>`, not `Vec<Task>` — always match/unwrap the Result.
- `dispatch::independent_chains(graph, max)` returns `Vec<Chain>` — not `build_slots`.
- `tests/conformance/` is a workspace member (`-p godmode-conformance`); add new test modules
  in `src/`, register in `lib.rs::all_tests()`, and add `pub mod` to `lib.rs`.
- `Task::started_at` is set by `Session::start_task`, not `graph::start` — duration tracking
  only works when transitions go through `Session`, not raw `graph::*` functions directly.
- `rx::validate_run` fires inside `Session::start_task` before state mutation — if the script
  doesn't exist and `rx` is on PATH, the task is rejected before being marked Running.
