# Hallucination Pattern Matrix (--deep only)

Read this before producing a `--deep` doublecheck report, when the user has asked for the full matrix.

When invoked with `--deep`, render this matrix for every claim reviewed. One row per claim.
Columns map directly to the seven patterns in `references/layer-3-adversarial-review.md` — mark each cell Y (present), N (not
present), or ? (cannot determine).

| Claim ID | Fabricated citation | Precise # w/o source | Confident on uncertain | Wrong association | Temporal confusion | Overgeneralization | Missing qualifiers | Confidence rating |
| -------- | :-----------------: | :------------------: | :--------------------: | :---------------: | :----------------: | :----------------: | :----------------: | ----------------- |
| C1       |                     |                      |                        |                   |                    |                    |                    |                   |
| C2       |                     |                      |                        |                   |                    |                    |                    |                   |

Fill every cell. A blank cell is not the same as N — unknown is `?`. Any `Y` in columns
1–2 escalates to FABRICATION RISK regardless of other columns. Any `Y` in columns 3–7
escalates to DISPUTED or UNVERIFIED depending on whether contradicting evidence was found.
