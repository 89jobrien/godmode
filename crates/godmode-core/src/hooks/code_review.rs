//! code-review — PostToolUse/Bash hook.
//! Suggests running code-review after `gh pr create`.

/// Run the code-review hook. Returns a message for stderr (may be empty).
pub fn run(command: &str) -> String {
    if command.contains("gh pr create") {
        "[godmode:code-review] PR created — run /godmode:code-review for a systematic quality pass before requesting review".to_string()
    } else {
        String::new()
    }
}
