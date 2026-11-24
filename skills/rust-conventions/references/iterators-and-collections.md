# Iterators and collections

Read this before writing a loop over a collection, building a chain of transformations, or accumulating a result.

- Use iterators instead of index-based loops — they are often faster, safer, and more
  expressive.
- Avoid premature `.collect()` — keep iterator chains lazy until a collection is actually
  needed.
- Prefer `.filter_map()` over `.filter().map()` when the two operations can be combined.
- Use `.fold()` for accumulation rather than a mutable variable outside the loop.
