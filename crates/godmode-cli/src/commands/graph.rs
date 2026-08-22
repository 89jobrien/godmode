//! Task graph construction and rendering.

use anyhow::Result;
use godmode_core::{builder, graph};
use std::path::Path;

use crate::GraphAction;

pub fn run_graph_action(root: &Path, json: bool, action: GraphAction) -> Result<()> {
    match action {
        GraphAction::Build { input, vars } => {
            let summary = match input {
                Some(path) => {
                    let p = std::path::PathBuf::from(&path);
                    builder::build_from_file(root, &p, &vars)?
                }
                None => builder::build_interactive(root)?,
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                println!(
                    "Added {} task(s), {} dep(s) wired.",
                    summary.added, summary.wired
                );
                if !summary.findings.is_empty() {
                    for f in &summary.findings {
                        eprintln!("! {}", f);
                    }
                }
                if summary.next.is_empty() {
                    std::process::exit(1);
                }
            }
            Ok(())
        }
    }
}

pub fn run_visualize_graph(
    root: &Path,
    json: bool,
    format: String,
    out: Option<String>,
) -> Result<()> {
    let g = graph::load(root)?;
    let dot = graph::to_dot(&g);
    let content = match format.as_str() {
        "dot" => dot,
        "svg" => {
            // Try piping DOT through `dot -Tsvg`; degrade gracefully if missing.
            match std::process::Command::new("dot")
                .args(["-Tsvg"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::inherit())
                .spawn()
            {
                Ok(mut child) => {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.take() {
                        let mut stdin = stdin;
                        let _ = stdin.write_all(dot.as_bytes());
                    }
                    let output = child.wait_with_output()?;
                    if !output.status.success() {
                        anyhow::bail!("graphviz dot exited with {}", output.status);
                    }
                    String::from_utf8_lossy(&output.stdout).into_owned()
                }
                Err(_) => {
                    eprintln!("warning: graphviz `dot` not found — falling back to DOT format");
                    dot
                }
            }
        }
        other => anyhow::bail!("unsupported format '{other}'; expected dot or svg"),
    };
    if let Some(path) = out {
        std::fs::write(&path, &content)?;
        if !json {
            println!("wrote {path}");
        } else {
            println!("{}", serde_json::json!({"path": path}));
        }
    } else {
        print!("{content}");
    }
    Ok(())
}
