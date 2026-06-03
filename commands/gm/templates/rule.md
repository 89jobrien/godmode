## Rules

- Log failed actions to `.ctx/pending-manual.txt` with format:
  `[TIMESTAMP] FAILED: <command> — manual URL: <url>`
- Move on to the next task immediately after logging. Do not retry.
- Provide the manual URL and exact steps the user needs.
