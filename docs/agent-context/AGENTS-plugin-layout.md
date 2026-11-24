# Plugin layout

Read this before changing the plugin manifest, skills directory, or agents directory.

This repo is also a Claude Code plugin installed via bazaar:

```text
.claude-plugin/plugin.json   # name, version, author, description only — no extra fields
skills/                      # discovered by directory scan, not declared in plugin.json
agents/                      # top-level *.md are GENERATED — Claude discovers these
  cfg/*.cfg.yaml             # source of truth: structured agent config
  prompts/*.prompt.txt       # source of truth: raw prompt text
  INDEX.md                   # generated: agent table
```

Plugin manifest schema accepts only: `name`, `version`, `author`, `description`. Extra fields
cause validation failure on `claude plugin install`.
