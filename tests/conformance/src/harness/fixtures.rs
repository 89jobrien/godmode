//! FixtureLoader — load expected outputs from tests/conformance/fixtures/.

use std::path::{Path, PathBuf};

use serde_json::Value;
use thiserror::Error;

/// Result type returned by fixture loading operations.
pub type FixtureResult<T> = Result<T, FixtureError>;

/// Errors that can occur while locating or parsing a fixture.
#[derive(Debug, Error)]
pub enum FixtureError {
    /// The requested fixture path does not exist.
    #[error("fixture file not found: {path}")]
    NotFound {
        /// Path at which the fixture was expected.
        path: PathBuf,
    },
    /// The fixture file could not be read or parsed as JSON.
    #[error("fixture parse error in {path}: {reason}")]
    ParseError {
        /// Path of the fixture that could not be parsed.
        path: PathBuf,
        /// Underlying read or JSON parsing error.
        reason: String,
    },
}

/// A loaded fixture.
#[derive(Debug, Clone)]
pub struct TestFixture {
    /// Filesystem path from which the fixture was loaded.
    pub path: PathBuf,
    /// Parsed JSON fixture contents.
    pub data: Value,
}

impl TestFixture {
    /// Get a field from the fixture data.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Get a string field.
    pub fn str_field(&self, key: &str) -> Option<&str> {
        self.data.get(key)?.as_str()
    }

    /// Get a string array field.
    pub fn str_array(&self, key: &str) -> Vec<&str> {
        self.data
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default()
    }
}

/// Loads fixtures from `tests/conformance/fixtures/expected/`.
pub struct FixtureLoader {
    fixtures_dir: PathBuf,
}

impl FixtureLoader {
    /// Creates a loader for the manifest's `fixtures/expected` directory.
    pub fn new() -> Self {
        // Resolve relative to CARGO_MANIFEST_DIR at runtime if possible,
        // otherwise fall back to cwd-relative path.
        let base = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        Self {
            fixtures_dir: base.join("fixtures").join("expected"),
        }
    }

    /// Creates a loader that reads fixtures from a specific directory.
    pub fn with_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            fixtures_dir: dir.into(),
        }
    }

    /// Load a fixture by name (filename stem, no extension).
    pub fn load(&self, name: &str) -> FixtureResult<TestFixture> {
        let path = self.fixtures_dir.join(format!("{}.json", name));
        if !path.exists() {
            return Err(FixtureError::NotFound { path });
        }
        let raw = std::fs::read_to_string(&path).map_err(|e| FixtureError::ParseError {
            path: path.clone(),
            reason: e.to_string(),
        })?;
        let data: Value = serde_json::from_str(&raw).map_err(|e| FixtureError::ParseError {
            path: path.clone(),
            reason: e.to_string(),
        })?;
        Ok(TestFixture { path, data })
    }

    /// List all available fixture names.
    pub fn list(&self) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(&self.fixtures_dir) else {
            return vec![];
        };
        entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let p = e.path();
                if p.extension()? == "json" {
                    Some(p.file_stem()?.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for FixtureLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience: resolve fixture dir from a repo root path.
pub fn fixture_dir(repo_root: &Path) -> PathBuf {
    repo_root
        .join("tests")
        .join("conformance")
        .join("fixtures")
        .join("expected")
}
