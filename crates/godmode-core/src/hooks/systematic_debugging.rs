//! systematic-debugging — PostToolUse/Bash hook.
//! After non-zero exit (not godmode/git), nudges systematic debugging.

/// Run the systematic-debugging hook. Returns a message for stderr (may be empty).
pub fn run(command: &str, exit_code: i64) -> String {
    if exit_code == 0 {
        return String::new();
    }

    // Skip godmode and git commands
    if command.contains("godmode") || command.contains("git") {
        return String::new();
    }

    format!(
        "[godmode:debug] Command failed (exit {exit_code}) — run /godmode:systematic-debugging before guessing a fix"
    )
}
