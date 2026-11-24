# Policy layout and category design

Read this before creating, moving, or reviewing a policy file under `policies/`, or before choosing which category an agent belongs to.

### Directory layout

```text
policies/
  default.yaml                # baseline — Standard level
  by-category/
    agent.yaml                # dispatchers (tdd-crate, moa, parallel-agents)
    plan.yaml                 # planners (brainstorm, planner, writing-plans)
    issue.yaml                # issue handlers (cross-issue, triage, tackle)
    qual.yaml                 # quality gates (code-review, health-score, dead-code)
    ops.yaml                  # operations (changelog, dep-audit, dep-bump, orchestrator)
    git.yaml                  # git agents (cap, pr-author, wave-integration)
    refac.yaml                # refactoring (refactoring, doc-maintainer, workspace-refactor)
    dbg.yaml                  # debugging (ci-fix, systematic-debugging, mistake-tracker)
    meta.yaml                 # meta-analysis (pattern-learner)
    gov.yaml                  # governance/observability (observability-as-infrastructure)
  levels/
    open.yaml                 # audit only — local dev
    standard.yaml             # tool allowlist + content filters — default
    strict.yaml               # read-only + human approval — sensitive ops
    locked.yaml               # read-only, no Bash, no Agent — compliance
```

### Category design principles

| Category | Can Write | Can Bash | Can Agent | Rationale                                   |
| -------- | --------- | -------- | --------- | ------------------------------------------- |
| agent    | yes       | yes      | **yes**   | Only category with Agent tool — dispatchers |
| plan     | yes       | no       | no        | Planners propose, not execute               |
| issue    | yes       | yes      | no        | Need gh CLI but no sub-delegation           |
| qual     | reports   | yes      | no        | Analyze and report, not fix                 |
| ops      | yes       | yes      | no        | Broad access, destructive ops gated         |
| git      | yes       | yes      | no        | Git operations, force-push gated            |
| refac    | yes       | yes      | no        | Source edits, tests required after          |
| dbg      | yes       | yes      | no        | Full diagnostic access                      |
| meta     | reports   | no       | no        | Read-only analysis                          |
| gov      | reports   | no       | no        | Observers don't modify                      |
