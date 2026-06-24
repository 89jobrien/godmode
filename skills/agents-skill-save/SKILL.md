---
name: agents-skill-save
description: Use when creating a new local skill or fixing an existing skill that was saved to the wrong skills tree. Symptoms - a skill was written under ~/.claude/skills, the active setup uses ~/.agents/skills, or skill instructions still reference the old path convention.
---

# Agents Skill Save

## When to Use

Use this when you need to save a new skill into the active local skills tree, or when an existing skill still points at the older `~/.claude/skills/` layout.
It is especially useful after creating a skill from session work and needing to place it in the right directory the first time.

## Commands

```bash
# 1. Confirm the active skills tree
ls -la ~/.agents ~/.agents/skills
# Use Glob tool: glob '/Users/joe/.agents/skills/**/SKILL.md'

# 2. Inspect the existing skill content before changing it
sed -n '1,260p' /path/to/SKILL.md

# 3. Create the destination directory for the skill
mkdir -p ~/.agents/skills/<skill-name>

# 4. If the skill was saved to the wrong tree, move it
mv ~/.claude/skills/<old-path>/SKILL.md ~/.agents/skills/<skill-name>/SKILL.md

# 5. Patch stale path references inside skill docs
# Replace ~/.claude/skills/... with ~/.agents/skills/...
# Remove invented personal namespaces unless the tree already uses them

# 6. Verify the final file
sed -n '1,220p' ~/.agents/skills/<skill-name>/SKILL.md
```

## Common Failures

| Symptom                                                                          | Fix                                                                                                       |
| -------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| `~/.agents/skills` exists but the new skill was written under `~/.claude/skills` | Move the file into `~/.agents/skills/<name>/SKILL.md` and remove the misplaced copy                       |
| Skill text still mentions `~/.claude/skills/`                                    | Patch the instructions so future skill writes follow the active tree                                      |
| Unsure whether to use a personal namespace like `joe/`                           | Do not invent one unless the existing `~/.agents/skills/` tree already uses it                            |
| Parent directory does not exist                                                  | Create it first with `mkdir -p ~/.agents/skills/<skill-name>`                                             |
| Not sure whether a manifest must be updated                                      | Check `~/.claude/plugins/` and `~/.agents/` metadata only if the current setup actually uses registration |
