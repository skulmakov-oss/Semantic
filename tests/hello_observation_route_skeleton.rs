use std::path::PathBuf;
use std::vec::Vec;

use sm_emit::hello_real_semcode::{emit_hello_real_semcode_skeleton, HelloRealSemCodeOp};
use sm_front::hello_parser::parse_hello_file;
use sm_front::hello_sema::validate_hello_file;
use sm_ir::hello_ir::lower_hello_checked_file;
use sm_runtime_core::hello_observation_route::{
    route_hello_observation_to_sink, HelloObservationRouteError, HelloObservationRouteInput,
    HelloObservationRouteResult,
};
use sm_runtime_core::hello_observation_sink::{
    HelloObservationClass, HelloObservationEvent, HelloObservationSequenceIndex,
    HelloObservationSink, HelloObservationSinkError,
};
use sm_verify::hello_real_semcode_admission::{
    admit_hello_real_semcode_skeleton, HelloRealSemCodeAdmissionDecision,
    HelloRealSemCodeAdmissionInput, HelloRealSemCodeAdmissionOp,
};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn fixture_text(rel: &str) -> String {
    std::fs::read_to_string(repo_path(rel))
        .unwrap_or_else(|err| panic!("failed to read fixture {rel}: {err}"))
}

fn parse_validate_lower() -> sm_ir::hello_ir::HelloIrModule {
    let input = fixture_text("tests/fixtures/pending/hello/positive_hello_verbose_directional.sm");
    let parsed = parse_hello_file(&input).unwrap_or_else(|err| {
        panic!("parser unexpectedly rejected canonical hello fixture: {err}")
    });
    let checked = validate_hello_file(parsed)
        .unwrap_or_else(|err| panic!("sema unexpectedly rejected canonical hello fixture: {err}"));
    lower_hello_checked_file(&checked).unwrap_or_else(|err| {
        panic!("lowering unexpectedly rejected canonical hello fixture: {err}")
    })
}

fn render_text_literal(text: &str) -> String {
    text.to_string()
}

fn skeleton_to_admission_input(ops: &[HelloRealSemCodeOp]) -> HelloRealSemCodeAdmissionInput {
    let mut admission_ops = Vec::with_capacity(ops.len());
    for op in ops {
        admission_ops.push(match op {
            HelloRealSemCodeOp::DeclareLocalQuad { name, value } => {
                HelloRealSemCodeAdmissionOp::DeclareLocalQuad {
                    name: name.clone(),
                    value: value.clone(),
                }
            }
            HelloRealSemCodeOp::RequireQuadEq { name, expected } => {
                HelloRealSemCodeAdmissionOp::RequireQuadEq {
                    name: name.clone(),
                    expected: expected.clone(),
                }
            }
            HelloRealSemCodeOp::ObserveTextLiteral { text } => {
                HelloRealSemCodeAdmissionOp::ObserveTextLiteral { text: text.clone() }
            }
            HelloRealSemCodeOp::CompleteQuad { value } => {
                HelloRealSemCodeAdmissionOp::CompleteQuad {
                    value: value.clone(),
                }
            }
        });
    }

    HelloRealSemCodeAdmissionInput { ops: admission_ops }
}

fn canonical_admission_input() -> HelloRealSemCodeAdmissionInput {
    let module = parse_validate_lower();
    let semcode = emit_hello_real_semcode_skeleton(&module)
        .expect("canonical verbose Hello should lower to real SemCode skeleton");
    skeleton_to_admission_input(&semcode.ops)
}

#[derive(Default)]
struct InMemorySink {
    events: Vec<HelloObservationEvent>,
    reject: bool,
}

impl HelloObservationSink for InMemorySink {
    fn observe(&mut self, event: HelloObservationEvent) -> Result<(), HelloObservationSinkError> {
        if self.reject {
            return Err(HelloObservationSinkError::Denied);
        }
        self.events.push(event);
        Ok(())
    }
}

fn admitted_route_input() -> HelloObservationRouteInput {
    HelloObservationRouteInput {
        admitted: true,
        text: "Hello, World!".to_string(),
        sequence_index: HelloObservationSequenceIndex(0),
    }
}

fn route_and_capture(
    input: HelloObservationRouteInput,
    sink: &mut InMemorySink,
) -> HelloObservationRouteResult {
    route_hello_observation_to_sink(input, sink)
}

#[test]
fn hello_observation_route_routes_admitted_hello_to_explicit_sink() {
    let mut sink = InMemorySink::default();
    let result = route_and_capture(admitted_route_input(), &mut sink);

    assert_eq!(result, HelloObservationRouteResult::Routed);
    assert_eq!(sink.events.len(), 1);
    let event = &sink.events[0];
    assert_eq!(event.operation_kind, "controlled_observation_text");
    assert_eq!(
        event.observation_class,
        HelloObservationClass::ControlledText
    );
    assert_eq!(event.text, "Hello, World!");
    assert_eq!(event.sequence_index, HelloObservationSequenceIndex(0));
}

#[test]
fn hello_observation_route_rejects_non_admitted_and_forbidden_inputs() {
    let mut sink = InMemorySink::default();

    let not_admitted = HelloObservationRouteInput {
        admitted: false,
        text: "Hello, World!".to_string(),
        sequence_index: HelloObservationSequenceIndex(9),
    };
    assert_eq!(
        route_and_capture(not_admitted, &mut sink),
        HelloObservationRouteResult::NotRouted(HelloObservationRouteError::NotAdmitted)
    );
    assert!(sink.events.is_empty());

    for forbidden in ["stdout", "print", "io.write", "file", "network", "stdin"] {
        let result = route_and_capture(
            HelloObservationRouteInput {
                admitted: true,
                text: render_text_literal(forbidden),
                sequence_index: HelloObservationSequenceIndex(1),
            },
            &mut sink,
        );
        assert_eq!(
            result,
            HelloObservationRouteResult::NotRouted(HelloObservationRouteError::ForbiddenHostOutput,)
        );
    }

    let non_controlled = route_and_capture(
        HelloObservationRouteInput {
            admitted: true,
            text: render_text_literal("Not Hello"),
            sequence_index: HelloObservationSequenceIndex(2),
        },
        &mut sink,
    );
    assert_eq!(
        non_controlled,
        HelloObservationRouteResult::NotRouted(HelloObservationRouteError::NonControlledText)
    );
}

#[test]
fn hello_observation_route_maps_sink_rejection_to_sink_rejected() {
    let mut sink = InMemorySink {
        events: Vec::new(),
        reject: true,
    };
    let result = route_and_capture(admitted_route_input(), &mut sink);

    assert_eq!(
        result,
        HelloObservationRouteResult::NotRouted(HelloObservationRouteError::SinkRejected)
    );
    assert!(sink.events.is_empty());
}

#[test]
fn hello_observation_route_composes_isolated_verifier_and_sink_layers() {
    let canonical = canonical_admission_input();
    assert_eq!(
        admit_hello_real_semcode_skeleton(&canonical),
        HelloRealSemCodeAdmissionDecision::Admit
    );

    let mut sink = InMemorySink::default();
    let result = route_and_capture(admitted_route_input(), &mut sink);

    assert_eq!(result, HelloObservationRouteResult::Routed);
    assert_eq!(sink.events.len(), 1);
    assert_eq!(
        sink.events[0].sequence_index,
        HelloObservationSequenceIndex(0)
    );
}
