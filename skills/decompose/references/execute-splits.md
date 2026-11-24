# Executing the splits

Read this after the user has approved the plan, before running `helpers/split-branch.nu` or the coverage check.

## Step 4 — Execute the splits

Run the splitter for each approved split:

```bash
nu skills/decompose/helpers/split-branch.nu \
  --source <source-branch> \
  --base main \
  --split-id 1 \
  --branch <split-branch-name> \
  --files "path/a.rs path/b.rs ..." \
  --output-dir .ctx/godmode/decomps/<decomp-name>
```

The splitter:

1. Checks out `main` (or base)
2. Creates the split branch
3. Cherry-picks by file: `git checkout <source-branch> -- <files>`
4. Runs `taskit check compile`
5. Runs `taskit test run` (or scoped to affected crates)
6. If checks pass: commits with a conventional message
7. If checks fail: reports failure, rolls back the branch, stops — does not continue to next split

Process splits sequentially, not in parallel. A failing split must be resolved before continuing.

After all splits pass:

```bash
# Verify no file was dropped — writes result to .ctx/godmode/decomps/
nu skills/decompose/helpers/analyze-diff.nu --verify-coverage \
  --source <source-branch> \
  --splits "split-1-branch split-2-branch ..." \
  | save --force .ctx/godmode/decomps/<decomp-name>/coverage.json
```

Coverage check: the union of all split files must equal the source branch file list. Report any
files that appear in the source but no split (orphaned files).
