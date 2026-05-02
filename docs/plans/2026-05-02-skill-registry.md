# Skill Registry & YAML Agent System

**Date**: 2026-05-02
**Status**: approved

## Goal

Replace the implicit filesystem-scan model for skills and agents with a structured registry
system. Skills and agents become first-class installable units with a global registry, a
defined schema, and a CLI for discovery and management — modelled on rx's registry pattern.

## Problems solved

- No way to install skills from external sources (URLs, GitHub)
- Agent `.md` files mix routing metadata with prose — hard to parse, validate, or generate
- No `godmode skill list` or `godmode agent list`
- Hook registration is manual — editing `hooks/hooks.json` by hand for every new skill
- No consistency enforcement on agent descriptions / trigger phrases

## Architecture

### Registry location

`~/.config/godmode/registry.json` — global, user-owned. Written by `godmode skill install`,
read by `godmode skill list`, `godmode agent list`, and the hook loader.

### Resolution order (skill lookup)

1. Plugin-local: `$CLAUDE_PLUGIN_ROOT/skills/<name>/` (highest priority)
2. Global registry: `~/.config/godmode/skills/<name>/`

Same-name collisions: plugin-local wins. Different names: both are available.

### Registry schema

```json
{
  "version": 1,
  "skills": [
    {
      "name": "godmode:brainstorm",
      "source": "local",
      "origin": "$CLAUDE_PLUGIN_ROOT/skills/brainstorm",
      "path": "$CLAUDE_PLUGIN_ROOT/skills/brainstorm",
      "hook": "hook.nu",
      "skill_md": "SKILL.md",
      "agent": "brainstorm-agent.yaml",
      "installed_at": "2026-05-02T00:00:00Z",
      "version": "1.1.1"
    }
  ]
}
```

`source` values: `local` (plugin bundle), `global` (user-installed), `remote` (fetched from URL).

### Agent YAML schema

Agents move from `.md` (frontmatter + prose) to `.yaml` (structured source of truth).
The `.md` file becomes a **generated artifact** produced by `godmode agent generate`.

```yaml
name: godmode:brainstorm-agent
version: "1.0.0"
description: >
  Use before any creative work — new features, architecture decisions, adding crates,
  designing APIs. Triggers on "let's build", "design X", "how should we structure".
triggers:
  - "let's build"
  - "I want to add"
  - "design X"
  - "how should we structure"
model: inherit
color: blue
tools: [Read, Write, Edit, Bash, Glob, Grep]
skills: [brainstorm]
prompt: skills/brainstorm/SKILL.md
hooks:
  - event: PreToolUse
    matcher: Write
    script: skills/brainstorm/hook.nu
metadata:
  author: Joseph O'Brien
  tags: [design, architecture, planning]
  since: "1.0.0"
  deprecated: false
```

Valid `hooks[].event` values (Claude Code canonical):
`PreToolUse`, `PostToolUse`, `PreCompact`, `SubagentStop`, `Stop`, `SessionStart`

`hooks[].matcher` is a tool name or `"*"` — only applies to `PreToolUse` / `PostToolUse`.

### `godmode agent generate`

Reads `agents/<name>-agent.yaml`, writes `agents/<name>-agent.md` with:

- YAML frontmatter: `name`, `description`, `model`, `color`, `tools`, `skills`
- Body: contents of the file referenced by `prompt:` field

Run automatically by `godmode release bump` and `nu hooks/install.nu`.

### `godmode skill install <source>`

Mirrors `rx install`:

- `<source>` can be a local path, directory, or GitHub URL
- Copies skill directory to `~/.config/godmode/skills/<name>/`
- Validates: `SKILL.md` present, `hook.nu` present (optional), `*-agent.yaml` present (optional)
- Upserts registry entry in `~/.config/godmode/registry.json`
- If agent YAML present, runs `godmode agent generate` to produce `.md`

### `godmode skill list`

Reads registry + plugin-local skills. Outputs table:

```
NAME                              SOURCE   VERSION  HOOK  AGENT
godmode:brainstorm                local    1.1.1    yes   yes
godmode:ci-fix                    local    1.1.1    yes   yes
godmode:my-custom-skill           global   0.1.0    no    yes
```

`--json` flag for machine output.

### `godmode agent list`

Reads all `agents/*.yaml` files (plugin-local + global registry).

```
NAME                              TRIGGERS                    SKILLS
godmode:brainstorm-agent          "let's build", "design X"   brainstorm
godmode:tdd-agent                 "implement", "add feature"  test-driven-development
```

`--json` and `--filter <keyword>` flags.

### Hook auto-registration

When `godmode skill install` runs, if the skill's agent YAML declares `hooks:`, those
entries are merged into `hooks/hooks.json` automatically. Existing entries with the same
script path are skipped (idempotent).

`godmode skill uninstall <name>` removes the registry entry and removes its hook entries
from `hooks/hooks.json`.

## Migration

Existing `.md` agent files are the source until their `.yaml` counterparts are authored.
`godmode agent migrate` reads each `agents/*.md`, extracts frontmatter, produces a
`agents/<name>-agent.yaml` stub with `prompt:` pointing to the skill's `SKILL.md`.
The original `.md` is kept until `godmode agent generate` overwrites it.

## Files affected

| File                                  | Action                                                                         |
| ------------------------------------- | ------------------------------------------------------------------------------ |
| `crates/godmode-core/src/registry.rs` | New — registry load/save/upsert                                                |
| `crates/godmode-core/src/skill.rs`    | New — skill install, list, resolution                                          |
| `crates/godmode-core/src/agent.rs`    | New — agent YAML parse, generate `.md`                                         |
| `crates/godmode-cli/src/main.rs`      | New subcommands: `skill install/list/uninstall`, `agent list/generate/migrate` |
| `agents/*.yaml`                       | New — YAML source for each agent                                               |
| `agents/*.md`                         | Generated — produced by `godmode agent generate`                               |
| `hooks/hooks.json`                    | Extended — auto-registration via `godmode skill install`                       |
| `hooks/install.nu`                    | Extended — run `godmode agent generate` after install                          |

## Out of scope

- Remote skill marketplace / index server
- Skill dependency resolution (skill A requires skill B)
- Agent composition (one agent calling another by registry name)
- Per-project registry overrides (global only for now)
