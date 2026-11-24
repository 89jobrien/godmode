# Stages 2-3: Validate and probe

Read this before running `crs validate` on the rules file, or before testing a rule against a command string with `crs probe` or `crs pre`.

## Stage 2: Validate

Check that all rules are well-formed, patterns compile, and alternative tools are on PATH:

```bash
crs validate
```

Fix any errors before continuing. Common failures:

- Invalid regex in `pattern` or `exceptions`
- Alternative tool not on PATH (install it or remove the alternative field)

---

## Stage 3: Probe

Interactively test a rule against a specific command string before wiring into the live hook:

```bash
# Pass a raw command string:
echo "grep -r foo ." | crs probe

# Pass a PreToolUse JSON payload (matches real hook input):
echo '{"tool":"Bash","input":{"command":"grep -r foo ."}}' | crs probe
```

`crs probe` prints per-rule verdicts — which rules matched, which exceptions fired, and the
final block/allow decision.

Note: `probe` matches the whole command string. The live `crs pre` hook additionally
splits pipeline commands on `;` / `&&` / `||` before matching. If a rule behaves
differently in production than in probe, test via `crs pre` with the full JSON payload.

```bash
echo '{"tool":"Bash","input":{"command":"grep -r foo . && echo done"}}' | crs pre
```
