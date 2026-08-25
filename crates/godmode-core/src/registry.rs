use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Kind of installable artifact tracked in the global registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Skill,
    Agent,
}

/// Single installed skill/agent entry persisted in `registry.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub kind: EntryKind,
    pub path: PathBuf,
    pub version: String,
}

/// Global registry of installed skills and agents.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Registry {
    pub entries: Vec<RegistryEntry>,
}

/// Resolve the global registry file path under `$HOME/.config/godmode/`.
fn global_registry_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("godmode")
        .join("registry.json")
}

impl Registry {
    /// Load from `~/.config/godmode/registry.json`; returns empty registry if absent.
    pub fn load_global() -> Result<Self> {
        let path = global_registry_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    /// Persist to `~/.config/godmode/registry.json`.
    pub fn save_global(&self) -> Result<()> {
        let path = global_registry_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Install an entry idempotently. Returns `true` if the entry was new.
    pub fn install(&mut self, entry: RegistryEntry) -> bool {
        if self.entries.iter().any(|e| e.name == entry.name) {
            return false;
        }
        self.entries.push(entry);
        true
    }

    /// Remove an entry by name. Returns `true` if something was removed.
    pub fn uninstall(&mut self, name: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.name != name);
        self.entries.len() < before
    }

    /// List entries, optionally filtered by kind.
    pub fn list(&self, kind: Option<EntryKind>) -> Vec<&RegistryEntry> {
        match kind {
            None => self.entries.iter().collect(),
            Some(k) => self.entries.iter().filter(|e| e.kind == k).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn mk_entry(name: &str, kind: EntryKind) -> RegistryEntry {
        RegistryEntry {
            name: name.to_string(),
            kind,
            path: PathBuf::from("/tmp/fake"),
            version: "1.0.0".to_string(),
        }
    }

    #[test]
    fn install_is_idempotent() {
        let mut reg = Registry::default();
        assert!(reg.install(mk_entry("my-skill", EntryKind::Skill)));
        assert!(!reg.install(mk_entry("my-skill", EntryKind::Skill)));
        assert_eq!(reg.entries.len(), 1);
    }

    #[test]
    fn uninstall_returns_true_when_removed() {
        let mut reg = Registry::default();
        reg.install(mk_entry("my-agent", EntryKind::Agent));
        assert!(reg.uninstall("my-agent"));
        assert!(!reg.uninstall("my-agent"));
        assert!(reg.entries.is_empty());
    }

    #[test]
    fn list_filters_by_kind() {
        let mut reg = Registry::default();
        reg.install(mk_entry("s1", EntryKind::Skill));
        reg.install(mk_entry("a1", EntryKind::Agent));
        assert_eq!(reg.list(Some(EntryKind::Skill)).len(), 1);
        assert_eq!(reg.list(Some(EntryKind::Agent)).len(), 1);
        assert_eq!(reg.list(None).len(), 2);
    }

    #[test]
    fn roundtrip_json() {
        let _dir = TempDir::new().unwrap();
        let mut reg = Registry::default();
        reg.install(mk_entry("x", EntryKind::Skill));
        let json = serde_json::to_string_pretty(&reg).unwrap();
        let reg2: Registry = serde_json::from_str(&json).unwrap();
        assert_eq!(reg2.entries.len(), 1);
        assert_eq!(reg2.entries[0].name, "x");
    }
}
