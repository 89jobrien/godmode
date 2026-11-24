# OpenCode parity

Read this before changing an agent, command, or hook that has an OpenCode projection.

`.opencode/` holds the OpenCode projection. Everything in it is GENERATED — edit the
`agents/cfg/*.cfg.yaml` sources or the hook registry, then regenerate.

| Surface  | Source of truth                                           | Generated output               | Regenerate with                           |
| -------- | --------------------------------------------------------- | ------------------------------ | ----------------------------------------- |
| Commands | `commands/gm/*.yaml`                                      | `.opencode/commands/gm-*.md`   | `nu commands/gm/generate.nu`              |
| Agents   | `agents/cfg/*.cfg.yaml` + `agents/prompts/*.txt`          | `.opencode/agents/*.md`        | `godmode agent generate-opencode --all`   |
| Hooks    | `REGISTRY` in `crates/godmode-core/src/hooks/registry.rs` | `.opencode/plugins/godmode.ts` | `godmode hook generate --client opencode` |
| Skills   | `skills/*/SKILL.md`                                       | none needed                    | —                                         |

Skills need no projection: OpenCode reads `.claude/skills/*/SKILL.md` natively.

Agent projections are deny-by-default. Granted tools become `permission: allow`, followed by a
`"*": deny` catch-all, because OpenCode applies the _last_ matching rule — the catch-all must
come last or it would be overridden.

Hooks map Claude's `PreToolUse`/`PostToolUse` to OpenCode's `tool.execute.before`/`after`.
Claude's `SessionStart` and `Stop` have **no OpenCode equivalent**, so these 6 hooks do not run
under OpenCode and are reported by a warning at plugin load:

`session-start`, `memory-bank-inject`, `task-management` (SessionStart) — `stop-guard`,
`memory-bank-update-remind`, `introspection` (Stop)

`UNSUPPORTED_OPENCODE_EVENTS` in `registry.rs` is the single source of truth for that list;
`opencode_unmapped()` derives the report from it. Do not hand-edit
`.opencode/plugins/godmode.ts` — three conformance tests fail if it drifts.
