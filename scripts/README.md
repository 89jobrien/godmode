# scripts/

Standalone utility scripts for godmode maintenance and development.

| Script                     | Purpose                                                                                            |
| -------------------------- | -------------------------------------------------------------------------------------------------- |
| `check-refs.nu`            | Audit broken skill references — finds `godmode:<name>` refs that don't resolve to a real skill dir |
| `check-skill-index.nu`     | Verify skill index is consistent with actual skill directories                                     |
| `gen-index.nu`             | Regenerate `agents/INDEX.md` from agent frontmatter                                                |
| `rebuild-reports-index.nu` | Rebuild the reports index under `docs/`                                                            |
| `bump-version.sh`          | Bump the plugin version in `.claude-plugin/plugin.json`                                            |
| `aichat-plan.nu`           | Generate aichat system prompts from godmode skill content                                          |
| `aichat-system-install.nu` | Install generated aichat system prompts to the aichat config dir                                   |

## Common Tasks

```bash
# After adding a new skill, verify no broken refs:
nu scripts/check-refs.nu

# After adding or renaming agents, regenerate the index:
nu scripts/gen-index.nu

# After committing, bump the plugin version to HEAD SHA:
nu scripts/bump-version.sh
# Then reinstall: claude plugin install godmode@local
```
