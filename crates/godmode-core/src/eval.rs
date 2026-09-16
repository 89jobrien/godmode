//! Skill evaluation runs, baselines, and CI gates.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EvalSuite {
    pub skill_name: String,
    pub evals: Vec<EvalCase>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvalCase {
    pub id: u64,
    pub prompt: String,
    pub expected_output: String,
    #[serde(default)]
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvalOutcome {
    pub passed: bool,
    #[serde(default)]
    pub output: String,
    #[serde(default)]
    pub cost_usd: f64,
}

pub trait EvalExecutor {
    fn evaluate(
        &mut self,
        case: &EvalCase,
        iteration: u32,
        remaining_budget_usd: f64,
    ) -> Result<EvalOutcome>;
}

#[derive(Debug, Clone, Copy)]
pub struct RunPolicy {
    pub repetitions: u32,
    pub max_cases: usize,
    pub max_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseResult {
    pub eval_id: u64,
    pub iteration: u32,
    pub passed: bool,
    pub cost_usd: f64,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub id: String,
    pub skill: String,
    pub created_at: DateTime<Utc>,
    pub repetitions: u32,
    pub passed: usize,
    pub total: usize,
    pub pass_rate: f64,
    pub cost_usd: f64,
    pub cases: Vec<CaseResult>,
}

#[derive(Debug, Clone, Copy)]
pub struct Gate {
    pub min_pass_rate: f64,
    pub max_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Comparison {
    pub skill: String,
    pub baseline_id: String,
    pub current_id: String,
    pub pass_rate_delta: f64,
    pub cost_delta_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalStatus {
    pub skill: String,
    pub stored_runs: usize,
    pub latest: Option<RunReport>,
    pub baseline: Option<RunReport>,
    pub gate_passed: bool,
}

pub const MAX_REPETITIONS: u32 = 10;

static RUN_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub fn suite_path(root: &Path, skill: &str) -> PathBuf {
    root.join("skills").join(skill).join("evals/evals.json")
}

fn state_dir(root: &Path, skill: &str) -> Result<PathBuf> {
    if skill.is_empty()
        || !skill
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        anyhow::bail!("invalid skill name '{skill}'");
    }
    Ok(root.join(".ctx/godmode/evaluations").join(skill))
}

fn runs_dir(root: &Path, skill: &str) -> Result<PathBuf> {
    Ok(state_dir(root, skill)?.join("runs"))
}

fn baseline_path(root: &Path, skill: &str) -> Result<PathBuf> {
    Ok(state_dir(root, skill)?.join("baseline.json"))
}

pub fn load_suite(root: &Path, skill: &str) -> Result<EvalSuite> {
    state_dir(root, skill)?;
    let path = suite_path(root, skill);
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("reading evaluation suite {}", path.display()))?;
    let suite: EvalSuite = serde_json::from_str(&raw)
        .with_context(|| format!("parsing evaluation suite {}", path.display()))?;
    if suite.evals.is_empty() {
        anyhow::bail!("evaluation suite '{skill}' has no cases");
    }
    Ok(suite)
}

pub fn run<E: EvalExecutor>(
    root: &Path,
    skill: &str,
    policy: RunPolicy,
    executor: &mut E,
) -> Result<RunReport> {
    validate_policy(policy)?;
    let suite = load_suite(root, skill)?;
    let total = suite
        .evals
        .len()
        .checked_mul(policy.repetitions as usize)
        .context("evaluation invocation count overflowed")?;
    if total > policy.max_cases {
        anyhow::bail!(
            "evaluation requires {total} cases, exceeding max_cases {}",
            policy.max_cases
        );
    }

    let mut cases = Vec::with_capacity(total);
    let mut cost_usd = 0.0;
    for iteration in 1..=policy.repetitions {
        for case in &suite.evals {
            let remaining = policy.max_cost_usd - cost_usd;
            if remaining <= 0.0 {
                anyhow::bail!("cost budget exhausted before eval {}", case.id);
            }
            let outcome = executor.evaluate(case, iteration, remaining)?;
            validate_cost(outcome.cost_usd)?;
            if outcome.cost_usd > remaining {
                anyhow::bail!(
                    "cost budget exceeded by eval {}: ${:.4} > ${remaining:.4} remaining",
                    case.id,
                    outcome.cost_usd
                );
            }
            cost_usd += outcome.cost_usd;
            cases.push(CaseResult {
                eval_id: case.id,
                iteration,
                passed: outcome.passed,
                cost_usd: outcome.cost_usd,
                output: outcome.output,
            });
        }
    }

    let passed = cases.iter().filter(|result| result.passed).count();
    let now = Utc::now();
    let report = RunReport {
        id: format!(
            "{}-{}-{}",
            now.format("%Y%m%dT%H%M%S%.9fZ"),
            std::process::id(),
            RUN_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ),
        skill: skill.to_owned(),
        created_at: now,
        repetitions: policy.repetitions,
        passed,
        total,
        pass_rate: passed as f64 / total as f64,
        cost_usd,
        cases,
    };
    let path = runs_dir(root, skill)?.join(format!("{}.json", report.id));
    write_json(&path, &report)?;
    Ok(report)
}

fn validate_policy(policy: RunPolicy) -> Result<()> {
    if policy.repetitions == 0 || policy.repetitions > MAX_REPETITIONS {
        anyhow::bail!(
            "repetitions must be between 1 and {MAX_REPETITIONS}, got {}",
            policy.repetitions
        );
    }
    if policy.max_cases == 0 {
        anyhow::bail!("max_cases must be greater than zero");
    }
    validate_cost(policy.max_cost_usd)?;
    if policy.max_cost_usd == 0.0 {
        anyhow::bail!("max_cost_usd must be greater than zero");
    }
    Ok(())
}

fn validate_gate(gate: Gate) -> Result<()> {
    if !gate.min_pass_rate.is_finite() || !(0.0..=1.0).contains(&gate.min_pass_rate) {
        anyhow::bail!("min_pass_rate must be between 0 and 1");
    }
    validate_cost(gate.max_cost_usd)
}

fn validate_cost(cost: f64) -> Result<()> {
    if !cost.is_finite() || cost < 0.0 {
        anyhow::bail!("cost must be a finite non-negative value");
    }
    Ok(())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let data = serde_json::to_vec_pretty(value)?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, data).with_context(|| format!("writing {}", temporary.display()))?;
    std::fs::rename(&temporary, path).with_context(|| format!("writing {}", path.display()))
}

fn read_report(path: &Path) -> Result<RunReport> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading evaluation report {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("parsing evaluation report {}", path.display()))
}

fn stored_runs(root: &Path, skill: &str) -> Result<Vec<RunReport>> {
    let dir = runs_dir(root, skill)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut reports = Vec::new();
    for entry in std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            reports.push(read_report(&path)?);
        }
    }
    reports.sort_by_key(|report| report.created_at);
    Ok(reports)
}

fn latest_run(root: &Path, skill: &str) -> Result<RunReport> {
    stored_runs(root, skill)?
        .pop()
        .with_context(|| format!("no stored evaluation runs for skill '{skill}'"))
}

fn load_baseline(root: &Path, skill: &str) -> Result<Option<RunReport>> {
    let path = baseline_path(root, skill)?;
    if path.exists() {
        Ok(Some(read_report(&path)?))
    } else {
        Ok(None)
    }
}

pub fn promote(root: &Path, skill: &str, gate: Gate) -> Result<RunReport> {
    validate_gate(gate)?;
    let latest = latest_run(root, skill)?;
    ensure_gate(&latest, gate)?;
    write_json(&baseline_path(root, skill)?, &latest)?;
    Ok(latest)
}

pub fn compare(root: &Path, skill: &str) -> Result<Comparison> {
    let current = latest_run(root, skill)?;
    let baseline = load_baseline(root, skill)?
        .with_context(|| format!("no promoted baseline for skill '{skill}'"))?;
    Ok(Comparison {
        skill: skill.to_owned(),
        baseline_id: baseline.id,
        current_id: current.id,
        pass_rate_delta: current.pass_rate - baseline.pass_rate,
        cost_delta_usd: current.cost_usd - baseline.cost_usd,
    })
}

pub fn status(root: &Path, skill: &str, gate: Gate) -> Result<EvalStatus> {
    validate_gate(gate)?;
    let mut runs = stored_runs(root, skill)?;
    let latest = runs.pop();
    let gate_passed = latest.as_ref().is_some_and(|report| {
        report.pass_rate >= gate.min_pass_rate && report.cost_usd <= gate.max_cost_usd
    });
    Ok(EvalStatus {
        skill: skill.to_owned(),
        stored_runs: runs.len() + usize::from(latest.is_some()),
        latest,
        baseline: load_baseline(root, skill)?,
        gate_passed,
    })
}

fn ensure_gate(report: &RunReport, gate: Gate) -> Result<()> {
    if report.pass_rate < gate.min_pass_rate {
        anyhow::bail!(
            "pass rate {:.1}% is below required {:.1}%",
            report.pass_rate * 100.0,
            gate.min_pass_rate * 100.0
        );
    }
    if report.cost_usd > gate.max_cost_usd {
        anyhow::bail!(
            "run cost ${:.4} exceeds maximum ${:.4}",
            report.cost_usd,
            gate.max_cost_usd
        );
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct FixtureFile {
    results: Vec<FixtureResult>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureResult {
    eval_id: u64,
    #[serde(flatten)]
    outcome: EvalOutcome,
}

/// Deterministic executor for local and CI runs. Real model adapters implement
/// [`EvalExecutor`] without coupling the evaluation domain to a model vendor.
pub struct FixtureExecutor {
    results: Vec<FixtureResult>,
}

impl FixtureExecutor {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("reading fixture results {}", path.display()))?;
        let fixture: FixtureFile = serde_json::from_str(&raw)
            .with_context(|| format!("parsing fixture results {}", path.display()))?;
        Ok(Self {
            results: fixture.results,
        })
    }
}

impl EvalExecutor for FixtureExecutor {
    fn evaluate(
        &mut self,
        case: &EvalCase,
        _iteration: u32,
        remaining_budget_usd: f64,
    ) -> Result<EvalOutcome> {
        let result = self
            .results
            .iter()
            .find(|result| result.eval_id == case.id)
            .with_context(|| format!("fixture has no result for eval {}", case.id))?;
        if result.outcome.cost_usd > remaining_budget_usd {
            anyhow::bail!(
                "fixture result for eval {} exceeds remaining cost budget",
                case.id
            );
        }
        Ok(result.outcome.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use anyhow::anyhow;
    use tempfile::tempdir;

    use super::*;

    struct FakeExecutor {
        outcomes: VecDeque<EvalOutcome>,
        calls: usize,
    }

    impl EvalExecutor for FakeExecutor {
        fn evaluate(
            &mut self,
            _case: &EvalCase,
            _iteration: u32,
            _remaining_budget_usd: f64,
        ) -> Result<EvalOutcome> {
            self.calls += 1;
            self.outcomes
                .pop_front()
                .ok_or_else(|| anyhow!("unexpected evaluation call"))
        }
    }

    fn write_suite(root: &Path) {
        let path = suite_path(root, "demo");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            path,
            r#"{"skill_name":"demo","evals":[{"id":1,"prompt":"p","expected_output":"e","files":[]}]}"#,
        )
        .unwrap();
    }

    fn executor(outcomes: Vec<EvalOutcome>) -> FakeExecutor {
        FakeExecutor {
            outcomes: outcomes.into(),
            calls: 0,
        }
    }

    fn policy(repetitions: u32, max_cost_usd: f64) -> RunPolicy {
        RunPolicy {
            repetitions,
            max_cases: 10,
            max_cost_usd,
        }
    }

    #[test]
    fn repeated_run_is_aggregated_and_persisted() {
        let dir = tempdir().unwrap();
        write_suite(dir.path());
        let mut fake = executor(vec![
            EvalOutcome {
                passed: true,
                output: "a".into(),
                cost_usd: 0.1,
            },
            EvalOutcome {
                passed: false,
                output: "b".into(),
                cost_usd: 0.2,
            },
        ]);

        let report = run(dir.path(), "demo", policy(2, 1.0), &mut fake).unwrap();

        assert_eq!(fake.calls, 2);
        assert_eq!(report.total, 2);
        assert_eq!(report.passed, 1);
        assert_eq!(report.pass_rate, 0.5);
        assert!((report.cost_usd - 0.3).abs() < f64::EPSILON);
        assert_eq!(
            status(
                dir.path(),
                "demo",
                Gate {
                    min_pass_rate: 0.0,
                    max_cost_usd: 1.0
                }
            )
            .unwrap()
            .stored_runs,
            1
        );
    }

    #[test]
    fn cost_budget_rejects_an_over_budget_result() {
        let dir = tempdir().unwrap();
        write_suite(dir.path());
        let mut fake = executor(vec![
            EvalOutcome {
                passed: true,
                output: String::new(),
                cost_usd: 0.4,
            },
            EvalOutcome {
                passed: true,
                output: String::new(),
                cost_usd: 0.2,
            },
        ]);

        let error = run(dir.path(), "demo", policy(2, 0.5), &mut fake).unwrap_err();

        assert!(error.to_string().contains("cost budget"));
        assert_eq!(fake.calls, 2);
        assert_eq!(
            status(
                dir.path(),
                "demo",
                Gate {
                    min_pass_rate: 0.0,
                    max_cost_usd: 1.0
                }
            )
            .unwrap()
            .stored_runs,
            0
        );
    }

    #[test]
    fn promote_compare_and_status_apply_gates() {
        let dir = tempdir().unwrap();
        write_suite(dir.path());
        let gate = Gate {
            min_pass_rate: 0.8,
            max_cost_usd: 0.5,
        };
        let mut first = executor(vec![EvalOutcome {
            passed: true,
            output: String::new(),
            cost_usd: 0.2,
        }]);
        let baseline = run(dir.path(), "demo", policy(1, 0.5), &mut first).unwrap();
        assert_eq!(promote(dir.path(), "demo", gate).unwrap().id, baseline.id);

        let mut second = executor(vec![EvalOutcome {
            passed: false,
            output: String::new(),
            cost_usd: 0.1,
        }]);
        let current = run(dir.path(), "demo", policy(1, 0.5), &mut second).unwrap();
        let comparison = compare(dir.path(), "demo").unwrap();
        assert_eq!(status(dir.path(), "demo", gate).unwrap().stored_runs, 2);
        assert_eq!(comparison.current_id, current.id);
        assert_eq!(comparison.pass_rate_delta, -1.0);

        let current_status = status(dir.path(), "demo", gate).unwrap();
        assert!(!current_status.gate_passed);
        assert_eq!(current_status.baseline.unwrap().id, baseline.id);
    }

    #[test]
    fn rejects_unbounded_run_before_executor_call() {
        let dir = tempdir().unwrap();
        write_suite(dir.path());
        let mut fake = executor(vec![]);

        let error = run(
            dir.path(),
            "demo",
            RunPolicy {
                repetitions: 11,
                max_cases: 10,
                max_cost_usd: 1.0,
            },
            &mut fake,
        )
        .unwrap_err();

        assert!(error.to_string().contains("repetitions"));
        assert_eq!(fake.calls, 0);
    }
}
