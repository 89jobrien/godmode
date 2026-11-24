# Scorecard format and health grade

Read this before writing a health scorecard for the user, including the trend table and the grade criteria.

## Scorecard Format

Markdown table showing current metrics, previous (if available), and trend:

```markdown
## Codebase Health Scorecard

Generated: 2026-06-02 14:35:22 UTC
Workspace: godmode

| Metric          | Current | Previous | Trend |
| --------------- | ------- | -------- | ----- |
| Tests           | 272     | 270      | ↑     |
| Clippy warnings | 0       | 0        | →     |
| TODO density    | 0.5/1k  | 0.6/1k   | ↑     |
| Public API      | 145     | 142      | →     |
| Dependencies    | 98      | 100      | ↑     |
| Avg module      | 180     | 182      | ↑     |
| Doc coverage    | 92.5%   | 91.0%    | ↑     |

### Summary

- **Improving**: 5 metrics
- **Stable**: 2 metrics
- **Degraded**: 0 metrics

### Health Grade: A

Criteria:

- A: test_count ≥ 200, clippy_warnings == 0, doc_coverage > 80%
- B: test_count ≥ 100, no severe clippy issues
- C: otherwise
- Subtract one grade if doc_coverage < 50% or todo_density > 2/1k
```
