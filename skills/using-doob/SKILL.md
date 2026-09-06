---
name: using-doob
description: Reference for working with doob, the workspace's todo/task tracker with GitHub sync and handoff workflows. Covers the doob CLI and doobdash TUI for managing todos/handoffs in ANY project, plus doob's own architecture (SurrealKV storage, HANDOFF.yaml sync) for when developing doob itself. Use whenever doob, HANDOFF.yaml/HANDOFF.*.yaml files, doobdash, or "sync handoff" come up in any repo — not just inside ~/dev/doob. Also use when the user asks to check todos, update task status, or reconcile a HANDOFF file against the doob database.
---

# Using doob

doob is a cross-project todo/task tracker. It stores todos in a local
SurrealKV database (`~/.ctx/doob/db/`) and syncs them against per-project
`HANDOFF.*.yaml` files and GitHub issues. Any repo in `~/dev` may have its own
`HANDOFF.<project>.<project>.yaml` (see naming convention below) — this skill
applies wherever you find one, not just inside `~/dev/doob`.

There are two ways this skill gets used:

1. **As a consumer** — managing todos/handoffs in some other project via the
   `doob` CLI or `doobdash` TUI.
2. **As a developer** — working on doob's own codebase (`~/dev/doob`), where
   its SurrealDB quirks and CI gate matter.

Read the section that matches what you're doing; skip the other.

For the full command surface (`todo`, `note`, `kan`, `watch`, `search`,
`stats`, `archive`, `schema`) beyond the handoff-focused commands below, see
`references/cli-reference.md`.

## Using doob in any project

### Core commands

```bash
doob handoff sync --file HANDOFF.<project>.<project>.yaml   # sync YAML -> DB
doob handoff update-status <id> <status>                    # set status in doob
```

Status values: `open` · `done` · `parked` · `blocked`.

**Sync conflict order matters**: doob's DB always wins over the YAML on
sync. If you edit both a status field in the YAML AND want to run
`update-status`, run `update-status` _before_ editing the YAML — otherwise
your manual YAML edit gets silently reverted on the next sync. When in doubt,
treat the DB as the source of truth and the YAML as a snapshot you pull into.

### HANDOFF file naming

HANDOFF files live in a project's `.ctx/` directory and are named with the
repo dirname twice: `HANDOFF.<dirname>.<dirname>.yaml` — e.g. for the `doob`
repo itself, `HANDOFF.doob.doob.yaml`. Don't use `workspace` as the base name
even when working from `~/dev`; use the actual repo directory name so the
file is unambiguous when viewed outside its directory.

After any manual edit to a HANDOFF.yaml, run `doob handoff sync` on it — the
DB is what `doobdash` and cross-project reporting read from, not the file
itself.

### doobdash (TUI)

Interactive view over the same DB plus whichever HANDOFF file is nearest the
current directory (it walks up from CWD looking for `HANDOFF.*.yaml`).

```bash
doobdash [path/to/HANDOFF.yaml]   # optional explicit path
```

Keybindings:

- `j`/`k` — move within a column, `h`/`l` — switch column
- `Enter` — open item overlay
- `Space` (leader) then: `s` status · `n` note · `w` save · `/` search ·
  `1`-`5` tabs · `?` help · `Esc` cancel
- `z` toggle side strip, `z` then `j`/`k` to resize it
- `q` quit
- Tab `5` ("DB") browses the raw SurrealKV todo table directly — useful when
  the HANDOFF file and DB have drifted and you need to see DB state without
  going through sync.

### Reconciling a HANDOFF file against the DB

When asked to "check todos" or "audit doob against HANDOFF" for a project:

1. Locate that project's `HANDOFF.*.yaml` in its `.ctx/` directory — use
   `scripts/find-handoffs.sh [root-dir]` to enumerate every HANDOFF file
   across the workspace (defaults to `~/dev`) if you don't already know the
   path.
2. Run `scripts/reconcile-handoff.sh <handoff-file> [project-name]`, which
   syncs the file (`doob handoff sync`) and then prints that project's DB
   state (`doob handoff list --json`) so drift is visible in one pass. It
   does not resolve conflicts itself — DB still wins, per above — it only
   surfaces what's there.
3. Compare statuses — any mismatch means the YAML is stale and should be
   treated as informational only, not authoritative.

`reconcile-handoff.sh` requires the `doob` binary on `PATH` and touches the real DB at
`~/.ctx/doob/db/` (via `doob handoff sync`); `find-handoffs.sh` is read-only. Do not run
`reconcile-handoff.sh` speculatively against a project you don't actually
want synced.

## Developing doob itself (working in ~/dev/doob)

Only relevant when editing doob's own source, not when merely using the CLI.

### Architecture

- Binary: `doob` / Library: `doob` — lib+bin structure at
  `crates/doob/src/main.rs` + `crates/doob/src/lib.rs`
- Storage: SurrealKV at `~/.ctx/doob/db/` — this is a _directory_, not a
  single file
- `handoff_item` table is intentionally SCHEMALESS: typed tables rejected
  nullable datetime fields via JSON, so don't "fix" this by adding a schema
  without checking that constraint first

### SurrealDB 2.x gotchas

These are load-bearing — SurrealDB's behavior here is surprising enough that
skipping them wastes a debugging cycle:

- **Parameterized queries silently no-op** (SurrealDB issue #6271) — use raw
  interpolated SQL strings instead of bound parameters for queries against
  this DB version.
- **Datetime fields**: ISO strings are rejected inside `CONTENT` blocks.
  Inject as `d"2026-01-01T00:00:00Z"` literals instead.
- **`UPDATE ... MERGE ... WHERE`** returns empty rows even when the update
  succeeded — `SELECT` first to check existence rather than trusting the
  `UPDATE` return value as a success signal.
- **`.take::<Vec<surrealdb::Value>>(0)`** fails on rows containing a `Thing`
  — deserialize into `serde_json::Value` instead, or ignore the typed
  extraction path for those rows.

### Development loop

```bash
cargo install --path crates/doob        # reinstall binary (release by default)
cargo nextest run --all-features
cargo clippy
```

Wipe the DB with `rm -rf ~/.ctx/doob/db` if it gets into a bad state — safe,
since it re-syncs from HANDOFF.yaml files to repopulate.

### CI gate — run before every push

```bash
./ci.sh
```

Do not rely on `ci.sh`'s legacy test runner. Run the preferred gate directly:

```bash
cargo fmt --all --check
cargo clippy -- -D warnings
cargo nextest run --all-features
cargo audit
```

### doobdash crate

`crates/doobdash/` is a separate workspace member (own binary, not a module
of `doob`). Install with `cargo install --path crates/doobdash`. If you're
changing keybindings or the DB tab described above, that's the crate to
touch — the core `doob` crate only owns the CLI and sync logic.

### Hooks note

`HANDOFF*.yaml` files are excluded from the `obfsck` pre-commit secret scan
in this repo — their `doob_uuid` fields match a container-ID-like pattern
that would otherwise false-positive as a secret.
