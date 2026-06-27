#!/usr/bin/env rust-script
//! Validate a SKILL.md frontmatter with strict key/field checks.

use std::env;
use std::fs;

const ALLOWED_KEYS: &[&str] = &[
    "name",
    "description",
    "license",
    "allowed-tools",
    "metadata",
    "requires",
    "next",
];

#[derive(Debug)]
struct Validation {
    errors: Vec<String>,
}

impl Validation {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }

    fn push(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }

    fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

fn parse_frontmatter(src: &str) -> Result<Vec<(String, String)>, String> {
    let mut lines = src.lines().peekable();
    let first = lines.next().ok_or("file is empty")?;
    if first.trim() != "---" {
        return Err("missing frontmatter: file must start with ---".into());
    }

    let mut fields = Vec::new();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed == "---" {
            return Ok(fields);
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = line.split_once(':') else {
            return Err(format!("invalid frontmatter line: {trimmed}"));
        };

        let key = raw_key.trim();
        if key.is_empty() {
            return Err("frontmatter key is empty".into());
        }

        let raw_value = raw_value.trim();
        let value = if raw_value.starts_with('>') || raw_value.starts_with('|') {
            let mut block = String::new();
            while let Some(next_line) = lines.peek().map(|s| s.trim_end()) {
                if next_line.trim().is_empty() {
                    block.push('\n');
                    lines.next();
                    continue;
                }
                if next_line.starts_with(' ') || next_line.starts_with('\t') {
                    block.push_str(next_line.trim_start());
                    block.push('\n');
                    lines.next();
                } else {
                    break;
                }
            }
            block.trim().to_string()
        } else {
            raw_value.to_string()
        };

        fields.push((key.to_string(), value));
        if value.is_empty() {
            // Keep behaviour consistent with the existing field-empty validation.
            continue;
        }
    }
    Err("unclosed frontmatter: missing closing ---".into())
}

fn validate(path: &str) -> Validation {
    let mut v = Validation::new();

    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            v.push(format!("failed to read '{path}': {e}"));
            return v;
        }
    };

    let fields = match parse_frontmatter(&src) {
        Ok(f) => f,
        Err(e) => {
            v.push(e);
            return v;
        }
    };

    let map: std::collections::HashMap<&str, &str> = fields
        .iter()
        .filter_map(|(k, v)| {
            if k.is_empty() {
                None
            } else {
                Some((k.as_str(), v.as_str()))
            }
        })
        .collect();

    for (name, value) in &fields {
        if !ALLOWED_KEYS.contains(&name.as_str()) {
            v.push(format!("unexpected key '{name}'"));
        }
        if value.trim().is_empty() {
            v.push(format!("field '{name}' is empty"));
        }
    }

    let name = map.get("name").copied().unwrap_or("");
    if name.is_empty() {
        v.push("missing required field 'name'");
    } else {
        if name.len() > 64 {
            v.push(format!("name is too long ({}) chars, max 64", name.len()));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            || name.starts_with('-')
            || name.ends_with('-')
            || name.contains("--")
        {
            v.push("name must be kebab-case: lowercase letters, digits, and single hyphens only"
                .to_string());
        }
    }

    let desc = map.get("description").copied().unwrap_or("");
    if desc.is_empty() {
        v.push("missing required field 'description'");
    } else {
        if desc.len() > 1024 {
            v.push(format!(
                "description is too long ({} chars), max 1024",
                desc.len()
            ));
        }
        if desc.contains('<') || desc.contains('>') {
            v.push("description must not contain angle brackets");
        }
    }

    v
}

fn main() -> std::process::ExitCode {
    let path = env::args().nth(1).unwrap_or_else(|| "SKILL.md".to_string());
    let validation = validate(&path);

    if validation.is_ok() {
        println!("SKILL is valid.");
        return std::process::ExitCode::SUCCESS;
    }

    eprintln!("SKILL validation failed:");
    for err in validation.errors {
        eprintln!(" - {err}");
    }
    std::process::ExitCode::FAILURE
}
