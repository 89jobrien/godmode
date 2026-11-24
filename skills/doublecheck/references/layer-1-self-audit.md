# Layer 1: Self-Audit

Read this before starting a doublecheck run, or when extracting claims from a response to verify. This layer runs before any web search.

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
