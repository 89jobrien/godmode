# Stage 6: Refine or retire

Read this before changing a rule based on observed activity, removing an obsolete rule, or checking rule health across sessions with insights and replay.

Based on Stage 5 observations:

### Rule fires too broadly (false positives)

- Narrow `pattern` regex, or
- Add more entries to `exceptions`

Then re-run Stage 2 (validate) and Stage 3 (probe) before deploying.

### Rule never fires

- Verify the pattern actually matches the commands you care about via `crs probe`
- Check if an exception is over-broad and eating all matches

### Rule is obsolete

Remove the rule from the JSON file. Run `crs validate` to confirm the file is still clean.

### Tracking rule health across sessions

```bash
crs insights    # session facets with git context
crs replay      # replay past session Bash commands through the current ruleset
```

`replay` is especially useful after narrowing a pattern — it shows how the change would
have affected past sessions.
