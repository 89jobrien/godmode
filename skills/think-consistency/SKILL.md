---
name: think-consistency
description:
  Self-Consistency reasoning — run multiple independent reasoning paths and
  take consensus. Use for high-stakes decisions where reasoning errors are costly.
  Trigger this skill when the user says "think-consistency", "self-consistency",
  "run consistency check", or when the user faces a complex decision where getting
  the reasoning wrong would be expensive. Also trigger when the user asks Claude to
  reason from multiple angles before committing to an answer, or when they want
  independent reasoning paths to vote on the best answer.
allowed-tools: Read, Bash
---

# Self-Consistency

## Start Session

Before beginning, register this think session:

```nu
# Check for active session
if ('/tmp/.claude-think-session' | path exists) {
  open /tmp/.claude-think-session
}
```

If another session is active, ask the user to finish or abandon it first. Otherwise:

```nu
'self-consistency' | save --force /tmp/.claude-think-session
```

## Work Through Stages

Read the strategy definition and follow its stages in order:

```nu
open ($env.SKILL_PATH | path join 'references/self-consistency.md')
```

If `$env.SKILL_PATH` is unavailable, fall back to the bundled path relative to this
file, or read `references/self-consistency.md` directly via the Read tool.

Work through each numbered stage sequentially. Do not skip stages. Present your
work for each stage before moving to the next.

## Complete Session

After the final stage, clean up so the think way can fire again:

```nu
ls /tmp | where name =~ '\.claude-(think-session|way-meta-think-)' | each { |f| rm $f.name }
```
