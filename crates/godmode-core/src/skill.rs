//! Discovery and index generation for repository-local skills.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Name and canonical filesystem path of a discovered skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    /// Skill directory name.
    pub name: String,
    /// Path to the skill directory.
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

/// Write `<root>/skills/INDEX.md` from the current local skills.
pub fn generate_skill_index(root: &Path, skills: &[SkillDef]) -> Result<()> {
    std::fs::write(root.join("skills/INDEX.md"), render_skill_index(skills)?)?;
    Ok(())
}

/// Return whether `skills/INDEX.md` matches the current local skill list.
pub fn skill_index_is_current(root: &Path, skills: &[SkillDef]) -> Result<bool> {
    let expected = render_skill_index(skills)?;
    Ok(std::fs::read_to_string(root.join("skills/INDEX.md"))
        .is_ok_and(|content| content == expected))
}

fn render_skill_index(skills: &[SkillDef]) -> Result<String> {
    let mut lines = vec![
        "# Skill Index".to_string(),
        String::new(),
        "| Name | Description |".to_string(),
        "| ---- | ----------- |".to_string(),
    ];
    for skill in skills {
        let description = skill_description(&skill.path)?.replace('|', "/");
        lines.push(format!("| {} | {} |", skill.name, description));
    }
    lines.push(String::new());
    Ok(lines.join("\n"))
}

fn skill_description(path: &Path) -> Result<String> {
    let skill_path = path.join("SKILL.md");
    let content = std::fs::read_to_string(&skill_path)
        .with_context(|| format!("reading skill manifest {}", skill_path.display()))?;
    let Some(frontmatter) = content
        .strip_prefix("---")
        .and_then(|rest| rest.split_once("\n---"))
        .map(|(frontmatter, _)| frontmatter)
    else {
        bail!("missing YAML frontmatter in {}", skill_path.display());
    };
    let value: serde_yaml::Value = serde_yaml::from_str(frontmatter)
        .with_context(|| format!("parsing skill manifest {}", skill_path.display()))?;
    Ok(value
        .get("description")
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(" "))
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

    #[test]
    fn generate_skill_index_writes_descriptions_and_detects_staleness() {
        let dir = TempDir::new().unwrap();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("my-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\ndescription: A focused test skill.\n---\n",
        )
        .unwrap();
        let skills = list_local(&skills_dir).unwrap();

        generate_skill_index(dir.path(), &skills).unwrap();
        let index = std::fs::read_to_string(skills_dir.join("INDEX.md")).unwrap();
        assert!(index.contains("| my-skill | A focused test skill. |"));
        assert!(skill_index_is_current(dir.path(), &skills).unwrap());

        std::fs::write(skills_dir.join("INDEX.md"), "stale").unwrap();
        assert!(!skill_index_is_current(dir.path(), &skills).unwrap());
    }

    #[test]
    fn generate_skill_index_reports_invalid_frontmatter() {
        let dir = TempDir::new().unwrap();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("broken");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "---\nname: [broken\n---\n").unwrap();
        let skills = list_local(&skills_dir).unwrap();

        let error = generate_skill_index(dir.path(), &skills).unwrap_err();

        assert!(error.to_string().contains("parsing skill manifest"));
    }
}
