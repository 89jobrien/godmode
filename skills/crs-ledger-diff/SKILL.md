---
name: "godmode:crs-ledger-diff"
description: >
  Diff live coursers rule-hit stats and the installed rule set against the
  hand-authored prose rule ledger, and propose skeleton entries for rules that
  have no prose yet. Discover/propose half of the rule-ledger pair.
requires: []
next: [crs-ledger-apply]
---

# crs-ledger-diff

Compare live course-correct rule hit stats and the installed rule set
against the hand-authored prose ledger, and propose skeleton entries for any
rule that has no prose yet. This is the discover/propose half of the
coursers rule-ledger pair; `crs-ledger-apply` is the merge/render half.

## When to Use

- Asked to "sync the rule ledger", "check for missing rule prose", or
  "diff the ledger against crs-stats"
- Periodically, as a sibling check to `crs-discover` (that pipeline finds new
  _rule candidates_; this one finds rules that exist but lack a prose entry)

## Steps

1. Read the live rule set from `~/.config/coursers/course-correct-rules.json`
   (same path `crs-validate` reads) and collect every rule `id`.
2. Read `.ctx/coursers/rule-ledger.yaml`. If it does not exist, treat it as
   an empty mapping — this is not an error.
3. Diff:
   - `missing` = live rule IDs with no key in `rule-ledger.yaml`
   - `orphaned` = keys in `rule-ledger.yaml` with no matching live rule ID
4. For each `missing` ID, write a skeleton entry to
   `.ctx/coursers/rule-ledger-proposed.yaml`. `pattern` is filled from the
   live rule's own `pattern` field (not left as TODO — it's already known);
   `category`/`prevention`/`notes` are genuinely hand-authored judgment and
   stay TODO for a human to fill in:

   ```yaml
   <rule-id>:
     category: "TODO"
     pattern: "<copied verbatim from course-correct-rules.json's pattern field for this rule>"
     prevention: "TODO"
     notes: ""
   ```

   If there are no missing rules, write `{}` to the output path — don't skip
   writing it, mirroring `crs-discover`'s "write empty, don't skip" rule for
   zero candidates.

5. Report: the count of missing rules now proposed, and the list of
   `orphaned` rule IDs (reported only — never written anywhere; a human
   decides whether to prune those entries from `rule-ledger.yaml` by hand).

## Contract

- **Input**: `~/.config/coursers/course-correct-rules.json` (read-only),
  `.ctx/coursers/rule-ledger.yaml` (read-only, optional)
- **Output**: `.ctx/coursers/rule-ledger-proposed.yaml` — object keyed by
  rule ID, skeleton entries only. `crs-ledger-apply` merges this file.
- Never writes to `.ctx/coursers/rule-ledger.yaml` or `rule-ledger.md` — this
  step only proposes, exactly like `crs-propose-rules` never installs.
- Never writes to `.ctx/coursers/crs-stats.json` or any file under
  `.ctx/memory-bank/` — occurrence counts and memory-bank's own mistake
  ledger are both out of scope for this pair of skills.
