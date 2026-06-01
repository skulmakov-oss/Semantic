use std::path::PathBuf;

use sm_front::hello_parser::parse_hello_file;
use sm_front::hello_sema::validate_hello_file;
use sm_ir::hello_ir::{
    lower_hello_checked_file, HelloIrObservationClass, HelloIrQuadLit, HelloIrStmt,
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

fn parse_validate(rel: &str) -> sm_front::hello_sema::HelloCheckedFile {
    let input = fixture_text(rel);
    let parsed = parse_hello_file(&input)
        .unwrap_or_else(|err| panic!("parser unexpectedly rejected {rel}: {err}"));
    validate_hello_file(parsed)
        .unwrap_or_else(|err| panic!("sema unexpectedly rejected {rel}: {err}"))
}

#[test]
fn hello_ir_lowering_pending_verbose_fixture_lowers_in_order() {
    let checked =
        parse_validate("tests/fixtures/pending/hello/positive_hello_verbose_directional.sm");
    let module =
        lower_hello_checked_file(&checked).expect("canonical verbose Hello should lower to IR");

    assert_eq!(module.entry.name, "HelloWorld");
    assert_eq!(module.entry.body.len(), 4);

    match &module.entry.body[..] {
        [HelloIrStmt::LocalQuad(state), HelloIrStmt::RequireQuadEq(require), HelloIrStmt::ObserveText(observe), HelloIrStmt::CompleteQuad(complete)] =>
        {
            assert_eq!(state.symbol, "boot");
            assert_eq!(state.value, HelloIrQuadLit::T);
            assert_eq!(require.symbol, "boot");
            assert_eq!(require.expected, HelloIrQuadLit::T);
            assert_eq!(observe.text, "\"Hello, World!\"");
            assert_eq!(
                observe.observation_class,
                HelloIrObservationClass::Controlled
            );
            assert_eq!(complete.value, HelloIrQuadLit::T);
        }
        other => panic!("unexpected Hello IR shape: {other:?}"),
    }
}

#[test]
fn hello_ir_lowering_pending_minimal_observe_shape_is_rejected() {
    let checked = parse_validate(
        "tests/fixtures/pending/hello/positive_hello_minimal_observe_directional.sm",
    );
    let err =
        lower_hello_checked_file(&checked).expect_err("secondary Hello shape should not lower yet");
    assert!(
        err.message
            .contains("secondary Hello shape is not admitted for IR lowering"),
        "expected secondary-shape IR rejection, got: {}",
        err.message
    );
}
