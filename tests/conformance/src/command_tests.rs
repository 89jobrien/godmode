//! Cross-target conformance tests for generated slash commands.

use std::collections::BTreeSet;
use std::path::PathBuf;

use godmode_core::command::{self, CommandTarget};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn command_projections_match_all_canonical_sources() {
    let root = repo_root();
    let source = root.join("command-support/gm");
    let definitions = command::load_command_definitions(&source).unwrap();
    let claude = command::render_commands(
        &definitions,
        CommandTarget::Claude,
        &source.join("templates"),
    )
    .unwrap();
    let opencode = command::render_commands(
        &definitions,
        CommandTarget::OpenCode,
        &source.join("templates"),
    )
    .unwrap();

    assert_eq!(definitions.len(), claude.len());
    assert_eq!(definitions.len(), opencode.len());
    assert_eq!(
        claude
            .iter()
            .map(|command| &command.file_name)
            .collect::<BTreeSet<_>>()
            .len(),
        definitions.len()
    );
    for (claude, opencode) in claude.iter().zip(&opencode) {
        assert_eq!(claude.file_name, opencode.file_name);
        assert!(claude.content.contains("allowed-tools:"));
        assert!(opencode.content.contains("subtask: false"));
        assert!(!opencode.content.contains("/gm:"));
        let tracked = std::fs::read_to_string(root.join("commands").join(&claude.file_name))
            .unwrap_or_else(|error| panic!("missing {}: {error}", claude.file_name));
        assert_eq!(tracked, claude.content, "stale {}", claude.file_name);
    }

    let gitignore = std::fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(!gitignore.lines().any(|line| line == "commands/gm-*.md"));
}
