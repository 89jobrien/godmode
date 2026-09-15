use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Insight { action } => {
            fn parse_date_or_today(d: &Option<String>) -> Result<insights::NaiveDate> {
                match d {
                    Some(s) => Ok(insights::NaiveDate::parse_from_str(s, "%Y-%m-%d")?),
                    None => Ok(insights::today()),
                }
            }

            match action {
                InsightAction::Add { title, body, tags } => {
                    let insight = insights::new_insight(title, body, tags);
                    insights::append(root, &insight)?;
                    if json {
                        println!("{}", serde_json::to_string(&insight)?);
                    } else {
                        println!("Recorded: {}", insight.title);
                    }
                }
                InsightAction::List { date } => {
                    let d = parse_date_or_today(&date)?;
                    let items = insights::list_for_date(root, d)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&items)?);
                    } else if items.is_empty() {
                        println!("No insights for {d}.");
                        std::process::exit(2);
                    } else {
                        for i in &items {
                            let tags = if i.tags.is_empty() {
                                String::new()
                            } else {
                                format!(" [{}]", i.tags.join(", "))
                            };
                            println!("- {}{}", i.title, tags);
                        }
                    }
                }
                InsightAction::Render { date } => {
                    let d = parse_date_or_today(&date)?;
                    let path = insights::render_markdown(root, d)?;
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({ "path": path.display().to_string() })
                        );
                    } else {
                        println!("Wrote {}", path.display());
                    }
                }
            }
            Ok(())
        }
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
