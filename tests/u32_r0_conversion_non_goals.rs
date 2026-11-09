//! R0 conversion non-goals for plain `u32` (`docs/spec/compiler_u32_v0.md` §14).
//!
//! `semantic.compiler.u32/0.1` admits no conversion between `u32` and `i32`,
//! `f64` or `fx`, and no `as` cast syntax. These vectors pin that every such
//! program stays rejected at both optimisation levels, and at which frontend
//! stage: unsupported `as` syntax and out-of-range literals are parser
//! rejections, implicit binding/return/argument conversions and mixed-family
//! operators are typecheck rejections. Diagnostic wording is deliberately not
//! pinned: it is not part of the frozen contract. Mixed-family operator probes
//! declare no result type, and each is paired with an admitted context control
//! and same-family controls, so only the operator itself can cause rejection.
//!
//! The positive vectors re-assert the checked semantics these non-goals must
//! not disturb (full coverage stays in `tests/shf3a_u32_qualification.rs`).

use sm_emit::{compile_program_to_semcode_with_options, CompileProfile, OptLevel};
use sm_front::{parse_program, type_check_program};
use sm_runtime_core::RuntimeTrap;
use sm_verify::verify_semcode_token;
use sm_vm::{run_verified_function_semcode_with_args, RuntimeError, Value};

#[derive(Debug, PartialEq, Eq)]
enum Stage {
    Parse,
    Typecheck,
}

fn program(body: &str) -> String {
    format!("{body}\nfn main() {{\n    return;\n}}\n")
}

/// Asserts `src` is rejected at `stage` and emits no SemCode at O0 or O1.
#[track_caller]
fn assert_rejected(stage: Stage, src: &str) {
    let observed = match parse_program(src) {
        Err(_) => Stage::Parse,
        Ok(p) => {
            type_check_program(&p).expect_err(&format!("must not typecheck:\n{src}"));
            Stage::Typecheck
        }
    };
    assert_eq!(observed, stage, "rejection stage for:\n{src}");
    for opt in [OptLevel::O0, OptLevel::O1] {
        let first = compile_program_to_semcode_with_options(src, CompileProfile::RustLike, opt)
            .expect_err(&format!("must not emit SemCode ({opt:?}):\n{src}"))
            .to_string();
        let again = compile_program_to_semcode_with_options(src, CompileProfile::RustLike, opt)
            .expect_err("second compile")
            .to_string();
        assert_eq!(first, again, "{opt:?}: diagnostic must be deterministic");
    }
}

/// `fn probe(a: <from>) -> <to>` bodies for an implicit `from -> to` conversion
/// through a typed binding, a return and a call argument.
fn implicit_conversions(from: &str, to: &str) -> [String; 3] {
    [
        program(&format!(
            "fn probe(a: {from}) -> {to} {{\n    let x: {to} = a;\n    return x;\n}}"
        )),
        program(&format!("fn probe(a: {from}) -> {to} {{\n    return a;\n}}")),
        program(&format!(
            "fn take(x: {to}) -> {to} {{\n    return x;\n}}\nfn probe(a: {from}) -> {to} {{\n    return take(a);\n}}"
        )),
    ]
}

#[test]
fn d_i2u_and_d_u2i_no_implicit_i32_u32_conversion() {
    for (from, to) in [("i32", "u32"), ("u32", "i32")] {
        for src in implicit_conversions(from, to) {
            assert_rejected(Stage::Typecheck, &src);
        }
    }
}

#[test]
fn d_f64_no_u32_f64_conversion() {
    for (from, to) in [("u32", "f64"), ("f64", "u32")] {
        for src in implicit_conversions(from, to) {
            assert_rejected(Stage::Typecheck, &src);
        }
    }
}

#[test]
fn d_fx_no_u32_fx_conversion() {
    for (from, to) in [("u32", "fx"), ("fx", "u32")] {
        for src in implicit_conversions(from, to) {
            assert_rejected(Stage::Typecheck, &src);
        }
    }
}

#[test]
fn d_csyn_no_as_cast_syntax() {
    for (from, to) in [
        ("i32", "u32"),
        ("u32", "i32"),
        ("u32", "f64"),
        ("f64", "u32"),
        ("u32", "fx"),
        ("fx", "u32"),
        ("u32", "u32"),
    ] {
        let src = program(&format!(
            "fn probe(a: {from}) -> {to} {{\n    let x: {to} = a as {to};\n    return x;\n}}"
        ));
        assert_rejected(Stage::Parse, &src);
    }
}

const BINARY_OPS: [&str; 11] = ["+", "-", "*", "/", "%", "<", "<=", ">", ">=", "==", "!="];

/// A unit `probe` whose only typed expression is `a <op> b`: no return type
/// and no binding annotation constrain the operator's result, so a rejection
/// cannot come from a result/signature mismatch.
fn binary_probe(l: &str, r: &str, op: &str) -> String {
    program(&format!(
        "fn probe(a: {l}, b: {r}) {{\n    let x = a {op} b;\n    return;\n}}"
    ))
}

/// The same `probe` with the binary operation removed: identical signature,
/// unannotated bindings and unit return. It must typecheck and compile, so the
/// operator is the only difference between an admitted and a rejected program.
fn context_control(l: &str, r: &str) -> String {
    program(&format!(
        "fn probe(a: {l}, b: {r}) {{\n    let x = a;\n    let y = b;\n    return;\n}}"
    ))
}

#[track_caller]
fn assert_admitted(src: &str) {
    let parsed = parse_program(src).unwrap_or_else(|e| panic!("must parse:\n{src}\n{e:?}"));
    type_check_program(&parsed).unwrap_or_else(|e| panic!("must typecheck:\n{src}\n{e:?}"));
    for opt in [OptLevel::O0, OptLevel::O1] {
        let bytes = compile_program_to_semcode_with_options(src, CompileProfile::RustLike, opt)
            .unwrap_or_else(|e| panic!("must compile ({opt:?}):\n{src}\n{e}"));
        verify_semcode_token(&bytes).unwrap_or_else(|e| panic!("must verify ({opt:?}): {e}"));
    }
}

#[test]
fn mixed_family_operators_stay_rejected_in_both_orders() {
    let mut rejected = 0;
    for other in ["i32", "f64", "fx"] {
        for (l, r) in [("u32", other), (other, "u32")] {
            assert_admitted(&context_control(l, r));
            for op in BINARY_OPS {
                assert_rejected(Stage::Typecheck, &binary_probe(l, r, op));
                rejected += 1;
            }
        }
    }
    assert_eq!(rejected, 66);
}

#[test]
fn same_family_operators_are_admitted_in_the_same_probe_shape() {
    for ty in ["u32", "i32"] {
        assert_admitted(&context_control(ty, ty));
        for op in BINARY_OPS {
            assert_admitted(&binary_probe(ty, ty, op));
        }
    }
}

#[test]
fn out_of_range_u32_literal_is_a_parse_rejection() {
    for lit in ["4294967296u32", "99999999999u32"] {
        assert_rejected(
            Stage::Parse,
            &program(&format!("fn probe() -> u32 {{\n    return {lit};\n}}")),
        );
    }
}

fn run_probe(src: &str, opt: OptLevel) -> Result<Value, RuntimeError> {
    let bytes = compile_program_to_semcode_with_options(src, CompileProfile::RustLike, opt)
        .unwrap_or_else(|e| panic!("compile ({opt:?}):\n{src}\n{e}"));
    let token = verify_semcode_token(&bytes).unwrap_or_else(|e| panic!("verify: {e}"));
    let entry = token.require_entry("probe").expect("probe entry");
    run_verified_function_semcode_with_args(&entry, vec![])
}

/// O0 and O1 must agree exactly; returns the shared outcome.
fn outcome(ret: &str, expr: &str) -> Result<Value, RuntimeError> {
    let src = program(&format!("fn probe() -> {ret} {{\n    return {expr};\n}}"));
    let o0 = run_probe(&src, OptLevel::O0);
    let o1 = run_probe(&src, OptLevel::O1);
    assert_eq!(
        format!("{o0:?}"),
        format!("{o1:?}"),
        "O0/O1 divergence: {expr}"
    );
    o0
}

#[test]
fn checked_semantics_are_preserved() {
    let value = |ret: &str, expr: &str| {
        format!(
            "{:?}",
            outcome(ret, expr).unwrap_or_else(|e| panic!("{expr}: {e:?}"))
        )
    };
    assert_eq!(
        value("u32", "4294967294u32 + 1u32"),
        format!("{:?}", Value::U32(u32::MAX))
    );
    assert_eq!(value("u32", "7u32 % 3u32"), format!("{:?}", Value::U32(1)));
    assert_eq!(
        value("bool", "4294967295u32 > 2147483647u32"),
        format!("{:?}", Value::Bool(true))
    );
    assert_eq!(
        value("bool", "to_text(4294967295u32) == \"4294967295\""),
        format!("{:?}", Value::Bool(true))
    );
    for (expr, trap) in [
        ("4294967295u32 + 1u32", RuntimeTrap::ArithmeticOverflow),
        ("0u32 - 1u32", RuntimeTrap::ArithmeticOverflow),
        ("65536u32 * 65536u32", RuntimeTrap::ArithmeticOverflow),
        ("1u32 / 0u32", RuntimeTrap::DivisionByZero),
        ("1u32 % 0u32", RuntimeTrap::DivisionByZero),
    ] {
        match outcome("u32", expr) {
            Err(RuntimeError::Trap(t)) => assert_eq!(t, trap, "{expr}"),
            other => panic!("{expr}: expected {trap:?}, got {other:?}"),
        }
    }
}
