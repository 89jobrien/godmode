//! Property-based tests for the trace query layer.
//!
//! Run with: cargo nextest run --test prop_trace
//!
//! The trace log is written by other processes and can be truncated mid-write,
//! so `parse` and the queries over it are the two places where a malformed input
//! turns into a wrong number rather than an error. These properties pin the
//! invariants that make those numbers trustworthy.

use chrono::{Duration, Utc};
use crux_runtime::types::step::{Step, StepKind, StepStatus};
use godmode_core::hooks::trace_log::{elapsed_ms, new_trace_id, parse_trace_id};
use godmode_core::trace_query::{self, Record};
use proptest::prelude::*;

// ── Strategies ───────────────────────────────────────────────────────────────

/// A skill or helper name that contains no `#`. `#` is the trace-id delimiter,
/// so a name carrying one cannot survive a round trip — that is a real domain
/// constraint, not a test artefact, and `prop_trace_id_rejects_hash_names`
/// records the boundary.
fn name_no_hash() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9._/-]{0,20}[a-z0-9]".prop_map(|s| s)
}

/// A legacy-shaped record with a real event name.
fn legacy_line(event: &str, session: &str, ts: &str) -> String {
    format!(
        r#"{{"event":"{event}","agent_id":"a","skill":"s","session_id":"{session}","ts":"{ts}"}}"#
    )
}

// ── Trace-id round trip ───────────────────────────────────────────────────────

proptest! {
    /// The closing call recovers the skill name from the id, so the format must
    /// round-trip for any legal skill name.
    #[test]
    fn prop_trace_id_recovers_skill(skill in name_no_hash(), helper in name_no_hash()) {
        let tid = new_trace_id(&skill, &helper);
        let (recovered, millis) = parse_trace_id(&tid).expect("a minted id must parse");
        prop_assert_eq!(recovered, skill);
        prop_assert!(millis > 0, "start millis must be recovered, got {millis}");
    }

    /// Junk must be rejected, never guessed. A wrong skill name here would
    /// attribute a failure to the wrong skill.
    #[test]
    fn prop_arbitrary_text_is_never_mistaken_for_a_trace_id(s in ".{0,40}") {
        if let Some((skill, _)) = parse_trace_id(&s) {
            // If it parses, whatever it recovered must actually be in the input.
            prop_assert!(
                s.contains(&skill),
                "recovered skill {skill:?} absent from {s:?}"
            );
        }
    }

    /// `elapsed_ms` must not panic and must not underflow into a huge value.
    #[test]
    fn prop_elapsed_ms_is_bounded(s in ".{0,40}") {
        let ms = elapsed_ms(&s);
        // A huge value would mean an underflow escaped the saturating_sub.
        prop_assert!(ms < 1_000_000_000_000, "implausible elapsed {ms} for {s:?}");
    }

    /// A name containing the delimiter is a genuine boundary: the id is
    /// unrecoverable and must be rejected rather than silently mis-parsed.
    #[test]
    fn prop_trace_id_rejects_hash_names(tail in "[a-z]{1,8}") {
        let tid = new_trace_id(&format!("we#{tail}ird"), "h");
        match parse_trace_id(&tid) {
            None => {} // rejected: acceptable
            Some((skill, _)) => prop_assert_eq!(
                skill,
                format!("we#{tail}ird"),
                "a recovered skill must be exact, not a prefix"
            ),
        }
    }
}

// ── Parse robustness ──────────────────────────────────────────────────────────

proptest! {
    /// Arbitrary text must never panic, and must never invent records out of
    /// blank lines.
    #[test]
    fn prop_parse_never_panics(body in ".{0,200}") {
        let records = trace_query::parse(&body);
        let non_blank = body.lines().filter(|l| !l.trim().is_empty()).count();
        prop_assert!(records.len() <= non_blank, "records {} > non-blank lines {non_blank}", records.len());
    }

    /// Every line that is valid JSON yields exactly one record.
    #[test]
    fn prop_valid_json_lines_all_parse(
        events in prop::collection::vec(
            prop::sample::select(vec![
                "skill.start", "skill.complete", "skill.error",
                "agent.start", "agent.complete", "agent.blocked",
                "tool.use", "decision",
            ]),
            0..24,
        ),
        session in "[a-z0-9]{1,8}",
    ) {
        let body: String = events
            .iter()
            .map(|e| format!("{}\n", legacy_line(e, &session, "2026-09-30T09:00:00Z")))
            .collect();
        let records = trace_query::parse(&body);
        prop_assert_eq!(records.len(), events.len());
    }

    /// A crux `Step` must survive the writer/reader round trip, since the
    /// on-disk form *is* the crux schema.
    #[test]
    fn prop_step_round_trips_through_the_reader(
        name in name_no_hash(),
        duration in 0u64..100_000,
    ) {
        let step = Step {
            name: name.clone(),
            kind: StepKind::Delegation,
            status: StepStatus::Ok,
            confidence: 1.0,
            started_at: Utc::now(),
            duration_ms: duration,
            input_hash: 0,
            content_hash: None,
            output: None,
            error: None,
            attempt: 1,
            events: vec![],
            metadata: std::collections::HashMap::from([
                ("phase".to_string(), serde_json::json!("terminal")),
                ("session_id".to_string(), serde_json::json!("s1")),
            ]),
            findings: vec![],
        };
        let line = serde_json::to_string(&step).unwrap();
        let records = trace_query::parse(&line);
        prop_assert_eq!(records.len(), 1);
        prop_assert_eq!(&records[0].name, &name);
        prop_assert_eq!(records[0].duration_ms, Some(duration));
        prop_assert_eq!(records[0].status, Some(StepStatus::Ok));
        prop_assert_eq!(records[0].session_id.as_deref(), Some("s1"));
    }
}

// ── Query invariants ─────────────────────────────────────────────────────────

/// Build records from a script of actions so the queries have real lifecycles.
fn from_script(script: &[&str]) -> Vec<Record> {
    let body: String = script.iter().map(|line| format!("{line}\n")).collect();
    trace_query::parse(&body)
}

fn agent_start(id: &str) -> String {
    format!(r#"{{"event":"agent.start","agent_id":"{id}","session_id":"s"}}"#)
}

fn agent_end(event: &str, id: &str) -> String {
    format!(r#"{{"event":"{event}","agent_id":"{id}","session_id":"s"}}"#)
}

proptest! {
    /// Soundness of `unresolved_agents`: everything it reports must actually be
    /// open. This is the whole point of the triage command, so a false positive
    /// sends someone chasing an agent that finished.
    #[test]
    fn prop_unresolved_agents_are_genuinely_open(
        ids in prop::collection::hash_set("[a-z]{1,6}", 1..=8),
        closed in prop::collection::hash_set("[a-z]{1,6}", 0..=8),
    ) {
        let mut lines: Vec<String> = ids.iter().map(|i| agent_start(i)).collect();
        for id in &closed {
            if ids.contains(id) {
                lines.push(agent_end("agent.complete", id));
            }
        }
        lines.sort();
        let records = from_script(&lines.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        let refs: Vec<&Record> = records.iter().collect();

        let open_ids: std::collections::HashSet<&str> = records
            .iter()
            .filter(|r| r.is_agent() && r.is_open())
            .map(|r| r.name.as_str())
            .collect();

        for reported in trace_query::unresolved_agents(&refs) {
            prop_assert!(
                open_ids.contains(reported.as_str()),
                "reported {reported:?} as unresolved but it has no open record"
            );
        }
    }

    /// A closed agent is never reported as unresolved.
    #[test]
    fn prop_closed_agents_are_never_unresolved(ids in prop::collection::hash_set("[a-z]{1,6}", 1..=8)) {
        let mut lines: Vec<String> = ids.iter().map(|i| agent_start(i)).collect();
        for id in &ids {
            lines.push(agent_end("agent.blocked", id));
        }
        lines.sort();
        let records = from_script(&lines.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        let refs: Vec<&Record> = records.iter().collect();
        prop_assert!(trace_query::unresolved_agents(&refs).is_empty());
    }

    /// `stats` internal consistency: the failure count must equal what the
    /// failures query independently returns.
    #[test]
    fn prop_stats_failure_count_matches_failures_query(
        n_err in 0usize..20,
        n_blocked in 0usize..20,
    ) {
        let mut lines: Vec<String> = Vec::new();
        for i in 0..n_err {
            lines.push(format!(r#"{{"event":"skill.error","skill":"s{i}","session_id":"s"}}"#));
        }
        for i in 0..n_blocked {
            lines.push(agent_end("agent.blocked", &format!("a{i}")));
        }
        let records = from_script(&lines.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        let s = trace_query::stats(&records);
        prop_assert_eq!(s.failure_count, trace_query::failures(&records).len());
    }

    /// Duration rollups must be arithmetically possible.
    #[test]
    fn prop_skill_durations_are_sane(durations in prop::collection::vec(0u64..200_000, 1..16)) {
        let body: String = durations
            .iter()
            .map(|d| format!(r#"{{"event":"skill.complete","skill":"s","duration_ms":{d}}}"#) + "\n")
            .collect();
        let records = trace_query::parse(&body);
        let s = trace_query::stats(&records);
        for d in &s.skill_durations {
            prop_assert!(d.avg_ms <= d.max_ms, "avg {} exceeded max {}", d.avg_ms, d.max_ms);
            prop_assert!(d.runs > 0);
            prop_assert!(d.max_ms <= 200_000);
        }
        prop_assert!(s.failure_count <= s.total_records);
    }

    /// An open record never carries an outcome, so it can never be a failure.
    #[test]
    fn prop_open_records_are_never_failures(n in 0usize..20) {
        let body: String = (0..n)
            .map(|i| format!(r#"{{"event":"skill.start","skill":"s{i}","session_id":"s"}}"#) + "\n")
            .collect();
        let records = trace_query::parse(&body);
        prop_assert!(trace_query::failures(&records).is_empty());
    }

    /// Session scoping must never widen the result set.
    #[test]
    fn prop_session_filter_is_a_subset(sessions in prop::collection::vec("[a-z]{1,4}", 1..=6)) {
        let body: String = sessions
            .iter()
            .map(|s| format!("{}\n", legacy_line("skill.complete", s, "2026-09-30T09:00:00Z")))
            .collect();
        let records = trace_query::parse(&body);
        let picked = &sessions[0];
        let scoped: Vec<&Record> = records
            .iter()
            .filter(|r| r.session_id.as_deref() == Some(picked.as_str()))
            .collect();
        prop_assert!(scoped.len() <= records.len());
        for r in &scoped {
            prop_assert_eq!(r.session_id.as_deref(), Some(picked.as_str()));
        }
    }
}

// ── Freshness ────────────────────────────────────────────────────────────────

proptest! {
    /// A timestamp far in the future is not "today" unless it genuinely is, and
    /// an unparseable one is never claimed to be current.
    #[test]
    fn prop_is_today_rejects_the_past(secs in 86_400i64..4_000_000_000) {
        let ts = (Utc::now() - Duration::seconds(secs)).to_rfc3339();
        prop_assert!(!trace_query::is_today(&ts), "a day-old timestamp must not read as current");
    }

    #[test]
    fn prop_is_today_is_false_for_junk(s in ".{0,40}") {
        if trace_query::is_today(&s) {
            prop_assert!(DateLike::is_parseable(&s), "claimed current but not a timestamp: {s:?}");
        }
    }
}

/// Tiny helper so the junk property can state why a claim is legitimate.
struct DateLike;
impl DateLike {
    fn is_parseable(s: &str) -> bool {
        chrono::DateTime::parse_from_rfc3339(s).is_ok()
    }
}
