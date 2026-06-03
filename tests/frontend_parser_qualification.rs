use sm_front::parse_rustlike_with_profile;
use sm_profile::ParserProfile;

fn parse(src: &str) -> sm_front::Program {
    let bundle = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
        .expect("parse failed");
    match bundle {
        sm_front::AstBundle::RustLike(p) => p,
        _ => panic!("Expected RustLike AST"),
    }
}

fn parse_err(src: &str) -> sm_front::FrontendError {
    parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
        .expect_err("expected parse to fail")
}

#[test]
fn test_top_level_declaration_parsing() {
    let src = r#"
        fn my_func() {}
        record MyRecord { field: i32 }
        enum MyEnum { Variant }
        trait MyTrait {}
        impl MyTrait for MyRecord {}
        schema MySchema {}
    "#;
    let prog = parse(src);
    assert!(!prog.functions.is_empty(), "missing functions");
    assert!(!prog.records.is_empty(), "missing records");
    assert!(!prog.adts.is_empty(), "missing adts");
    assert!(!prog.traits.is_empty(), "missing traits");
    assert!(!prog.impls.is_empty(), "missing impls");
    assert!(!prog.schemas.is_empty(), "missing schemas");
}

#[test]
fn test_function_shape_parsing() {
    let src = r#"
        fn empty_body() {}
        fn with_params(a: i32, b: String) {}
        fn with_return() -> i32 { return 0; }
        fn expr_bodied() -> i32 = 42;
        fn with_contracts(x: i32) -> i32
            requires(x > 0)
            ensures(result > 0)
            invariant(x == x)
        { return x; }
    "#;
    let prog = parse(src);
    assert!(prog.functions.len() >= 5);
}

#[test]
fn test_statement_control_parsing() {
    let src = r#"
        fn control_flow() {
            let x = 1;
            let mut y = 2;
            y = 3;
            if x == 1 {
                return;
            } else {
                let z = 4;
            }
            while y > 0 {
                y = y - 1;
                continue;
            }
            loop {
                break;
            }
            let m = match x {
                1..=1 => { 10 }
                _ => { 20 }
            };
        }
    "#;
    let prog = parse(src);
    assert_eq!(prog.functions.len(), 1);
}

#[test]
fn test_data_surface_parsing() {
    let src = r#"
        fn data_surface() {
            let r = MyRecord { field: 42 };
            let f = r.field;
            let e = Option::Some(1);
            let m = match e {
                Option::Some(x) => { 1 }
                _ => { 2 }
            };
        }
    "#;
    let prog = parse(src);
    assert_eq!(prog.functions.len(), 1);
}

#[test]
fn test_negative_parser_diagnostics() {
    let err = parse_err("fn");
    assert!(!err.to_string().is_empty());

    let err = parse_err("fn my_func(a: i32 { }");
    assert!(!err.to_string().is_empty());

    let err = parse_err("fn my_func() {");
    assert!(!err.to_string().is_empty());

    let err = parse_err("fn my_func() { let x = 1 }");
    assert!(!err.to_string().is_empty());

    let err = parse_err("match x { 1 => }");
    assert!(!err.to_string().is_empty());

    let err = parse_err("record MyRecord { field: }");
    assert!(!err.to_string().is_empty());

    let err = parse_err("enum MyEnum { Variant, } enum");
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_parser_profile_boundary() {
    let mut disabled_profile = ParserProfile::foundation_default();
    disabled_profile.features.allow_schema_surface = false;

    let src = "schema MySchema {}";

    // Should fail if disabled
    let res = parse_rustlike_with_profile(src, &disabled_profile);
    assert!(res.is_err(), "Expected parse to fail with schema disabled");

    let mut enabled_profile = ParserProfile::foundation_default();
    enabled_profile.features.allow_schema_surface = true;

    let res2 = parse_rustlike_with_profile(src, &enabled_profile);
    assert!(
        res2.is_ok(),
        "Expected parse to succeed with schema enabled"
    );
}

#[test]
fn test_source_mark_sanity() {
    let err = parse_err("fn 123()");
    let msg = err.to_string();
    assert!(!msg.is_empty(), "expected some diagnostic text");
}
