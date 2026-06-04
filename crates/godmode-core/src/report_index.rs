//! Report index — tracks files across categorised report subdirectories.
//!
//! # Architecture
//!
//! **Port**: [`ReportIndexPort`] — the trait that any index storage backend
//! must implement.
//!
//! **Adapter**: [`JsonFileIndex`] — reads/writes `godmode-reports.index.json`.
//!
//! Writers (Rust code like `insights::render_markdown`, or skill prompts like
//! `self-reflect` and `introspection`) call [`JsonFileIndex::add_entry`] after
//! producing a report file. The rebuild script (`scripts/rebuild-reports-index.nu`)
//! calls [`JsonFileIndex::rebuild`] for bulk reconciliation.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReportIndex {
    pub version: u32,
    pub generated: String,
    pub categories: BTreeMap<String, ReportCategory>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReportCategory {
    pub path: String,
    pub description: String,
    pub files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
}

// ---------------------------------------------------------------------------
// Port (trait)
// ---------------------------------------------------------------------------

pub trait ReportIndexPort {
    /// Add a file entry under a category. Creates the category if absent.
    fn add_entry(&self, category: &str, filename: &str) -> Result<()>;

    /// Add an item (sub-file) entry under a category.
    fn add_item(&self, category: &str, item_path: &str) -> Result<()>;

    /// Rebuild the entire index by scanning report subdirectories on disk.
    fn rebuild(&self) -> Result<ReportIndex>;

    /// Load the current index.
    fn load(&self) -> Result<ReportIndex>;
}

// ---------------------------------------------------------------------------
// Adapter: JSON file
// ---------------------------------------------------------------------------

/// Reports root directory (`.ctx/godmode/reports/`).
pub struct JsonFileIndex {
    reports_dir: PathBuf,
}

impl JsonFileIndex {
    pub fn new(project_root: &Path) -> Self {
        Self {
            reports_dir: project_root.join(".ctx").join("godmode").join("reports"),
        }
    }

    fn index_path(&self) -> PathBuf {
        self.reports_dir.join("godmode-reports.index.json")
    }

    fn load_or_default(&self) -> Result<ReportIndex> {
        let path = self.index_path();
        if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
        } else {
            Ok(ReportIndex {
                version: 1,
                generated: today_str(),
                categories: BTreeMap::new(),
            })
        }
    }

    fn save(&self, index: &ReportIndex) -> Result<()> {
        let path = self.index_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(index)?;
        std::fs::write(&path, format!("{json}\n"))?;
        Ok(())
    }
}

fn today_str() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

/// Known category metadata — used by `rebuild` to populate descriptions.
fn category_meta(name: &str) -> (&str, &str) {
    match name {
        "reflect" => ("reflect/", "Session self-reflection reports"),
        "introspection" => ("introspection/", "Skill/agent/plugin consistency audits"),
        "insights" => (
            "insights/",
            "Session insight captures and individual insight items",
        ),
        _ => ("", ""),
    }
}

impl ReportIndexPort for JsonFileIndex {
    fn add_entry(&self, category: &str, filename: &str) -> Result<()> {
        let mut index = self.load_or_default()?;
        index.generated = today_str();

        let cat = index
            .categories
            .entry(category.to_string())
            .or_insert_with(|| {
                let (path, desc) = category_meta(category);
                ReportCategory {
                    path: path.to_string(),
                    description: desc.to_string(),
                    files: Vec::new(),
                    items: Vec::new(),
                }
            });

        let name = filename.to_string();
        if !cat.files.contains(&name) {
            cat.files.push(name);
            cat.files.sort();
        }

        self.save(&index)
    }

    fn add_item(&self, category: &str, item_path: &str) -> Result<()> {
        let mut index = self.load_or_default()?;
        index.generated = today_str();

        let cat = index
            .categories
            .entry(category.to_string())
            .or_insert_with(|| {
                let (path, desc) = category_meta(category);
                ReportCategory {
                    path: path.to_string(),
                    description: desc.to_string(),
                    files: Vec::new(),
                    items: Vec::new(),
                }
            });

        let p = item_path.to_string();
        if !cat.items.contains(&p) {
            cat.items.push(p);
            cat.items.sort();
        }

        self.save(&index)
    }

    fn rebuild(&self) -> Result<ReportIndex> {
        let mut index = ReportIndex {
            version: 1,
            generated: today_str(),
            categories: BTreeMap::new(),
        };

        for known in &["reflect", "introspection", "insights"] {
            let dir = self.reports_dir.join(known);
            if !dir.is_dir() {
                continue;
            }

            let (path, desc) = category_meta(known);
            let mut cat = ReportCategory {
                path: path.to_string(),
                description: desc.to_string(),
                files: Vec::new(),
                items: Vec::new(),
            };

            // Top-level files in the category dir
            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                let ft = entry.file_type()?;
                let name = entry.file_name().to_string_lossy().to_string();
                if ft.is_file() && name.ends_with(".md") {
                    cat.files.push(name);
                } else if ft.is_dir() && name == "items" {
                    // Scan items/ subdirectory
                    for item in std::fs::read_dir(entry.path())? {
                        let item = item?;
                        if item.file_type()?.is_file() {
                            let item_name = format!("items/{}", item.file_name().to_string_lossy());
                            cat.items.push(item_name);
                        }
                    }
                    cat.items.sort();
                }
            }
            cat.files.sort();
            index.categories.insert(known.to_string(), cat);
        }

        self.save(&index)?;
        Ok(index)
    }

    fn load(&self) -> Result<ReportIndex> {
        self.load_or_default()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, JsonFileIndex) {
        let dir = TempDir::new().unwrap();
        let idx = JsonFileIndex::new(dir.path());
        // Create the reports dir so save() works
        std::fs::create_dir_all(idx.reports_dir.clone()).unwrap();
        (dir, idx)
    }

    #[test]
    fn add_entry_creates_category_and_file() {
        let (_dir, idx) = setup();
        idx.add_entry("reflect", "reflect-2026-06-04.md").unwrap();

        let loaded = idx.load().unwrap();
        assert!(loaded.categories.contains_key("reflect"));
        assert_eq!(loaded.categories["reflect"].files.len(), 1);
        assert_eq!(
            loaded.categories["reflect"].files[0],
            "reflect-2026-06-04.md"
        );
    }

    #[test]
    fn add_entry_is_idempotent() {
        let (_dir, idx) = setup();
        idx.add_entry("reflect", "reflect-2026-06-04.md").unwrap();
        idx.add_entry("reflect", "reflect-2026-06-04.md").unwrap();

        let loaded = idx.load().unwrap();
        assert_eq!(loaded.categories["reflect"].files.len(), 1);
    }

    #[test]
    fn add_item_populates_items_vec() {
        let (_dir, idx) = setup();
        idx.add_item("insights", "items/2026-06-04-001-test.md")
            .unwrap();

        let loaded = idx.load().unwrap();
        assert_eq!(loaded.categories["insights"].items.len(), 1);
    }

    #[test]
    fn rebuild_scans_disk() {
        let (dir, idx) = setup();
        // Create reflect subdir with a file
        let reflect_dir = dir.path().join(".ctx/godmode/reports/reflect");
        std::fs::create_dir_all(&reflect_dir).unwrap();
        std::fs::write(reflect_dir.join("reflect-2026-01-01.md"), "# test").unwrap();

        let rebuilt = idx.rebuild().unwrap();
        assert_eq!(rebuilt.categories["reflect"].files.len(), 1);
        assert_eq!(
            rebuilt.categories["reflect"].files[0],
            "reflect-2026-01-01.md"
        );
    }

    #[test]
    fn load_returns_default_when_no_file() {
        let (_dir, idx) = setup();
        let loaded = idx.load().unwrap();
        assert!(loaded.categories.is_empty());
        assert_eq!(loaded.version, 1);
    }
}
