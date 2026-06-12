# Handler Catalog

Complete reference for all built-in crux pipeline handlers.

## Shell

| Handler          | Behavior                          | Fails on non-zero? |
| ---------------- | --------------------------------- | ------------------ |
| `shell::exec`    | Run command, return stdout/stderr | No                 |
| `shell::capture` | Run command, return stdout/stderr | Yes                |

Output: `{ "exit_code": 0, "stdout": "...", "stderr": "" }`

## Filesystem

| Handler      | Args              | Output                  |
| ------------ | ----------------- | ----------------------- |
| `fs::read`   | `path`            | File contents as string |
| `fs::write`  | `path`, `content` | Success confirmation    |
| `fs::glob`   | `pattern`         | Array of matching paths |
| `fs::exists` | `path`            | Boolean                 |

## Git

| Handler             | Args       | Output                          |
| ------------------- | ---------- | ------------------------------- |
| `git::staged_files` | —          | `git diff --cached --name-only` |
| `git::diff`         | `revision` | `git diff [revision]`           |
| `git::log`          | `count`    | `git log -N --format=%H\t%s`    |
| `git::status`       | —          | `git status --porcelain`        |

## JSON

| Handler                 | Args            | Output                            |
| ----------------------- | --------------- | --------------------------------- |
| `json::pick`            | field names     | Extract named fields from input   |
| `json::merge`           | `with` (object) | Merge static object into input    |
| `json::jq`              | dot-path        | Dot-path traversal (not full jq)  |
| `json::group_by`        | `key`           | Group array items by field        |
| `json::filter_nonempty` | `field`         | Filter items with non-empty field |

## Text Parsing

| Handler                   | Input              | Output                      |
| ------------------------- | ------------------ | --------------------------- |
| `text::parse_vimgrep`     | rg --vimgrep lines | `[{file, line, col, text}]` |
| `text::parse_jsonl`       | NDJSON text        | Array (skips invalid lines) |
| `text::parse_frontmatter` | Markdown text      | YAML frontmatter object     |
| `text::parse_diff`        | Unified diff       | `[{file, hunks}]`           |
| `text::parse_branch_list` | git branch output  | `[{name, current}]`         |

## Control

| Handler        | Behavior                                  |
| -------------- | ----------------------------------------- |
| `ctrl::noop`   | Pass input through unchanged              |
| `ctrl::log`    | Log to stderr and pass through            |
| `ctrl::assert` | Assert `args.condition` is truthy or fail |

## LLM

| Handler          | Behavior                                     | Feature flag |
| ---------------- | -------------------------------------------- | ------------ |
| `llm::invoke`    | Raw LLM completion (OpenAI/Anthropic/Ollama) | —            |
| `llm::extract`   | BAML structured extraction                   | `baml`       |
| `llm::decompose` | BAML spec decomposition into task list       | `baml`       |
| `llm::plan`      | BAML pipeline generation from goal           | `baml`       |

## Container / Harness

| Handler           | Behavior                                           |
| ----------------- | -------------------------------------------------- |
| `container::run`  | Start container from HarnessProfile                |
| `container::wait` | Block until container exits, emit exit code + logs |
| `harness::evolve` | Run EvolutionPlanner against RunMetrics            |
| `harness::canary` | Deploy canary image (`traffic_percent` arg)        |

## Rx (Script Registry)

| Handler    | Args                    | Behavior                    |
| ---------- | ----------------------- | --------------------------- |
| `rx::run`  | `name`, optional `args` | Run script from rx registry |
| `rx::list` | optional `registry`     | List all rx commands        |

## Analysis

Trace analysis for completed agent runs.

| Handler                        | Behavior                                         |
| ------------------------------ | ------------------------------------------------ |
| `analysis::latency_profile`    | Flag steps exceeding 2x median duration          |
| `analysis::token_spend`        | Token counts per step, top-3 consumers           |
| `analysis::failure_clusters`   | Group failed steps by CruxErr kind               |
| `analysis::replay_cache_hits`  | ReplayCache hit/miss ratio per step name         |
| `analysis::tighten_budget`     | Suggest tighter Budget if spend > 80%            |
| `analysis::compress_stages`    | Flag pipe stages consuming > 40% of tokens       |
| `analysis::tune_retry`         | Suggest retry config for steps with > 2 failures |
| `analysis::patch_schema_check` | Validate YAML patch string                       |
| `analysis::replay_dry_run`     | Re-run trace in lenient replay against a patch   |

## CI

| Handler                 | Behavior                             |
| ----------------------- | ------------------------------------ |
| `ci::compile_errors`    | Parse rustc errors from CI logs      |
| `ci::clippy_violations` | Parse clippy warnings                |
| `ci::nextest_failures`  | Parse nextest FAIL lines             |
| `ci::deny_violations`   | Parse cargo-deny violations          |
| `ci::deduplicate_spans` | Collapse findings sharing file:line  |
| `ci::classify_severity` | Rank: compile > deny > test > clippy |
| `ci::attach_owners`     | Map file paths to crate names        |
| `ci::score_fixability`  | Auto-fix score as confidence         |

## Review

| Handler                       | Behavior                                    |
| ----------------------------- | ------------------------------------------- |
| `review::arch_boundary_check` | Detect domain->adapter imports              |
| `review::normalize_findings`  | Merge clippy, arch, coverage into Finding[] |
| `review::apply_severity`      | Classify: blocking/suggestion/observation   |
| `review::compute_score`       | Reduce findings to 0.0-1.0 confidence       |
| `review::approve`             | Run `gh pr review --approve`                |

## Triage

| Handler                      | Behavior                                  |
| ---------------------------- | ----------------------------------------- |
| `triage::parse_repo_tags`    | Extract repo field from todo metadata     |
| `triage::score_urgency`      | Score by age \* priority, sort descending |
| `triage::deduplicate_intent` | Cluster similar titles via edit distance  |
| `triage::group_by_repo`      | Partition todos into per-repo buckets     |
