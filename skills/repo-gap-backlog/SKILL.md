---
name: repo-gap-backlog
description: Use when a local project exists but does not yet have a GitHub repo or issue backlog, and you want to turn a completion review into concrete GitHub issues. Symptoms - local code is ahead of repo setup, there is no README/spec, or you need to answer "is this done?" and capture the gaps as issues.
---

# Repo Gap Backlog

## When to Use

Use this when you have a local repo or workspace that needs a strict "done or not done" review, and you want the missing work captured as GitHub issues immediately.
It is especially useful when the GitHub repo may not exist yet, or when the local project name does not match an existing published repo.

## Commands

```bash
# 1. Inspect the local workspace for completion signals
ls -la
# Use Glob tool: glob './**/.git' (maxdepth 2 equivalent)
# Use Glob tool: glob './**/README*', './**/HANDOFF*', './**/TODO*'
rg -n "TODO|FIXME|WIP|TBD|XXX|not implemented|unimplemented!|panic!\(|todo!\(" . --glob '!target' --glob '!.git'

# 2. Read the product surface and verify the code actually works
cargo test
./target/debug/<bin-name> --help
./target/debug/<bin-name> <subcommand> --help

# 3. Check whether a GitHub repo already exists
gh auth status
gh repo view <owner>/<repo>
gh repo list <owner> --limit 200

# 4. If missing, create the repo
gh repo create <owner>/<repo> --public --description "<one-line description>"
# or
gh repo create <owner>/<repo> --private --description "<one-line description>"

# 5. File one issue per concrete product gap
gh issue create -R <owner>/<repo> \
  --title 'Document the product contract and usage' \
  --body $'Current gap:\n- no README\n- no install instructions\n\nDone when:\n- repo docs define the supported contract'

gh issue create -R <owner>/<repo> \
  --title 'Add end-to-end integration tests for the CLI workflow' \
  --body $'Current gap:\n- unit coverage exists but workflow coverage is thin\n\nDone when:\n- integration tests cover the real user flow'
```

## Common Failures

| Symptom                                                       | Fix                                                                                                                     |
| ------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `gh repo view` fails with "Could not resolve to a Repository" | Confirm the repo slug; it may not exist yet or may be under a different name                                            |
| `error connecting to api.github.com`                          | Re-run with escalated network permissions                                                                               |
| zsh interprets issue body text as commands                    | Use single quotes or `$'...'` bodies instead of unescaped markdown-ish text                                             |
| Local workspace is not a git repo root                        | Treat it as a workspace, inspect member crates/packages, and avoid assuming repo-level status                           |
| Project appears "green" because tests pass                    | Separate "builds/tests" from "done"; inspect docs, integration depth, data model completeness, and user-facing contract |
