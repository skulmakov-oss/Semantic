//! SSF-09 C1A P1A0-R1P-D3: a match pattern exposes an enum payload with the
//! same canonical semantic type the enum constructor accepts. The frontend's
//! match-family spec resolves declared payloads through
//! `canonicalize_declared_type` (a named enum is `Adt`, never the parser's raw
//! `Record` spelling), recursively through containers; the ADT declaration
//! table, lowering and every legitimate type error are unchanged.

use sm_emit::compile_program_to_semcode;
use sm_front::{parse_program, type_check_program};
use sm_ir::{compile_program_to_ir, CompilePipelineError};
use sm_vm::run_verified_semcode;

const INNER: &str = "enum Inner {\n    A,\n    B,\n}\n";
const OUTER: &str = "enum Outer {\n    Wrap(Inner),\n    Nil,\n}\n";
const OTHER: &str = "enum Other {\n    Q,\n}\n";
const REC: &str = "record P {\n    v: i32,\n}\n";
const IS_B: &str = "fn is_b(x: Inner) -> bool {\n    return match x {\n        Inner::B => { true }\n        _ => { false }\n    };\n}\n";

fn with_main(decls: &str, body: &str) -> String {
    format!("{decls}fn main() {{\n{body}\n    return;\n}}\n")
}

/// `match scr { pat => { body } other => { assert(false); } }`
fn arm(scr: &str, pat: &str, body: &str, other: &str) -> String {
    format!(
        "    match {scr} {{\n        {pat} => {{\n{body}\n        }}\n        {other} => {{\n            assert(false);\n        }}\n    }}"
    )
}

/// Typecheck, lower, emit SemCode, verify and run; the program's own
/// `assert`s check the bound payload's value.
fn runs(src: &str) {
    let bytes = match compile_program_to_semcode(src) {
        Ok(bytes) => bytes,
        Err(err) => panic!("valid program must compile, got {err}"),
    };
    run_verified_semcode(&bytes).expect("verified execution");
}

fn typechecks(src: &str) {
    let program = parse_program(src).expect("parse");
    if let Err(err) = type_check_program(&program) {
        panic!("valid program must typecheck, got {}", err.message);
    }
}

/// The program is rejected by the frontend; returns the message.
fn rejected(src: &str) -> String {
    let program = parse_program(src).expect("negative witnesses must parse");
    let tc = type_check_program(&program).expect_err("typecheck must reject");
    match compile_program_to_ir(src).expect_err("the pipeline must reject") {
        CompilePipelineError::Frontend(err) => assert_eq!(err.message, tc.message),
        other => panic!("expected a frontend error, got {other:?}"),
    }
    tc.message
}

#[test]
fn d3_p1_direct_payload_reaches_a_canonical_consumer() {
    let src = with_main(
        &format!("{INNER}{OUTER}{IS_B}"),
        &format!(
            "    let x = Outer::Wrap(Inner::B);\n{}",
            arm(
                "x",
                "Outer::Wrap(v)",
                "            assert(is_b(v));",
                "Outer::Nil"
            )
        ),
    );
    runs(&src);
}

#[test]
fn d3_p2_bound_payload_is_matched_again() {
    let src = with_main(
        &format!("{INNER}{OUTER}"),
        &format!(
            "    let x = Outer::Wrap(Inner::B);\n{}",
            arm(
                "x",
                "Outer::Wrap(v)",
                &arm("v", "Inner::B", "            return;", "Inner::A"),
                "Outer::Nil"
            )
        ),
    );
    runs(&src);
}

#[test]
fn d3_p3_container_payloads_canonicalize_recursively() {
    let shapes = [
        (
            "Option(Inner)",
            "Option::Some(Inner::B)",
            "    return match p {\n        Option::Some(i) => { is_b(i) }\n        _ => { false }\n    };",
        ),
        (
            "Sequence(Inner)",
            "[Inner::A, Inner::B]",
            "    return is_b(p[1]);",
        ),
        (
            "(Inner, i32)",
            "(Inner::B, 3)",
            "    let (i, n) = p;\n    return is_b(i);",
        ),
        (
            "Result(Inner, i32)",
            "Result::Ok(Inner::B)",
            "    return match p {\n        Result::Ok(i) => { is_b(i) }\n        _ => { false }\n    };",
        ),
    ];
    for (ty, value, check) in shapes {
        let decls = format!(
            "{INNER}{IS_B}enum O {{\n    W({ty}),\n    Nn,\n}}\nfn check(p: {ty}) -> bool {{\n{check}\n}}\n"
        );
        let body = format!(
            "    let x = O::W({value});\n{}",
            arm("x", "O::W(p)", "            assert(check(p));", "O::Nn")
        );
        runs(&with_main(&decls, &body));
    }
}

#[test]
fn d3_p4_nested_nominal_payloads_resolve_at_every_level() {
    let src = with_main(
        &format!("{INNER}{OUTER}{IS_B}enum Top {{\n    Tt(Outer),\n}}\n"),
        &format!(
            "    let x = Top::Tt(Outer::Wrap(Inner::B));\n    match x {{\n        Top::Tt(o) => {{\n{}\n        }}\n    }}",
            arm("o", "Outer::Wrap(i)", "            assert(is_b(i));", "Outer::Nil")
        ),
    );
    runs(&src);
}

#[test]
fn d3_p5_inferred_binding_keeps_the_canonical_type() {
    let src = with_main(
        &format!("{INNER}{OUTER}{IS_B}"),
        &format!(
            "    let x = Outer::Wrap(Inner::B);\n{}",
            arm(
                "x",
                "Outer::Wrap(v)",
                "            let y = v;\n            let z: Inner = y;\n            assert(is_b(z));",
                "Outer::Nil"
            )
        ),
    );
    runs(&src);
}

#[test]
fn d3_p6_payload_flows_through_branch_and_match_values() {
    let decls = format!(
        "{INNER}{OUTER}{IS_B}fn unwrap(o: Outer) -> Inner {{\n    return match o {{\n        Outer::Wrap(v) => {{ v }}\n        Outer::Nil => {{ Inner::A }}\n    }};\n}}\n"
    );
    let body = format!(
        "    let c: bool = true;\n    assert(is_b(unwrap(Outer::Wrap(Inner::B))));\n    let x = Outer::Wrap(Inner::B);\n{}",
        arm(
            "x",
            "Outer::Wrap(v)",
            "            let y = if c == true { v } else { Inner::A };\n            assert(is_b(y));",
            "Outer::Nil"
        )
    );
    runs(&with_main(&decls, &body));
}

#[test]
fn d3_p7_mixed_primitive_and_nominal_items() {
    let src = with_main(
        &format!("{INNER}{IS_B}enum O {{\n    W(i32, Inner, bool),\n    Nn,\n}}\n"),
        &format!(
            "    let x = O::W(5, Inner::B, true);\n{}",
            arm(
                "x",
                "O::W(n, v, f)",
                "            assert(n == 5);\n            assert(is_b(v));\n            assert(f);",
                "O::Nn"
            )
        ),
    );
    runs(&src);
}

#[test]
fn d3_p8_constructor_and_destructor_agree_on_the_payload_type() {
    // The constructor accepts `inner: Inner`; the pattern must expose `v` as
    // that same type: annotation, parameter and constructor all take it back.
    let src = with_main(
        &format!("{INNER}{OUTER}{IS_B}"),
        &format!(
            "    let inner: Inner = Inner::B;\n    let x = Outer::Wrap(inner);\n{}",
            arm(
                "x",
                "Outer::Wrap(v)",
                "            let same: Inner = v;\n            let again: Outer = Outer::Wrap(v);\n            assert(is_b(same));\n            match again {\n                Outer::Wrap(w) => {\n                    assert(is_b(w));\n                }\n                Outer::Nil => {\n                    assert(false);\n                }\n            }",
                "Outer::Nil"
            )
        ),
    );
    runs(&src);
}

#[test]
fn d3_p9_guard_sees_the_canonical_binding() {
    let src = with_main(
        &format!("{INNER}{OUTER}{IS_B}"),
        "    let x = Outer::Wrap(Inner::B);\n    match x {\n        Outer::Wrap(v) if is_b(v) => {\n            return;\n        }\n        _ => {\n            assert(false);\n        }\n    }",
    );
    runs(&src);
}

#[test]
fn d3_p10_if_let_binding_typechecks_with_the_canonical_type() {
    // Typecheck only: if-let lowering is a separate, pre-existing gap.
    let src = with_main(
        &format!("{INNER}{OUTER}{IS_B}"),
        "    let x = Outer::Wrap(Inner::B);\n    let b: bool = if let Outer::Wrap(v) = x { is_b(v) } else { false };",
    );
    typechecks(&src);
}

#[test]
fn d3_n1_pattern_shape_errors_stay_rejected() {
    let decls = format!("{INNER}{OUTER}{OTHER}");
    let cases = [
        (
            "    match x {\n        Other::Q => {\n            return;\n        }\n        _ => {\n            return;\n        }\n    }".to_string(),
            "does not match scrutinee enum 'Outer'",
        ),
        (
            arm("x", "Outer::Nope(v)", "            return;", "Outer::Nil"),
            "has no variant named 'Nope'",
        ),
        (
            arm("x", "Outer::Wrap(a, b)", "            return;", "Outer::Nil"),
            "arity mismatch",
        ),
        (
            "    match x {\n        Outer::Wrap(v) => {\n            return;\n        }\n    }".to_string(),
            "missing variants: Nil",
        ),
    ];
    for (body, expected) in cases {
        let src = with_main(
            &decls,
            &format!("    let x = Outer::Wrap(Inner::A);\n{body}"),
        );
        let msg = rejected(&src);
        assert!(msg.contains(expected), "{expected:?} not in {msg:?}");
    }
}

#[test]
fn d3_n2_real_type_mismatches_on_bound_payloads_stay_rejected() {
    let fns = "fn take_other(x: Other) {\n    return;\n}\nfn take_p(p: P) {\n    return;\n}\nfn take_oo(o: Option(Other)) {\n    return;\n}\nfn consume(x: Inner) {\n    return;\n}\n";
    let decls = format!(
        "{INNER}{OTHER}{REC}{fns}enum O {{\n    W(Inner),\n    I(i32),\n    R(P),\n    Op(Option(Inner)),\n}}\n"
    );
    let cases = [
        (
            "O::W(Inner::A)",
            "O::W(v)",
            "take_other(v);",
            "arg 0 for 'take_other'",
        ),
        (
            "O::W(Inner::A)",
            "O::W(v)",
            "take_p(v);",
            "arg 0 for 'take_p'",
        ),
        ("O::I(1)", "O::I(v)", "consume(v);", "arg 0 for 'consume'"),
        (
            "O::R(P { v: 1 })",
            "O::R(v)",
            "consume(v);",
            "arg 0 for 'consume'",
        ),
        (
            "O::Op(Option::Some(Inner::A))",
            "O::Op(v)",
            "take_oo(v);",
            "arg 0 for 'take_oo'",
        ),
        (
            "O::W(Inner::A)",
            "O::W(v)",
            "let y: Other = v;",
            "type mismatch in let 'y'",
        ),
        (
            "O::W(Inner::A)",
            "O::W(v)",
            "if v {\n                return;\n            }",
            "if condition must be bool",
        ),
    ];
    for (value, pat, stmt, expected) in cases {
        let body = format!(
            "    let x = {value};\n    match x {{\n        {pat} => {{\n            {stmt}\n        }}\n        _ => {{\n            return;\n        }}\n    }}"
        );
        let msg = rejected(&with_main(&decls, &body));
        assert!(msg.contains(expected), "{expected:?} not in {msg:?}");
    }
}

#[test]
fn d3_n3_wrong_constructor_payloads_stay_rejected() {
    for value in ["Other::Q", "P { v: 1 }", "7"] {
        let src = with_main(
            &format!("{INNER}{OUTER}{OTHER}{REC}"),
            &format!("    let x = Outer::Wrap({value});"),
        );
        let msg = rejected(&src);
        assert!(
            msg.contains("enum constructor 'Outer::Wrap' payload item 0"),
            "{msg:?}"
        );
    }
}

#[test]
fn d3_declaration_and_generic_boundaries_are_unchanged() {
    // Declared payloads are still validated raw, before any match exists.
    assert_eq!(
        rejected(&with_main("enum O {\n    W(Zq),\n}\n", "")),
        "unknown record type 'Zq' in variant 'O::W' payload item 0"
    );
    assert!(rejected(&with_main("enum G<X> {\n    W(X),\n}\n", "")).contains("generic enum 'G'"));
    // A match inside a generic function now typechecks; lowering admission
    // of the generic function itself is unchanged.
    let src = with_main(
        &format!(
            "{INNER}{OUTER}{IS_B}fn pick<X>(o: Outer, t: X) -> X {{\n{}\n    return t;\n}}\n",
            arm(
                "o",
                "Outer::Wrap(v)",
                "            assert(is_b(v));",
                "Outer::Nil"
            )
        ),
        "    let r: i32 = pick(Outer::Wrap(Inner::B), 3);",
    );
    typechecks(&src);
    match compile_program_to_ir(&src).expect_err("generic lowering stays gated") {
        CompilePipelineError::Frontend(err) => {
            assert!(
                err.message.contains("generic function 'pick'"),
                "{}",
                err.message
            )
        }
        other => panic!("expected the generic admission error, got {other:?}"),
    }
}
