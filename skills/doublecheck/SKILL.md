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

---

## Layer 1: Self-Audit

Re-read the target output with a critical lens. No web searches yet — this layer is extraction
and internal analysis only.

### Step 1: Extract Claims

Go through the text sentence by sentence and pull out every statement that asserts something
verifiable. Assign each a temporary ID (C1, C2, ...) for tracking.

| Category        | What to look for                                     | Examples                                                         |
| --------------- | ---------------------------------------------------- | ---------------------------------------------------------------- |
| **Factual**     | How things are or were                               | "Python was created in 1991"                                     |
| **Statistical** | Numbers, percentages, quantities                     | "95% of enterprises use cloud services"                          |
| **Citation**    | Specific documents, cases, laws, papers, standards   | "Under Section 230 of the CDA...", "_Mayo v. Prometheus_ (2012)" |
| **Entity**      | Claims about people, organizations, products, places | "OpenAI was founded by Sam Altman and Elon Musk"                 |
| **Causal**      | X caused Y, X leads to Y                             | "This vulnerability allows remote code execution"                |
| **Temporal**    | Dates, timelines, sequences                          | "Version 2.0 was released before the security patch"             |

### Step 2: Check Internal Consistency

Review extracted claims against each other:

- Does the text contradict itself? (two different dates for the same event)
- Are any claims logically incompatible?
- Does it make an assumption in one section that it contradicts in another?

Flag contradictions immediately — these need no external search to identify as problems.

### Step 3: Initial Confidence Assessment

For each claim, assess from internal knowledge only:

- Is this accurate to your recollection?
- Is this a high-risk hallucination category? (Specific citations, precise statistics, and
  exact dates are the most frequently fabricated.)
- Is the claim specific enough to be falsifiable at all?

Record initial confidence as input to Layer 2. Do not surface these ratings as findings yet.

---

## Layer 2: Source Verification

Search for external evidence for each extracted claim. The goal is to give the user URLs they
can visit themselves — not your summary of what those URLs say.

### Search Strategy

For each claim:

1. Formulate a search query targeting the primary source — exact case name, specific statistic
   and topic, key entities and relationship.
2. Run the search with `web_search`. If the first query returns nothing relevant, try once
   more with different terms. Stop at two attempts per claim.
3. Evaluate results:
   - Primary or authoritative source that directly addresses the claim? Record it.
   - Contradicting information from a credible source? Record it and the contradiction.
   - Nothing relevant? Record the absence — real things usually have a web footprint.
4. Always record the URL. Do not summarize without linking.

### Source Quality

Prefer in order: primary sources (official docs, court records, legislative text, spec bodies,
regulatory filings) → peer-reviewed publications → established reference works → news/blogs.
Note when a source is secondary so the user can weigh it.

### Citations Require Special Treatment

Citations are the highest-risk hallucination category. For any claim citing a specific case,
statute, paper, standard, or document:

1. Search for the exact citation (case name, title, section number).
2. If found, confirm the cited content actually says what the target text claims.
3. If not found at all: **FABRICATION RISK**. Models routinely generate plausible-sounding
   citations for things that do not exist.

---

## Layer 3: Adversarial Review

Switch posture entirely. Assume the output contains errors and actively try to find them.

### Hallucination Pattern Checklist

Check every claim for these patterns:

1. **Fabricated citation** — specific case, paper, or statute that could not be found in
   Layer 2. Most dangerous pattern: looks authoritative, is invented.

2. **Precise number without source** — specific statistic (e.g., "78% of companies…") with
   no indicated origin. Models generate plausible statistics that are entirely fabricated.

3. **Confident specificity on uncertain topics** — exact dates, precise amounts, definitive
   attributions in areas where experts disagree or facts are genuinely unknown.

4. **Plausible-but-wrong association** — ruling attributed to wrong court, quote to wrong
   person, law's provision described incorrectly while the law's name is right.

5. **Temporal confusion** — something described as current that may be outdated; events
   described in wrong sequence.

6. **Overgeneralization** — stated as universally true when it applies only in specific
   jurisdictions, contexts, or time periods. Very common in legal and regulatory content.

7. **Missing qualifiers** — nuanced topic presented as settled when significant exceptions,
   limitations, or counterarguments exist.

### Hallucination Pattern Matrix (--deep only)

When invoked with `--deep`, render this matrix for every claim reviewed. One row per claim.
Columns map directly to the seven patterns above — mark each cell Y (present), N (not
present), or ? (cannot determine).

| Claim ID | Fabricated citation | Precise # w/o source | Confident on uncertain | Wrong association | Temporal confusion | Overgeneralization | Missing qualifiers | Confidence rating |
| -------- | :-----------------: | :------------------: | :--------------------: | :---------------: | :----------------: | :----------------: | :----------------: | ----------------- |
| C1       |                     |                      |                        |                   |                    |                    |                    |                   |
| C2       |                     |                      |                        |                   |                    |                    |                    |                   |

Fill every cell. A blank cell is not the same as N — unknown is `?`. Any `Y` in columns
1–2 escalates to FABRICATION RISK regardless of other columns. Any `Y` in columns 3–7
escalates to DISPUTED or UNVERIFIED depending on whether contradicting evidence was found.

### Adversarial Questions

For each major claim that passed Layers 1 and 2:

- What would make this wrong?
- Is there a common misconception in this area the model might have absorbed?
- Would a domain expert object to how this is stated?
- Is this claim from after the training cutoff — could it be outdated?

### Red Flags to Escalate

Escalate these prominently — do not bury them in the table:

- A citation that cannot be found anywhere
- A statistic with no identifiable source
- A legal or regulatory claim contradicting what authoritative sources say
- A claim stated with high confidence that is actually disputed or uncertain

---

## Output: Verification Report

After completing all three layers, produce the report using
`references/verification-report-template.md`.

### Confidence Ratings

| Rating               | Meaning                                                          | User action                                             |
| -------------------- | ---------------------------------------------------------------- | ------------------------------------------------------- |
| **VERIFIED**         | Supporting source found and linked                               | Spot-check the link if critical                         |
| **PLAUSIBLE**        | Consistent with general knowledge; no specific source found      | Reasonable but unconfirmed; verify before relying on it |
| **UNVERIFIED**       | No supporting or contradicting evidence found                    | Do not rely on without independent verification         |
| **DISPUTED**         | Contradicting evidence found from a credible source              | Review the contradicting source; this may be wrong      |
| **FABRICATION RISK** | Matches hallucination pattern (unfindable citation or statistic) | Assume wrong until confirmed from a primary source      |

### Report Principles

- Provide links, not verdicts. The user decides what is true.
- When contradicting information was found, present both sides with sources. Do not pick a winner.
- If a claim is unfalsifiable (too vague or subjective to verify), say so explicitly.
- Be explicit about what could not be checked. "Could not verify" differs from "this is wrong."
- Lead with the highest-severity findings.

### Limitations Disclosure

Always append to the report:

> **Limitations:** This pipeline accelerates human verification; it does not replace it.
> Web search may not reach paywalled or very recent sources. The adversarial layer uses the
> same underlying model that may have produced the original output — it catches many issues
> but not all. VERIFIED means a supporting source was found, not that the claim is definitely
> correct. PLAUSIBLE claims may still be wrong.

---

## Domain Guidance

### Legal Content

Elevated hallucination risk: case citations and holdings are frequently fabricated, jurisdictional
nuances are flattened, statutory language gets paraphrased in meaning-changing ways, majority/
minority distinctions disappear.

Extra scrutiny: case citations, statutory references, regulatory interpretations, and any
jurisdictional claim. Search legal databases when available.

### Medical and Scientific Content

- Verify cited studies exist and that results are accurately described
- Flag outdated guidelines presented as current
- Flag dosages, treatment protocols, diagnostic criteria — these change, errors are dangerous

### Financial and Regulatory Content

- Verify specific dollar amounts, dates, and thresholds
- Confirm regulatory requirements are for the correct jurisdiction and are current
- Watch for tax law claims outdated by recent legislative changes

### Technical and Security Content

- Verify CVE numbers, vulnerability descriptions, and affected versions
- Check API specs and configuration instructions against current documentation
- Flag version-specific information that may be outdated

---

## Related

- `skills/systematic-debugging/SKILL.md` — same evidence-first posture applied to code
- `skills/code-review/SKILL.md` — adversarial review applied to implementation
