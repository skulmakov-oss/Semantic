use std::collections::BTreeMap;
use std::path::PathBuf;

use sm_verify::hello_pending_admission::{
    evaluate_hello_pending_policy, HelloPendingPolicyInput, HelloPendingPolicyReason,
    HelloPendingPolicyResult,
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

fn parse_fixture(rel: &str) -> BTreeMap<String, String> {
    let input = fixture_text(rel);
    let mut map = BTreeMap::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let (key, value) = trimmed
            .split_once('=')
            .unwrap_or_else(|| panic!("invalid fixture line in {rel}: {trimmed}"));
        let key = key.trim().to_string();
        let value = value.trim();
        let parsed = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value[1..value.len() - 1].to_string()
        } else {
            value.to_string()
        };
        map.insert(key, parsed);
    }

    map
}

fn parse_bool(value: &str, rel: &str, key: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        other => panic!("expected boolean for {key} in {rel}, got {other}"),
    }
}

fn fixture_input(rel: &str) -> HelloPendingPolicyInput {
    let fields = parse_fixture(rel);
    let requested_host_channel = fields.get("requested_host_channel").cloned();

    HelloPendingPolicyInput {
        operation_kind: fields
            .get("operation_kind")
            .cloned()
            .unwrap_or_else(|| "controlled_observation_text".to_string()),
        payload_type: fields
            .get("payload_type")
            .cloned()
            .unwrap_or_else(|| "text_literal".to_string()),
        capability_observation_sink: fields
            .get("capability_observation_sink")
            .cloned()
            .unwrap_or_else(|| "present".to_string()),
        sink_available: fields
            .get("sink_available")
            .map(|value| parse_bool(value, rel, "sink_available"))
            .unwrap_or(true),
        audit_policy: fields
            .get("audit_policy")
            .cloned()
            .unwrap_or_else(|| "required".to_string()),
        audit_available: fields
            .get("audit_available")
            .map(|value| parse_bool(value, rel, "audit_available"))
            .unwrap_or(true),
        deterministic_order: fields
            .get("deterministic_order")
            .map(|value| parse_bool(value, rel, "deterministic_order"))
            .unwrap_or(true),
        requested_host_channel,
    }
}

fn assert_admission(rel: &str, expected: HelloPendingPolicyResult) -> HelloPendingPolicyInput {
    let input = fixture_input(rel);
    let actual = evaluate_hello_pending_policy(&input);
    assert_eq!(actual, expected, "fixture {rel} evaluated unexpectedly");
    input
}

#[test]
fn hello_pending_verifier_admission_evaluates_all_policy_fixtures() {
    let positive = assert_admission(
        "tests/fixtures/pending/hello_policy/positive_observation_policy_admitted.toml",
        HelloPendingPolicyResult::Admit,
    );
    assert_eq!(positive.operation_kind, "controlled_observation_text");
    assert_eq!(positive.payload_type, "text_literal");

    assert_admission(
        "tests/fixtures/pending/hello_policy/negative_missing_observation_capability.toml",
        HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::MissingObservationCapability,
        },
    );
    // Preservation, not a new regression: stdout keeps its own distinct
    // diagnostic and is not folded into GenericIoNotAllowed.
    assert_admission(
        "tests/fixtures/pending/hello_policy/negative_stdout_fallback.toml",
        HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::StdoutNotDefaultSink,
        },
    );
    assert_admission(
        "tests/fixtures/pending/hello_policy/negative_missing_audit_when_required.toml",
        HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::AuditRequiredButUnavailable,
        },
    );
    assert_admission(
        "tests/fixtures/pending/hello_policy/negative_nondeterministic_sink.toml",
        HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::NondeterministicSinkConfiguration,
        },
    );
}

#[test]
fn hello_pending_verifier_admission_rejects_unsupported_operation_kind() {
    let mut input = fixture_input(
        "tests/fixtures/pending/hello_policy/positive_observation_policy_admitted.toml",
    );
    input.operation_kind = "generic_io".to_string();
    let actual = evaluate_hello_pending_policy(&input);
    assert_eq!(
        actual,
        HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::UnsupportedOperationKind,
        }
    );
}

#[test]
fn hello_pending_verifier_admission_rejects_unsupported_payload_type() {
    let mut input = fixture_input(
        "tests/fixtures/pending/hello_policy/positive_observation_policy_admitted.toml",
    );
    input.payload_type = "bytes".to_string();
    let actual = evaluate_hello_pending_policy(&input);
    assert_eq!(
        actual,
        HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::UnsupportedPayloadType,
        }
    );
}

#[test]
fn hello_pending_verifier_admission_rejects_both_unsupported_with_operation_kind_priority() {
    // evaluate_hello_pending_policy checks operation_kind before payload_type
    // (both checks run right after the capability gate), so when both fields
    // are unsupported the operation_kind check wins deterministically.
    let mut input = fixture_input(
        "tests/fixtures/pending/hello_policy/positive_observation_policy_admitted.toml",
    );
    input.operation_kind = "generic_io".to_string();
    input.payload_type = "bytes".to_string();
    let actual = evaluate_hello_pending_policy(&input);
    assert_eq!(
        actual,
        HelloPendingPolicyResult::Deny {
            reason: HelloPendingPolicyReason::UnsupportedOperationKind,
        }
    );
}

#[test]
fn hello_pending_verifier_admission_rejects_forbidden_host_channels() {
    // The controlled-observation contract excludes six host channels.
    // stdout gets its own diagnostic (covered above); the remaining five
    // are generic-IO denials.
    for channel in ["print", "io.write", "file", "network", "stdin"] {
        let mut input = fixture_input(
            "tests/fixtures/pending/hello_policy/positive_observation_policy_admitted.toml",
        );
        input.requested_host_channel = Some(channel.to_string());
        let actual = evaluate_hello_pending_policy(&input);
        assert_eq!(
            actual,
            HelloPendingPolicyResult::Deny {
                reason: HelloPendingPolicyReason::GenericIoNotAllowed,
            },
            "channel {channel}"
        );
    }
}

#[test]
fn hello_pending_verifier_admission_policy_registry_stays_pending_only() {
    let registry = fixture_text("docs/language/semantic_hello_policy_fixtures.md");
    let readme = fixture_text("tests/fixtures/pending/hello_policy/README.md");

    for needle in ["pending", "not executable", "not production verifier"] {
        assert!(
            registry.contains(needle) || readme.contains(needle),
            "policy registry/readme should mention {needle}"
        );
    }
}
