//! Declarative hook metadata, manifest projection, and coverage diagnostics.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HookClient {
    Claude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookStatus {
    Active,
    Superseded,
    Internal,
}

#[derive(Debug, Clone, Copy)]
pub struct HookRegistration {
    pub id: &'static str,
    pub client: Option<HookClient>,
    pub event: Option<&'static str>,
    pub matcher: Option<&'static str>,
    pub command: &'static str,
    pub timeout: Option<u64>,
    pub status: HookStatus,
    pub superseded_by: Option<&'static str>,
}

impl HookRegistration {
    pub const fn active(
        id: &'static str,
        client: HookClient,
        event: &'static str,
        matcher: &'static str,
        command: &'static str,
        timeout: u64,
    ) -> Self {
        Self {
            id,
            client: Some(client),
            event: Some(event),
            matcher: Some(matcher),
            command,
            timeout: Some(timeout),
            status: HookStatus::Active,
            superseded_by: None,
        }
    }
    pub const fn superseded(
        id: &'static str,
        superseded_by: &'static str,
        command: &'static str,
    ) -> Self {
        Self {
            id,
            client: None,
            event: None,
            matcher: None,
            command,
            timeout: None,
            status: HookStatus::Superseded,
            superseded_by: Some(superseded_by),
        }
    }
    pub const fn internal(id: &'static str, command: &'static str) -> Self {
        Self {
            id,
            client: None,
            event: None,
            matcher: None,
            command,
            timeout: None,
            status: HookStatus::Internal,
            superseded_by: None,
        }
    }
}

const CLAUDE: HookClient = HookClient::Claude;

/// Canonical metadata for supported-client hooks and retired scripts.
pub const REGISTRY: &[HookRegistration] = &[
    HookRegistration::active(
        "session-start",
        CLAUDE,
        "SessionStart",
        "*",
        "rust-script $CLAUDE_PLUGIN_ROOT/hooks/scripts/session-start.rs",
        15,
    ),
    HookRegistration::active(
        "memory-bank-inject",
        CLAUDE,
        "SessionStart",
        "*",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/memory-bank-inject.nu",
        10,
    ),
    HookRegistration::active(
        "task-management",
        CLAUDE,
        "SessionStart",
        "*",
        "godmode hook run task-management",
        10,
    ),
    HookRegistration::active(
        "pre-agent-task-context",
        CLAUDE,
        "PreToolUse",
        "Agent",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/pre-agent-task-context.nu",
        10,
    ),
    HookRegistration::active(
        "agent-governance",
        CLAUDE,
        "PreToolUse",
        "Agent",
        "godmode hook run agent-governance",
        10,
    ),
    HookRegistration::active(
        "pre-bash-nag",
        CLAUDE,
        "PreToolUse",
        "Bash",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/pre-bash-nag.nu",
        10,
    ),
    HookRegistration::active(
        "pre-commit-gate",
        CLAUDE,
        "PreToolUse",
        "Bash",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/pre-commit-gate.nu",
        30,
    ),
    HookRegistration::active(
        "moa",
        CLAUDE,
        "PreToolUse",
        "Bash",
        "godmode hook run moa",
        10,
    ),
    HookRegistration::active(
        "brainstorm",
        CLAUDE,
        "PreToolUse",
        "Write",
        "godmode hook run brainstorm",
        10,
    ),
    HookRegistration::active(
        "check-blocked",
        CLAUDE,
        "PostToolUse",
        "Agent",
        "bash $CLAUDE_PLUGIN_ROOT/hooks/scripts/check-blocked.sh",
        10,
    ),
    HookRegistration::active(
        "parallel-agents",
        CLAUDE,
        "PostToolUse",
        "Agent",
        "godmode hook run parallel-agents",
        10,
    ),
    HookRegistration::active(
        "task-done-sync",
        CLAUDE,
        "PostToolUse",
        "Bash",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/task-done-sync.nu",
        15,
    ),
    HookRegistration::active(
        "post-pipeline-step",
        CLAUDE,
        "PostToolUse",
        "Bash",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-pipeline-step.nu",
        10,
    ),
    HookRegistration::active(
        "auto-block",
        CLAUDE,
        "PostToolUse",
        "Bash",
        "godmode hook run auto-block",
        10,
    ),
    HookRegistration::active(
        "ci-fix",
        CLAUDE,
        "PostToolUse",
        "Bash",
        "godmode hook run ci-fix",
        15,
    ),
    HookRegistration::active(
        "code-review",
        CLAUDE,
        "PostToolUse",
        "Bash",
        "godmode hook run code-review",
        10,
    ),
    HookRegistration::active(
        "wave-integration",
        CLAUDE,
        "PostToolUse",
        "Bash",
        "godmode hook run wave-integration",
        10,
    ),
    HookRegistration::active(
        "post-json-write",
        CLAUDE,
        "PostToolUse",
        "Write",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-json-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-toml-write",
        CLAUDE,
        "PostToolUse",
        "Write",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-toml-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-yaml-write",
        CLAUDE,
        "PostToolUse",
        "Write",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-yaml-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-nu-write",
        CLAUDE,
        "PostToolUse",
        "Write",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-nu-check.nu",
        10,
    ),
    HookRegistration::active(
        "post-write-plan-ingest",
        CLAUDE,
        "PostToolUse",
        "Write",
        "rust-script $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-write-plan-ingest.rs",
        15,
    ),
    HookRegistration::active(
        "post-json-edit",
        CLAUDE,
        "PostToolUse",
        "Edit",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-json-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-toml-edit",
        CLAUDE,
        "PostToolUse",
        "Edit",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-toml-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-yaml-edit",
        CLAUDE,
        "PostToolUse",
        "Edit",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-yaml-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-nu-edit",
        CLAUDE,
        "PostToolUse",
        "Edit",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-nu-check.nu",
        10,
    ),
    HookRegistration::active(
        "stop-guard",
        CLAUDE,
        "Stop",
        "*",
        "godmode hook run stop-guard",
        15,
    ),
    HookRegistration::active(
        "memory-bank-update-remind",
        CLAUDE,
        "Stop",
        "*",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/memory-bank-update-remind.nu",
        10,
    ),
    HookRegistration::active(
        "introspection",
        CLAUDE,
        "Stop",
        "*",
        "godmode hook run introspection",
        20,
    ),
    HookRegistration::superseded(
        "session-start-nu",
        "session-start",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/session-start.nu",
    ),
    HookRegistration::superseded(
        "post-write-plan-ingest-nu",
        "post-write-plan-ingest",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-write-plan-ingest.nu",
    ),
    HookRegistration::superseded(
        "post-bash-auto-block",
        "auto-block",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-bash-auto-block.nu",
    ),
    HookRegistration::superseded(
        "stop-guard-nu",
        "stop-guard",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/stop-guard.nu",
    ),
    HookRegistration::internal(
        "godmode-trace",
        "rust-script $CLAUDE_PLUGIN_ROOT/hooks/scripts/godmode-trace.rs",
    ),
];

pub fn generate_manifest(client: HookClient) -> serde_json::Value {
    let mut events = serde_json::Map::new();
    for entry in REGISTRY
        .iter()
        .filter(|entry| entry.status == HookStatus::Active && entry.client == Some(client))
    {
        let event = entry.event.expect("active hook event");
        let matcher = entry.matcher.expect("active hook matcher");
        let groups = events
            .entry(event.to_string())
            .or_insert_with(|| serde_json::Value::Array(Vec::new()))
            .as_array_mut()
            .expect("event groups are arrays");
        let hook = serde_json::json!({"type": "command", "command": entry.command, "timeout": entry.timeout.expect("active hook timeout")});
        if let Some(group) = groups.iter_mut().find(|group| group["matcher"] == matcher) {
            group["hooks"]
                .as_array_mut()
                .expect("hooks array")
                .push(hook);
        } else {
            groups.push(serde_json::json!({"matcher": matcher, "hooks": [hook]}));
        }
    }
    serde_json::json!({"hooks": events})
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CoverageIssueKind {
    Unregistered,
    Missing,
    Duplicate,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CoverageIssue {
    pub kind: CoverageIssueKind,
    pub hook: String,
    pub detail: String,
}

pub fn diagnose_coverage(
    root: &Path,
    client: HookClient,
    registry: &[HookRegistration],
    manifest: &serde_json::Value,
) -> Vec<CoverageIssue> {
    let actual = crate::integrations::hook_runner::list_hooks_from_json(manifest);
    let mut issues = Vec::new();
    let known: BTreeMap<_, _> = registry
        .iter()
        .map(|entry| (entry.command, entry))
        .collect();
    let mut seen = BTreeSet::new();
    for (event, matcher, command) in &actual {
        if !seen.insert((event.as_str(), matcher.as_str(), command.as_str())) {
            issues.push(issue(
                CoverageIssueKind::Duplicate,
                command,
                format!("duplicate {event}/{matcher} manifest entry"),
            ));
        }
        match known.get(command.as_str()) {
            None => issues.push(issue(
                CoverageIssueKind::Unregistered,
                command,
                "manifest command is absent from the registry",
            )),
            Some(entry) if entry.status == HookStatus::Superseded => issues.push(issue(
                CoverageIssueKind::Superseded,
                command,
                format!("superseded by {}", entry.superseded_by.unwrap_or("unknown")),
            )),
            _ => {}
        }
    }
    for entry in registry
        .iter()
        .filter(|entry| entry.status == HookStatus::Active && entry.client == Some(client))
    {
        let present = actual.iter().any(|(event, matcher, command)| {
            Some(event.as_str()) == entry.event
                && Some(matcher.as_str()) == entry.matcher
                && command == entry.command
        });
        if !present {
            issues.push(issue(
                CoverageIssueKind::Missing,
                entry.id,
                "active registry entry is absent from the manifest",
            ));
        }
        if let Some(path) = script_path(entry.command)
            && !root.join(path).is_file()
        {
            issues.push(issue(
                CoverageIssueKind::Missing,
                entry.id,
                format!("script does not exist: {path}"),
            ));
        }
    }
    let registered_paths: BTreeSet<_> = registry
        .iter()
        .filter_map(|entry| script_path(entry.command))
        .collect();
    if let Ok(entries) = std::fs::read_dir(root.join("hooks/scripts")) {
        for path in entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
        {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");
            if !registered_paths.contains(relative.as_str()) {
                issues.push(issue(
                    CoverageIssueKind::Unregistered,
                    &relative,
                    "hook script is absent from the registry",
                ));
            }
        }
    }
    issues
}

pub fn diagnose_repository(root: &Path, client: HookClient) -> anyhow::Result<Vec<CoverageIssue>> {
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("hooks/hooks.json"))?)?;
    Ok(diagnose_coverage(root, client, REGISTRY, &manifest))
}

fn script_path(command: &str) -> Option<&str> {
    command.split("$CLAUDE_PLUGIN_ROOT/").nth(1)
}
fn issue(
    kind: CoverageIssueKind,
    hook: impl Into<String>,
    detail: impl Into<String>,
) -> CoverageIssue {
    CoverageIssue {
        kind,
        hook: hook.into(),
        detail: detail.into(),
    }
}
