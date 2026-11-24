# Phase 3: eval cases and runs

Read this before writing or running eval cases for Phase 3. It holds the eval YAML, the promptfoo run command, and the shell fallback.

## Eval case template

One case per cluster from Phase 2:

```yaml
# .ctx/_WORKING_DIR/evals-<date>.yaml
description: "<cluster.id> — <cluster.diagnosis>"
providers:
  - id: exec
    config:
      command: "echo '<sample_command>' | coursers pre --profile test"

tests:
  - description: "should block <pattern>"
    assert:
      - type: javascript
        value: "output.includes('deny')"

  - description: "should NOT block <exception>"
    assert:
      - type: javascript
        value: "!output.includes('deny')"
```

## Run evals

```nu
promptfoo eval --config .ctx/_WORKING_DIR/evals-<date>.yaml --output .ctx/_WORKING_DIR/eval-results-<date>.json
```

## Fallback when promptfoo is not available

```nu
# Smoke test equivalent
echo '{"tool_name":"Bash","tool_input":{"command":"<sample>"}}' | coursers pre
```
