## Rules

- Retry recoverable failures up to 3 times with a distinct corrective action.
- After 3 failed attempts, write `BLOCKED.md` with the attempts and root cause,
  then stop that task.
- Authentication failures are non-retriable: log them to
  `.ctx/pending-manual.txt`, include a manual URL when one exists, and continue
  only with independent work.
