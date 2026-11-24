# Policy schema

Read this before writing or editing a policy YAML file. The fields are the same for `default.yaml`, the category files, and the level overlays. `helpers/policy-template.yaml` is the blank starting point, and `references/policy-example.yaml` is a filled example.

```yaml
name: "<policy-name>"
level: standard # open | standard | strict | locked
category: "<category>" # matches agents/cfg/*.cfg.yaml
inherits: default

allowed_tools: [Read, Write, ...] # empty = no restriction
blocked_tools: [Agent, ...] # always denied
blocked_patterns: # regex, any match = deny
  - "(?i)--no-verify"
max_calls_per_dispatch: 200 # rate limit per agent dispatch

require_human_approval: # operations needing user OK
  - cargo publish

subagent: # constraints on sub-agents
  max_concurrent: 5
  must_verify_branch: true
  no_commit_to_main: true
  max_retries_on_failure: 3
  require_commit_before_done: true
  blocked_flags: ["--no-verify"]

audit:
  enabled: true
  format: jsonl
  path: .ctx/godmode/traces/governance-audit.jsonl
  log_allowed: false
  log_denied: true
  log_reviews: true
```
