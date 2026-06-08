//! receiving-review — PreToolUse/Edit hook.
//! Warns if editing src/ with an open PR (should process review comments first).
//! Depends on `gh` at runtime.

/// Run the receiving-review hook. Returns a message for stderr (may be empty).
pub fn run(file_path: &str) -> String {
    if !file_path.contains("/src/") {
        return String::new();
    }

    // Check for open PR on current branch via gh
    let output = std::process::Command::new("gh")
        .args(["pr", "view", "--json", "state"])
        .output();

    let state = match output {
        Ok(o) if o.status.success() => {
            let json: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap_or_default();
            json.get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }
        _ => return String::new(),
    };

    if state == "OPEN" {
        "[godmode:receiving-review] Editing src/ with open PR — process review comments via /godmode:receiving-review first".to_string()
    } else {
        String::new()
    }
}
