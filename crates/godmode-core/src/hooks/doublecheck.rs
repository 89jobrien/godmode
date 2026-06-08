//! doublecheck — PostToolUse/Bash hook.
//! After web-fetching commands, reminds to verify factual claims.

/// Run the doublecheck hook. Returns a message for stderr (may be empty).
pub fn run(command: &str, exit_code: i64) -> String {
    if exit_code != 0 {
        return String::new();
    }

    let is_web_fetch = command.contains("curl ")
        || command.contains("wget ")
        || command.contains("web_search")
        || command.contains("http");

    if is_web_fetch {
        "[godmode:doublecheck] Web content fetched — verify factual claims before committing to a plan".to_string()
    } else {
        String::new()
    }
}
