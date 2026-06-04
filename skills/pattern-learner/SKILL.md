---
name: "godmode:pattern-learner"
description: >
  Cross-session pattern extraction. Analyzes session traces, git history, and
  self-reflection data to discover recurring workflows, crate coupling patterns,
  and problem-solving conventions. Writes findings to memory-bank.
requires: []
next: []
---

# Pattern Learner

Extract recurring patterns from session history and codebases to discover your
workflow conventions, crate coupling relationships, and preferred API shapes.

## When to Use

- After completing multiple sessions of work (5+ sessions preferred)
- When asked to "find patterns", "learn from sessions", "what do I keep doing"
- To audit recurring failure modes and their fixes
- To document emerging conventions before they become implicit
- To identify which skills, tools, and crates form your core workflow

## Analysis Dimensions

### 1. Skill Co-occurrence

Identify which skills are invoked together in sessions:

**What to measure**:

- Which skill pairs or triplets appear together consistently?
- Do certain skills always precede or follow others?
- What is the most frequent skill ordering in your workflow?

**Output**:

```
Skill co-occurrence: [skill-a, skill-b]
Occurrences: 5/8 sessions
Confidence: high
Pattern: skill-a always runs before skill-b in code-review workflows
```

**Confidence threshold**: Require 3+ occurrences out of recent 8 sessions for
`high` confidence. Fewer occurrences = `medium` or `low`.

### 2. Crate Coupling

Identify which crates change together in commits:

**What to measure**:

- Which crate pairs consistently update in the same commit?
- Are there obvious source-of-truth dependencies revealed by change patterns?
- Do certain modules always require parallel updates?

**Output**:

```
Crate coupling: crux-core ↔ devkit
Occurrences: 5/8 recent commits
Confidence: high
Pattern: Changes to core runtime (crux-core) trigger toolkit updates (devkit)
```

**Confidence threshold**: Require 3+ occurrences in recent 10 commits for `high`
confidence.

### 3. Failure-Fix Pairs

Identify recurring test failure → fix sequences:

**What to measure**:

- Do the same tests fail repeatedly?
- What command sequence fixes each failure type?
- Is there a root-cause pattern (e.g., "missing unsafe block comment")?

**Output**:

```
Failure-fix pair: clippy unsafe warnings
Failure cause: Missing SAFETY comment on unsafe block
Fix sequence: [1. Add SAFETY doc comment, 2. Run cargo clippy, 3. Commit]
Frequency: 3/10 recent sessions
Confidence: high
```

**Confidence threshold**: Require 3+ occurrences for a pattern to be `high`
confidence. One or two occurrences = `low`.

### 4. Conventions and API Shapes

Identify recurring naming, structure, and coding patterns:

**What to measure**:

- Are there preferred naming schemes (e.g., `godmode:domain-service`)?
- Do function signatures follow a consistent pattern?
- What error handling style dominates?
- Are there recurring doc comment templates or structures?

**Output**:

```
Naming convention: godmode:<domain>-<service>
Examples: godmode:pattern-learner, godmode:code-review, godmode:introspection
Frequency: 7/8 recent agent definitions
Confidence: high
Rationale: Consistent naming aids discoverability and mental model alignment
```

**Confidence threshold**: Require 60%+ adherence in recent samples for `high`.

## Output Format

All patterns must be documented as structured entries:

```markdown
## Patterns — <YYYY-MM-DD>

### <Pattern Category>: <Pattern Name>

**Frequency**: X/Y recent observations
**Confidence**: [high|medium|low]
**Description**: One-sentence summary of the pattern
**Evidence**: Numbered list of recent examples (commits, sessions, files)
**Recommended actions**: Optional; what to do with this knowledge
```

If a pattern has fewer than 3 supporting observations, mark it `Confidence:
low`. Do not promote uncertain patterns to established knowledge.

## Confidence Rules

| Observations | Confidence | Action                                   |
| ------------ | ---------- | ---------------------------------------- |
| 1–2          | low        | Record as tentative; flag for monitoring |
| 3–5          | medium     | Likely real; worth documenting           |
| 6+           | high       | Established pattern; apply widely        |

**Special cases**:

- If a single failure-fix pair solves a blocking issue 2+ times, mark as
  `medium` even with 2 occurrences.
- If a naming convention appears in 100% of recent agents (>5), mark as `high`
  even with fewer observations.
- If a pattern contradicts a documented convention, flag as `Confidence: low`
  and investigate before promoting.

## Guardrails

- **Read-only analysis**: Never modify source code, tests, or agent definitions.
- **Append-only memory**: Append new patterns to memory-bank; never delete or
  overwrite existing patterns.
- **Minimum evidence threshold**: Require 3+ occurrences before declaring a
  pattern "established". Use `low` confidence liberally for emerging patterns.
- **Date all patterns**: Include the observation date; older patterns may
  become stale.
- **Ground in evidence**: Every pattern must cite specific commits, session IDs,
  or files. Never invent patterns from intuition alone.
- **Flag contradictions**: If a new pattern contradicts an existing one, note
  the conflict and suggest investigation.

## See Also

- `.ctx/memory-bank/patterns.md` — main pattern repository
- `skills/introspection/SKILL.md` — plugin and skill consistency auditing
- `agents/pattern-learner-agent.md` — agent definition
