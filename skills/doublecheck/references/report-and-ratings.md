# Verification report, ratings, and limitations

Read this before writing the output of a doublecheck run, choosing a confidence rating for a claim, or adding the limitations disclosure.

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
