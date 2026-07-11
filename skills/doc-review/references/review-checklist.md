# Doc Review Checklist

## Accuracy

- [ ] Every CLI command shown: run it, confirm output matches documentation
- [ ] Every file path: confirm it exists on disk
- [ ] Every flag/option: confirm it appears in `--help`
- [ ] Every type, function, module name: confirm it exists in source
- [ ] Every code example: confirm it compiles/runs without error
- [ ] No invented behaviour, flags, or return values

## Completeness

- [ ] Install/setup: everything needed to get started is present
- [ ] CLI reference: all subcommands and their flags are documented
- [ ] Architecture doc: all major components have ownership described
- [ ] API reference: every public export has purpose, parameters, return, and example
- [ ] No section left with only placeholder text ("TODO", "TBD", "coming soon")

## Clarity

- [ ] Entry point (quickstart, install, or overview) is the first thing visible
- [ ] Every term is defined before it is used, or links to a definition
- [ ] Examples are concrete — real commands, real output, not pseudocode
- [ ] Tables used where items have multiple attributes (not nested bullet lists)
- [ ] Active voice: "run X" not "X can be run"

## Navigability

- [ ] Headings exist at the right granularity (not every paragraph, not absent)
- [ ] Docs > 200 lines: table of contents present
- [ ] All cross-reference links resolve (internal anchors and external URLs)
- [ ] Code blocks use correct language hints (` ```bash `, ` ```rust `, ` ```yaml `)

## Verdict

| Finding count | Verdict |
| ------------- | ------- |
| 0 Blocking    | PASS    |
| ≥1 Blocking   | FAIL    |

Suggestions and nitpicks do not block PASS but must be recorded.
