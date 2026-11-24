# Layer 2: Source Verification

Read this before searching for external evidence for extracted claims, or before verifying a citation against its source.

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
