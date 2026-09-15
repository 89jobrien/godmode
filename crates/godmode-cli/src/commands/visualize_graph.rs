use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::VisualizeGraph { format, out } => {
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
                            eprintln!(
                                "warning: graphviz `dot` not found — falling back to DOT format"
                            );
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
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
