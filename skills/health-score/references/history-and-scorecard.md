# History file and scorecard format

Read this before appending to the health history file, or before writing a scorecard for the user.

## History Format

File: `.ctx/godmode/memory-bank/health-history.jsonl`

Each line is a valid JSON object with timestamp and all seven metrics:

```json
{
  "timestamp": "2026-06-02T14:35:22Z",
  "test_count": 272,
  "clippy_warnings": 0,
  "todo_density": 0.5,
  "api_surface": 145,
  "dependency_count": 98,
  "avg_module_lines": 180,
  "doc_coverage": 92.5
}
```

- One measurement per line
- Timestamp is ISO 8601 UTC
- All numeric fields are floats or integers as appropriate
- New measurements are appended to the end (immutable log)
- File is created automatically on first write if missing

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
