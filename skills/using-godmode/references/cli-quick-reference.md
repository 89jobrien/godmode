# CLI quick reference

Read this before running, adding, or changing a godmode subcommand.

```bash
godmode handon                          # session start triage
godmode handoff                         # session end closeout
godmode plan ingest <plan.md>           # ingest plan → task graph
godmode task list [--json]              # all tasks
godmode task next [--json]              # next runnable (exit 2 if none)
godmode task add <title> [--id <id>] [opts]  # add task
godmode task start <id>                 # mark running
godmode task done <id> [--commit <sha>] # mark done
godmode task block <id> "<reason>"      # mark blocked
godmode task unblock <id>               # reset to pending
godmode task unblock-all                # reset ALL blocked tasks to pending
godmode task run <id>                   # run task's run: command
godmode task remove <id>               # remove a task from the graph
godmode dispatch [--max 5] [--json]     # parallel chains for orca-strait
godmode agent dispatch <path> [--max N] # ingest + dispatch in one step
godmode status                          # graph counts + next runnable
godmode context [--json]                # session context for hooks/subagents
godmode verify [--crate-name <crate>]   # nextest + clippy + fmt gate
```
