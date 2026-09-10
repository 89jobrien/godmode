# pipelines/crux/

`.crux` pipeline files, executed by the `crux` CLI (from the sibling `../crux`
checkout's `crux-agentic` crate) via `just crux-*` recipes. Different format
from the sibling `pipelines/*.yaml` skill-orchestration files one level up --
those are godmode's own skill-sequencing format, not crux-script.

## Pipelines

| Pipeline          | Description                                                                              |
| ----------------- | ---------------------------------------------------------------------------------------- |
| `check_refs.crux` | Runs `scripts/check-refs.nu --ci`; fails the pipeline on any broken skill doc reference. |

## Run

```bash
just crux-check-refs
```

The recipe requires a sibling `../crux` source checkout because the `crux` CLI is provided by
that workspace’s `crux-agentic` package. Godmode’s runtime integration instead uses the published
`crux-runtime` crate; that dependency neither supplies the CLI nor requires a local checkout.
