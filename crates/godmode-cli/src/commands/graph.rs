use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Graph { action } => match action {
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
        },
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
