# Phase 1: trace commands

Read this before collecting traces with the coursers/crs adapter. It holds the commands for SKILL.md Phase 1.

```nu
# Recent session history for current project
crs discover --since 30 --format json

# Richer facet view with git context
crs insights --format json

# Block stats — which rules fired most
crs stats
```
