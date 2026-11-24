# Agent scratch space

Read this before writing drafts, plans, or BLOCKED.md files during a session.

`.ctx/_WORKING_DIR/` is the canonical scratch directory for all agents and helpers. Use it for:

- Intermediate proposals, plans, and drafts produced during a session
- `BLOCKED.md` files written by stalled parallel agents
- MoA proposal files (`moa-proposal-<n>.txt`)
- Any artifact that should survive within a session but need not be committed

**Naming convention**: `<agent-or-skill>-<artifact>.<ext>`
(e.g. `moa-proposal-1.txt`, `introspection-2026-05-02.md`, `parallel-blocked-crate-foo.md`)

`godmode handon` reports a count of files present. `godmode handoff` may snapshot keepers into
`.ctx/` proper. Everything in `_WORKING_DIR/` is gitignored via the `.ctx/*` rule.
