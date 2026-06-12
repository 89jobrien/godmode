# rustqual Fix Patterns

Concrete before/after examples for each finding type.

## IOSP VIOLATION: Extract I/O from logic

**Before** (logic + calls in one function):

```rust
pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
    let path = path.into();
    let data = if path.exists() {           // logic
        let raw = fs::read_to_string(&path)?; // call
        serde_json::from_str(&raw)?           // call
    } else {
        Data::default()                       // logic
    };
    Ok(Self::from_data(path, data))
}
```

**After** (separate I/O helper + pure constructor):

```rust
// qual:allow(iosp) reason: "I/O boundary — existence check + read"
fn read_data(path: &Path) -> Result<Data> {
    if path.exists() {
        let raw = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    } else {
        Ok(Data::default())
    }
}

pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
    let path = path.into();
    let data = read_data(&path)?;     // call only
    Ok(Self::from_data(path, data))   // call only
}

fn from_data(path: PathBuf, data: Data) -> Self {
    // pure logic — no I/O calls
}
```

The `read_data` function still has a VIOLATION (inherent at the I/O
boundary), but `open` is now clean and `from_data` is pure logic.

## IOSP VIOLATION: Separate pure logic from integration

**Before** (lint mixes I/O iteration with set logic):

```rust
pub fn lint(&self) -> Result<LintReport> {
    let mut report = LintReport::default();
    for (slug, content) in self.iter_pages()? {  // I/O call
        let links = extract_wikilinks(&content); // logic
        if links.is_empty() {                    // logic
            report.isolated_pages.push(slug);
        }
    }
    Ok(report)
}
```

**After** (I/O in lint, logic in build_lint_report):

```rust
pub fn lint(&self) -> Result<LintReport> {
    let pages = self.iter_pages()?;       // I/O only
    Ok(build_lint_report(&pages))         // delegate to pure logic
}

// qual:allow(iosp) reason: "pure logic calling extract_wikilinks"
fn build_lint_report(pages: &[(String, String)]) -> LintReport {
    // all logic, no I/O — testable without filesystem
}
```

## TQ_UNTESTED: Move logic from binary to library

**Before** (logic lives in CLI binary, untestable):

```rust
// in kgx-cli/src/main.rs
fn ingest_entities(graph: &mut GraphStore, input: &IngestInput) -> usize {
    for e in &input.entities {
        graph.add_node(&e.name, &e.entity_type, ...);
    }
    input.entities.len()
}
```

**After** (logic in library crate with unit tests):

```rust
// in kgx/src/ingest.rs
pub fn ingest_entities(
    graph: &mut GraphStore,
    doc_id: &str,
    entities: &[IngestEntity<'_>],
) -> usize { ... }

#[cfg(test)]
mod tests {
    #[test]
    fn ingest_entities_adds_nodes() { ... }
}
```

The CLI now calls `kgx::ingest_entities(...)` — a thin delegation.

## TQ_UNTESTED: FromStr instead of ad-hoc parsing

**Before** (parse function in binary, untested):

```rust
fn parse_category(s: &str) -> Result<WikiCategory> {
    match s {
        "summary" => Ok(WikiCategory::Summary),
        ...
        _ => bail!("unknown category"),
    }
}
```

**After** (FromStr in library with tests):

```rust
// in types.rs
impl FromStr for WikiCategory {
    type Err = WikiCategoryParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { ... }
}

#[test]
fn from_str_rejects_invalid_category() {
    let err = WikiCategory::from_str("bogus").unwrap_err();
    assert!(err.to_string().contains("bogus"));
}
```

CLI becomes: `s.parse::<WikiCategory>().map_err(|e| anyhow!("{e}"))`

## BOILERPLATE: Use thiserror

**Before** (manual Display + Error):

```rust
#[derive(Debug, Clone)]
pub struct MyError(pub String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}", self.0)
    }
}
impl std::error::Error for MyError {}
```

**After**:

```rust
#[derive(Debug, Clone, thiserror::Error)]
#[error("error: {0}")]
pub struct MyError(pub String);
```

## SRP_MODULE: Split test files

**Before** (one flat file with all tests):

```rust
// tests/cli.rs — 243 lines, SRP_MODULE finding
#[test] fn init_creates_workspace() { ... }
#[test] fn graph_add_node() { ... }
#[test] fn wiki_write_read() { ... }
```

**After** (organized by feature):

```rust
// tests/cli.rs
fn kgx() -> Command { ... }
fn temp_root() -> String { ... }
fn init_workspace(root: &str) { ... }

mod init {
    use super::*;
    #[test] fn creates_workspace() { ... }
}
mod graph {
    use super::*;
    #[test] fn add_node_and_search() { ... }
}
mod wiki {
    use super::*;
    #[test] fn write_read_list_search_lint() { ... }
}
```

## High Parameter Count: Input struct

**Before** (6 parameters):

```rust
pub fn add_edge(
    &mut self, source: NodeId, target: NodeId,
    relation_type: &str, confidence: f64,
    supporting_text: Option<&str>, source_doc: Option<&str>,
) -> Option<EdgeId> { ... }
```

**After**:

```rust
pub struct EdgeInput<'a> {
    pub source: NodeId,
    pub target: NodeId,
    pub relation_type: &'a str,
    pub confidence: f64,
    pub supporting_text: Option<&'a str>,
    pub source_doc: Option<&'a str>,
}

pub fn add_edge(&mut self, input: EdgeInput<'_>) -> Option<EdgeId> { ... }
```
