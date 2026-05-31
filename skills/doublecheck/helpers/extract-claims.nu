#!/usr/bin/env nu
# doublecheck/helpers/extract-claims.nu
# Reference template for claim extraction output format.
# Not executed automatically — shows the expected structure.

# Expected claim extraction format (JSON):
# [
#   {
#     "id": "C1",
#     "text": "Python was created in 1991",
#     "category": "factual",
#     "initial_confidence": "high",
#     "requires_search": true
#   },
#   {
#     "id": "C2",
#     "text": "95% of enterprises use cloud services",
#     "category": "statistical",
#     "initial_confidence": "low",
#     "requires_search": true
#   }
# ]

print "This is a reference helper — see references/claim-categories.md for the full taxonomy."
