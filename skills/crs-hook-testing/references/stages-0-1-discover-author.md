# Stages 0-1: Discover and author

Read this before discovering candidate commands for a new rule, or before adding or editing a rule in `course-correct-rules.json`.

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

---

## Stage 1: Author

Edit `~/.config/coursers/course-correct-rules.json` (or the path in `$COURSERS_RULES`).

Each rule has:

```json
{
  "id": "no-raw-grep",
  "description": "Use ripgrep (rg) instead of grep",
  "pattern": "\\bgrep\\b",
  "message": "Use `rg` instead of `grep` — faster and respects .gitignore",
  "exceptions": [
    "git (commit|log|tag|stash)",
    "git -C .* (commit|log|tag|stash)"
  ],
  "alternative": "rg",
  "failure_threshold": null
}
```

Key fields:

- `pattern` — regex matched against the full Bash command string (not per-segment)
- `exceptions` — list of regexes; if any match, the rule does not fire
- `alternative` — tool name used in the block message and checked by `validate`
- `failure_threshold` — optional: block a command after this many consecutive failures
