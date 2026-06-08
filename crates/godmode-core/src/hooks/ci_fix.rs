//! ci-fix — PostToolUse/Bash hook.
//! Notifies after a successful git push that a CI run may have started.
//! Note: this hook checks `gh` availability at runtime.

/// Run the ci-fix hook. Returns a message for stderr (may be empty).
pub fn run(command: &str, exit_code: i64) -> String {
    if !command.contains("git push") || exit_code != 0 {
        return String::new();
    }

    // Check if gh is available
    let gh_available = std::process::Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !gh_available {
        return String::new();
    }

    // Brief pause then check for a queued run
    std::thread::sleep(std::time::Duration::from_secs(3));

    let output = std::process::Command::new("gh")
        .args(["run", "list", "--limit", "1", "--json", "status"])
        .output();

    let status = match output {
        Ok(o) if o.status.success() => {
            let json: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap_or_default();
            json.as_array()
                .and_then(|a| a.first())
                .and_then(|r| r.get("status"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string()
        }
        _ => return String::new(),
    };

    if status == "queued" || status == "in_progress" {
        "[godmode:ci-fix] CI run started — check status with `gh run list --limit 3`".to_string()
    } else {
        String::new()
    }
}
