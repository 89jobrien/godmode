//! Governance policy resolution, checking, listing, and audit.

use anyhow::Result;
use godmode_core::{insights, policy};
use std::path::Path;

use crate::PolicyCmdAction;

pub fn run_policy_action(root: &Path, json: bool, action: PolicyCmdAction) -> Result<()> {
    match action {
        PolicyCmdAction::Resolve { agent, level } => {
            let level_parsed = level
                .as_deref()
                .map(|l| l.parse::<policy::GovernanceLevel>())
                .transpose()?;
            let resolved = policy::resolve(root, &agent, level_parsed.as_ref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resolved)?);
            } else {
                println!("Agent:    {}", resolved.agent);
                println!("Category: {}", resolved.category);
                println!("Level:    {}", resolved.level);
                println!("Sources:  {}", resolved.sources.join(" + "));
                println!();
                let p = &resolved.policy;
                if p.allowed_tools.is_empty() {
                    println!("Allowed tools: (all)");
                } else {
                    println!("Allowed tools: {}", p.allowed_tools.join(", "));
                }
                if !p.blocked_tools.is_empty() {
                    println!("Blocked tools: {}", p.blocked_tools.join(", "));
                }
                println!("Max calls/dispatch: {}", p.max_calls_per_dispatch);
                if !p.require_human_approval.is_empty() {
                    println!("Require approval: {}", p.require_human_approval.join(", "));
                }
                println!();
                println!("Subagent constraints:");
                println!("  max_concurrent: {}", p.subagent.max_concurrent);
                println!("  verify_branch:  {}", p.subagent.must_verify_branch);
                println!("  no_main:        {}", p.subagent.no_commit_to_main);
                println!("  max_retries:    {}", p.subagent.max_retries_on_failure);
                println!(
                    "  require_commit: {}",
                    p.subagent.require_commit_before_done
                );
                if !p.subagent.blocked_flags.is_empty() {
                    println!("  blocked_flags:  {}", p.subagent.blocked_flags.join(", "));
                }
            }
        }
        PolicyCmdAction::Check {
            agent,
            tool,
            input,
            level,
        } => {
            let level_parsed = level
                .as_deref()
                .map(|l| l.parse::<policy::GovernanceLevel>())
                .transpose()?;
            let resolved = policy::resolve(root, &agent, level_parsed.as_ref())?;
            let result = policy::check_tool(&resolved.policy, &tool, input.as_deref());
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                let symbol = match result.action {
                    policy::PolicyAction::Allow => "ALLOW",
                    policy::PolicyAction::Deny => "DENY",
                    policy::PolicyAction::Review => "REVIEW",
                };
                println!("{symbol}: {}", result.reason);
            }
            // Exit 1 on deny for scripting
            if result.action == policy::PolicyAction::Deny {
                std::process::exit(1);
            }
        }
        PolicyCmdAction::List => {
            let index = policy::list_policies(root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&index)?);
            } else {
                if let Some(ref d) = index.default {
                    println!("Default: {} (level: {})", d.name, d.level);
                }
                if !index.categories.is_empty() {
                    println!();
                    println!("Categories:");
                    let mut cats: Vec<_> = index.categories.keys().collect();
                    cats.sort();
                    for cat in cats {
                        let p = &index.categories[cat];
                        println!(
                            "  {cat:<8}  tools: {}  max: {}",
                            if p.allowed_tools.is_empty() {
                                "(all)".to_string()
                            } else {
                                p.allowed_tools.join(",")
                            },
                            p.max_calls_per_dispatch,
                        );
                    }
                }
                if !index.levels.is_empty() {
                    println!();
                    println!("Levels:");
                    for level_name in &["open", "standard", "strict", "locked"] {
                        if let Some(p) = index.levels.get(*level_name) {
                            println!("  {:<10}  max: {}", level_name, p.max_calls_per_dispatch,);
                        }
                    }
                }
            }
        }
        PolicyCmdAction::Audit { date } => {
            let date_str = date.unwrap_or_else(|| insights::today().to_string());
            let events = policy::read_audit_events(root, Some(&date_str))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&events)?);
            } else if events.is_empty() {
                println!("No governance events for {date_str}.");
            } else {
                let denied = events.iter().filter(|e| e.action == "denied").count();
                let reviews = events
                    .iter()
                    .filter(|e| e.action == "review" || e.action == "warn")
                    .count();
                let allowed = events.iter().filter(|e| e.action == "allowed").count();
                println!("Governance audit for {date_str}:");
                println!(
                    "  {} events: {} denied, {} review, {} allowed",
                    events.len(),
                    denied,
                    reviews,
                    allowed,
                );
                println!();
                for ev in &events {
                    if ev.action == "denied" || ev.action == "review" || ev.action == "warn" {
                        println!(
                            "  [{action}] {agent} -> {tool}: {reason}",
                            action = ev.action.to_uppercase(),
                            agent = ev.agent_id,
                            tool = ev.tool_name,
                            reason = ev.reason,
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
