# Ingestion Rules

## Entity Extraction

- Extract only entities explicitly named in the source text.
- Do NOT infer entities not directly mentioned.
- Each entity must have a `type` from the ontology (see `ontology.md`).
- Each entity must have `supporting_text` — the exact quote or paraphrase.

## Relation Extraction

- Only extract relations with direct textual evidence.
- Minimum confidence: 0.6 (skip weaker signals).
- Each relation must have `supporting_text`.
- Prefer specific relation types over generic "related_to".

## Document Processing

1. Read the full document.
2. Identify all named entities (people, systems, concepts, issues, tools).
3. Identify relations between entities (causes, depends_on, implements, etc.).
4. Assign confidence scores based on directness of evidence.
5. Pipe the structured payload to `kgx ingest`.

## Payload Schema

```json
{
  "doc_id": "unique_doc_id",
  "title": "Document Title",
  "source": "/path/to/source.md",
  "raw_content": "full text of the document",
  "entities": [
    {
      "name": "entity name (lowercase, normalized)",
      "type": "ontology type",
      "supporting_text": "evidence from source"
    }
  ],
  "relations": [
    {
      "source": "entity name",
      "target": "entity name",
      "type": "relation type",
      "confidence": 0.9,
      "supporting_text": "evidence from source"
    }
  ]
}
```

## Post-Ingest

After `kgx ingest` succeeds:

1. Write a wiki summary page via `kgx wiki write --category summary`.
2. Write or update entity pages for each new entity.
3. Update topic pages if the document touches existing synthesis topics.
