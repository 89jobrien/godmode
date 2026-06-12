---
name: token-cost-optimizer
description: Use when analyzing Claude or agent token usage, ccusage reports, output-to-input token ratios, model spend, or ways to lower AI coding costs. Symptoms - the user asks "why is Claude expensive", "analyze token usage", "reduce token usage", "optimize costs", "make agents less verbose", "lower output/input ratio", or shares `ccusage` output. Produces concise technical strategies, model-routing rules, and copy-paste cost-control policies.
---

# Token Cost Optimizer

## When to Use

Use this when the user wants to understand or reduce LLM/agent costs.
Focus on measured drivers before giving advice: model mix, output volume, cache creation, session length, subagent fan-out, and repeated verbose reporting.
Do not assume input tokens are the problem just because token totals are large; generated output and expensive model choice are often the largest controllable costs.

## Workflow

1. Gather usage data.
   - Prefer already-attached command output when available.
   - If more data is needed and `ccusage` is available, run read-only reports.
   - Use JSON output for aggregates and rendered output for quick inspection.

```bash
ccusage monthly --json --offline
ccusage weekly --json --offline
ccusage daily --breakdown --offline
ccusage session --breakdown --offline
ccusage daily --instances --breakdown --offline
ccusage blocks --offline
```

2. Identify cost drivers.
   - Total cost and trend by month/week/day.
   - Output/input ratio using direct input and output tokens.
   - Per-model cost share, especially Opus and Sonnet.
   - High-cost days, sessions, blocks, or projects.
   - Cache creation versus cache reads; high cache creation can signal repeated context rebuilds.
   - Multipliers from parallel agents, long reviews, broad planning, or verbose final summaries.

3. Recommend controls in priority order.
   - Reduce generated output first.
   - Route simple work to cheaper models.
   - Make Opus explicit/escalation-only.
   - Use tighter response formats and review filters.
   - Limit subagent fan-out and long-running sessions.
   - Audit the specific sessions/projects causing spikes.

4. Keep the response concise.
   - Give numbers first, then actions.
   - Avoid broad generic advice when a measured driver is available.
   - Do not paste full command output back to the user.

## Cost Controls

### Output budget

- Default final answers to 150 words unless detail is requested.
- For code changes, report only `Changed`, `Validated`, and `Risks`.
- Do not paste full files, full diffs, or full logs unless explicitly requested.
- Prefer file paths, symbols, and short conceptual summaries over copied code.
- Keep progress updates brief and save detailed findings for final conclusions.

### Model routing

- Use Haiku for triage, summarization, log clustering, issue extraction, and simple classification.
- Use Sonnet for normal coding, debugging, test repair, and code review.
- Use Opus only for ambiguous root cause analysis, architectural tradeoffs, security-sensitive review, or after cheaper models fail.
- If a session becomes Opus-heavy, stop and confirm the escalation is still worth the spend.

### Review and debugging limits

- For code review, report only actionable correctness, security, data-loss, race-condition, and broken-test findings.
- Suppress compliments and low-value style nits.
- For logs, extract failing test names, file paths, line numbers, and the first causal error.
- Avoid repeatedly re-explaining the same root cause after it is established.

### Subagent controls

- Use parallel agents only for independent tasks.
- Give each subagent a strict return format: answer, evidence, confidence, max five bullets.
- Avoid broad "research everything" prompts that cause duplicated summaries.

### Session hygiene

- Start a fresh session after unrelated tasks to avoid carrying excess context.
- Prefer targeted file reads/searches over pasting large context into prompts.
- Keep stable rules short; long always-on rules increase cache creation and can make every turn heavier.

## Copy-Paste Policy

Use this as a standing cost-control rule:

```text
Default to concise responses. Keep final answers under 150 words unless I ask for detail. Do not paste full files, full diffs, or full logs unless requested. For implementation work, report only Changed, Validated, and Risks. Use cheaper models for triage and summarization. Treat Opus as escalation-only for hard root-cause, architecture, or security-sensitive work.
```

Use this for review tasks:

```text
Review only for correctness, security, data loss, race conditions, broken tests, and maintainability issues that can cause real bugs. Suppress compliments, style nits, and restatements of the diff. One concise paragraph per finding.
```

Use this for debugging:

```text
Extract only the failing command, failing test or operation, file paths, line numbers, and first causal error. State the likely root cause only after evidence is collected. Do not include full logs unless I ask.
```

## Report Shape

When presenting analysis, use this structure:

- Trend: cost and token direction over time.
- Drivers: model mix, output/input ratio, high-cost sessions, cache behavior.
- Actions: the smallest set of controls likely to reduce spend.
- Next audit: one or two commands to identify the next bottleneck.

## Common Misreads

- High total token count may be cache reads, not direct expensive generation.
- A low input token count with high cost usually points to output volume or expensive model choice.
- Monthly cost can look lower early in a month; compare run rate, not just absolute spend.
- Broad review, planning, and subagent workflows can cost more than implementation.
- Adding more input does not fix a high output/input ratio; reduce unnecessary output instead.
