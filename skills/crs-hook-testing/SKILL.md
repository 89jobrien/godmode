---
name: "godmode:crs-hook-testing"
description: >
  Coursers rule lifecycle pipeline. Walk a course-correct rule (or candidate rule) through
  the full lifecycle: author → validate → probe → fire (pre/post) → observe → refine/retire.
  Use when adding a new crs/coursers rule, auditing an existing one, or diagnosing a rule
  that is misfiring or dead.
requires: []
next: []
argument-hint: "[<rule-name-or-pattern>]"
---

# CRS Hook Testing — Coursers Rule Lifecycle

A structured pipeline for managing the lifecycle of a `course-correct` rule in
`~/.config/coursers/course-correct-rules.json`.

Each stage maps to one or more `crs` / `coursers` subcommands. Walk through stages in
order for a new rule. Jump to the relevant stage when debugging or refining an existing
rule.

---

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

---

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

---

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
crs log --limit 200 | grep "<rule-id>"
```

Also export a full snapshot for offline analysis:

```bash
crs export > /tmp/crs-snapshot-$(date +%Y%m%d).json
```

---

## Stage 6: Refine or Retire

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

---

## Quick Reference

| Stage       | Command(s)                                          |
| ----------- | --------------------------------------------------- |
| Discover    | `crs discover`, `crs suggest`                       |
| Author      | Edit `~/.config/coursers/course-correct-rules.json` |
| Validate    | `crs validate`                                      |
| Probe       | `echo '<cmd>' \| crs probe`                         |
| Fire (pre)  | `echo '<json>' \| coursers pre`                     |
| Fire (post) | `echo '<json>' \| coursers post`                    |
| Observe     | `crs stats`, `crs heat`, `crs history`, `crs log`   |
| Export      | `crs export`                                        |
| Refine      | Edit rules → validate → probe → observe             |
| Replay      | `crs replay`, `crs insights`                        |

---

## Common Pitfalls

- `probe` vs `pre` divergence: `probe` matches the whole command string; `pre` splits on
  shell operators first. Always test with `crs pre` before trusting `crs probe` alone.
- After editing `crates/core` or `crates/coursers`, the live `crs`/`coursers` binaries
  are NOT updated automatically. Run `cargo install --path crates/coursers` to deploy.
- The `no-find-use-glob` rule matches `find .ctx` in commit messages. Exceptions for
  `git (commit|log|tag|stash)` (including `git -C` form) suppress this.
- State files: global at `~/.config/coursers/course-correct-state.json`; project-local
  at `.ctx/course-correct-state.json` (wins over global).
