use sm_front::parse_rustlike_with_profile;
use sm_front::types::MatchPattern;
use sm_front::{Expr, Stmt};
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
fn test_expression_bodied_function_lowering_shape_matches_block_return() {
    let src = r#"
        fn expr_bodied(x: quad) -> quad = x;
        fn block_bodied(x: quad) -> quad { return x; }
    "#;
    let prog = parse(src);

    assert_eq!(prog.functions.len(), 2);

    let expr_func = &prog.functions[0];
    let block_func = &prog.functions[1];

    let expr_stmt = prog.arena.stmt(expr_func.body[0]);
    let block_stmt = prog.arena.stmt(block_func.body[0]);

    let Stmt::Return(Some(expr_value)) = expr_stmt else {
        panic!("expected expression-bodied function to lower to return statement");
    };
    let Stmt::Return(Some(block_value)) = block_stmt else {
        panic!("expected block-bodied function to contain return statement");
    };

    let Expr::Var(_) = prog.arena.expr(*expr_value) else {
        panic!("expected expression-bodied return value to be the function parameter");
    };
    let Expr::Var(_) = prog.arena.expr(*block_value) else {
        panic!("expected block-bodied return value to be the function parameter");
    };

    assert_eq!(expr_func.body.len(), 1);
    assert_eq!(block_func.body.len(), 1);
}

#[test]
fn test_match_integer_literal_cases_lower_to_singleton_ranges() {
    let src = r#"
        fn main() {
            let index: u32 = 1u32;
            let out: quad = match index {
                0 => { N }
                1 => { F }
                2 => { T }
                _ => { S }
            };
            return;
        }
    "#;
    let prog = parse(src);

    let func = &prog.functions[0];
    let Stmt::Let { value, .. } = prog.arena.stmt(func.body[1]) else {
        panic!("expected leading let statement");
    };
    let Expr::Match(match_expr) = prog.arena.expr(*value) else {
        panic!("expected match expression");
    };

    assert_eq!(match_expr.arms.len(), 3);
    for (arm, expected) in match_expr.arms.iter().zip([0, 1, 2]) {
        let MatchPattern::IntRange(range) = &arm.pat else {
            panic!("expected integer literal case to lower to a singleton range");
        };
        assert_eq!(range.start, expected);
        assert_eq!(range.end, expected);
        assert!(range.inclusive);
    }
    assert!(match_expr.default.is_some());
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
