use std::path::PathBuf;

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
use std::vec::Vec;

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
    let mut admission_ops = Vec::new();
    for op in ops {
        let admission_op = match op {
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
        };
        admission_ops.push(admission_op);
    }

    HelloRealSemCodeAdmissionInput { ops: admission_ops }
}

fn canonical_admission_input() -> HelloRealSemCodeAdmissionInput {
    let module = parse_validate_lower();
    let semcode = emit_hello_real_semcode_skeleton(&module)
        .expect("canonical verbose Hello should lower to real SemCode skeleton");
    skeleton_to_admission_input(&semcode.ops)
}

fn assert_reject(input: HelloRealSemCodeAdmissionInput, expected: HelloRealSemCodeAdmissionError) {
    match admit_hello_real_semcode_skeleton(&input) {
        HelloRealSemCodeAdmissionDecision::Admit => {
            panic!("expected rejection, got admit")
        }
        HelloRealSemCodeAdmissionDecision::Reject(reason) => assert_eq!(reason, expected),
    }
}

#[test]
fn hello_real_semcode_admission_accepts_canonical_skeleton() {
    let input = canonical_admission_input();
    assert_eq!(
        admit_hello_real_semcode_skeleton(&input),
        HelloRealSemCodeAdmissionDecision::Admit
    );
}

#[test]
fn hello_real_semcode_admission_rejects_order_and_shape_mutations() {
    let canonical = canonical_admission_input();

    let mut swapped = canonical.clone();
    swapped.ops.swap(1, 2);
    assert_reject(
        swapped,
        HelloRealSemCodeAdmissionError::InvalidOperationOrder,
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

    let mut wrong_name = canonical.clone();
    if let HelloRealSemCodeAdmissionOp::DeclareLocalQuad { name, .. } = &mut wrong_name.ops[0] {
        *name = "not_boot".to_string();
    }
    assert_reject(wrong_name, HelloRealSemCodeAdmissionError::UnsupportedShape);

    let mut wrong_value = canonical.clone();
    if let HelloRealSemCodeAdmissionOp::DeclareLocalQuad { value, .. } = &mut wrong_value.ops[0] {
        *value = "F".to_string();
    }
    assert_reject(wrong_value, HelloRealSemCodeAdmissionError::UnsupportedShape);

    let mut wrong_completion = canonical.clone();
    if let HelloRealSemCodeAdmissionOp::CompleteQuad { value } = &mut wrong_completion.ops[3] {
        *value = "F".to_string();
    }
    assert_reject(
        wrong_completion,
        HelloRealSemCodeAdmissionError::MissingCompletion,
    );
}

#[test]
fn hello_real_semcode_admission_rejects_stdout_print_and_generic_io_text() {
    let canonical = canonical_admission_input();

    let mut stdout = canonical.clone();
    if let HelloRealSemCodeAdmissionOp::ObserveTextLiteral { text } = &mut stdout.ops[2] {
        *text = render_text_literal("stdout");
    }
    assert_reject(stdout, HelloRealSemCodeAdmissionError::StdoutNotAllowed);

    let mut print = canonical.clone();
    if let HelloRealSemCodeAdmissionOp::ObserveTextLiteral { text } = &mut print.ops[2] {
        *text = render_text_literal("print");
    }
    assert_reject(print, HelloRealSemCodeAdmissionError::PrintNotAllowed);

    let mut io_write = canonical.clone();
    if let HelloRealSemCodeAdmissionOp::ObserveTextLiteral { text } = &mut io_write.ops[2] {
        *text = render_text_literal("io.write");
    }
    assert_reject(io_write, HelloRealSemCodeAdmissionError::GenericIoNotAllowed);
}

#[test]
fn hello_real_semcode_admission_rejects_opcode_and_bytecode_markers() {
    let canonical = canonical_admission_input();

    let mut opcode = canonical.clone();
    if let HelloRealSemCodeAdmissionOp::ObserveTextLiteral { text } = &mut opcode.ops[2] {
        *text = render_text_literal("opcode");
    }
    assert_reject(opcode, HelloRealSemCodeAdmissionError::OpcodeOrBytecodeNotAllowed);

    let mut bytecode = canonical.clone();
    if let HelloRealSemCodeAdmissionOp::ObserveTextLiteral { text } = &mut bytecode.ops[2] {
        *text = render_text_literal("bytecode");
    }
    assert_reject(
        bytecode,
        HelloRealSemCodeAdmissionError::OpcodeOrBytecodeNotAllowed,
    );
}
