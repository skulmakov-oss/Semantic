use std::path::PathBuf;

use sm_emit::hello_real_semcode::{
    emit_hello_real_semcode_skeleton, render_hello_real_semcode_skeleton_text,
    HelloRealSemCodeOp,
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
    let parsed = parse_hello_file(&input)
        .unwrap_or_else(|err| panic!("parser unexpectedly rejected canonical hello fixture: {err}"));
    let checked = validate_hello_file(parsed)
        .unwrap_or_else(|err| panic!("sema unexpectedly rejected canonical hello fixture: {err}"));
    lower_hello_checked_file(&checked)
        .unwrap_or_else(|err| panic!("lowering unexpectedly rejected canonical hello fixture: {err}"))
}

#[test]
fn hello_real_semcode_skeleton_emits_decided_operation_order() {
    let module = parse_validate_lower();
    let semcode = emit_hello_real_semcode_skeleton(&module)
        .expect("canonical verbose Hello should lower to real SemCode skeleton");

    assert_eq!(semcode.ops.len(), 4);
    match &semcode.ops[..] {
        [
            HelloRealSemCodeOp::DeclareLocalQuad { name, value },
            HelloRealSemCodeOp::RequireQuadEq { name: req_name, expected },
            HelloRealSemCodeOp::ObserveTextLiteral { text },
            HelloRealSemCodeOp::CompleteQuad { value: complete_value },
        ] => {
            assert_eq!(name, "boot");
            assert_eq!(value, "T");
            assert_eq!(req_name, "boot");
            assert_eq!(expected, "T");
            assert_eq!(text, "\"Hello, World!\"");
            assert_eq!(complete_value, "T");
        }
        other => panic!("unexpected Hello real SemCode skeleton order: {other:?}"),
    }
}

#[test]
fn hello_real_semcode_skeleton_renders_expected_text() {
    let module = parse_validate_lower();
    let rendered = render_hello_real_semcode_skeleton_text(&module)
        .expect("canonical verbose Hello should render real SemCode skeleton");
    let expected = "\
declare_local_quad boot = T
require_quad_eq boot T
observe_text_literal \"Hello, World!\"
complete_quad T
";

    assert_eq!(rendered.join("\n"), expected.trim_end());
}

#[test]
fn hello_real_semcode_skeleton_no_stdout_print_or_opcode_words() {
    let module = parse_validate_lower();
    let rendered = render_hello_real_semcode_skeleton_text(&module)
        .expect("canonical verbose Hello should render real SemCode skeleton");
    let joined = rendered.join("\n");

    assert!(joined.contains("observe_text_literal \"Hello, World!\""));
    assert!(!joined.contains("request_observation_text"));
    assert!(!joined.contains("print"));
    assert!(!joined.contains("stdout"));
    assert!(!joined.contains("io.write"));
    assert!(!joined.contains("opcode"));
    assert!(!joined.contains("bytecode"));
}
