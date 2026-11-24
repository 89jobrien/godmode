# Phases 1-2: Reproduce and locate the root cause

Read this before confirming a bug reproduces (Phase 1), or before tracing a failure back to its root cause (Phase 2).

## Phase 1 — Reproduce

Run the loop. Watch the bug appear. Then confirm:

- The loop produces the failure mode the **user** described, not a different failure that
  happens to be nearby. Wrong bug = wrong fix.
- It reproduces across multiple runs, or at a high enough rate to debug against.
- You captured the exact symptom (error message, wrong output, slow timing) so later phases
  can verify the fix actually addresses it.

Record it in `helpers/repro-template.md`. For Rust, `helpers/debug-session.nu` automates
the git + nextest capture; pass the crate and an optional test name as arguments.

## Phase 2 — Locate the root cause

1. **Parse the error exactly.** Read the full message, stack trace, and surrounding context.
   Do not skim.
2. **Check environment first.** Missing env vars and unresolved `op://` refs are the most
   common root cause. Verify before investigating code.
3. **Check recent changes.**

   ```bash
   git diff HEAD~1
   git log --oneline -10
   ```

4. **For multi-component failures**: add diagnostic output at each boundary to locate where
   the failure originates. Trace data flow backward from the symptom.
5. **Find a working analog** in the codebase — similar code that does work. Compare working
   vs. broken line by line. Document every difference, however insignificant it looks.
