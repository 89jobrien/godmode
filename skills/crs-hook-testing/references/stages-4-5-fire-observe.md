# Stage 4: Fire (pre and post hooks)

Read this before firing a rule through the real pre or post hook chain. Checking the rule activity afterwards is in stage-5-observe.md.

## Stage 4: Fire (pre/post)

Exercise the rule through the real hook chain:

**Pre-tool (blocking):**

```bash
echo '{"tool":"Bash","input":{"command":"<command-to-test>"}}' | coursers pre
```

Exit 0 = allowed. Exit 2 = blocked (rule fired). The response JSON is printed.

**Post-tool (failure learning):**

```bash
echo '{"tool":"Bash","input":{"command":"<cmd>"},"output":{"exit_code":1}}' | coursers post
```

Records the failure to the rolling state log. After `failure_threshold` hits, `crs pre`
will block the command even without a pattern match.
