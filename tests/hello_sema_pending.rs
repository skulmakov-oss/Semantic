use std::path::PathBuf;

use sm_front::hello_parser::parse_hello_file;
use sm_front::hello_sema::validate_hello_file;

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

fn parse_and_validate(rel: &str) -> sm_front::hello_sema::HelloCheckedFile {
    let input = fixture_text(rel);
    let parsed = parse_hello_file(&input)
        .unwrap_or_else(|err| panic!("parser unexpectedly rejected {rel}: {err}"));
    validate_hello_file(parsed)
        .unwrap_or_else(|err| panic!("sema unexpectedly rejected {rel}: {err}"))
}

fn parse_err(rel: &str) -> String {
    let input = fixture_text(rel);
    parse_hello_file(&input)
        .expect_err(&format!("parser unexpectedly accepted {rel}"))
        .message
}

fn validate_src(src: &str) -> Result<sm_front::hello_sema::HelloCheckedFile, String> {
    let parsed = parse_hello_file(src).map_err(|err| err.message)?;
    validate_hello_file(parsed).map_err(|err| err.message)
}

#[test]
fn hello_sema_pending_positive_verbose_directional_is_architecture_bearing() {
    let checked =
        parse_and_validate("tests/fixtures/pending/hello/positive_hello_verbose_directional.sm");
    assert!(checked.report.canonical_shape);
    assert!(checked.report.architecture_bearing);
    assert!(!checked.report.secondary_shape);
    assert!(checked.report.complete_present);
    assert_eq!(checked.report.state_count, 1);
    assert_eq!(checked.report.require_count, 1);
    assert_eq!(checked.report.observe_count, 1);
}

#[test]
fn hello_sema_pending_positive_minimal_observe_is_secondary_shape() {
    let checked = parse_and_validate(
        "tests/fixtures/pending/hello/positive_hello_minimal_observe_directional.sm",
    );
    assert!(!checked.report.canonical_shape);
    assert!(!checked.report.architecture_bearing);
    assert!(checked.report.secondary_shape);
    assert!(!checked.report.complete_present);
    assert_eq!(checked.report.observe_count, 1);
}

#[test]
fn hello_sema_pending_duplicate_state_rejects() {
    let src = r#"
entry HelloWorld {
    state boot: quad = T;
    state boot: quad = F;
    observe "Hello, World!";
    complete T;
}
"#;
    let err = validate_src(src).expect_err("duplicate state should be rejected");
    assert!(
        err.contains("duplicate Hello state symbol"),
        "expected duplicate state rejection, got: {err}"
    );
}

#[test]
fn hello_sema_pending_unknown_required_state_rejects() {
    let src = r#"
entry HelloWorld {
    state boot: quad = T;
    require missing == T;
    observe "Hello, World!";
    complete T;
}
"#;
    let err = validate_src(src).expect_err("unknown state should be rejected");
    assert!(
        err.contains("unknown Hello state symbol"),
        "expected unknown state rejection, got: {err}"
    );
}

#[test]
fn hello_sema_pending_parser_negative_shapes_stay_parser_rejected() {
    for rel in [
        "tests/fixtures/pending/hello/negative_hello_print_legacy_canonical.sm",
        "tests/fixtures/pending/hello/negative_hello_observe_non_text_payload.sm",
        "tests/fixtures/pending/hello/negative_hello_require_side_effect_shape.sm",
        "tests/fixtures/pending/hello/negative_hello_general_io_shape.sm",
    ] {
        let err = parse_err(rel);
        assert!(
            err.contains("expected") || err.contains("legacy Hello syntax"),
            "expected parser rejection for {rel}, got: {err}"
        );
    }
}
