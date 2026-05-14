use std::path::PathBuf;

use sm_front::hello_parser::{
    parse_hello_file, HelloEntry, HelloObserveText, HelloQuadLit, HelloStmt,
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

fn parse_fixture(rel: &str) -> HelloEntry {
    let input = fixture_text(rel);
    let parsed = parse_hello_file(&input)
        .unwrap_or_else(|err| panic!("parser unexpectedly rejected {rel}: {err}"));
    parsed.entry
}

fn parse_fixture_err(rel: &str) -> String {
    let input = fixture_text(rel);
    parse_hello_file(&input)
        .expect_err(&format!("parser unexpectedly accepted {rel}"))
        .message
}

#[test]
fn hello_parser_pending_positive_verbose_directional_parses() {
    let entry = parse_fixture("tests/fixtures/pending/hello/positive_hello_verbose_directional.sm");
    assert_eq!(entry.name, "HelloWorld");
    assert_eq!(entry.body.len(), 4);

    match &entry.body[..] {
        [HelloStmt::State(state), HelloStmt::Require(require), HelloStmt::Observe(observe), HelloStmt::Complete(complete)] =>
        {
            assert_eq!(state.name, "boot");
            assert_eq!(state.value, HelloQuadLit::T);
            assert_eq!(require.state, "boot");
            assert_eq!(require.value, HelloQuadLit::T);
            assert_eq!(observe.text, "\"Hello, World!\"");
            assert_eq!(complete.value, HelloQuadLit::T);
        }
        other => panic!("unexpected Hello statement shape: {other:?}"),
    }
}

#[test]
fn hello_parser_pending_positive_minimal_observe_parses() {
    let entry =
        parse_fixture("tests/fixtures/pending/hello/positive_hello_minimal_observe_directional.sm");
    assert_eq!(entry.name, "HelloWorld");
    assert_eq!(entry.body.len(), 1);
    match &entry.body[0] {
        HelloStmt::Observe(HelloObserveText { text }) => {
            assert_eq!(text, "\"Hello, World!\"");
        }
        other => panic!("unexpected Hello statement shape: {other:?}"),
    }
}

#[test]
fn hello_parser_pending_legacy_print_shape_is_unsupported() {
    let err =
        parse_fixture_err("tests/fixtures/pending/hello/negative_hello_print_legacy_canonical.sm");
    assert!(
        err.contains("legacy Hello syntax") || err.contains("expected `entry`"),
        "expected legacy Hello rejection, got: {err}"
    );
}

#[test]
fn hello_parser_pending_general_io_shape_is_unsupported() {
    let err = parse_fixture_err("tests/fixtures/pending/hello/negative_hello_general_io_shape.sm");
    assert!(
        err.contains("expected `entry`")
            || err.contains("legacy Hello syntax")
            || err.contains("expected Hello statement"),
        "expected general I/O rejection to stay parser-level, got: {err}"
    );
}

#[test]
fn hello_parser_pending_non_text_observe_is_parser_rejected() {
    let err = parse_fixture_err(
        "tests/fixtures/pending/hello/negative_hello_observe_non_text_payload.sm",
    );
    assert!(
        err.contains("expected text literal after `observe`"),
        "expected non-text observe parser rejection, got: {err}"
    );
}

#[test]
fn hello_parser_pending_require_side_effect_shape_is_parser_rejected() {
    let err = parse_fixture_err(
        "tests/fixtures/pending/hello/negative_hello_require_side_effect_shape.sm",
    );
    assert!(
        err.contains("expected `==` after required state symbol"),
        "expected require-side-effect parser rejection, got: {err}"
    );
}
