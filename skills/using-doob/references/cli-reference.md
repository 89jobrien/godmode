# doob CLI reference

Full subcommand map, current as of `doob --help`. Global flags on every
subcommand: `--json` (machine-readable output — prefer this when parsing
results programmatically instead of scraping table output), `--db <DB>`
(override the default `~/.ctx/doob/db/` path).

## todo — everyday task management

```bash
doob todo add "description"          # add one todo
doob todo list [--status S] [-p PROJECT] [-l LIMIT]
doob todo complete <id>...
doob todo remove <id>...
doob todo undo <id>                  # revert a completion
doob todo due <id> <date|clear>
doob todo deps <id>                  # show dependency chain
doob todo update <id> [fields...]    # edit fields in place
doob todo gh-sync                    # sync todos to GitHub issues
```

Use `doob todo list --status open -p <project> --json` when you need a
scriptable snapshot of a project's open work — this is the same data
`doobdash`'s kanban view reads from.

## handoff — HANDOFF.yaml <-> DB sync

```bash
doob handoff sync --file <path/to/HANDOFF.x.x.yaml>
doob handoff list [--status S] [-p PROJECT]
doob handoff add-extra <id> <text>       # append a note/extra to an item
doob handoff update-status <id> <status>
```

`sync` is bidirectional but DB wins on conflict — see the "Sync conflict
order" note in SKILL.md. `list --json` is the fastest way to check DB state
for a project without going through `doobdash`.

## note — freeform notes

```bash
doob note add "text"
doob note list
```

Not covered in depth here — used less often than todo/handoff in this
workspace's workflows.

## kan / watch — visual boards

```bash
doob kan             # static kanban snapshot in terminal
doob watch            # live-updating kanban board
```

Prefer `doobdash` (separate `doobdash` binary, richer keybindings) over
`doob kan`/`watch` for interactive use — these are lighter-weight
alternatives when doobdash isn't installed.

## search / stats / archive

```bash
doob search "query"                          # full-text across todos + notes
doob stats [-p PROJECT] [--window DAYS]      # activity analytics, default 7-day window
doob archive                                 # archive completed/cancelled todos
```

## schema

```bash
doob schema
```

Prints a machine-readable JSON manifest of every command and its
parameters — use this instead of guessing at flags if a command's behavior
seems to have drifted from this reference, since the CLI can add
subcommands between doob releases.
