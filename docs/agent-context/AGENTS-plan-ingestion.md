# Plan ingestion format

Read this before ingesting plan markdown into the task graph.

Plan markdown must use `### Task N: <title>` headings. Optionally annotate with:

```markdown
**Crate**: `crate-name`
**Run**: `cargo nextest run -p crate-name`
```

`plan::parse` builds sequential `depends_on` chains automatically. The CLI prepares parsed tasks
against the current graph: the first plan keeps `tN` IDs, later collisions use a deterministic
file-stem namespace, and provenance keeps re-ingestion idempotent. The recorded path is one signal,
not the whole rule. A plan whose recorded path no longer exists on disk is matched as relocated
when its file stem is unambiguous and its recorded tasks are still in the graph with matching title,
crate, and run. That match keeps the existing task IDs and rebinds the provenance entry to the new
path, so re-ingestion is idempotent after a move. Two plan files that share a stem and both still
exist remain distinct plans. Prefix an explicit existing-graph dependency with `graph:` so it is not
remapped with internal plan dependencies.
