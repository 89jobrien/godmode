# Workspace finding triage decision tree

Read this before deciding how to handle a workspace rustqual finding, when the code location alone does not settle the fix.

```text
Finding: DEAD_CODE
  └─ Is the function pub?
       ├─ Yes → Is it used by another crate?
       │          ├─ Yes → detect_dead_code = false (or exclude_files)
       │          └─ No  → Real dead code. Remove or add a test.
       └─ No  → Real dead code. Remove.

Finding: TQ_UNTESTED
  └─ Which crate?
       ├─ Binary/CLI → Is it dispatch wiring?
       │                 ├─ Yes → ignore_functions
       │                 └─ No  → Move logic to lib crate + add unit test
       └─ Lib        → Add a unit test. No exception.

Finding: ORPHAN_SUPPRESSION after qual:allow(dry)
  └─ You tried to suppress DEAD_CODE with qual:allow(dry).
     Remove the qual:allow annotation.
     Use detect_dead_code = false instead.
```
