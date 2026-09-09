//! Architectural boundary tests for the CLI entry point.

#[test]
fn main_delegates_command_behavior_to_adapters() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let main = std::fs::read_to_string(manifest.join("src/main.rs")).unwrap();
    let commands = std::fs::read_to_string(manifest.join("src/commands/mod.rs")).unwrap();

    assert!(
        main.contains("commands::dispatch(root, json, sarif, cli.cmd)"),
        "main must delegate parsed commands to the adapter boundary"
    );
    assert!(
        !main.contains("mod command_dispatch"),
        "main must not own command-family dispatch"
    );
    assert!(
        !main.contains("std::fs")
            && !main.contains("println!")
            && !main.contains("std::process::Command"),
        "main must not own formatting, filesystem, or subprocess policy"
    );
    assert!(
        commands.contains("mod command;")
            && commands.contains("mod session;")
            && commands.contains("mod workspace;"),
        "command adapters must separate command, session, and workspace policies"
    );
}
