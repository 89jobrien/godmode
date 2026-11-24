# Stage 0: Discover

Read this before discovering candidate commands for a new rule. Adding or editing a rule in course-correct-rules.json is in stage-1-author.md.

## Stage 0: Discover (optional — new rules only)

Surface Bash commands that coursers is not yet handling:

```bash
crs discover
```

Output: commands seen in Claude Code session logs that have no matching rule.
Pick candidates for a new rule from this list.

Also useful:

```bash
crs suggest
```

Output: heuristically proposed new rules based on unhandled command frequency.
