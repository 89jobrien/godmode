# Phase 4: HALO scoring and change list

Read this before scoring failing eval clusters or writing the ranked change list in Phase 4. It holds the score formula and the output shape.

## Score formula

```text
impact    = evidence_count × severity_weight  (P1=3, P2=2, P3=1)
confidence = promptfoo pass rate on existing cases (0.0–1.0)
effort     = estimated lines of change (1=<5, 2=5-20, 3=>20)
halo_score = (impact × confidence) / effort
```

## Change list

```json
{
  "changes": [
    {
      "rank": 1,
      "cluster_id": "no-sleep-in-codex",
      "type": "missing-rule",
      "halo_score": 4.2,
      "action": "add rule to ~/.config/coursers/profiles/codex/rules.json",
      "target_file": "~/.config/coursers/profiles/codex/rules.json",
      "evidence_count": 14,
      "eval_ids": ["eval-003", "eval-007"]
    }
  ]
}
```
