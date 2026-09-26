//! SSF-09 C1A P1A0-R1P-D2: `unit` is a Semantic value. A user function call
//! that returns `unit`, used where typecheck already admits a `unit` value,
//! lowers through the ordinary call-expression path (`Call { dst: Some(r) }`,
//! the callee's `Ret` supplies `Value::Unit`); a discarded call keeps
//! `Call { dst: None }`. Source syntax and frontend admission are unchanged.

use sm_emit::compile_program_to_semcode;
use sm_ir::{compile_program_to_ir, CompilePipelineError, IrFunction, IrInstr};
use sm_runtime_core::RuntimeTrap;
use sm_vm::{run_verified_semcode, RuntimeError};

const U: &str = "fn u() {\n    return;\n}\n";
const BOOM: &str = "fn boom() {\n    assert(false);\n    return;\n}\n";

fn with_main(decls: &str, body: &str) -> String {
    format!("{decls}fn main() {{\n{body}\n    return;\n}}\n")
}

fn lower(src: &str) -> Vec<IrFunction> {
    match compile_program_to_ir(src) {
        Ok(funcs) => funcs,
        Err(err) => panic!("typechecked program must lower, got {err:?}"),
    }
}

fn func<'a>(funcs: &'a [IrFunction], name: &str) -> &'a IrFunction {
    funcs
        .iter()
        .find(|f| f.name == name)
        .expect("function present")
}

/// Destination registers of every `Call` to `callee` in `f`, in program order.
fn call_dsts(f: &IrFunction, callee: &str) -> Vec<Option<u16>> {
    f.instrs
        .iter()
        .filter_map(|i| match i {
            IrInstr::Call { dst, name, .. } if name == callee => Some(*dst),
            _ => None,
        })
        .collect()
}

/// The single value-carrying call to `callee` in `f`; its register must be a real destination.
fn value_call(f: &IrFunction, callee: &str) -> u16 {
    match call_dsts(f, callee).as_slice() {
        [Some(r)] => *r,
        other => panic!("expected exactly one value call to '{callee}', got {other:?}"),
    }
}

fn runs(src: &str) {
    let bytes = compile_program_to_semcode(src).expect("compile to SemCode");
    run_verified_semcode(&bytes).expect("verified execution");
}

fn traps_on_assert(src: &str) {
    let bytes = compile_program_to_semcode(src).expect("compile to SemCode");
    let err = run_verified_semcode(&bytes).expect_err("the side effect must execute");
    assert!(
        matches!(err, RuntimeError::Trap(RuntimeTrap::AssertionFailed)),
        "{err:?}"
    );
}

fn frontend_error(src: &str) -> String {
    match compile_program_to_ir(src).expect_err("must be rejected") {
        CompilePipelineError::Frontend(err) => err.message,
        other => panic!("expected a frontend error, got {other:?}"),
    }
}

#[test]
fn d2_u1_return_of_unit_call_is_an_ordinary_value_return() {
    let src = with_main(&format!("{U}fn f() {{\n    return u();\n}}\n"), "    f();");
    let funcs = lower(&src);
    let f = func(&funcs, "f");
    let r = value_call(f, "u");
    assert!(
        matches!(f.instrs.last(), Some(IrInstr::Ret { src: Some(s) }) if *s == r),
        "{:?}",
        f.instrs
    );
    runs(&src);
}

#[test]
fn d2_u2_inferred_let_binds_the_call_result() {
    let src = with_main(U, "    let x = u();\n    let y = u();\n    assert(x == y);");
    let funcs = lower(&src);
    let main = func(&funcs, "main");
    let dsts = call_dsts(main, "u");
    assert_eq!(dsts.len(), 2, "{:?}", main.instrs);
    for dst in dsts {
        let r = dst.expect("value call has a destination");
        assert!(
            main.instrs
                .iter()
                .any(|i| matches!(i, IrInstr::StoreVar { src, .. } if *src == r)),
            "the stored value must be the call's own result register {r}: {:?}",
            main.instrs
        );
    }
    runs(&src);
}

#[test]
fn d2_u3_tuple_item_consumes_the_call_result() {
    let src = with_main(U, "    let t = (u(), 1);");
    let funcs = lower(&src);
    let main = func(&funcs, "main");
    let r = value_call(main, "u");
    assert!(
        main.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::MakeTuple { items, .. } if items.first() == Some(&r))),
        "{:?}",
        main.instrs
    );
    runs(&src);
}

#[test]
fn d2_u4_sequence_item_consumes_the_call_result() {
    let src = with_main(U, "    let s = [u(), u()];");
    let funcs = lower(&src);
    let main = func(&funcs, "main");
    let dsts: Vec<u16> = call_dsts(main, "u")
        .into_iter()
        .map(|d| d.expect("value call"))
        .collect();
    assert_eq!(dsts.len(), 2);
    assert!(
        main.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::MakeSequence { items, .. } if *items == dsts)),
        "{:?}",
        main.instrs
    );
    runs(&src);
}

#[test]
fn d2_u5_option_payload_consumes_the_call_result() {
    let src = with_main(U, "    let o = Option::Some(u());");
    let funcs = lower(&src);
    let main = func(&funcs, "main");
    let r = value_call(main, "u");
    assert!(
        main.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::MakeAdt { items, .. } if *items == vec![r])),
        "{:?}",
        main.instrs
    );
    runs(&src);
}

#[test]
fn d2_u6_if_expression_with_unit_branches() {
    let src = with_main(
        U,
        "    let c: bool = true;\n    let x = if c { u() } else { u() };",
    );
    let funcs = lower(&src);
    let dsts = call_dsts(func(&funcs, "main"), "u");
    assert_eq!(dsts.len(), 2);
    assert!(dsts.iter().all(Option::is_some), "{dsts:?}");
    runs(&src);
}

#[test]
fn d2_u7_match_expression_with_unit_arms() {
    let src = with_main(
        &format!("{U}enum E {{\n    A,\n    B,\n}}\n"),
        "    let e: E = E::A;\n    let x = match e {\n        E::A => { u() }\n        _ => { u() }\n    };",
    );
    let funcs = lower(&src);
    let dsts = call_dsts(func(&funcs, "main"), "u");
    assert_eq!(dsts.len(), 2);
    assert!(dsts.iter().all(Option::is_some), "{dsts:?}");
    runs(&src);
}

#[test]
fn d2_u8_unit_equality_compares_two_real_call_results() {
    let src = with_main(U, "    let b: bool = u() == u();\n    assert(b);");
    let funcs = lower(&src);
    let main = func(&funcs, "main");
    let dsts: Vec<u16> = call_dsts(main, "u")
        .into_iter()
        .map(|d| d.expect("value call"))
        .collect();
    assert_eq!(
        dsts.len(),
        2,
        "each operand call executes exactly once: {:?}",
        main.instrs
    );
    assert!(
        main.instrs.iter().any(
            |i| matches!(i, IrInstr::CmpEq { lhs, rhs, .. } if [*lhs, *rhs] == [dsts[0], dsts[1]])
        ),
        "{:?}",
        main.instrs
    );
    // `assert(b)` passing at runtime proves both registers hold `Value::Unit`
    runs(&src);
}

#[test]
fn d2_u9_chained_unit_returns() {
    let src = with_main(
        &format!("{U}fn g() {{\n    return u();\n}}\nfn f() {{\n    return g();\n}}\n"),
        "    f();",
    );
    let funcs = lower(&src);
    for (f, callee) in [("g", "u"), ("f", "g")] {
        let body = func(&funcs, f);
        let r = value_call(body, callee);
        assert!(matches!(body.instrs.last(), Some(IrInstr::Ret { src: Some(s) }) if *s == r));
    }
    runs(&src);
}

#[test]
fn d2_u10_discarded_unit_call_stays_value_less() {
    let src = with_main(
        U,
        "    u();\n    let n: i32 = {\n        u();\n        1\n    };",
    );
    let funcs = lower(&src);
    assert_eq!(call_dsts(func(&funcs, "main"), "u"), vec![None, None]);
    runs(&src);
}

#[test]
fn d2_user_unit_call_matches_builtin_unit_call_shape() {
    // parity only: `print` (a builtin returning unit) already lowered as a value
    for (decls, callee) in [("", "print"), (U, "u")] {
        let arg = if callee == "print" { "\"p\"" } else { "" };
        let src = with_main(
            &format!("{decls}fn f() {{\n    return {callee}({arg});\n}}\n"),
            "    f();",
        );
        let funcs = lower(&src);
        let f = func(&funcs, "f");
        let r = value_call(f, callee);
        assert!(
            matches!(f.instrs.last(), Some(IrInstr::Ret { src: Some(s) }) if *s == r),
            "{callee}"
        );
        runs(&src);
    }
}

#[test]
fn d2_se1_return_context_executes_the_call_exactly_once() {
    let src = with_main(
        &format!("{BOOM}fn f() {{\n    return boom();\n}}\n"),
        "    f();",
    );
    let funcs = lower(&src);
    value_call(func(&funcs, "f"), "boom");
    traps_on_assert(&src);
}

#[test]
fn d2_se2_let_context_executes_the_call_exactly_once_in_order() {
    let src = with_main(
        &format!("{U}{BOOM}"),
        "    let a = u();\n    let x = boom();",
    );
    let funcs = lower(&src);
    let main = func(&funcs, "main");
    let order: Vec<&str> = main
        .instrs
        .iter()
        .filter_map(|i| match i {
            IrInstr::Call {
                name, dst: Some(_), ..
            } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(order, ["u", "boom"]);
    traps_on_assert(&src);
}

#[test]
fn d2_frontend_rules_for_unit_are_unchanged() {
    let takes_i32 = "fn g(a: i32) -> i32 {\n    return a;\n}\n";
    assert_eq!(
        frontend_error(&with_main(
            &format!("{U}{takes_i32}"),
            "    let n: i32 = g(u());"
        )),
        "arg 0 for 'g' has type Unit, expected I32"
    );
    assert_eq!(
        frontend_error(&with_main(U, "    if u() {\n        return;\n    }")),
        "if condition must be bool; explicit compare is required for quad"
    );
    assert_eq!(
        frontend_error(&format!(
            "{U}fn f() -> i32 {{\n    return u();\n}}\nfn main() {{\n    return;\n}}\n"
        )),
        "return type mismatch: expected I32, got Unit"
    );
    assert_eq!(
        frontend_error(&with_main(U, "    let x: unit = u();")),
        "unknown record type 'unit' in let 'x'"
    );
    assert_eq!(
        frontend_error(&with_main(U, "    let x: () = u();")),
        "empty tuple type is not supported in v0"
    );
    // no unit-returning closure type is spellable, so the direct-closure
    // guard in lowering stays unreachable from source
    assert_eq!(
        frontend_error(&with_main(U, "    let c = (x => u());")),
        "first-class closure literals currently require contextual Closure(T -> U) type in M8.4 Wave 2"
    );
}
