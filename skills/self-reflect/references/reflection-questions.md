# Reflection Questions

Answer these based on evidence (commits, diffs, task state), not memory:

1. **What took longer than expected?**
   - Look for repeated edits to the same file.
   - Multiple fix-up commits on one topic.
   - Tasks that went from Running back to Blocked.

2. **What went smoothly?**
   - First-pass successes (single commit, no fixups).
   - Clean test runs on first attempt.
   - No hook failures or CI issues.

3. **What was discovered that was not in the original plan?**
   - TODOs added during the session.
   - Unexpected coupling found between modules.
   - Agents that surfaced issues not on the task graph.

4. **What would speed up the next session?**
   - Missing tools or scripts.
   - Unclear specs that required investigation.
   - Slow feedback loops (CI, builds, test runs).
   - Friction points in the workflow.

## Guardrails

- Do not modify source code during reflection — read-only.
- Do not rewrite history to look cleaner.
- Empty "Shipped" is honest and useful if nothing landed.
- Never write deferred placeholders like "(fill in later)".
