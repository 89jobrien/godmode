---
description: 'If any gh or external CLI command (gh, jira, linear) fails with an auth or permission error:'
subtask: false
---
## Rules

- Retry recoverable failures up to 3 times with a distinct corrective action.
- After 3 failed attempts, write `BLOCKED.md` with the attempts and root cause,
  then stop that task.
- Authentication failures are non-retriable: log them to
  `.ctx/pending-manual.txt`, include a manual URL when one exists, and continue
  only with independent work.


If any gh or external CLI command (gh, jira, linear) fails with an auth or permission error:

1. Do NOT retry the command.
2. Print the manual URL and exact steps the user needs to complete the action themselves.
   - For gh: provide the GitHub web URL for the PR, issue, or action.
   - For jira: provide the Jira board URL.
   - For linear: provide the Linear team URL.
3. Log the failed action to `.ctx/pending-manual.txt` with format:
   `[TIMESTAMP] FAILED: <command> — manual URL: <url>`
4. Move on to the next task immediately.

This command is a reminder — invoke it mentally before any gh/jira/linear operation.
If those commands succeed, ignore this prompt entirely.
