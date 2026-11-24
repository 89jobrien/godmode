---
name: "godmode:mistake-tracker"
description: >
  Maintain a catalog of recurring mistakes, error patterns, and failure modes detected
  across sessions, commits, and CI runs. Use this skill to detect what keeps breaking,
  find root patterns, and capture prevention strategies.
requires: []
next: []
---

# Mistake Tracker

A persistent ledger at `.ctx/godmode/memory-bank/mistakes.md` that catalogs recurring error
patterns, failure modes, and process mistakes. The ledger is grounded in actual session
traces and git history — not assumptions.

## When to Use

- When asked "what mistakes keep happening?" or "why does this keep breaking?"
- After recurring CI failures or local test flakes
- Before a major release, to surface systemic issues
- When tracking prevents the same bug from shipping multiple times
- As a feedback loop to improve process and infrastructure

## Reference files

Read the file named in a row before doing the task in its right column. These files are not
loaded automatically.

| File                                 | Read before                                                                                    |
| ------------------------------------ | ---------------------------------------------------------------------------------------------- |
| `references/detection-categories.md` | classifying a failure, or deciding which of the five categories a pattern belongs to           |
| `references/ledger-format.md`        | creating or appending to `.ctx/godmode/memory-bank/mistakes.md`, or checking an entry's fields |
| `references/output-format.md`        | reporting findings to the user                                                                 |

## Integration with Other Skills

### godmode:verification-before-completion

Check the mistake ledger before shipping. Flag if:

- A pattern in the ledger affects the code being shipped
- Prevention strategies have not been applied
- The same mistake appears in new commits

### godmode:self-reflect

Reference the mistake ledger when reflecting on session outcomes. Use findings to
improve processes and identify where reviews or tests need strengthening.

### godmode:introspection

Include mistake ledger review in conformance checks. Verify:

- Ledger is current (updated within last 5 sessions)
- Prevention strategies are actionable
- Patterns have not been resolved but forgotten in the ledger

## Update Rules

- **After each session**: append any newly detected patterns
- **Weekly review**: check if prevention strategies have been implemented
- **When patterns are resolved**: add resolution note with date
- **Never delete entries**: keep full history for post-mortems and trend analysis
- **Consolidate duplicates**: if the same pattern appears under different names,
  merge entries and update counts

## Guardrails

- All patterns must be backed by actual traces or git history — never invent
- Occurrence counts must be exact; estimate only if source is unclear
- Dates must be accurate (use commit dates or .jsonl timestamps)
- Prevention strategies must be specific and actionable
- Do not modify source code; only catalog and recommend
- Keep the ledger append-only for audit trail purposes
