# Session Workflow

## Full session pattern

```
# Start
godmode handon

# Work loop
godmode task next
godmode task start <id>
# ... implement ...
godmode task done <id> --commit <sha>
godmode task next   # repeat

# End
godmode handoff
```

## Ingest a plan then work

```
godmode plan ingest docs/plans/YYYY-MM-DD-feature.md
godmode handon
godmode task start t1
# implement t1 ...
godmode task done t1 --commit abc1234
godmode task next   # → t2 now runnable
godmode task start t2
# ...
godmode handoff
```

## Parallel dispatch

```
godmode dispatch --json               # review chains
godmode agent dispatch docs/plans/feature.md  # ingest + dispatch payload
# paste chains into godmode:parallel-agents
```

## Mid-session state check

```
godmode status    # fast — no external calls
```

## Clear completed tasks between sessions

```
godmode task clear --done   # prune done tasks, keep pending/running/blocked
```
