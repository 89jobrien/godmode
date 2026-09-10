# Architecture

## Data flow

```text
$HOME/.claude/projects/<project-slug>/<session-id>.jsonl
                         |
                         v
                    harvest.rs
  - scans Claude Code JSONL session files for the selected date
  - reads top-level sessionId, timestamp, cwd, gitBranch, and slug fields
  - extracts user text from human message content blocks
  - extracts assistant text and tool_use names from assistant content blocks
  - filters empty, confirmation-only, and injected system turns
  - emits a JSON array of normalized session records
                         |
                         v
                    analyze.rs
  - reads the harvested JSON and builds a bounded transcript
  - loads prompts/analysis.whatidid.txt
  - calls https://api.anthropic.com/v1/messages with claude-haiku-4-5-20251001
  - validates the response against the Rust Digest/Goal/Task structs
  - caches JSON under $HOME/.cache/whatidid/YYYY-MM-DD.json
  - optionally encrypts the cache with AES-256-GCM when WHATIDID_CACHE_KEY is set
                         |
                         v
                    report.rs
  - loads model_pricing.json and reports its SHA-256 hash
  - computes cost and effort summaries
  - renders an HTML report and optionally opens it locally
```

`whatidid.rs` is the Rust orchestrator. It invokes each helper through
`rust-script`, passing harvest and digest output through date-scoped files in
`/tmp`. `ANTHROPIC_API_KEY` is required only when analysis is not satisfied by
an existing cache entry.

## Claude Code JSONL input

Claude Code stores one JSON object per line beneath
`$HOME/.claude/projects/`. The harvester handles malformed lines by skipping
them and recognizes these top-level event types:

| `type`      | Fields consumed                                     |
| ----------- | --------------------------------------------------- |
| `human`     | `message.content[]` text blocks                     |
| `assistant` | `message.content[]` text and `tool_use.name` blocks |

Session identity and context come from top-level `sessionId`, `timestamp`,
`cwd`, `gitBranch`, and `slug`. There is no Copilot `workspace.yaml`,
`session.shutdown`, premium-request counter, or Outlook/PowerShell stage in
this implementation.

The normalized session record includes message turns and counts for all tools,
reads, edits/writes, and Bash calls. `model_metrics` is present in the wire
shape for report compatibility; the current Claude JSONL harvester does not
populate token usage from assistant events.

## Anthropic analysis

`analyze.rs` sends one user message containing the analysis prompt and the
normalized transcript. It strips optional Markdown JSON fences, deserializes
the response into Rust types, and rejects invalid model output before caching.
The request uses Anthropic's native headers (`x-api-key` and
`anthropic-version`), not GitHub Models or an OpenAI-compatible endpoint.

Cache files can contain activity details. Set `WHATIDID_CACHE_KEY` to encrypt
new cache entries. Existing plaintext cache entries remain readable with a
warning for migration.

## Pricing and leverage

Pricing is loaded at runtime from `skills/whatidid/model_pricing.json`; the
fallback rates in that file apply when no model prefix matches. Update that
JSON file rather than embedding rates in documentation.

```text
human_value = total_human_hours * $72/hour
leverage    = human_value / $39 monthly seat cost
```
