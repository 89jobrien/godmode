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

Requires a sibling `../crux` checkout (godmode depends on `crux-runtime` via a
local path dependency already; this reuses that same checkout to run the CLI).
