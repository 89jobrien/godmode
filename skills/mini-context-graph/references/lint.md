# Wiki Lint Workflow

Run `kgx wiki lint` periodically to health-check the knowledge base.

## Lint Output

```json
{
  "orphan_pages": ["page with no inbound wikilinks"],
  "missing_pages": ["[[linked-but-nonexistent]]"],
  "broken_wikilinks": [{ "page": "source.md", "link": "[[broken]]" }],
  "isolated_pages": ["page with no wikilinks in or out"]
}
```

## Fix Procedure

### Orphan Pages (no inbound links)

1. Read the orphan page.
2. Find related pages that should link to it.
3. Add `[[orphan-name]]` wikilinks in those pages.
4. If truly unrelated to anything, consider merging into a topic page or deleting.

### Missing Pages (linked but don't exist)

1. Check if the entity exists in the graph: `kgx graph search "entity name"`.
2. If it exists in the graph, write the wiki page: `kgx wiki write --category entity`.
3. If it doesn't exist, either ingest a source for it or remove the broken link.

### Broken Wikilinks

1. Check for typos in the link target.
2. Check if the page was renamed — update the link.
3. If the target was deleted intentionally, remove the link.

### Isolated Pages

Pages with no links in or out are candidates for:

- Adding cross-references to related entities.
- Merging into a broader topic page.
- Deletion if the content is stale or superseded.

## Schedule

Run lint after every 5+ ingests, or weekly during active knowledge building.
