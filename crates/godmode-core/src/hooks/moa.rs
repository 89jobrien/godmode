//! moa — PreToolUse/Bash hook.
//! Informs when an LLM API call is detected during an active task.

use std::path::Path;

use crate::graph;
use crate::model::Status;

const LLM_PATTERNS: &[&str] = &["openai", "anthropic", "claude"];

/// Check if the command looks like an LLM API call.
fn is_llm_call(cmd: &str) -> bool {
    if LLM_PATTERNS.iter().any(|p| cmd.contains(p)) {
        return true;
    }
    cmd.contains("curl") && cmd.contains("api.")
}

/// Run the moa hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, command: &str) -> String {
    if !is_llm_call(command) {
        return String::new();
    }

    let graph = match graph::load(root) {
        Ok(g) => g,
        Err(_) => return String::new(),
    };

    let has_running = graph.tasks.iter().any(|t| t.status == Status::Running);
    if !has_running {
        return String::new();
    }

    "[godmode:moa] LLM call detected during active task — consider /godmode:moa for multi-model synthesis".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_llm_calls() {
        assert!(is_llm_call("curl -s https://api.openai.com/v1/chat"));
        assert!(is_llm_call("curl https://api.anthropic.com/v1/messages"));
        assert!(is_llm_call("python -c 'import openai'"));
        assert!(!is_llm_call("cargo test"));
        assert!(!is_llm_call("curl https://example.com"));
    }
}
