use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub name: String,
    pub path: PathBuf,
}

/// List all skills in `dir` by scanning for subdirectories containing `SKILL.md`.
pub fn list_local(dir: &Path) -> Result<Vec<SkillDef>> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut skills = vec![];
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").exists() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            skills.push(SkillDef {
                name,
                path: path.canonicalize().unwrap_or(path),
            });
        }
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn list_local_empty_when_no_skills() {
        let dir = TempDir::new().unwrap();
        let skills = list_local(dir.path()).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn list_local_finds_skill_dirs() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("my-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "# my-skill").unwrap();

        // A dir without SKILL.md should not appear
        let other = dir.path().join("not-a-skill");
        std::fs::create_dir(&other).unwrap();

        let skills = list_local(dir.path()).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "my-skill");
    }

    #[test]
    fn list_local_absent_dir_returns_empty() {
        let path = PathBuf::from("/tmp/godmode-test-nonexistent-dir-xyz");
        let skills = list_local(&path).unwrap();
        assert!(skills.is_empty());
    }
}
