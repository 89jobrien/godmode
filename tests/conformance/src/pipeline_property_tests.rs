//! Property-based conformance tests for the pipeline state machine.
//!
//! Generated pipeline definitions and transition sequences verify that advancing
//! or skipping a step updates progress and history consistently, completion is
//! reported exactly at the end of a pipeline, and entry-point selection begins at
//! the expected step. YAML round-trip properties also protect the persisted
//! pipeline definition and state formats.

#[cfg(test)]
mod pipeline_properties {
    use godmode_core::pipeline::{
        self, LoopMode, Pipeline, PipelineState, PipelineStep, StepStatus,
    };
    use proptest::prelude::*;

    fn arb_skill_name() -> impl Strategy<Value = String> {
        "[a-z][a-z-]{2,15}".prop_map(|s| s)
    }

    fn arb_step() -> impl Strategy<Value = PipelineStep> {
        (arb_skill_name(), any::<bool>(), any::<bool>()).prop_map(|(skill, optional, has_loop)| {
            PipelineStep {
                skill,
                optional,
                r#loop: if has_loop {
                    Some(LoopMode::PerTask)
                } else {
                    None
                },
                parallel_with: vec![],
            }
        })
    }

    fn arb_pipeline(min_steps: usize, max_steps: usize) -> impl Strategy<Value = Pipeline> {
        prop::collection::vec(arb_step(), min_steps..max_steps).prop_map(|steps| {
            let entry_points = if steps.is_empty() {
                vec![]
            } else {
                vec![steps[0].skill.clone()]
            };
            Pipeline {
                name: "test-pipeline".into(),
                description: "generated".into(),
                steps,
                entry_points,
            }
        })
    }

    /// Generate a sequence of operations: true = advance, false = skip.
    fn arb_ops(max_len: usize) -> impl Strategy<Value = Vec<bool>> {
        prop::collection::vec(any::<bool>(), 0..max_len)
    }

    proptest! {
        /// current_step after N operations always equals N.
        #[test]
        fn step_count_equals_transitions(
            p in arb_pipeline(1, 10),
            ops in arb_ops(12),
        ) {
            let mut state = pipeline::start(&p, None).unwrap();
            let mut transitions = 0usize;

            for op in ops {
                if pipeline::is_complete(&state, &p) {
                    break;
                }
                if op {
                    pipeline::advance(&mut state, &p);
                } else {
                    pipeline::skip(&mut state, &p);
                }
                transitions += 1;
            }

            prop_assert_eq!(state.current_step, transitions);
        }

        /// history.len() always equals the number of transitions performed.
        #[test]
        fn history_tracks_all_transitions(
            p in arb_pipeline(1, 10),
            ops in arb_ops(12),
        ) {
            let mut state = pipeline::start(&p, None).unwrap();
            let mut transitions = 0usize;

            for op in ops {
                if pipeline::is_complete(&state, &p) {
                    break;
                }
                if op {
                    pipeline::advance(&mut state, &p);
                } else {
                    pipeline::skip(&mut state, &p);
                }
                transitions += 1;
            }

            prop_assert_eq!(state.history.len(), transitions);
        }

        /// is_complete is true iff current_step >= steps.len().
        #[test]
        fn is_complete_iff_past_end(
            p in arb_pipeline(1, 8),
            ops in arb_ops(10),
        ) {
            let mut state = pipeline::start(&p, None).unwrap();

            for op in ops {
                if pipeline::is_complete(&state, &p) {
                    break;
                }
                if op {
                    pipeline::advance(&mut state, &p);
                } else {
                    pipeline::skip(&mut state, &p);
                }
            }

            let complete = pipeline::is_complete(&state, &p);
            prop_assert_eq!(complete, state.current_step >= p.steps.len());
        }

        /// Exhausting all steps always results in is_complete.
        #[test]
        fn exhaust_all_steps_completes(p in arb_pipeline(1, 10)) {
            let mut state = pipeline::start(&p, None).unwrap();

            for _ in 0..p.steps.len() {
                prop_assert!(!pipeline::is_complete(&state, &p));
                pipeline::advance(&mut state, &p);
            }

            prop_assert!(pipeline::is_complete(&state, &p));
        }

        /// remaining() + current_step always equals steps.len().
        #[test]
        fn remaining_plus_current_equals_total(
            p in arb_pipeline(1, 10),
            ops in arb_ops(12),
        ) {
            let mut state = pipeline::start(&p, None).unwrap();

            for op in ops {
                if pipeline::is_complete(&state, &p) {
                    break;
                }
                let (done, total) = pipeline::progress(&state, &p);
                let rem = pipeline::remaining(&state, &p);
                prop_assert_eq!(done + rem, total);

                if op {
                    pipeline::advance(&mut state, &p);
                } else {
                    pipeline::skip(&mut state, &p);
                }
            }
        }

        /// advance records Done status; skip records Skipped status.
        #[test]
        fn advance_done_skip_skipped(
            p in arb_pipeline(2, 8),
            ops in arb_ops(10),
        ) {
            let mut state = pipeline::start(&p, None).unwrap();

            for op in ops {
                if pipeline::is_complete(&state, &p) {
                    break;
                }
                let idx = state.history.len();
                if op {
                    pipeline::advance(&mut state, &p);
                    prop_assert_eq!(&state.history[idx].status, &StepStatus::Done);
                } else {
                    pipeline::skip(&mut state, &p);
                    prop_assert_eq!(&state.history[idx].status, &StepStatus::Skipped);
                }
            }
        }

        /// PipelineState round-trips through YAML serialization.
        #[test]
        fn state_yaml_roundtrip(
            p in arb_pipeline(1, 6),
            n_advance in 0usize..6,
        ) {
            let mut state = pipeline::start(&p, None).unwrap();
            for _ in 0..n_advance.min(p.steps.len()) {
                pipeline::advance(&mut state, &p);
            }

            let yaml = serde_yaml::to_string(&state).unwrap();
            let back: PipelineState = serde_yaml::from_str(&yaml).unwrap();

            prop_assert_eq!(back.active, state.active);
            prop_assert_eq!(back.current_step, state.current_step);
            prop_assert_eq!(back.history.len(), state.history.len());
        }

        /// Pipeline definition round-trips through YAML.
        #[test]
        fn pipeline_yaml_roundtrip(p in arb_pipeline(1, 8)) {
            let yaml = serde_yaml::to_string(&p).unwrap();
            let back: Pipeline = serde_yaml::from_str(&yaml).unwrap();

            prop_assert_eq!(back.name, p.name);
            prop_assert_eq!(back.steps.len(), p.steps.len());
            for (orig, rt) in p.steps.iter().zip(back.steps.iter()) {
                prop_assert_eq!(&orig.skill, &rt.skill);
                prop_assert_eq!(orig.optional, rt.optional);
                prop_assert_eq!(&orig.r#loop, &rt.r#loop);
            }
        }

        /// start(from: first_skill) produces current_step == 0.
        #[test]
        fn start_from_first_entry_point(p in arb_pipeline(1, 8)) {
            let first = &p.steps[0].skill;
            let state = pipeline::start(&p, Some(first)).unwrap();
            prop_assert_eq!(state.current_step, 0);
        }
    }
}
