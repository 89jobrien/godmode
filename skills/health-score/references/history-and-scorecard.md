# History file format

Read this before appending to the health history file. The scorecard format and health grade are in scorecard-format-and-grade.md.

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
