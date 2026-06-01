use std::path::PathBuf;

use sm_front::hello_parser::parse_hello_file;
use sm_front::hello_sema::validate_hello_file;
use sm_ir::hello_ir::{
    lower_hello_checked_file, HelloIrModule, HelloIrObservationClass, HelloIrQuadLit, HelloIrStmt,
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

fn render_hello_ir(module: &HelloIrModule) -> String {
    let mut out = String::new();
    out.push_str("HelloIrModule\n");
    out.push_str(&format!("  entry: {}\n", module.entry.name));
    for (idx, stmt) in module.entry.body.iter().enumerate() {
        match stmt {
            HelloIrStmt::LocalQuad(local) => {
                out.push_str(&format!(
                    "  stmt[{idx}]: LocalQuad {} = {}\n",
                    local.symbol,
                    render_quad(local.value)
                ));
            }
            HelloIrStmt::RequireQuadEq(require) => {
                out.push_str(&format!(
                    "  stmt[{idx}]: RequireQuadEq {} == {}\n",
                    require.symbol,
                    render_quad(require.expected)
                ));
            }
            HelloIrStmt::ObserveText(observe) => {
                out.push_str(&format!(
                    "  stmt[{idx}]: ObserveText class={:?} text={}\n",
                    observe.observation_class, observe.text
                ));
            }
            HelloIrStmt::CompleteQuad(complete) => {
                out.push_str(&format!(
                    "  stmt[{idx}]: CompleteQuad {}\n",
                    render_quad(complete.value)
                ));
            }
        }
    }
    out
}

fn render_quad(value: HelloIrQuadLit) -> &'static str {
    match value {
        HelloIrQuadLit::T => "T",
        HelloIrQuadLit::F => "F",
        HelloIrQuadLit::N => "N",
        HelloIrQuadLit::S => "S",
    }
}

#[test]
fn hello_ir_shape_pending_rendered_shape_is_stable() {
    let checked =
        parse_validate("tests/fixtures/pending/hello/positive_hello_verbose_directional.sm");
    let module =
        lower_hello_checked_file(&checked).expect("canonical verbose Hello should lower to IR");

    let rendered = render_hello_ir(&module);
    let expected = "\
HelloIrModule
  entry: HelloWorld
  stmt[0]: LocalQuad boot = T
  stmt[1]: RequireQuadEq boot == T
  stmt[2]: ObserveText class=Controlled text=\"Hello, World!\"
  stmt[3]: CompleteQuad T
";

    assert_eq!(rendered, expected);
}

#[test]
fn hello_ir_shape_pending_preserves_statement_order_and_controlled_observation_boundary() {
    let checked =
        parse_validate("tests/fixtures/pending/hello/positive_hello_verbose_directional.sm");
    let module =
        lower_hello_checked_file(&checked).expect("canonical verbose Hello should lower to IR");

    assert_eq!(module.entry.body.len(), 4);
    match &module.entry.body[..] {
        [HelloIrStmt::LocalQuad(_), HelloIrStmt::RequireQuadEq(_), HelloIrStmt::ObserveText(observe), HelloIrStmt::CompleteQuad(_)] =>
        {
            assert_eq!(
                observe.observation_class,
                HelloIrObservationClass::Controlled
            );
            let rendered = render_hello_ir(&module);
            assert!(rendered.contains("Controlled"));
            assert!(!rendered.contains("stdout"));
            assert!(!rendered.contains("I/O"));
        }
        other => panic!("unexpected Hello IR order: {other:?}"),
    }
}

#[test]
fn hello_ir_shape_pending_secondary_shape_does_not_lower() {
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

#[test]
fn hello_ir_shape_tests_do_not_emit_semcode() {
    let checked =
        parse_validate("tests/fixtures/pending/hello/positive_hello_verbose_directional.sm");
    let module =
        lower_hello_checked_file(&checked).expect("canonical verbose Hello should lower to IR");
    let rendered = render_hello_ir(&module);

    // SemCode emission is intentionally out of scope for M-HELLO-4D.
    assert!(rendered.starts_with("HelloIrModule"));
    assert!(rendered.contains("ObserveText class=Controlled"));
}
