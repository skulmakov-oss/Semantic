use std::path::PathBuf;

use sm_emit::hello_observation_bytes::{
    emit_hello_observation_bytes_from_real_semcode_skeleton, emit_hello_observation_bytes_gated,
    render_hello_observation_bytes_gated, HelloObservationByteEmissionError,
    HelloObservationByteRecord,
};
use sm_emit::hello_real_semcode::{
    emit_hello_real_semcode_skeleton, HelloRealSemCodeModule, HelloRealSemCodeOp,
};
use sm_front::hello_parser::parse_hello_file;
use sm_front::hello_sema::validate_hello_file;
use sm_ir::hello_ir::lower_hello_checked_file;

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

fn canonical_real_semcode() -> HelloRealSemCodeModule {
    let module = parse_validate_lower();
    emit_hello_real_semcode_skeleton(&module)
        .expect("canonical verbose Hello should lower to real SemCode skeleton")
}

fn render_text_literal(text: &str) -> String {
    format!("{text:?}")
}

fn assert_reject(module: &HelloRealSemCodeModule, expected: HelloObservationByteEmissionError) {
    assert_eq!(
        emit_hello_observation_bytes_gated(module),
        Err(expected.clone())
    );
    assert_eq!(render_hello_observation_bytes_gated(module), Err(expected));
}

#[test]
fn hello_observation_byte_bridge_emits_gated_provisional_representation() {
    let module = canonical_real_semcode();
    let emitted = emit_hello_observation_bytes_from_real_semcode_skeleton(&module)
        .expect("canonical hello should emit gated observation bytes");
    let gated_emitted = emit_hello_observation_bytes_gated(&module)
        .expect("canonical hello should emit gated observation bytes");
    let rendered = render_hello_observation_bytes_gated(&module)
        .expect("canonical hello should render gated observation bytes");
    let rendered_text = String::from_utf8(rendered).expect("utf8 rendered bytes");

    assert_eq!(emitted.format_status, "gated_provisional_not_production");
    assert_eq!(gated_emitted, emitted);
    assert_eq!(emitted.records.len(), 4);
    match &emitted.records[..] {
        [HelloObservationByteRecord::DeclareLocalQuad { symbol, value }, HelloObservationByteRecord::RequireQuadEq {
            symbol: req_symbol,
            expected,
        }, HelloObservationByteRecord::ObserveTextLiteral {
            symbolic_op,
            text_const_index,
            observation_class,
            sequence_index,
            policy_ref,
        }, HelloObservationByteRecord::CompleteQuad {
            value: complete_value,
        }] => {
            assert_eq!(symbol, "boot");
            assert_eq!(value, "T");
            assert_eq!(req_symbol, "boot");
            assert_eq!(expected, "T");
            assert_eq!(*symbolic_op, "OBSERVE_TEXT_LITERAL");
            assert_eq!(*text_const_index, 0);
            assert_eq!(*observation_class, "ControlledText");
            assert_eq!(*sequence_index, 0);
            assert_eq!(*policy_ref, 0);
            assert_eq!(complete_value, "T");
        }
        other => panic!("unexpected gated observation byte record order: {other:?}"),
    }

    let expected = "\
FORMAT_STATUS gated_provisional_not_production
DECLARE_LOCAL_QUAD boot T
REQUIRE_QUAD_EQ boot T
OBSERVE_TEXT_LITERAL const#0 ControlledText seq#0 policy#0
COMPLETE_QUAD T
CONST_TEXT #0 \"Hello, World!\"
";
    assert_eq!(rendered_text, expected.trim_end());
}

#[test]
fn hello_observation_byte_emission_rejects_host_output_markers() {
    let module = canonical_real_semcode();

    for marker in ["stdout", "print", "io.write", "file", "network", "stdin"] {
        let mut mutated = module.clone();
        if let HelloRealSemCodeOp::ObserveTextLiteral { text } = &mut mutated.ops[2] {
            *text = render_text_literal(marker);
        }
        assert_reject(
            &mutated,
            HelloObservationByteEmissionError::ForbiddenHostOutput,
        );
    }
}

#[test]
fn hello_observation_byte_emission_rejects_missing_observation_and_wrong_order() {
    let module = canonical_real_semcode();

    let mut missing_observation = module.clone();
    missing_observation.ops.remove(2);
    assert_reject(
        &missing_observation,
        HelloObservationByteEmissionError::MissingControlledObservation,
    );

    let mut wrong_order = module.clone();
    wrong_order.ops.swap(1, 2);
    assert_reject(
        &wrong_order,
        HelloObservationByteEmissionError::UnsupportedShape,
    );
}

#[test]
fn hello_observation_byte_emission_rejects_non_hello_text() {
    let module = canonical_real_semcode();

    let mut mutated = module.clone();
    if let HelloRealSemCodeOp::ObserveTextLiteral { text } = &mut mutated.ops[2] {
        *text = render_text_literal("Not Hello");
    }

    assert_reject(
        &mutated,
        HelloObservationByteEmissionError::UnsupportedShape,
    );
}
