# Limitations of the health metrics

Read this before presenting a health score as a judgment of quality, or before comparing scores across repos.

## Limitations

- **Approximate counts**: Line counts and pattern matches are heuristic-based, not
  exact AST analysis.
- **Not a replacement for coverage tools**: Use `cargo-tarpaulin` or `cargo-llvm-cov`
  for precise code coverage metrics.
- **Clippy is not exhaustive**: Only counts printed warnings; does not assess overall
  code quality.
- **API surface ≠ API complexity**: Counts items, not API design quality or backward
  compatibility.
- **TODO density is contextual**: Some repos legitimately have more TODOs than others
  depending on development stage.
- **Module size varies by domain**: 180 lines average may be appropriate for one crate
  but high for another.

Use these metrics for **trend detection** and **relative comparison**, not absolute
quality judgment.
