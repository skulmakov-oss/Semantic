use semantic_language::frontend::parse_logos_program;

// The canonical documented System form: `System Name(param = value, ...)`.
// Mirrors the fixture at examples/canonical/quad_cycle_logos/src/main.sm.
const DOCUMENTED_SYSTEM_EXAMPLE: &str = r#"
System QuadCycle(sample_count = 48, state_period = 4):

Entity Sensor:
    state val: quad
    prop threshold: f64

Law "CheckSignal" [priority 10]:
    When Sensor.val == T -> Log.emit("Signal OK")
    When Sensor.val == S -> System.recovery()
"#;

#[test]
fn documented_system_example_parses_with_name_equals_value_params() {
    let program = parse_logos_program(DOCUMENTED_SYSTEM_EXAMPLE).expect("logos parse");
    let system = program.system.expect("System declaration");
    assert_eq!(system.name, "QuadCycle");
    assert_eq!(
        system.params,
        vec![
            ("sample_count".to_string(), "48".to_string()),
            ("state_period".to_string(), "4".to_string()),
        ]
    );
}

#[test]
fn logos_md_previously_documented_typed_param_form_is_rejected() {
    // docs/spec/logos.md previously (incorrectly) documented `System Name(param: type, ...)`.
    // The actual parser requires `name = value`, not `name: type`. This proves the
    // previously-documented form was never what the parser admitted, and that the
    // corrected documentation now matches real rejection behavior.
    let source = r#"
System QuadCycle(sample_count: u32):

Entity Sensor:
    state val: quad
"#;
    let error = parse_logos_program(source).expect_err("typed `:` params must be rejected");
    assert!(
        error.message.contains("expected '='") && error.message.contains("E0202"),
        "unexpected error: {}",
        error.message
    );
}

#[test]
fn system_with_no_parameters_still_parses() {
    let source = r#"
System QuadCycle:

Entity Sensor:
    state val: quad
"#;
    let program = parse_logos_program(source).expect("logos parse");
    let system = program.system.expect("System declaration");
    assert_eq!(system.name, "QuadCycle");
    assert!(system.params.is_empty());
}

#[test]
fn documentation_matches_the_parser_accepted_system_form() {
    let logos_md = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/spec/logos.md"),
    )
    .expect("read logos.md");
    assert!(
        logos_md.contains("System Name(param = value, ...)"),
        "logos.md must document the parser's actual `name = value` System param form"
    );
    assert!(
        !logos_md.contains("System Name(param: type, ...)"),
        "logos.md must not still document the unimplemented typed `param: type` System form"
    );
}
