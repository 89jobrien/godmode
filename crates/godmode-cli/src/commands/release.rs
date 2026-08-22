//! Plugin release: version bump, tag, push, changelog.

use anyhow::Result;
use godmode_core::release;
use std::path::Path;

use crate::ReleaseAction;

pub fn run_release_action(root: &Path, json: bool, action: ReleaseAction) -> Result<()> {
    match action {
        ReleaseAction::Current => {
            let v = release::current_version(root)?;
            if json {
                println!("{}", serde_json::json!({"version": v}));
            } else {
                println!("{}", v);
            }
            Ok(())
        }
        ReleaseAction::Bump { version } => {
            let info = release::bump(root, version.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&info)?);
            } else {
                println!("{} → {}", info.old_version, info.new_version);
            }
            Ok(())
        }
        ReleaseAction::Tag => {
            let tag = release::tag(root)?;
            if json {
                println!("{}", serde_json::json!({"tag": tag}));
            } else {
                println!("Tagged {}", tag);
            }
            Ok(())
        }
        ReleaseAction::Push => {
            release::push(root)?;
            if json {
                println!("{}", serde_json::json!({"ok": true}));
            } else {
                println!("Pushed branch and tag.");
            }
            Ok(())
        }
        ReleaseAction::Changelog => {
            let entry = release::generate_changelog(root)?;
            release::write_changelog(root, &entry)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "version": entry.version, "date": entry.date})
                );
            } else {
                println!("Updated CHANGELOG.md for version {}.", entry.version);
            }
            Ok(())
        }

        ReleaseAction::Validate => {
            let warnings = release::validate_versions(root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&warnings)?);
            } else if warnings.is_empty() {
                println!("All versions consistent.");
            } else {
                println!("Version drift detected:");
                for w in &warnings {
                    println!("  - {w}");
                }
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
