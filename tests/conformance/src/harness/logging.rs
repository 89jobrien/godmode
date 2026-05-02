//! TestLogger — structured hierarchical output for conformance tests.

use std::fmt::Debug;

/// Logger for conformance test output.
pub struct TestLogger {
    entries: Vec<LogEntry>,
    indent: usize,
    test_name: String,
}

#[derive(Debug, Clone)]
struct LogEntry {
    level: LogLevel,
    indent: usize,
    message: String,
}

#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Info,
    Error,
}

impl TestLogger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            indent: 0,
            test_name: String::new(),
        }
    }

    pub fn set_test_name(&mut self, name: &str) {
        self.test_name = name.to_string();
    }

    pub fn clear_test_name(&mut self) {
        self.test_name.clear();
    }

    pub fn info(&mut self, msg: &str) {
        self.entries.push(LogEntry {
            level: LogLevel::Info,
            indent: self.indent,
            message: msg.to_string(),
        });
    }

    pub fn error(&mut self, msg: &str) {
        self.entries.push(LogEntry {
            level: LogLevel::Error,
            indent: self.indent,
            message: msg.to_string(),
        });
    }

    pub fn log_input<T: Debug>(&mut self, name: &str, value: &T) {
        self.info(&format!("input  {}: {:?}", name, value));
    }

    pub fn log_expected<T: Debug>(&mut self, name: &str, value: &T) {
        self.info(&format!("expect {}: {:?}", name, value));
    }

    pub fn log_actual<T: Debug>(&mut self, name: &str, value: &T) {
        self.info(&format!("actual {}: {:?}", name, value));
    }

    pub fn indent(&mut self) {
        self.indent += 2;
    }

    pub fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(2);
    }

    pub fn section<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.info(&format!("--- {} ---", name));
        self.indent();
        f(self);
        self.dedent();
    }

    /// Render all log entries to a string (used in failure output).
    pub fn render(&self) -> String {
        let mut out = String::new();
        for entry in &self.entries {
            let prefix = match entry.level {
                LogLevel::Info => "  ",
                LogLevel::Error => "! ",
            };
            out.push_str(&format!(
                "{}{}{}\n",
                " ".repeat(entry.indent),
                prefix,
                entry.message
            ));
        }
        out
    }
}

impl Default for TestLogger {
    fn default() -> Self {
        Self::new()
    }
}
