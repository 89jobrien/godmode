# Governance audit trail

Read this before reading, aggregating, or changing the governance audit log, or before changing what gets logged under `audit:` in a policy.

All governance decisions are logged to
`.ctx/godmode/traces/governance-audit.jsonl`:

```json
{
  "ts": "2026-06-04T14:30:00+0000",
  "event": "governance.check",
  "action": "denied",
  "agent_id": "gm-dispatch",
  "tool_name": "Agent",
  "reason": "content matches blocked pattern: (?i)--no-verify",
  "pattern": "(?i)--no-verify",
  "session_id": "abc1234-1717500000000"
}
```

Use `audit-report.nu` to aggregate and summarize.
