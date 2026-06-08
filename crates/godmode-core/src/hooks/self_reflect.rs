//! self-reflect — UserPromptSubmit hook.
//! Nudges self-reflect when user signals session end.

const TRIGGERS: &[&str] = &[
    "reflect",
    "self-reflect",
    "what did we do",
    "session summary",
    "end of session",
    "wrap up",
];

/// Run the self-reflect hook. Returns a message for stderr (may be empty).
pub fn run(prompt: &str) -> String {
    let lower = prompt.to_lowercase();
    if TRIGGERS.iter().any(|t| lower.contains(t)) {
        "[godmode:self-reflect] Session close detected — consider running /godmode:self-reflect"
            .to_string()
    } else {
        String::new()
    }
}
