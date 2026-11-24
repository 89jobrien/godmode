# Phase 7: validation commands

Read this before running the validation gates in Phase 7. Run all four before committing. The branch check and commit steps are in SKILL.md.

```nu
# 1. Run the evals that were failing in Phase 3
promptfoo eval --config .ctx/_WORKING_DIR/evals-<date>.yaml

# 2. Run the full rule health check
crs validate [--profile <profile>]

# 3. Run workspace tests
taskit test run

# 4. Smoke test the hook pipeline
echo '{"tool_name":"Bash","tool_input":{"command":"<sample>"}}' | coursers pre [--profile <profile>]
```
