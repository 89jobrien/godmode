#!/usr/bin/env rust-script
//! Render a whatidid digest JSON into an Outlook-compatible HTML report.
//!
//! Usage: report.rs <digest.json> [YYYY-MM-DD]
//!
//! Loads token pricing from model_pricing.json (sibling of helpers/).
//! Computes SHA-256 of the pricing file and prints it so drift is visible.
//!
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! anyhow = "1"
//! chrono = "0.4"
//! sha2 = "0.10"
//! hex = "0.4"
//! ```

use anyhow::{Context, Result};
use chrono::Local;
use hex::ToHex;
use serde::Deserialize;
use sha2::{Digest as Sha2Digest, Sha256};
use std::{fs, path::PathBuf, process::Command};

// ── Pricing types (loaded from model_pricing.json) ───────────────────────────

#[derive(Debug, Deserialize)]
struct PricingFile {
    models: Vec<ModelEntry>,
    fallback: FallbackEntry,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    prefix: String,
    input: f64,
    output: f64,
}

#[derive(Debug, Deserialize)]
struct FallbackEntry {
    input: f64,
    output: f64,
}

impl PricingFile {
    fn cost_for(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        let (inp_rate, out_rate) = self
            .models
            .iter()
            .find(|m| model.starts_with(&m.prefix))
            .map(|m| (m.input, m.output))
            .unwrap_or((self.fallback.input, self.fallback.output));
        (input_tokens as f64 / 1_000_000.0) * inp_rate
            + (output_tokens as f64 / 1_000_000.0) * out_rate
    }
}

// ── Digest types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Digest {
    headline: String,
    primary_focus: String,
    day_narrative: String,
    goals: Vec<Goal>,
}

#[derive(Debug, Deserialize)]
struct Goal {
    label: String,
    human_hours: f32,
    project: String,
    tasks: Vec<Task>,
}

#[derive(Debug, Deserialize)]
struct Task {
    title: String,
    what_got_done: String,
    tech_skills: Vec<String>,
    task_type: String,
}

const HOURLY_RATE: f64 = 72.0;
const SEAT_COST_PER_MONTH: f64 = 39.0;

fn main() -> Result<()> {
    let digest_path = std::env::args()
        .nth(1)
        .context("usage: report.rs <digest.json> [YYYY-MM-DD]")?;
    let date_str = std::env::args()
        .nth(2)
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());

    // Load and hash pricing
    let skill_dir = skill_dir()?;
    let pricing_path = skill_dir.join("model_pricing.json");
    let pricing_raw = fs::read_to_string(&pricing_path)
        .with_context(|| format!("read {}", pricing_path.display()))?;

    let hash: String = Sha256::digest(pricing_raw.as_bytes()).encode_hex::<String>();
    eprintln!("pricing sha256: {}", &hash[..16]);

    let pricing: PricingFile =
        serde_json::from_str(&pricing_raw).context("parse model_pricing.json")?;

    // Load digest
    let raw = fs::read_to_string(&digest_path)
        .with_context(|| format!("read {digest_path}"))?;
    let digest: Digest = {
        let v: serde_json::Value = serde_json::from_str(&raw)?;
        if let Some(d) = v.get("digest") {
            serde_json::from_value(d.clone())?
        } else {
            serde_json::from_str(&raw)?
        }
    };

    let total_hours: f32 = digest.goals.iter().map(|g| g.human_hours).sum();
    let human_value = total_hours as f64 * HOURLY_RATE;
    let leverage = human_value / SEAT_COST_PER_MONTH;

    let html = render_html(&digest, &date_str, total_hours, human_value, leverage);

    let out_path = format!("/tmp/whatidid-{date_str}.html");
    fs::write(&out_path, &html)
        .with_context(|| format!("write {out_path}"))?;

    let _ = Command::new("open").arg(&out_path).spawn();

    eprintln!("Report: {out_path}");
    eprintln!(
        "Date: {date_str}  |  Goals: {}  |  Hours: {:.1}h",
        digest.goals.len(),
        total_hours
    );
    eprintln!("Human value: ${human_value:.0}  |  Leverage: {leverage:.0}x");

    // Demonstrate pricing lookup (printed only if model metrics available)
    let _ = pricing.cost_for("gpt-4o-mini", 0, 0);

    Ok(())
}

fn render_html(
    digest: &Digest,
    date: &str,
    total_hours: f32,
    human_value: f64,
    leverage: f64,
) -> String {
    let goals_rows: String = digest.goals.iter().map(|g| {
        let skills: Vec<String> = g.tasks.iter()
            .flat_map(|t| t.tech_skills.iter().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let tasks_html: String = g.tasks.iter().map(|t| {
            format!(
                "<li><strong>{}</strong> — {} \
                 <span style='color:#555;font-size:11px'>[{}]</span></li>",
                esc(&t.title),
                esc(&t.what_got_done),
                esc(&t.task_type)
            )
        }).collect();
        format!(
            "<tr>\
               <td style='padding:8px;border-bottom:1px solid #eee;vertical-align:top'>\
                 <strong>{}</strong><br/>\
                 <span style='color:#555;font-size:12px'>{}</span>\
               </td>\
               <td style='padding:8px;border-bottom:1px solid #eee'>\
                 <ul style='margin:0;padding-left:18px'>{tasks_html}</ul>\
               </td>\
               <td style='padding:8px;border-bottom:1px solid #eee;text-align:right;white-space:nowrap'>{:.1}h</td>\
               <td style='padding:8px;border-bottom:1px solid #eee;font-size:12px;color:#555'>{}</td>\
             </tr>",
            esc(&g.label),
            esc(&g.project),
            g.human_hours,
            esc(&skills.join(", ")),
        )
    }).collect();

    format!(r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>What I Did — {date}</title></head>
<body style="font-family:Segoe UI,Arial,sans-serif;max-width:900px;margin:0 auto;padding:20px;color:#222">

<div style="background:#0078d4;color:#fff;padding:16px 24px;border-radius:4px;margin-bottom:20px">
  <h1 style="margin:0;font-size:20px">What I Did — {date}</h1>
  <p style="margin:4px 0 0;opacity:.85;font-size:14px">{}</p>
</div>

<p style="font-size:15px;line-height:1.6;margin-bottom:24px">{}</p>

<div style="display:flex;gap:16px;margin-bottom:28px;flex-wrap:wrap">
  {}
  {}
  {}
</div>

<h2 style="font-size:16px;border-bottom:2px solid #0078d4;padding-bottom:6px">Goals &amp; Tasks</h2>
<table style="width:100%;border-collapse:collapse;font-size:14px">
  <thead>
    <tr style="background:#f5f5f5">
      <th style="padding:8px;text-align:left;width:20%">Goal</th>
      <th style="padding:8px;text-align:left">Tasks</th>
      <th style="padding:8px;text-align:right;width:60px">Hours</th>
      <th style="padding:8px;text-align:left;width:15%">Skills</th>
    </tr>
  </thead>
  <tbody>{goals_rows}</tbody>
</table>

</body></html>"#,
        esc(&digest.primary_focus),
        esc(&digest.day_narrative),
        kpi_card("Hours", &format!("{total_hours:.1}h"), "#0078d4"),
        kpi_card("Human Value", &format!("${human_value:.0}"), "#107c10"),
        kpi_card("Leverage", &format!("{leverage:.0}×"), "#8764b8"),
    )
}

fn kpi_card(label: &str, value: &str, color: &str) -> String {
    format!(
        "<div style='background:{color};color:#fff;padding:14px 20px;\
         border-radius:4px;min-width:120px;text-align:center'>\
           <div style='font-size:24px;font-weight:bold'>{value}</div>\
           <div style='font-size:12px;opacity:.85;margin-top:2px'>{label}</div>\
         </div>"
    )
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

fn skill_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join("dev/minibox/.claude/skills/whatidid"))
}
