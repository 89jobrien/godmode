//! Property-based tests for `config::Config` TOML parsing.
//!
//! Run with: cargo nextest run --test prop_config

use godmode_core::config::{Config, Handoff, Integrations};
use proptest::prelude::*;
use tempfile::TempDir;

// ── Strategies ────────────────────────────────────────────────────────────────

fn arb_project_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,30}".prop_map(|s| s)
}

fn arb_max_commits() -> impl Strategy<Value = usize> {
    0usize..=1000
}

prop_compose! {
    fn arb_integrations()(doob in any::<bool>(), hj in any::<bool>(),
                          crux in any::<bool>(), rx in any::<bool>()) -> Integrations {
        Integrations { doob, hj, crux, rx }
    }
}

prop_compose! {
    fn arb_handoff()(enabled in any::<bool>(), doob_sync in any::<bool>(),
                     max_commits in arb_max_commits()) -> Handoff {
        Handoff { enabled, doob_sync, max_commits }
    }
}

prop_compose! {
    fn arb_config()(
        project_name in prop::option::of(arb_project_name()),
        integrations in arb_integrations(),
        handoff in arb_handoff(),
    ) -> Config {
        Config { project_name, integrations, handoff }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn write_and_load(dir: &std::path::Path, toml: &str) -> Config {
    std::fs::write(dir.join(".godmode.toml"), toml).unwrap();
    Config::load(dir)
}

// ── Properties ────────────────────────────────────────────────────────────────

proptest! {
    /// Any Config serialises to TOML and parses back to an identical value.
    #[test]
    fn prop_config_toml_roundtrip(cfg in arb_config()) {
        let dir = TempDir::new().unwrap();
        let toml = toml::to_string(&cfg).expect("serialize");
        let restored = write_and_load(dir.path(), &toml);

        prop_assert_eq!(&restored.project_name, &cfg.project_name);
        prop_assert_eq!(restored.integrations.doob,       cfg.integrations.doob);
        prop_assert_eq!(restored.integrations.hj,         cfg.integrations.hj);
        prop_assert_eq!(restored.integrations.crux,       cfg.integrations.crux);
        prop_assert_eq!(restored.integrations.rx,         cfg.integrations.rx);
        prop_assert_eq!(restored.handoff.enabled,         cfg.handoff.enabled);
        prop_assert_eq!(restored.handoff.doob_sync,       cfg.handoff.doob_sync);
        prop_assert_eq!(restored.handoff.max_commits,     cfg.handoff.max_commits);
    }

    /// When only `project_name` is set, all other fields keep their defaults.
    #[test]
    fn prop_partial_toml_preserves_defaults(name in arb_project_name()) {
        let dir = TempDir::new().unwrap();
        let toml = format!("project_name = \"{name}\"\n");
        let cfg = write_and_load(dir.path(), &toml);
        let defaults = Config::default();

        prop_assert_eq!(cfg.project_name.as_deref(), Some(name.as_str()));
        prop_assert_eq!(cfg.integrations.doob,   defaults.integrations.doob);
        prop_assert_eq!(cfg.integrations.hj,     defaults.integrations.hj);
        prop_assert_eq!(cfg.integrations.crux,   defaults.integrations.crux);
        prop_assert_eq!(cfg.integrations.rx,     defaults.integrations.rx);
        prop_assert_eq!(cfg.handoff.enabled,     defaults.handoff.enabled);
        prop_assert_eq!(cfg.handoff.doob_sync,   defaults.handoff.doob_sync);
        prop_assert_eq!(cfg.handoff.max_commits, defaults.handoff.max_commits);
    }

    /// `project_name` config override always wins over directory-name fallback.
    #[test]
    fn prop_project_name_override_wins(
        name in arb_project_name(),
        dir_name in "[a-z]{3,10}",
    ) {
        prop_assume!(name != dir_name);
        let dir = TempDir::new().unwrap();
        let fake_root = dir.path().join(&dir_name);
        std::fs::create_dir(&fake_root).unwrap();
        let toml = format!("project_name = \"{name}\"\n");
        std::fs::write(fake_root.join(".godmode.toml"), &toml).unwrap();
        let cfg = Config::load(&fake_root);
        let resolved = cfg.project_name(&fake_root);
        prop_assert_eq!(resolved, name);
    }

    /// Any valid `max_commits` value round-trips exactly.
    #[test]
    fn prop_max_commits_roundtrip(n in arb_max_commits()) {
        let dir = TempDir::new().unwrap();
        let toml = format!("[handoff]\nmax_commits = {n}\n");
        let cfg = write_and_load(dir.path(), &toml);
        prop_assert_eq!(cfg.handoff.max_commits, n);
    }

    /// Missing `.godmode.toml` always yields the struct default — never panics.
    #[test]
    fn prop_missing_file_never_panics(suffix in "[a-z]{4,12}") {
        let path = std::path::Path::new("/tmp").join(format!("gm-test-{suffix}"));
        // Deliberately do not create this directory.
        let cfg = Config::load(&path);
        let defaults = Config::default();
        prop_assert_eq!(cfg.integrations.doob,   defaults.integrations.doob);
        prop_assert_eq!(cfg.handoff.max_commits, defaults.handoff.max_commits);
    }
}
