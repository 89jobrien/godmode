# Phase 2: LLM assist prompt

Read this before running the LLM pass that labels trace clusters in Phase 2. Run it inline with the trace samples filled in.

```text
Given these agent trace samples, identify:
1. Patterns that should be blocked but aren't
2. Patterns that are blocked but shouldn't be (false positives)
3. Block messages that are unhelpful or wrong
4. Threshold values that fire too early or too late

Traces:
<traces>{{traces}}</traces>

Respond as JSON: { clusters: [{ id, type, severity, evidence_count, sample, diagnosis }] }
```
