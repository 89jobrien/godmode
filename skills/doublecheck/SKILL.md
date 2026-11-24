---
name: "godmode:doublecheck"
description: >
  Three-layer verification pipeline for AI-generated output. Extracts every verifiable claim,
  searches for supporting or contradicting sources, runs an adversarial hallucination review,
  and produces a structured report with source links for human review. Use after any response
  that contains citations, statistics, legal/regulatory analysis, or specific factual claims
  you intend to rely on. Also use inline after your own substantive responses when accuracy
  matters.
requires: []
next: []
argument-hint: "[--deep]"
---

# Doublecheck

Run a three-layer verification pipeline on AI-generated output. The goal is not to declare
what is true — it is to extract every verifiable claim, surface sources the user can check
independently, and flag hallucination patterns before they propagate.

## When to Run

- After producing any response containing citations, statistics, legal or regulatory claims,
  case references, or specific factual assertions
- When the user says "doublecheck that", "verify that", or "full report"
- Before committing analysis, recommendations, or sourced content to a plan or document
- Any time a response felt confident about something specific — confidence is not accuracy

## Reference files

Read the file named in a row before doing the step in its right column. These files are not
loaded automatically.

| File                                                        | Read before                                                                                    |
| ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| `references/layer-1-self-audit.md`                          | starting a run: extracting claims, checking internal consistency, initial confidence           |
| `references/layer-2-source-verification.md`                 | searching for sources, or checking a citation against its source                               |
| `references/layer-3-adversarial-review.md`                  | the adversarial pass, or the hallucination pattern checklist                                   |
| `references/layer-3-adversarial-questions-and-red-flags.md` | asking the adversarial questions for a major claim, or deciding which red flags to escalate    |
| `references/layer-3-deep-matrix.md`                         | running with `--deep`, which requires the per-claim pattern matrix                             |
| `references/report-and-ratings.md`                          | writing the report, choosing a confidence rating, or adding the limitations disclosure         |
| `references/verification-report-template.md`                | laying out the report itself                                                                   |
| `references/claim-categories.md`                            | classifying a claim when the category is unclear                                               |
| `references/domain-guidance.md`                             | the output is legal, medical or scientific, financial or regulatory, or technical and security |

## Layer summary

1. **Self-audit** (no web search). Extract claims as C1, C2, ..., check them against each other,
   and record initial confidence from internal knowledge. See `layer-1-self-audit.md`.
2. **Source verification**. Search for each claim, record the URL, and treat citations as the
   highest-risk category. A citation that cannot be found is FABRICATION RISK. See
   `layer-2-source-verification.md`.
3. **Adversarial review**. Assume the output is wrong and check every claim against the seven
   hallucination patterns. Escalate red flags prominently. See `layer-3-adversarial-review.md`.

With `--deep`, render the per-claim matrix from `layer-3-deep-matrix.md`.

## Output

Produce the report with `references/verification-report-template.md`, using the confidence
ratings and report principles in `references/report-and-ratings.md`. Always append the
limitations disclosure from that file. Lead with the highest-severity findings.

## Key rules

- Provide links, not verdicts. The user decides what is true.
- A `Y` in the fabrication or precise-number columns escalates to FABRICATION RISK.
- Present both sides when sources conflict. Do not pick a winner.
- Say what could not be checked. "Could not verify" is different from "this is wrong."
- Do not rely on a claim's own confidence. Confidence is not accuracy.

## Related

- `skills/systematic-debugging/SKILL.md` — same evidence-first posture applied to code
- `skills/code-review/SKILL.md` — adversarial review applied to implementation
