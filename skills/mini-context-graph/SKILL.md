---
name: "godmode:mini-context-graph"
description: >
  A persistent, compounding knowledge base combining Karpathy's LLM Wiki pattern with a
  structured knowledge graph. Ingest documents once — the LLM writes wiki pages, extracts
  entities and relations into the graph, and stores raw content for evidence retrieval.
  Knowledge accumulates and cross-references across sessions; it is never re-derived from
  scratch. Use when building or querying a knowledge base where provenance, structural
  traversal, and cumulative synthesis matter.
requires: []
next: []
---

# Mini Context Graph

Standard RAG re-discovers knowledge from scratch on every query. This skill is different:

1. **Wiki layer** — Persistent markdown pages (summaries, entity pages, topic syntheses).
   Cross-references are already there. The wiki gets richer with every ingest.
2. **Graph layer** — Entities and relations are extracted once and stored as a navigable
   knowledge graph. BFS traversal answers structural queries without re-reading sources.
3. **Raw source layer** — Original documents stored immutably with chunks. Provenance links
   tie every graph node and edge back to the exact text that supports it.

The LLM writes; `kgx` handles all bookkeeping.

---

## Three Layers

| Layer           | Where                      | What the LLM does             | What kgx does                     |
| --------------- | -------------------------- | ----------------------------- | --------------------------------- |
| **Raw Sources** | `.kgx/data/documents.json` | Reads (never modifies)        | Stores chunks + metadata          |
| **Wiki**        | `.kgx/wiki/` (markdown)    | Writes/updates pages          | Manages index.md + log.md         |
| **Graph**       | `.kgx/data/graph.json`     | Extracts entities + relations | Persists, deduplicates, traverses |

---

## Setup

Install the `kgx` CLI from the kgx workspace:

```bash
cargo install --path ~/dev/kgx/crates/kgx-cli
```

All commands use `--root <dir>` to set the data directory (default: `.kgx` in cwd).

---

## Ingest

When a new document arrives:

1. Read `references/ingestion.md` — entity/relation extraction rules.
2. Read `references/ontology.md` — type normalization rules.
3. Extract entities and relations via LLM reasoning.
4. Pipe a JSON payload to `kgx ingest` — stores raw content, chunks, graph nodes,
   provenance.
5. Write a wiki summary page via `kgx wiki write`.
6. Update entity pages — for each new or updated entity, write or update via
   `kgx wiki write --category entity`.
7. Update topic pages if the document touches an existing synthesis topic.

A single document ingest typically touches 3-10 wiki pages.

```bash
# Ingest a document with entities and relations
'{"doc_id":"doc_001","title":"System Crash Analysis","source":"/docs/incident_report.pdf","raw_content":"System crashes due to memory leaks. Memory leaks occur when objects are not released.","entities":[{"name":"memory leak","type":"issue","supporting_text":"memory leaks cause crashes"},{"name":"system crash","type":"issue","supporting_text":"system crashes due to memory leaks"}],"relations":[{"source":"memory leak","target":"system crash","type":"causes","confidence":1.0,"supporting_text":"System crashes due to memory leaks."}]}' | kgx ingest
# => {"doc_id": "doc_001", "chunk_count": 1, "nodes_added": 2, "edges_added": 1}

# Write a wiki summary page (content from a file)
# Write the markdown to a temp file, then pipe it:
open /tmp/wiki-page.md | kgx wiki write --category summary --title "System Crash Analysis Summary" \
  --summary "Incident report: memory leaks cause system crashes."
```

---

## Query

When answering a question:

1. Check the wiki first — `kgx wiki search <query>` to find relevant pages. Read them.
2. If the wiki has a good answer, synthesize from wiki pages (fast path).
3. If deeper graph traversal is needed, call `kgx query <entity-name>`.
4. Return the answer with evidence citations from `supporting_chunks`.
5. If the answer produces new synthesis, file it back as a wiki topic page.

```bash
# Fast path — search wiki
kgx wiki search "memory leak"
# => [{"slug": "...", "category": "summary", "path": "...", "snippet": "..."}]

# Full path with provenance — BFS from a seed entity
kgx query "memory leak"
# => {"query": "...", "nodes": [...], "edges": [...], "supporting_chunks": [...]}
```

---

## Lint

Periodically health-check the wiki:

```bash
kgx wiki lint
# => {"orphan_pages": [...], "missing_pages": [...], "broken_wikilinks": [...], "isolated_pages": [...]}
```

Review and fix: broken links, orphan pages, stale claims, missing cross-references.
See `references/lint.md` for the full lint workflow.

---

## Ingestion Rules

- Do NOT add entities not explicitly present in the source text
- Do NOT add relations without direct textual evidence
- Do NOT add edges with confidence below 0.6
- Provide `supporting_text` for every entity and relation — this enables provenance
- Write a wiki summary page for every ingested document
- Update existing entity pages when new information arrives
- Flag contradictions in wiki pages when new data conflicts with old claims

---

## Retrieval Constraints

- Traversal depth must not exceed 2 (`MAX_GRAPH_DEPTH`)
- Only edges with confidence >= 0.6 (`MIN_CONFIDENCE`)
- Maximum 50 nodes returned (`MAX_NODES`)
- Do NOT fabricate nodes or edges not present in the graph

---

## CLI Reference

All commands output JSON to stdout. Use `--root <dir>` to override the data directory.

| Command                                                           | Purpose                                    | When to use                          |
| ----------------------------------------------------------------- | ------------------------------------------ | ------------------------------------ |
| `kgx ingest` (stdin JSON)                                         | Full ingest: raw docs + graph + provenance | Every new document                   |
| `kgx query <name>`                                                | BFS subgraph + provenance chunks           | Queries requiring citations          |
| `kgx graph add-node <name> --entity-type <t>`                     | Add single entity                          | Quick additions without a source doc |
| `kgx graph add-edge <src> <tgt> --relation-type <t>`              | Add single relation                        | Quick additions without a source doc |
| `kgx graph search <query>`                                        | Search nodes by keyword                    | Finding entities                     |
| `kgx wiki write --category <c> --title <t> --summary <s>` (stdin) | Write/update a wiki page                   | After every ingest; after queries    |
| `kgx wiki read --category <c> --title <t>`                        | Read a wiki page                           | Before answering; cross-referencing  |
| `kgx wiki search <query>`                                         | Keyword search across wiki                 | Fast path before graph traversal     |
| `kgx wiki list --category <c>`                                    | List all wiki pages                        | Getting an overview                  |
| `kgx wiki lint`                                                   | Health check                               | Periodic maintenance                 |
| `kgx docs list`                                                   | List all ingested raw sources              | Audit / provenance checking          |
| `kgx docs search <query>`                                         | Chunk-level search                         | Finding specific evidence            |
| `kgx export --format json --output <dir>`                         | Export full context as JSON                | Snapshots, backups, data exchange    |
| `kgx export --format markdown --output <dir>`                     | Export as Obsidian-compatible vault        | Human review, Obsidian integration   |
| `kgx stats`                                                       | Node/edge/document counts                  | Quick overview                       |

---

## Responsibility Split

| Layer               | What happens                             | Owned by                            |
| ------------------- | ---------------------------------------- | ----------------------------------- |
| **LLM Reasoning**   | Extraction, synthesis, wiki writing      | Agent (this skill + reference docs) |
| **All Persistence** | Graph, wiki, documents, dedup, traversal | `kgx` CLI (Rust)                    |

The human curates sources and asks questions. The LLM writes the wiki, extracts the
graph, and answers with citations. `kgx` handles all bookkeeping.

---

## Related

- `skills/context-map/SKILL.md` — pre-implementation codebase mapping (same evidence-first
  pattern)
- `skills/doublecheck/SKILL.md` — verify claims extracted during ingest before committing them
- `references/ingestion.md` — entity/relation extraction rules (read before every ingest)
- `references/ontology.md` — type normalization rules
- `references/lint.md` — wiki health-check workflow
- Source: https://github.com/89jobrien/kgx
