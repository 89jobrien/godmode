# Governance Levels

| Level        | Controls                                        | Use Case                     |
| ------------ | ----------------------------------------------- | ---------------------------- |
| **Open**     | Audit only, no restrictions                     | Internal dev/testing         |
| **Standard** | Tool allowlist + content filters                | General production agents    |
| **Strict**   | All controls + human approval for sensitive ops | Financial, healthcare, legal |
| **Locked**   | Allowlist only, no dynamic tools, full audit    | Compliance-critical systems  |

## Choosing a Level

- Start at **Standard** for any agent with tool access.
- Escalate to **Strict** when the agent handles PII, financial data, or infrastructure.
- Use **Locked** for regulatory environments (SOC2, HIPAA, PCI-DSS).
- **Open** is for local dev only — never deploy Open to production.

## Composition Rules

When multiple policies are composed:

- Blocked lists: union (most restrictive wins)
- Allowed lists: intersection (narrowest scope wins)
- Rate limits: minimum (lowest wins)
- Human approval: union (any policy requiring review wins)
