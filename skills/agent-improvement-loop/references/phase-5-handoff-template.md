# Phase 5: handoff document template

Read this before writing the Codex handoff in Phase 5. Fill the template from the ranked change list in `diagnosis.json`.

```markdown
# Agent Improvement Handoff — <date>

## Context

<one paragraph: what agent, what loop iteration, what source of traces>

## Changes (ranked by HALO score)

### Change 1: <cluster_id>

- **File**: `<target_file>`
- **Action**: <add rule | update threshold | fix message | add exception>
- **Evidence**: <evidence_count> occurrences in last 30 days
- **Eval gate**: `<eval_id>` must pass after change
- **Spec**:
  <exact JSON/TOML/code block to add or modify>

### Change 2: ...
```
