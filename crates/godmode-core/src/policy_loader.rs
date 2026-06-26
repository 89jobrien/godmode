//! Policy loading, path resolution, and default/category/agent layering logic.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::agent;
use crate::policy::{
    AllowedToolsMode, GovernanceLevel, GovernancePolicy, PolicyIndex, ResolvedPolicy,
};

pub(crate) fn policies_dir(root: &Path) -> PathBuf {
    root.join("skills")
        .join("agent-governance")
        .join("policies")
}

pub(crate) fn load_policy(path: &Path) -> Result<GovernancePolicy> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading policy {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parsing policy YAML {}", path.display()))
}

/// Look up the category for an agent from `agents/cfg/<name>.cfg.yaml`.
pub fn agent_category(root: &Path, agent_name: &str) -> Option<String> {
    let cfg_dir = root.join("agents").join("cfg");

    for candidate in [
        cfg_dir.join(format!("{agent_name}.cfg.yaml")),
        cfg_dir.join(format!("{agent_name}-agent.cfg.yaml")),
    ] {
        if candidate.exists()
            && let Ok(def) = agent::load(&candidate)
            && !def.category.is_empty()
        {
            return Some(def.category);
        }
    }

    if let Ok(entries) = std::fs::read_dir(&cfg_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml")
                && let Ok(def) = agent::load(&path)
                && def.name == agent_name
                && !def.category.is_empty()
            {
                return Some(def.category);
            }
        }
    }

    None
}

/// Resolve the effective governance policy for an agent.
pub fn resolve(
    root: &Path,
    agent_name: &str,
    level_override: Option<&GovernanceLevel>,
) -> Result<ResolvedPolicy> {
    let dir = policies_dir(root);
    let default_path = dir.join("default.yaml");

    if !default_path.exists() {
        bail!("no default governance policy at {}", default_path.display());
    }

    let mut policy = load_policy(&default_path)?;
    let mut sources = vec!["default.yaml".to_string()];

    let category = agent_category(root, agent_name).unwrap_or_default();
    if !category.is_empty() {
        let cat_path = dir.join("by-category").join(format!("{category}.yaml"));
        if cat_path.exists() {
            let cat_policy = load_policy(&cat_path)?;
            compose(&mut policy, &cat_policy, AllowedToolsMode::Replace);
            sources.push(format!("by-category/{category}.yaml"));
        }
    }

    let effective_level = match level_override {
        Some(l) => l.to_string(),
        None => {
            if policy.level.is_empty() {
                "standard".to_string()
            } else {
                policy.level.clone()
            }
        }
    };
    let level_path = dir.join("levels").join(format!("{effective_level}.yaml"));
    if level_path.exists() {
        let level_policy = load_policy(&level_path)?;
        compose(&mut policy, &level_policy, AllowedToolsMode::Intersect);
        sources.push(format!("levels/{effective_level}.yaml"));
    }

    Ok(ResolvedPolicy {
        policy,
        agent: agent_name.to_string(),
        category,
        level: effective_level,
        sources,
    })
}

/// List all available policies (default + categories + levels).
pub fn list_policies(root: &Path) -> Result<PolicyIndex> {
    let dir = policies_dir(root);
    let mut index = PolicyIndex {
        default: None,
        categories: HashMap::new(),
        levels: HashMap::new(),
    };

    let default_path = dir.join("default.yaml");
    if default_path.exists() {
        index.default = Some(load_policy(&default_path)?);
    }

    let cat_dir = dir.join("by-category");
    if cat_dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&cat_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if let Ok(p) = load_policy(&path) {
                    index.categories.insert(name, p);
                }
            }
        }
    }

    let level_dir = dir.join("levels");
    if level_dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&level_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if let Ok(p) = load_policy(&path) {
                    index.levels.insert(name, p);
                }
            }
        }
    }

    Ok(index)
}

/// Compose an overlay on top of a base policy.
pub(crate) fn compose(
    base: &mut GovernancePolicy,
    overlay: &GovernancePolicy,
    mode: AllowedToolsMode,
) {
    for tool in &overlay.blocked_tools {
        if !base.blocked_tools.contains(tool) {
            base.blocked_tools.push(tool.clone());
        }
    }

    for pat in &overlay.blocked_patterns {
        if !base.blocked_patterns.contains(pat) {
            base.blocked_patterns.push(pat.clone());
        }
    }

    for item in &overlay.require_human_approval {
        if !base.require_human_approval.contains(item) {
            base.require_human_approval.push(item.clone());
        }
    }

    if !overlay.allowed_tools.is_empty() {
        match mode {
            AllowedToolsMode::Replace => {
                base.allowed_tools.clone_from(&overlay.allowed_tools);
            }
            AllowedToolsMode::Intersect => {
                if base.allowed_tools.is_empty() {
                    base.allowed_tools.clone_from(&overlay.allowed_tools);
                } else {
                    base.allowed_tools
                        .retain(|t| overlay.allowed_tools.contains(t));
                }
            }
        }
    }

    base.max_calls_per_dispatch = base
        .max_calls_per_dispatch
        .min(overlay.max_calls_per_dispatch);

    base.subagent.max_concurrent = base
        .subagent
        .max_concurrent
        .min(overlay.subagent.max_concurrent);
    base.subagent.must_verify_branch |= overlay.subagent.must_verify_branch;
    base.subagent.no_commit_to_main |= overlay.subagent.no_commit_to_main;
    base.subagent.max_retries_on_failure = base
        .subagent
        .max_retries_on_failure
        .min(overlay.subagent.max_retries_on_failure);
    base.subagent.require_commit_before_done |= overlay.subagent.require_commit_before_done;
    for flag in &overlay.subagent.blocked_flags {
        if !base.subagent.blocked_flags.contains(flag) {
            base.subagent.blocked_flags.push(flag.clone());
        }
    }
}
