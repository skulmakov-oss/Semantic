#![allow(clippy::clone_on_copy)]
use std::path::PathBuf;
use std::vec::Vec;

use prom_audit::hello_observation_audit::{
    build_hello_observation_audit_event, HelloObservationAuditEvent,
    HelloObservationAuditEventKind, HelloObservationAuditLinkage, HelloObservationAuditPayloadRef,
    HelloObservationAuditPolicyClass,
};
use prom_cap::hello_observation_capability::{
    evaluate_hello_observation_capability, HelloObservationCapabilityContext,
    HelloObservationCapabilityDecision,
};
use sm_emit::hello_real_semcode::{
    emit_hello_real_semcode_skeleton, render_hello_real_semcode_skeleton_text,
    HelloRealSemCodeModule, HelloRealSemCodeOp,
};
use sm_front::hello_parser::parse_hello_file;
use sm_front::hello_sema::validate_hello_file;
use sm_ir::hello_ir::lower_hello_checked_file;
use sm_runtime_core::hello_observation_route::{
    route_hello_observation_to_sink, HelloObservationRouteInput, HelloObservationRouteResult,
};
use sm_runtime_core::hello_observation_sink::{
    HelloObservationClass, HelloObservationEvent, HelloObservationSequenceIndex,
    HelloObservationSink, HelloObservationSinkError,
};
use sm_verify::hello_real_semcode_admission::{
    admit_hello_real_semcode_skeleton, HelloRealSemCodeAdmissionDecision,
    HelloRealSemCodeAdmissionInput, HelloRealSemCodeAdmissionOp,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum HelloCliSmokeHarnessResult {
    Accepted {
        semcode_ops: usize,
        sink_events: usize,
        audit_recorded: bool,
    },
    Rejected {
        stage: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HelloObservationAuditDecision {
    Recorded(HelloObservationAuditEvent),
    Deferred,
    NotRecorded(&'static str),
}

#[derive(Debug, Clone)]
struct HelloCliSmokeAcceptedDetails {
    result: HelloCliSmokeHarnessResult,
    admission_decision: HelloRealSemCodeAdmissionDecision,
    capability_decision: HelloObservationCapabilityDecision,
    route_result: HelloObservationRouteResult,
    semcode: HelloRealSemCodeModule,
    rendered_semcode: String,
    sink_events: Vec<HelloObservationEvent>,
    audit_decision: HelloObservationAuditDecision,
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

fn parse_validate_lower(rel: &str) -> Result<sm_ir::hello_ir::HelloIrModule, &'static str> {
    let input = fixture_text(rel);
    let parsed = parse_hello_file(&input).map_err(|_| "parser")?;
    let checked = validate_hello_file(parsed).map_err(|_| "sema")?;
    lower_hello_checked_file(&checked).map_err(|_| "lower")
}

fn canonical_observation_literal(module: &HelloRealSemCodeModule) -> &str {
    assert_eq!(module.ops.len(), 4);
    match &module.ops[..] {
        [HelloRealSemCodeOp::DeclareLocalQuad { name, value }, HelloRealSemCodeOp::RequireQuadEq {
            name: require_name,
            expected,
        }, HelloRealSemCodeOp::ObserveTextLiteral { text }, HelloRealSemCodeOp::CompleteQuad {
            value: complete_value,
        }] => {
            assert_eq!(name, "boot");
            assert_eq!(value, "T");
            assert_eq!(require_name, "boot");
            assert_eq!(expected, "T");
            assert_eq!(text, "\"Hello, World!\"");
            assert_eq!(complete_value, "T");
            text.as_str()
        }
        other => panic!("unexpected canonical Hello real SemCode shape: {other:?}"),
    }
}

fn runtime_text_from_literal(text: &str) -> String {
    text.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(text)
        .to_string()
}

fn skeleton_to_admission_input(module: &HelloRealSemCodeModule) -> HelloRealSemCodeAdmissionInput {
    let mut admission_ops = Vec::with_capacity(module.ops.len());
    for op in &module.ops {
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

fn deterministic_text_hash(text: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn audit_after_route(
    route_result: HelloObservationRouteResult,
    text: &str,
    sequence_index: u64,
    audit_policy_class: HelloObservationAuditPolicyClass,
    linkage: HelloObservationAuditLinkage,
) -> HelloObservationAuditDecision {
    match route_result {
        HelloObservationRouteResult::Routed => {
            if matches!(
                audit_policy_class,
                HelloObservationAuditPolicyClass::Deferred
            ) {
                HelloObservationAuditDecision::Deferred
            } else {
                HelloObservationAuditDecision::Recorded(build_hello_observation_audit_event(
                    deterministic_text_hash(text),
                    sequence_index,
                    audit_policy_class,
                    linkage,
                ))
            }
        }
        HelloObservationRouteResult::NotRouted(_) => {
            HelloObservationAuditDecision::NotRecorded("route_not_routed")
        }
    }
}

fn canonical_cli_smoke_pipeline() -> HelloCliSmokeAcceptedDetails {
    let rel = "tests/fixtures/pending/hello/positive_hello_verbose_directional.sm";
    let module = parse_validate_lower(rel).unwrap_or_else(|stage| {
        panic!("canonical Hello fixture should reach lowering, failed at {stage}")
    });

    let semcode = emit_hello_real_semcode_skeleton(&module)
        .expect("canonical verbose Hello should emit real SemCode skeleton");
    let rendered_semcode = render_hello_real_semcode_skeleton_text(&module)
        .expect("canonical verbose Hello should render real SemCode skeleton")
        .join("\n");
    let route_text = runtime_text_from_literal(canonical_observation_literal(&semcode));

    let admission_input = skeleton_to_admission_input(&semcode);
    let admission_decision = admit_hello_real_semcode_skeleton(&admission_input);
    assert_eq!(admission_decision, HelloRealSemCodeAdmissionDecision::Admit);

    let capability_context = HelloObservationCapabilityContext {
        observation_sink_present: true,
        sink_available: true,
        requested_host_channel: None,
    };
    let capability_decision = evaluate_hello_observation_capability(&capability_context);
    assert_eq!(
        capability_decision,
        HelloObservationCapabilityDecision::Allow
    );

    let mut sink = InMemorySink::default();
    let route_result = route_hello_observation_to_sink(
        HelloObservationRouteInput {
            admitted: true,
            text: route_text.clone(),
            sequence_index: HelloObservationSequenceIndex(0),
        },
        &mut sink,
    );
    assert_eq!(route_result, HelloObservationRouteResult::Routed);
    assert_eq!(sink.events.len(), 1);

    let linkage = HelloObservationAuditLinkage {
        verifier_admission_ref: Some(11),
        capability_policy_ref: Some(22),
        sink_policy_ref: Some(33),
    };
    let audit_decision = audit_after_route(
        route_result.clone(),
        &route_text,
        0,
        HelloObservationAuditPolicyClass::Required,
        linkage,
    );
    let audit_recorded = matches!(audit_decision, HelloObservationAuditDecision::Recorded(_));
    assert!(audit_recorded);

    HelloCliSmokeAcceptedDetails {
        result: HelloCliSmokeHarnessResult::Accepted {
            semcode_ops: semcode.ops.len(),
            sink_events: sink.events.len(),
            audit_recorded,
        },
        admission_decision,
        capability_decision,
        route_result,
        semcode,
        rendered_semcode,
        sink_events: sink.events,
        audit_decision,
    }
}

fn classify_cli_smoke_fixture(rel: &str) -> HelloCliSmokeHarnessResult {
    let module = match parse_validate_lower(rel) {
        Ok(module) => module,
        Err(stage) => {
            return HelloCliSmokeHarnessResult::Rejected { stage };
        }
    };

    let semcode = match emit_hello_real_semcode_skeleton(&module) {
        Ok(module) => module,
        Err(_) => {
            return HelloCliSmokeHarnessResult::Rejected { stage: "semcode" };
        }
    };

    let rendered_semcode = match render_hello_real_semcode_skeleton_text(&module) {
        Ok(lines) => lines.join("\n"),
        Err(_) => {
            return HelloCliSmokeHarnessResult::Rejected { stage: "semcode" };
        }
    };
    assert!(
        rendered_semcode.contains("observe_text_literal \"Hello, World!\"")
            || rendered_semcode.contains("observe_text_literal"),
        "rendered skeleton should remain controlled"
    );

    let admission_input = skeleton_to_admission_input(&semcode);
    if admit_hello_real_semcode_skeleton(&admission_input)
        != HelloRealSemCodeAdmissionDecision::Admit
    {
        return HelloCliSmokeHarnessResult::Rejected { stage: "verify" };
    }

    let capability_context = HelloObservationCapabilityContext {
        observation_sink_present: true,
        sink_available: true,
        requested_host_channel: None,
    };
    if evaluate_hello_observation_capability(&capability_context)
        != HelloObservationCapabilityDecision::Allow
    {
        return HelloCliSmokeHarnessResult::Rejected {
            stage: "capability",
        };
    }

    let mut sink = InMemorySink::default();
    let route_result = route_hello_observation_to_sink(
        HelloObservationRouteInput {
            admitted: true,
            text: runtime_text_from_literal(canonical_observation_literal(&semcode)),
            sequence_index: HelloObservationSequenceIndex(0),
        },
        &mut sink,
    );

    match route_result.clone() {
        HelloObservationRouteResult::Routed => {
            let audit_decision = audit_after_route(
                route_result,
                "Hello, World!",
                0,
                HelloObservationAuditPolicyClass::Required,
                HelloObservationAuditLinkage {
                    verifier_admission_ref: Some(11),
                    capability_policy_ref: Some(22),
                    sink_policy_ref: Some(33),
                },
            );
            match audit_decision {
                HelloObservationAuditDecision::Recorded(_) => {
                    HelloCliSmokeHarnessResult::Accepted {
                        semcode_ops: semcode.ops.len(),
                        sink_events: sink.events.len(),
                        audit_recorded: true,
                    }
                }
                HelloObservationAuditDecision::Deferred => {
                    HelloCliSmokeHarnessResult::Rejected { stage: "audit" }
                }
                HelloObservationAuditDecision::NotRecorded(_) => {
                    HelloCliSmokeHarnessResult::Rejected { stage: "audit" }
                }
            }
        }
        HelloObservationRouteResult::NotRouted(_) => {
            HelloCliSmokeHarnessResult::Rejected { stage: "route" }
        }
    }
}

#[test]
fn hello_cli_smoke_pipeline_harness_accepts_canonical_verbose_fixture() {
    let details = canonical_cli_smoke_pipeline();

    assert_eq!(
        details.result,
        HelloCliSmokeHarnessResult::Accepted {
            semcode_ops: 4,
            sink_events: 1,
            audit_recorded: true,
        }
    );
    assert_eq!(
        details.admission_decision,
        HelloRealSemCodeAdmissionDecision::Admit
    );
    assert_eq!(
        details.capability_decision,
        HelloObservationCapabilityDecision::Allow
    );
    assert_eq!(details.route_result, HelloObservationRouteResult::Routed);
    assert_eq!(details.semcode.ops.len(), 4);

    match &details.semcode.ops[..] {
        [HelloRealSemCodeOp::DeclareLocalQuad { name, value }, HelloRealSemCodeOp::RequireQuadEq {
            name: req_name,
            expected,
        }, HelloRealSemCodeOp::ObserveTextLiteral { text }, HelloRealSemCodeOp::CompleteQuad {
            value: complete_value,
        }] => {
            assert_eq!(name, "boot");
            assert_eq!(value, "T");
            assert_eq!(req_name, "boot");
            assert_eq!(expected, "T");
            assert_eq!(text, "\"Hello, World!\"");
            assert_eq!(complete_value, "T");
        }
        other => panic!("unexpected Hello real SemCode shape: {other:?}"),
    }

    assert_eq!(
        details.rendered_semcode,
        "\
declare_local_quad boot = T
require_quad_eq boot T
observe_text_literal \"Hello, World!\"
complete_quad T"
    );
    assert!(details
        .rendered_semcode
        .contains("observe_text_literal \"Hello, World!\""));
    assert!(!details
        .rendered_semcode
        .contains("request_observation_text"));
    assert!(!details.rendered_semcode.contains("print"));
    assert!(!details.rendered_semcode.contains("stdout"));
    assert!(!details.rendered_semcode.contains("io.write"));
    assert!(!details.rendered_semcode.contains("opcode"));
    assert!(!details.rendered_semcode.contains("bytecode"));

    assert_eq!(details.sink_events.len(), 1);
    let event = &details.sink_events[0];
    assert_eq!(event.operation_kind, "controlled_observation_text");
    assert_eq!(
        event.observation_class,
        HelloObservationClass::ControlledText
    );
    assert_eq!(event.text, "Hello, World!");
    assert_eq!(event.sequence_index, HelloObservationSequenceIndex(0));

    match &details.audit_decision {
        HelloObservationAuditDecision::Recorded(event) => {
            assert_eq!(
                event.event_kind,
                HelloObservationAuditEventKind::Observation
            );
            assert_eq!(event.operation_kind, "controlled_observation_text");
            assert_eq!(event.observation_class, "controlled");
            assert_eq!(
                event.payload_ref,
                HelloObservationAuditPayloadRef::LiteralTextHash(deterministic_text_hash(
                    "Hello, World!"
                ))
            );
            assert_eq!(event.sequence_index.0, 0);
            assert_eq!(
                event.audit_policy_class,
                HelloObservationAuditPolicyClass::Required
            );
            assert_eq!(event.linkage.verifier_admission_ref, Some(11));
            assert_eq!(event.linkage.capability_policy_ref, Some(22));
            assert_eq!(event.linkage.sink_policy_ref, Some(33));
        }
        other => panic!("expected local audit record, got {other:?}"),
    }
}

#[test]
fn hello_cli_smoke_pipeline_harness_rejects_negative_fixtures_before_route_or_audit() {
    for rel in [
        "tests/fixtures/pending/hello/negative_hello_print_legacy_canonical.sm",
        "tests/fixtures/pending/hello/negative_hello_general_io_shape.sm",
        "tests/fixtures/pending/hello/negative_hello_observe_non_text_payload.sm",
        "tests/fixtures/pending/hello/negative_hello_require_side_effect_shape.sm",
    ] {
        assert_eq!(
            classify_cli_smoke_fixture(rel),
            HelloCliSmokeHarnessResult::Rejected { stage: "parser" },
            "expected {rel} to reject before route/audit"
        );
    }
}
