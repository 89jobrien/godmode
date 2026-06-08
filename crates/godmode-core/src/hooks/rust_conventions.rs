//! rust-conventions — PreToolUse/Edit hook.
//! Warns about bare unwrap() calls in non-test Rust code.

/// Run the rust-conventions hook. Returns a message for stderr (may be empty).
pub fn run(file_path: &str) -> String {
    if !file_path.ends_with(".rs") {
        return String::new();
    }

    // Skip test infrastructure
    if file_path.contains("/tests/") || file_path.contains("/testing/") {
        return String::new();
    }

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    // Only check code above #[cfg(test)]
    let check_content = if let Some(pos) = content.find("#[cfg(test)]") {
        &content[..pos]
    } else {
        &content
    };

    let bare_unwraps = check_content
        .lines()
        .filter(|line| line.contains(".unwrap()") && !line.contains(".expect("))
        .count();

    if bare_unwraps > 0 {
        let basename = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path);
        format!(
            "[godmode:rust-conventions] {basename} has {bare_unwraps} bare unwrap() — use .expect(\"reason\") or return Result"
        )
    } else {
        String::new()
    }
}
