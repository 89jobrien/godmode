//! Declarative hook metadata, manifest projection, and coverage diagnostics.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HookClient {
    Claude,
    /// OpenCode. Hooks are TypeScript plugin functions rather than a
    /// subprocess manifest, so only `PreToolUse`/`PostToolUse` entries port.
    /// See [`UNSUPPORTED_OPENCODE_EVENTS`].
    OpenCode,
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
    /// Creates an active client hook registration.
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
    /// Creates a retired registration that points to its replacement.
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
    /// Creates a registration for a non-client-facing internal hook.
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
const OPENCODE: HookClient = HookClient::OpenCode;

/// Claude events with no OpenCode equivalent.
///
/// OpenCode exposes no session-start or turn-end lifecycle hook, so
/// registrations on these events cannot be projected. `PreToolUse` maps to
/// `tool.execute.before` and `PostToolUse` to `tool.execute.after`; the
/// `event` hook fires for many unrelated events and is not a session-start
/// substitute.
pub const UNSUPPORTED_OPENCODE_EVENTS: &[&str] = &["SessionStart", "Stop"];

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
    // OpenCode mirrors only the tool-lifecycle events. `SessionStart` and
    // `Stop` are listed in UNSUPPORTED_OPENCODE_EVENTS instead of registered
    // here, so the OpenCode plugin reports them as unmapped rather than
    // silently dropping them.
    HookRegistration::active(
        "pre-agent-task-context",
        OPENCODE,
        "PreToolUse",
        "task",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/pre-agent-task-context.nu",
        10,
    ),
    HookRegistration::active(
        "agent-governance",
        OPENCODE,
        "PreToolUse",
        "task",
        "godmode hook run agent-governance",
        10,
    ),
    HookRegistration::active(
        "pre-bash-nag",
        OPENCODE,
        "PreToolUse",
        "bash",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/pre-bash-nag.nu",
        10,
    ),
    HookRegistration::active(
        "pre-commit-gate",
        OPENCODE,
        "PreToolUse",
        "bash",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/pre-commit-gate.nu",
        30,
    ),
    HookRegistration::active(
        "moa",
        OPENCODE,
        "PreToolUse",
        "bash",
        "godmode hook run moa",
        10,
    ),
    HookRegistration::active(
        "brainstorm",
        OPENCODE,
        "PreToolUse",
        "write",
        "godmode hook run brainstorm",
        10,
    ),
    HookRegistration::active(
        "check-blocked",
        OPENCODE,
        "PostToolUse",
        "task",
        "bash $CLAUDE_PLUGIN_ROOT/hooks/scripts/check-blocked.sh",
        10,
    ),
    HookRegistration::active(
        "parallel-agents",
        OPENCODE,
        "PostToolUse",
        "task",
        "godmode hook run parallel-agents",
        10,
    ),
    HookRegistration::active(
        "task-done-sync",
        OPENCODE,
        "PostToolUse",
        "bash",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/task-done-sync.nu",
        15,
    ),
    HookRegistration::active(
        "post-pipeline-step",
        OPENCODE,
        "PostToolUse",
        "bash",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-pipeline-step.nu",
        10,
    ),
    HookRegistration::active(
        "auto-block",
        OPENCODE,
        "PostToolUse",
        "bash",
        "godmode hook run auto-block",
        10,
    ),
    HookRegistration::active(
        "ci-fix",
        OPENCODE,
        "PostToolUse",
        "bash",
        "godmode hook run ci-fix",
        15,
    ),
    HookRegistration::active(
        "code-review",
        OPENCODE,
        "PostToolUse",
        "bash",
        "godmode hook run code-review",
        10,
    ),
    HookRegistration::active(
        "wave-integration",
        OPENCODE,
        "PostToolUse",
        "bash",
        "godmode hook run wave-integration",
        10,
    ),
    HookRegistration::active(
        "post-json-write",
        OPENCODE,
        "PostToolUse",
        "write",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-json-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-toml-write",
        OPENCODE,
        "PostToolUse",
        "write",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-toml-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-yaml-write",
        OPENCODE,
        "PostToolUse",
        "write",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-yaml-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-nu-write",
        OPENCODE,
        "PostToolUse",
        "write",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-nu-check.nu",
        10,
    ),
    HookRegistration::active(
        "post-write-plan-ingest",
        OPENCODE,
        "PostToolUse",
        "write",
        "rust-script $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-write-plan-ingest.rs",
        15,
    ),
    HookRegistration::active(
        "post-json-edit",
        OPENCODE,
        "PostToolUse",
        "edit",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-json-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-toml-edit",
        OPENCODE,
        "PostToolUse",
        "edit",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-toml-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-yaml-edit",
        OPENCODE,
        "PostToolUse",
        "edit",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-yaml-validate.nu",
        10,
    ),
    HookRegistration::active(
        "post-nu-edit",
        OPENCODE,
        "PostToolUse",
        "edit",
        "nu $CLAUDE_PLUGIN_ROOT/hooks/scripts/post-nu-check.nu",
        10,
    ),
];

/// Builds a hook manifest from active registrations for the client.
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

/// Maps a Claude matcher to the OpenCode tool name it gates.
fn opencode_tool(matcher: &str) -> String {
    match matcher {
        "Agent" => "task".to_string(),
        "Bash" => "bash".to_string(),
        "Write" => "write".to_string(),
        "Edit" => "edit".to_string(),
        other => other.to_string(),
    }
}

/// Claude event to the OpenCode plugin hook that replaces it.
fn opencode_hook_name(event: &str) -> Option<&'static str> {
    match event {
        "PreToolUse" => Some("tool.execute.before"),
        "PostToolUse" => Some("tool.execute.after"),
        _ => None,
    }
}

/// Active OpenCode registrations that map onto a plugin hook, in registry order.
pub fn opencode_hooks(event: &str) -> Vec<&'static HookRegistration> {
    REGISTRY
        .iter()
        .filter(|entry| {
            entry.status == HookStatus::Active
                && entry.client == Some(HookClient::OpenCode)
                && opencode_hook_name(entry.event.unwrap_or_default()) == Some(event)
        })
        .collect()
}

/// Claude hooks with no OpenCode counterpart, as `(id, claude_event, command)`.
pub fn opencode_unmapped() -> Vec<(&'static str, &'static str, &'static str)> {
    REGISTRY
        .iter()
        .filter(|entry| {
            entry.status == HookStatus::Active
                && entry.client == Some(HookClient::Claude)
                && UNSUPPORTED_OPENCODE_EVENTS.contains(&entry.event.unwrap_or_default())
        })
        .map(|entry| (entry.id, entry.event.unwrap_or("?"), entry.command))
        .collect()
}

/// Renders `.opencode/plugins/godmode.ts` from the registry.
///
/// Generated rather than hand-written so the OpenCode hook surface cannot drift
/// from [`REGISTRY`]. Hook scripts keep the Claude `$CLAUDE_PLUGIN_ROOT`
/// contract; the plugin resolves that variable itself.
pub fn generate_opencode_plugin() -> String {
    let before = opencode_hooks("tool.execute.before");
    let after = opencode_hooks("tool.execute.after");
    let unmapped = opencode_unmapped();

    let mut unmapped_json = serde_json::Value::Array(
        unmapped
            .iter()
            .map(|(id, event, command)| {
                serde_json::json!({"id": id, "event": event, "command": command})
            })
            .collect(),
    );
    unmapped_json
        .as_array_mut()
        .expect("array")
        .sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));

    // `before` hooks may block, so each accumulates a decision; `after` hooks
    // are advisory and just fire.
    let render = |entries: &[&'static HookRegistration], blocking: bool| {
        let mut by_tool: BTreeMap<String, Vec<(&str, u64)>> = BTreeMap::new();
        for entry in entries {
            let tool = opencode_tool(entry.matcher.unwrap_or("*"));
            by_tool
                .entry(tool)
                .or_default()
                .push((entry.command, entry.timeout.unwrap_or(10)));
        }
        let mut out = String::new();
        for (tool, hooks) in by_tool {
            out.push_str(&format!("      if (tool === {tool:?}) {{\n"));
            for (command, timeout) in hooks {
                if blocking {
                    out.push_str(&format!(
                        "        verdict ??= await decide({command:?}, {timeout}, payload)\n"
                    ));
                } else {
                    out.push_str(&format!(
                        "        await run({command:?}, {timeout}, payload)\n"
                    ));
                }
            }
            out.push_str("      }\n");
        }
        out
    };

    format!(
        r#"// Generated by `godmode hook generate --client opencode`. Do not edit.
//
// Ports the godmode hook surface to OpenCode. Claude's `PreToolUse` and
// `PostToolUse` become `tool.execute.before` / `tool.execute.after`.
//
// Claude's `SessionStart` and `Stop` have no OpenCode equivalent, so the
// hooks on those events do not run here. UNMAPPED lists them; the warning
// below fires once at plugin load rather than failing silently.

import type {{ Plugin }} from "@opencode-ai/plugin"

const PLUGIN_ROOT =
  process.env.CLAUDE_PLUGIN_ROOT ?? process.env.GODMODE_PLUGIN_ROOT ?? process.cwd()

/** Claude hooks with no OpenCode lifecycle counterpart. */
const UNMAPPED: {{ id: string; event: string; command: string }}[] = {unmapped}

/** A Claude PreToolUse decision, as printed by the hook scripts. */
type Decision = {{ decision: "approve" | "block"; reason?: string }}

/**
 * Run a godmode hook script with the OpenCode tool payload on stdin and
 * return its Claude-style decision.
 *
 * Scripts keep the Claude contract: JSON on stdin, a
 * `{{"decision": "approve" | "block", ...}}` document on stdout. Unparseable
 * or absent output is treated as approve, because a hook that cannot be
 * understood must not wedge the tool.
 */
async function decide(command: string, timeout: number, payload: unknown): Promise<Decision | undefined> {{
  const argv = command.replaceAll("$CLAUDE_PLUGIN_ROOT", PLUGIN_ROOT).split(" ")
  try {{
    const proc = Bun.spawn(argv, {{
      stdin: "pipe",
      stdout: "pipe",
      stderr: "pipe",
      env: {{ ...process.env, CLAUDE_PLUGIN_ROOT: PLUGIN_ROOT }},
    }})
    proc.stdin.write(JSON.stringify(payload))
    proc.stdin.end()
    const timer = setTimeout(() => proc.kill(), timeout * 1000)
    const [, stdout] = await Promise.all([proc.exited, new Response(proc.stdout).text()])
    clearTimeout(timer)
    const verdict = stdout.match(/\{{[^{{}}]*"decision"[^{{}}]*\}}/)
    if (!verdict) return undefined
    return JSON.parse(verdict[0]) as Decision
  }} catch (error) {{
    console.error(`[godmode] hook failed: ${{command}} (${{error}})`)
    return undefined
  }}
}}

/** PostToolUse hooks are advisory: they report, they never block. */
async function run(command: string, timeout: number, payload: unknown): Promise<void> {{
  await decide(command, timeout, payload)
}}

export const GodmodePlugin: Plugin = async () => {{
  if (UNMAPPED.length > 0) {{
    console.warn(
      `[godmode] ${{UNMAPPED.length}} Claude hook(s) have no OpenCode equivalent and will not run: ` +
        UNMAPPED.map((h) => `${{h.id}} (${{h.event}})`).join(", "),
    )
  }}

  return {{
    "tool.execute.before": async (input, output) => {{
      const tool = input.tool
      const payload = {{ ...input, ...output }}
      let verdict: Decision | undefined
{before}      // A blocking PreToolUse hook (pre-commit-gate) denies the call by
      // throwing, which is OpenCode's equivalent of Claude's block decision.
      if (verdict?.decision === "block") {{
        throw new Error(verdict.reason ?? "[godmode] blocked by hook")
      }}
    }},
    "tool.execute.after": async (input, output) => {{
      const tool = input.tool
      const payload = {{ ...input, ...output }}
{after}    }},
  }}
}}
"#,
        unmapped = serde_json::to_string_pretty(&unmapped_json).unwrap_or_else(|_| "[]".into()),
        before = render(&before, true),
        after = render(&after, false),
    )
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

/// Reports manifest and script coverage issues against a hook registry.
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

/// Diagnoses `hooks/hooks.json` against the canonical registry.
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

#[cfg(test)]
mod opencode_tests {
    use super::*;

    #[test]
    fn maps_claude_matchers_to_opencode_tool_names() {
        assert_eq!(opencode_tool("Agent"), "task");
        assert_eq!(opencode_tool("Bash"), "bash");
        assert_eq!(opencode_tool("Write"), "write");
        assert_eq!(opencode_tool("Edit"), "edit");
        assert_eq!(opencode_tool("CustomTool"), "CustomTool");
    }

    #[test]
    fn only_tool_lifecycle_events_map_to_plugin_hooks() {
        assert_eq!(
            opencode_hook_name("PreToolUse"),
            Some("tool.execute.before")
        );
        assert_eq!(
            opencode_hook_name("PostToolUse"),
            Some("tool.execute.after")
        );
        assert_eq!(opencode_hook_name("SessionStart"), None);
        assert_eq!(opencode_hook_name("Stop"), None);
    }

    #[test]
    fn before_and_after_hooks_are_both_populated() {
        assert!(!opencode_hooks("tool.execute.before").is_empty());
        assert!(!opencode_hooks("tool.execute.after").is_empty());
    }

    #[test]
    fn no_opencode_registration_uses_an_unsupported_event() {
        for entry in REGISTRY
            .iter()
            .filter(|e| e.client == Some(HookClient::OpenCode))
        {
            let event = entry.event.expect("active hook event");
            assert!(
                opencode_hook_name(event).is_some(),
                "OpenCode hook '{}' is registered on unmappable event {event}",
                entry.id
            );
        }
    }

    #[test]
    fn unmapped_hooks_are_exactly_the_session_and_stop_claude_hooks() {
        let unmapped = opencode_unmapped();
        assert_eq!(unmapped.len(), 6, "expected 6 unmapped hooks: {unmapped:?}");
        for (id, event, _) in &unmapped {
            assert!(
                UNSUPPORTED_OPENCODE_EVENTS.contains(event),
                "{id} reported unmapped but sits on {event}"
            );
        }
    }

    #[test]
    fn generated_plugin_is_internally_consistent() {
        let ts = generate_opencode_plugin();

        let opens = ts.matches('{').count();
        let closes = ts.matches('}').count();
        assert_eq!(opens, closes, "unbalanced braces in generated plugin");

        for event in ["tool.execute.before", "tool.execute.after"] {
            for entry in opencode_hooks(event) {
                // A hook registered for two matchers (the validators run on
                // both write and edit) emits one call per tool branch.
                // Before-hooks accumulate a verdict; after-hooks just run.
                let needle = if event == "tool.execute.before" {
                    format!("decide({:?}, ", entry.command)
                } else {
                    format!("run({:?}, ", entry.command)
                };
                let expected = REGISTRY
                    .iter()
                    .filter(|other| {
                        other.status == HookStatus::Active
                            && other.client == Some(HookClient::OpenCode)
                            && opencode_hook_name(other.event.unwrap_or_default()) == Some(event)
                            && other.command == entry.command
                    })
                    .count();
                assert_eq!(
                    ts.matches(&needle).count(),
                    expected,
                    "hook '{}' should emit {expected} call(s) in {event}",
                    entry.id
                );
            }
        }

        for (id, _, command) in opencode_unmapped() {
            assert!(
                ts.contains(&format!("{id:?}")),
                "unmapped hook {id} absent from generated plugin"
            );
            assert!(
                ts.contains(command),
                "unmapped hook {id} command absent from generated plugin"
            );
        }
    }

    #[test]
    fn generated_plugin_does_not_claim_unmapped_hooks_are_active() {
        let ts = generate_opencode_plugin();
        assert!(!ts.contains("\"SessionStart\": ["));
        assert!(!ts.contains("\"Stop\": ["));
    }

    /// Claude PreToolUse hooks block by printing a decision document, not by
    /// exit code. If the OpenCode projection drops stdout, every blocking
    /// gate (notably pre-commit-gate) silently degrades to advisory.
    #[test]
    fn before_hooks_read_the_decision_document() {
        let ts = generate_opencode_plugin();
        assert!(
            ts.contains("JSON.parse(verdict[0]) as Decision"),
            "plugin must parse the hook decision document"
        );
        assert!(
            ts.contains("stdout.match"),
            "plugin must capture hook stdout"
        );
        assert!(
            ts.contains("throw new Error(verdict.reason"),
            "a block decision must deny the tool call"
        );
    }

    #[test]
    fn only_before_hooks_accumulate_a_verdict() {
        let ts = generate_opencode_plugin();
        let before_block = ts
            .find("verdict ??= await decide")
            .expect("before hooks use decide");
        let after_section = ts
            .find("\"tool.execute.after\": async")
            .expect("after hook present");
        assert!(
            before_block < after_section,
            "verdict accumulation belongs to the before hook only"
        );
        let after_body = &ts[after_section..];
        assert!(
            !after_body.contains("verdict ??="),
            "PostToolUse hooks are advisory and must not block:\n{after_body}"
        );
    }

    /// An unreadable decision must fall through to approve, or a malformed
    /// script would wedge every tool call.
    #[test]
    fn undecidable_hook_falls_through_to_approve() {
        let ts = generate_opencode_plugin();
        assert!(
            ts.contains("return undefined"),
            "a hook that cannot be parsed must not block"
        );
    }
}
