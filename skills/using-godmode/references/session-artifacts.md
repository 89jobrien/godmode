# Session artifacts

Read this before writing or reading the handoff snapshot or task files, or before running a reflection.

Neither command writes an external handoff record. `handoff` writes a tracked `HANDOFF.md` at the
repository root — the durable, human-readable snapshot. Everything else lives in `.ctx/`:

| Path                               | Contents                             |
| ---------------------------------- | ------------------------------------ |
| `HANDOFF.md`                       | Tracked session snapshot (repo root) |
| `.ctx/godmode/tasks.yaml`          | Task graph — the source of truth     |
| `.ctx/godmode/sessions/`           | Per-session event and summary traces |
| `.ctx/HANDOFF.<project>.<id>.yaml` | Machine-readable handoff record      |

Or with reflection:

```bash
godmode handoff && godmode:self-reflect   # status update + structured retrospective
```
