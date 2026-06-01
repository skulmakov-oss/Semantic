use prom_audit::hello_observation_audit::{
    build_hello_observation_audit_event, HelloObservationAuditEvent,
    HelloObservationAuditEventKind, HelloObservationAuditLinkage, HelloObservationAuditPayloadRef,
    HelloObservationAuditPolicyClass, HelloObservationAuditSequenceIndex,
};
use prom_cap::hello_observation_capability::{
    evaluate_hello_observation_capability, HelloObservationCapability,
    HelloObservationCapabilityContext, HelloObservationCapabilityDecision,
    HelloObservationCapabilityDenial, HelloObservationCapabilityPolicy,
};
use sm_runtime_core::hello_observation_sink::{
    HelloObservationClass, HelloObservationEvent, HelloObservationSequenceIndex,
    HelloObservationSink, HelloObservationSinkError,
};
use sm_verify::hello_pending_admission::{
    evaluate_hello_pending_policy, HelloPendingPolicyInput, HelloPendingPolicyReason,
    HelloPendingPolicyResult,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HelloIsolatedHarnessResult {
    Admit {
        sink_events: usize,
        audit_events: usize,
    },
    Deny {
        reason: &'static str,
    },
}

#[derive(Debug, Default)]
struct InMemoryHelloObservationSink {
    events: Vec<HelloObservationEvent>,
}

impl HelloObservationSink for InMemoryHelloObservationSink {
    fn observe(&mut self, event: HelloObservationEvent) -> Result<(), HelloObservationSinkError> {
        let expected_index = self.events.len() as u64;
        if event.sequence_index.0 != expected_index {
            return Err(HelloObservationSinkError::NondeterministicOrder);
        }
        self.events.push(event);
        Ok(())
    }
}

fn fixture_root() -> PathBuf {
    PathBuf::from("tests/fixtures/pending/hello_policy")
}

fn fixture_path(name: &str) -> PathBuf {
    fixture_root().join(name)
}

fn parse_fixture_kv(path: &Path) -> BTreeMap<String, String> {
    let text =
        fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let mut map = BTreeMap::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("invalid fixture line in {}: {line}", path.display()));
        let key = key.trim().to_string();
        let value = value.trim().trim_matches('"').to_string();
        map.insert(key, value);
    }
    map
}

fn parse_bool_or(map: &BTreeMap<String, String>, key: &str, default: bool) -> bool {
    map.get(key).map(|value| value == "true").unwrap_or(default)
}

fn parse_requested_host_channel(map: &BTreeMap<String, String>) -> Option<String> {
    map.get("requested_host_channel").cloned()
}

fn capability_requested_host_channel(map: &BTreeMap<String, String>) -> Option<&'static str> {
    match map.get("requested_host_channel").map(String::as_str) {
        Some("stdout") => Some("stdout"),
        Some("print") => Some("print"),
        Some("io.write") => Some("io.write"),
        Some("file") => Some("file"),
        Some("network") => Some("network"),
        Some("stdin") => Some("stdin"),
        _ => None,
    }
}

fn to_pending_input(map: &BTreeMap<String, String>) -> HelloPendingPolicyInput {
    HelloPendingPolicyInput {
        operation_kind: map
            .get("operation_kind")
            .cloned()
            .unwrap_or_else(|| "controlled_observation_text".to_string()),
        payload_type: map
            .get("payload_type")
            .cloned()
            .unwrap_or_else(|| "text_literal".to_string()),
        capability_observation_sink: map
            .get("capability_observation_sink")
            .cloned()
            .unwrap_or_else(|| "present".to_string()),
        sink_available: parse_bool_or(map, "sink_available", true),
        audit_policy: map
            .get("audit_policy")
            .cloned()
            .unwrap_or_else(|| "required".to_string()),
        audit_available: parse_bool_or(map, "audit_available", true),
        deterministic_order: parse_bool_or(map, "deterministic_order", true),
        requested_host_channel: parse_requested_host_channel(map),
    }
}

fn to_capability_context(map: &BTreeMap<String, String>) -> HelloObservationCapabilityContext {
    HelloObservationCapabilityContext {
        observation_sink_present: map
            .get("capability_observation_sink")
            .map(|value| value == "present")
            .unwrap_or(true),
        sink_available: map
            .get("sink_available")
            .map(|value| value == "true")
            .unwrap_or(true),
        requested_host_channel: capability_requested_host_channel(map),
    }
}

fn audit_policy_class_from_input(
    input: &HelloPendingPolicyInput,
) -> HelloObservationAuditPolicyClass {
    if input.audit_policy == "required" {
        HelloObservationAuditPolicyClass::Required
    } else {
        HelloObservationAuditPolicyClass::Deferred
    }
}

fn denial_reason_from_pending(reason: HelloPendingPolicyReason) -> &'static str {
    match reason {
        HelloPendingPolicyReason::MissingObservationCapability => "missing_observation_capability",
        HelloPendingPolicyReason::StdoutNotDefaultSink => "stdout_not_default_sink",
        HelloPendingPolicyReason::AuditRequiredButUnavailable => "audit_required_but_unavailable",
        HelloPendingPolicyReason::NondeterministicSinkConfiguration => {
            "nondeterministic_sink_configuration"
        }
    }
}

fn denial_reason_from_capability(reason: HelloObservationCapabilityDenial) -> &'static str {
    match reason {
        HelloObservationCapabilityDenial::MissingObservationCapability => {
            "missing_observation_capability"
        }
        HelloObservationCapabilityDenial::SinkUnavailable => "sink_unavailable",
        HelloObservationCapabilityDenial::StdoutNotDefaultSink => "stdout_not_default_sink",
        HelloObservationCapabilityDenial::GenericIoNotAllowed => "generic_io_not_allowed",
    }
}

fn stable_payload_hash(text: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn build_controlled_observation_event(text: &str, sequence_index: u64) -> HelloObservationEvent {
    HelloObservationEvent {
        operation_kind: "controlled_observation_text",
        observation_class: HelloObservationClass::ControlledText,
        text: text.to_string(),
        sequence_index: HelloObservationSequenceIndex(sequence_index),
    }
}

fn run_isolated_harness(path: &Path) -> (HelloIsolatedHarnessResult, usize, usize) {
    let map = parse_fixture_kv(path);
    let pending_input = to_pending_input(&map);
    let capability_context = to_capability_context(&map);

    match evaluate_hello_pending_policy(&pending_input) {
        HelloPendingPolicyResult::Deny { reason } => {
            let reason = denial_reason_from_pending(reason);
            return (HelloIsolatedHarnessResult::Deny { reason }, 0, 0);
        }
        HelloPendingPolicyResult::Admit => {}
    }

    match evaluate_hello_observation_capability(&capability_context) {
        HelloObservationCapabilityDecision::Deny(reason) => {
            let reason = denial_reason_from_capability(reason);
            return (HelloIsolatedHarnessResult::Deny { reason }, 0, 0);
        }
        HelloObservationCapabilityDecision::Allow => {}
    }

    let _policy = HelloObservationCapabilityPolicy;
    let capability = HelloObservationCapability::ObservationSink;
    assert_eq!(capability, HelloObservationCapability::ObservationSink);

    let mut sink = InMemoryHelloObservationSink::default();
    let text = "Hello, World!";
    let sink_event = build_controlled_observation_event(text, 0);
    sink.observe(sink_event)
        .expect("controlled sink must admit one ordered event");
    assert_eq!(sink.events.len(), 1);
    assert_eq!(sink.events[0].operation_kind, "controlled_observation_text");
    assert_eq!(
        sink.events[0].observation_class,
        HelloObservationClass::ControlledText
    );
    assert_eq!(sink.events[0].text, text);
    assert_eq!(
        sink.events[0].sequence_index,
        HelloObservationSequenceIndex(0)
    );

    let linkage = HelloObservationAuditLinkage {
        verifier_admission_ref: Some(1),
        capability_policy_ref: Some(2),
        sink_policy_ref: Some(3),
    };
    let audit_event: HelloObservationAuditEvent = build_hello_observation_audit_event(
        stable_payload_hash(text),
        0,
        audit_policy_class_from_input(&pending_input),
        linkage,
    );

    assert_eq!(
        audit_event.event_kind,
        HelloObservationAuditEventKind::Observation
    );
    assert_eq!(audit_event.operation_kind, "controlled_observation_text");
    assert_eq!(audit_event.observation_class, "controlled");
    assert_eq!(
        audit_event.sequence_index,
        HelloObservationAuditSequenceIndex(0)
    );
    assert_eq!(
        audit_event.payload_ref,
        HelloObservationAuditPayloadRef::LiteralTextHash(stable_payload_hash(text))
    );

    (
        HelloIsolatedHarnessResult::Admit {
            sink_events: sink.events.len(),
            audit_events: 1,
        },
        sink.events.len(),
        1,
    )
}

fn assert_fixture_outcome(
    name: &str,
    expected_result: &str,
    expected_reason: Option<&'static str>,
) {
    let path = fixture_path(name);
    let (result, sink_events, audit_events) = run_isolated_harness(&path);

    match expected_result {
        "admit" => {
            assert_eq!(
                result,
                HelloIsolatedHarnessResult::Admit {
                    sink_events: 1,
                    audit_events: 1,
                }
            );
            assert_eq!(sink_events, 1);
            assert_eq!(audit_events, 1);
        }
        "deny" => {
            let reason = expected_reason.expect("deny fixtures must have a reason");
            assert_eq!(result, HelloIsolatedHarnessResult::Deny { reason });
            assert_eq!(sink_events, 0);
            assert_eq!(audit_events, 0);
        }
        other => panic!(
            "unexpected expected_policy_result in {}: {other}",
            path.display()
        ),
    }
}

#[test]
fn hello_isolated_policy_harness_covers_all_pending_policy_fixtures() {
    let fixtures = [
        ("positive_observation_policy_admitted.toml", "admit", None),
        (
            "negative_missing_observation_capability.toml",
            "deny",
            Some("missing_observation_capability"),
        ),
        (
            "negative_stdout_fallback.toml",
            "deny",
            Some("stdout_not_default_sink"),
        ),
        (
            "negative_missing_audit_when_required.toml",
            "deny",
            Some("audit_required_but_unavailable"),
        ),
        (
            "negative_nondeterministic_sink.toml",
            "deny",
            Some("nondeterministic_sink_configuration"),
        ),
    ];

    for (name, expected_result, expected_reason) in fixtures {
        assert_fixture_outcome(name, expected_result, expected_reason);
    }
}

#[test]
fn hello_isolated_policy_harness_preserves_controlled_operation_shape() {
    let path = fixture_path("positive_observation_policy_admitted.toml");
    let (result, _, _) = run_isolated_harness(&path);

    assert_eq!(
        result,
        HelloIsolatedHarnessResult::Admit {
            sink_events: 1,
            audit_events: 1,
        }
    );

    let map = parse_fixture_kv(&path);
    let pending_input = to_pending_input(&map);
    assert_eq!(pending_input.operation_kind, "controlled_observation_text");
    assert_eq!(pending_input.payload_type, "text_literal");
    assert_eq!(pending_input.requested_host_channel, None);
}
