use std::path::PathBuf;

use sm_front::hello_parser::parse_hello_file;
use sm_front::hello_sema::validate_hello_file;
use sm_ir::hello_ir::lower_hello_checked_file;
use sm_ir::hello_semcode::{emit_hello_conceptual_semcode, render_hello_conceptual_semcode};

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

fn fixture_body_lines() -> String {
    fixture_text("tests/fixtures/pending/hello_semcode/positive_hello_verbose_conceptual.semcode.txt")
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn hello_semcode_conceptual_emitter_pending_matches_fixture_body() {
    let module = parse_validate_lower();
    let conceptual = emit_hello_conceptual_semcode(&module)
        .expect("conceptual emitter should render planning text");
    let rendered = conceptual.lines.join("\n");
    let expected = fixture_body_lines();
    assert_eq!(rendered.trim_end(), expected.trim_end());
}

#[test]
fn hello_semcode_conceptual_emitter_pending_no_print_stdout_or_io() {
    let module = parse_validate_lower();
    let conceptual = render_hello_conceptual_semcode(&module)
        .expect("conceptual emitter should render planning text");
    let rendered = conceptual.join("\n");

    assert!(rendered.contains("request_observation_text \"Hello, World!\""));
    assert!(!rendered.contains("print"));
    assert!(!rendered.contains("stdout"));
    assert!(!rendered.contains("io.write"));
    assert!(!rendered.contains("opcode"));
    assert!(!rendered.contains("bytecode"));
}
