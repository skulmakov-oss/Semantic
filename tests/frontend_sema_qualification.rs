use sm_front::{parse_rustlike_with_profile, type_check_program};
use sm_profile::ParserProfile;

fn parse(src: &str, profile: &ParserProfile) -> sm_front::Program {
    let bundle = parse_rustlike_with_profile(src, profile).expect("parse failed");
    match bundle {
        sm_front::AstBundle::RustLike(p) => p,
        _ => panic!("Expected RustLike AST"),
    }
}

fn check_ok(src: &str) {
    let p = parse(src, &ParserProfile::foundation_default());
    type_check_program(&p).expect("typecheck failed");
}

fn check_err(src: &str) -> String {
    let p = parse(src, &ParserProfile::foundation_default());
    let err = type_check_program(&p).expect_err("expected typecheck to fail");
    err.to_string()
}

#[test]
fn test_valid_primitive_bindings() {
    let src = r#"
        fn get_quad() -> quad { return get_quad(); }
        fn get_f64() -> f64 { return get_f64(); }
        fn main() {
            let q: quad = get_quad();
            let b: bool = true;
            let i: i32 = 42;
            let f: f64 = get_f64();
        }
    "#;
    check_ok(src);
}

#[test]
fn test_invalid_primitive_assignments() {
    let err1 = check_err("fn main() { let b: bool = 42; }");
    assert!(err1.contains("mismatch"), "got: {}", err1);

    let err2 = check_err("fn main() { let i: i32 = true; }");
    assert!(err2.contains("mismatch"), "got: {}", err2);

    let err3 =
        check_err("fn get_quad() -> quad { return get_quad(); } fn main() { let q: quad = true; }");
    assert!(err3.contains("mismatch"), "got: {}", err3);
}

#[test]
fn test_function_call_checking() {
    let src_ok = r#"
        fn add(x: i32, y: i32) -> i32 { return x + y; }
        fn main() { let z = add(1, 2); }
    "#;
    check_ok(src_ok);

    let err1 = check_err("fn add(x: i32) -> i32 { return x; } fn main() { add(true); }");
    assert!(
        err1.contains("expected") || err1.contains("mismatch"),
        "got: {}",
        err1
    );

    let err2 = check_err("fn add(x: i32) -> i32 { return x; } fn main() { add(1, 2); }");
    assert!(err2.contains("arg"), "got: {}", err2);

    let err3 = check_err("fn main() { unknown_func(); }");
    assert!(
        err3.contains("unknown") || err3.contains("not found"),
        "got: {}",
        err3
    );
}

#[test]
fn test_control_flow_semantic_checks() {
    let err1 = check_err("fn main() { if 1 { } }");
    assert!(err1.contains("bool"), "got: {}", err1);

    let err2 = check_err("fn main() { while 1 { } }");
    assert!(err2.contains("bool"), "got: {}", err2);

    let err3 = check_err("fn main() { break; }");
    assert!(err3.contains("break"), "got: {}", err3);

    let err4 = check_err("fn main() { continue; }");
    assert!(err4.contains("continue"), "got: {}", err4);

    let src_ok = r#"
        fn main() {
            while true {
                if true { break; }
                continue;
            }
        }
    "#;
    check_ok(src_ok);
}

#[test]
fn test_record_semantic_checks() {
    let src_ok = r#"
        record R { x: i32 }
        fn main() {
            let r = R { x: 1 };
            let v = r.x;
        }
    "#;
    check_ok(src_ok);

    let err1 = check_err("record R { x: i32 } fn main() { let r = R { }; }");
    assert!(err1.contains("missing"), "got: {}", err1);

    let err2 = check_err("record R { x: i32 } fn main() { let r = R { x: 1, x: 2 }; }");
    assert!(
        err2.contains("repeat") || err2.contains("duplicate") || err2.contains("already"),
        "got: {}",
        err2
    );

    let err3 = check_err("record R { x: i32 } fn main() { let r = R { x: 1 }; let v = r.y; }");
    assert!(
        err3.contains("unknown field")
            || err3.contains("not found")
            || err3.contains("no field")
            || err3.contains("does not exist"),
        "got: {}",
        err3
    );
}

#[test]
fn test_adt_match_semantic_checks() {
    let src_ok = r#"
        enum E { A, B(i32) }
        fn main() {
            let e = E::A;
            let m = match e {
                E::A => { 1 }
                E::B(x) => { 2 }
            };
        }
    "#;
    check_ok(src_ok);

    let err1 =
        check_err("enum E { A, B } fn main() { let e = E::A; let m = match e { E::A => { 1 } }; }");
    assert!(
        err1.contains("missing") || err1.contains("exhaustive"),
        "got: {}",
        err1
    );

    let err2 = check_err("enum E { A } fn main() { let e = E::Unknown; }");
    assert!(err2.contains("variant"), "got: {}", err2);
}

#[test]
fn test_option_result_semantic_checks() {
    let src_ok = r#"
        fn main() {
            let o: Option(i32) = Option::Some(1);
            let r: Result(i32, i32) = Result::Ok(1);
        }
    "#;
    check_ok(src_ok);

    let err1 = check_err("fn main() { let o: Option(i32) = Option::Some(true); }");
    assert!(
        err1.contains("mismatch") || err1.contains("payload"),
        "got: {}",
        err1
    );
}

#[test]
fn test_collections_semantic_checks() {
    let src_ok = r#"
        fn use_seq(s: Sequence(i32)) {}
        fn use_map(m: Map(i32, i32)) {}
        fn main() {}
    "#;
    check_ok(src_ok);
}

#[test]
fn test_diagnostics_sanity() {
    let err = check_err("fn main() { let x: i32 = true; }");
    assert!(!err.is_empty());
}
