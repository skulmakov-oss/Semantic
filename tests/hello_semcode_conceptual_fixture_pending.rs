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

fn parse_validate_lower() -> HelloIrModule {
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

fn render_conceptual_semcode(module: &HelloIrModule) -> String {
    let mut out = String::new();
    for stmt in &module.entry.body {
        match stmt {
            HelloIrStmt::LocalQuad(local) => {
                out.push_str(&format!(
                    "declare_local_quad {} = {}\n",
                    local.symbol,
                    render_quad(local.value)
                ));
            }
            HelloIrStmt::RequireQuadEq(require) => {
                out.push_str(&format!(
                    "require_quad_eq {} {}\n",
                    require.symbol,
                    render_quad(require.expected)
                ));
            }
            HelloIrStmt::ObserveText(observe) => {
                out.push_str(&format!("request_observation_text {}\n", observe.text));
                assert_eq!(
                    observe.observation_class,
                    HelloIrObservationClass::Controlled
                );
            }
            HelloIrStmt::CompleteQuad(complete) => {
                out.push_str(&format!("complete_quad {}\n", render_quad(complete.value)));
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

fn fixture_body_lines() -> String {
    fixture_text(
        "tests/fixtures/pending/hello_semcode/positive_hello_verbose_conceptual.semcode.txt",
    )
    .lines()
    .filter(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.starts_with('#')
    })
    .collect::<Vec<_>>()
    .join("\n")
}

#[test]
fn hello_semcode_conceptual_fixture_matches_conceptual_shape() {
    let module = parse_validate_lower();
    let rendered = render_conceptual_semcode(&module);
    let expected = fixture_body_lines();
    assert_eq!(rendered.trim_end(), expected.trim_end());
}

#[test]
fn hello_semcode_conceptual_fixture_stays_out_of_emitter_scope() {
    let module = parse_validate_lower();
    let rendered = render_conceptual_semcode(&module);

    // No SemCode emission exists yet; this test only checks the planning shape.
    assert!(rendered.contains("declare_local_quad boot = T"));
    assert!(rendered.contains("request_observation_text \"Hello, World!\""));
    assert!(!rendered.contains("opcode"));
    assert!(!rendered.contains("bytecode"));
}
