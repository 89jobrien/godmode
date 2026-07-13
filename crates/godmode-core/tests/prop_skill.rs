//! Property-based tests for `skill::list_local`.
//!
//! Run with: cargo nextest run --test prop_skill

use godmode_core::skill::{SkillDef, list_local};
use proptest::prelude::*;
use std::collections::HashSet;
use tempfile::TempDir;

// ── Strategies ───────────────────────────────────────────────────────────────

/// Generate a valid skill directory name: lowercase alphanumeric + hyphens, 1-24 chars.
fn skill_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,22}[a-z0-9]".prop_map(|s| s)
}

/// Generate a set of 0-10 unique skill names.
fn unique_skill_names() -> impl Strategy<Value = Vec<String>> {
    prop::collection::hash_set(skill_name(), 0..=10).prop_map(|s| s.into_iter().collect::<Vec<_>>())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn create_skill(dir: &std::path::Path, name: &str) {
    let skill_dir = dir.join(name);
    std::fs::create_dir(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), format!("# {name}")).unwrap();
}

fn create_non_skill(dir: &std::path::Path, name: &str) {
    // Directory without SKILL.md — must not appear in results.
    std::fs::create_dir(dir.join(name)).unwrap();
}

// ── Properties ────────────────────────────────────────────────────────────────

proptest! {
    /// The result length equals the number of skill dirs that were created.
    #[test]
    fn prop_list_local_len_matches_skill_count(names in unique_skill_names()) {
        let dir = TempDir::new().unwrap();
        for name in &names {
            create_skill(dir.path(), name);
        }
        let skills = list_local(dir.path()).unwrap();
        prop_assert_eq!(skills.len(), names.len());
    }

    /// Results are always sorted lexicographically by name.
    #[test]
    fn prop_list_local_is_sorted(names in unique_skill_names()) {
        let dir = TempDir::new().unwrap();
        for name in &names {
            create_skill(dir.path(), name);
        }
        let skills = list_local(dir.path()).unwrap();
        let names_out: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        let mut sorted = names_out.clone();
        sorted.sort();
        prop_assert_eq!(names_out, sorted);
    }

    /// Dirs without SKILL.md are never included in the results.
    #[test]
    fn prop_non_skill_dirs_excluded(
        skill_names in unique_skill_names(),
        non_skill_suffix in "[a-z]{3,8}",
    ) {
        let dir = TempDir::new().unwrap();
        for name in &skill_names {
            create_skill(dir.path(), name);
        }
        // Add a non-skill dir whose name won't collide with any skill name.
        let decoy = format!("__{non_skill_suffix}");
        create_non_skill(dir.path(), &decoy);

        let skills = list_local(dir.path()).unwrap();
        let found_names: HashSet<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        prop_assert!(!found_names.contains(decoy.as_str()),
            "non-skill dir {:?} must not appear in results", decoy);
    }

    /// Every returned skill name matches the directory name on disk.
    #[test]
    fn prop_name_matches_dir(names in unique_skill_names()) {
        let dir = TempDir::new().unwrap();
        for name in &names {
            create_skill(dir.path(), name);
        }
        let skills = list_local(dir.path()).unwrap();
        let expected: HashSet<&str> = names.iter().map(|s| s.as_str()).collect();
        for skill in &skills {
            prop_assert!(expected.contains(skill.name.as_str()),
                "unexpected skill name {:?}", skill.name);
        }
    }

    /// SkillDef round-trips through JSON without loss.
    #[test]
    fn prop_skill_def_json_roundtrip(
        name in "[a-z][a-z0-9-]{0,20}",
        path_suffix in "[a-z]{3,10}",
    ) {
        let original = SkillDef {
            name: name.clone(),
            path: std::path::PathBuf::from(format!("/tmp/{path_suffix}")),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: SkillDef = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&restored.name, &original.name);
        prop_assert_eq!(&restored.path, &original.path);
    }
}
