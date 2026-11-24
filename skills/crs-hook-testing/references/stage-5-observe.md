# Stage 5: Observe

Read this once a rule has been live for a session or more, to check its activity with stats, heat, history, log, or export.

After the rule has been live for a session or more, check its activity:

```bash
# Count by rule (all time):
crs stats

# Heatmap — visual frequency chart:
crs heat

# Recent blocked commands:
crs history

# Full hook execution log (last N entries):
crs log --limit 50

# Filter to a specific rule:
crs log --limit 200 | rg "<rule-id>"
```

Also export a full snapshot for offline analysis:

```bash
crs export > /tmp/crs-snapshot-$(date +%Y%m%d).json
```
