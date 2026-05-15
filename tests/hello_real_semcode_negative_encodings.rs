use std::path::PathBuf;
use std::vec::Vec;

use sm_emit::hello_real_semcode::{
    emit_hello_real_semcode_skeleton, HelloRealSemCodeOp,
};
use sm_front::hello_parser::parse_hello_file;
use sm_front::hello_sema::validate_hello_file;
use sm_ir::hello_ir::lower_hello_checked_file;
use sm_verify::hello_real_semcode_admission::{
    admit_hello_real_semcode_skeleton, HelloRealSemCodeAdmissionDecision,
    HelloRealSemCodeAdmissionError, HelloRealSemCodeAdmissionInput,
    HelloRealSemCodeAdmissionOp,
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
    let parsed = parse_hello_file(&input)
        .unwrap_or_else(|err| panic!("parser unexpectedly rejected canonical hello fixture: {err}"));
    let checked = validate_hello_file(parsed)
        .unwrap_or_else(|err| panic!("sema unexpectedly rejected canonical hello fixture: {err}"));
    lower_hello_checked_file(&checked)
        .unwrap_or_else(|err| panic!("lowering unexpectedly rejected canonical hello fixture: {err}"))
}

fn render_text_literal(text: &str) -> String {
    format!("{text:?}")
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
                HelloRealSemCodeAdmissionOp::ObserveTextLiteral {
                    text: text.clone(),
                }
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

fn assert_admit(input: &HelloRealSemCodeAdmissionInput) {
    assert_eq!(
        admit_hello_real_semcode_skeleton(input),
        HelloRealSemCodeAdmissionDecision::Admit
    );
}

fn assert_reject(input: HelloRealSemCodeAdmissionInput, expected: HelloRealSemCodeAdmissionError) {
    match admit_hello_real_semcode_skeleton(&input) {
        HelloRealSemCodeAdmissionDecision::Admit => panic!("expected rejection, got admit"),
        HelloRealSemCodeAdmissionDecision::Reject(reason) => assert_eq!(reason, expected),
    }
}

#[test]
fn hello_real_semcode_negative_encodings_accept_canonical_and_reject_forbidden_variants() {
    let canonical = canonical_admission_input();
    assert_admit(&canonical);

    for (replacement, expected) in [
        ("stdout", HelloRealSemCodeAdmissionError::StdoutNotAllowed),
        ("print", HelloRealSemCodeAdmissionError::PrintNotAllowed),
        ("io.write", HelloRealSemCodeAdmissionError::GenericIoNotAllowed),
        ("file", HelloRealSemCodeAdmissionError::GenericIoNotAllowed),
        ("network", HelloRealSemCodeAdmissionError::GenericIoNotAllowed),
        ("stdin", HelloRealSemCodeAdmissionError::GenericIoNotAllowed),
        ("opcode", HelloRealSemCodeAdmissionError::OpcodeOrBytecodeNotAllowed),
        ("bytecode", HelloRealSemCodeAdmissionError::OpcodeOrBytecodeNotAllowed),
        ("Not Hello", HelloRealSemCodeAdmissionError::NonTextObservation),
    ] {
        let mut input = canonical.clone();
        if let HelloRealSemCodeAdmissionOp::ObserveTextLiteral { text } = &mut input.ops[2] {
            *text = render_text_literal(replacement);
        } else {
            panic!("canonical observation slot changed");
        }
        assert_reject(input, expected);
    }
}

#[test]
fn hello_real_semcode_negative_encodings_reject_order_bypasses_and_missing_ops() {
    let canonical = canonical_admission_input();

    let mut observation_before_requirement = canonical.clone();
    observation_before_requirement.ops.swap(1, 2);
    assert_reject(
        observation_before_requirement,
        HelloRealSemCodeAdmissionError::InvalidOperationOrder,
    );

    let mut completion_before_observation = canonical.clone();
    completion_before_observation.ops.swap(2, 3);
    assert_reject(
        completion_before_observation,
        HelloRealSemCodeAdmissionError::InvalidOperationOrder,
    );

    let mut extra_observation = canonical.clone();
    extra_observation.ops.insert(
        3,
        HelloRealSemCodeAdmissionOp::ObserveTextLiteral {
            text: render_text_literal("extra observation"),
        },
    );
    assert_reject(
        extra_observation,
        HelloRealSemCodeAdmissionError::UnsupportedShape,
    );

    let mut missing_requirement = canonical.clone();
    missing_requirement.ops.remove(1);
    assert_reject(
        missing_requirement,
        HelloRealSemCodeAdmissionError::MissingRequirement,
    );

    let mut missing_observation = canonical.clone();
    missing_observation.ops.remove(2);
    assert_reject(
        missing_observation,
        HelloRealSemCodeAdmissionError::MissingObservation,
    );

    let mut missing_completion = canonical.clone();
    missing_completion.ops.remove(3);
    assert_reject(
        missing_completion,
        HelloRealSemCodeAdmissionError::MissingCompletion,
    );
}

