# Verification Report

**Target**: <!-- brief description of the output being verified -->
**Date**: <!-- YYYY-MM-DD -->
**Layers completed**: Self-Audit / Source Verification / Adversarial Review

---

## Summary

| Stat                    | Count |
| ----------------------- | ----- |
| Total claims extracted  |       |
| VERIFIED                |       |
| PLAUSIBLE               |       |
| UNVERIFIED              |       |
| DISPUTED                |       |
| FABRICATION RISK        |       |
| Internal contradictions |       |

<!-- If any claim is DISPUTED or FABRICATION RISK, add a callout here: -->
<!--
> **Heads up:** [specific claim] could not be verified / contradicts [source]. Verify
> independently before relying on it.
-->

---

## Claim-by-Claim Findings

<!-- Repeat this block for each extracted claim -->

### C1 — [brief claim text]

**Category**: Factual / Statistical / Citation / Entity / Causal / Temporal
**Rating**: VERIFIED / PLAUSIBLE / UNVERIFIED / DISPUTED / FABRICATION RISK

**Evidence**:

- Source: [URL, or `file:line` / commit hash for repo-grounded claims] — [one-line summary of what the source says]
- Contradicting source (if any): [URL or `file:line`] — [one-line summary]

**Notes**: <!-- anything the user should know about this claim -->

---

<!-- ... repeat for C2, C3, ... -->

---

## Internal Contradictions

<!-- List any contradictions found within the target text itself -->

| Location | Contradiction |
| -------- | ------------- |
|          |               |

---

## Adversarial Findings

<!-- Items identified in Layer 3 that warrant explicit callout -->

| Pattern                         | Claim ID | Detail |
| ------------------------------- | -------- | ------ |
| Fabricated citation             |          |        |
| Precise number without source   |          |        |
| Overgeneralization              |          |        |
| Missing qualifier               |          |        |
| Temporal confusion              |          |        |
| Plausible-but-wrong association |          |        |

---

## What to Do Before Relying on This Output

- [ ] Review all FABRICATION RISK and DISPUTED claims independently
- [ ] Visit source links for any VERIFIED claim critical to your decision
- [ ] Treat PLAUSIBLE claims as unconfirmed until you check a primary source
- [ ] Check UNVERIFIED claims manually — absence of a web result is not absence of error
- [ ] Re-run repo-grounded checks if the tree has changed since the verification run

---

> **Limitations:** This pipeline accelerates human verification; it does not replace it.
> Web search may not reach paywalled or very recent sources. The adversarial layer uses the
> same underlying model that may have produced the original output — it catches many issues
> but not all. VERIFIED means a supporting source was found, not that the claim is definitely
> correct. PLAUSIBLE claims may still be wrong.
