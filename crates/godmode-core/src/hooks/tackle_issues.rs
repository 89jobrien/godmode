//! tackle-issues — PostToolUse/Bash hook.
//! After `gh issue close`, reminds to verify the merge.

/// Run the tackle-issues hook. Returns a message for stderr (may be empty).
pub fn run(command: &str) -> String {
    if !command.contains("gh issue close") {
        return String::new();
    }

    // Extract issue number (token after "close")
    let parts: Vec<&str> = command.split_whitespace().collect();
    let issue_num = parts
        .iter()
        .position(|&p| p == "close")
        .and_then(|i| parts.get(i + 1))
        .copied()
        .unwrap_or("");

    if issue_num.is_empty() {
        "[godmode:tackle-issues] Issue closed — verify merge in git log: `git log --oneline main | head -5`".to_string()
    } else {
        format!(
            "[godmode:tackle-issues] Issue #{issue_num} closed — verify merge in git log: `git log --oneline main | head -5`"
        )
    }
}
