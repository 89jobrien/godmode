# Stages 4-5: Fire and observe

Read this before firing a rule through the real pre or post hook chain, or before checking a rule's activity with stats, heat, history, log, or export.

## Stage 4: Fire (pre/post)

Exercise the rule through the real hook chain:

**Pre-tool (blocking):**

```bash
echo '{"tool":"Bash","input":{"command":"<command-to-test>"}}' | coursers pre
```

Exit 0 = allowed. Exit 2 = blocked (rule fired). The response JSON is printed.

**Post-tool (failure learning):**

```bash
echo '{"tool":"Bash","input":{"command":"<cmd>"},"output":{"exit_code":1}}' | coursers post
```

Records the failure to the rolling state log. After `failure_threshold` hits, `crs pre`
will block the command even without a pattern match.

---

## Stage 5: Observe

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
