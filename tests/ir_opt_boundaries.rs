use std::fs;

use sm_ir::{
    emit_ir_to_semcode,
    passes::{crystalfold::CrystalFoldPass, IrModule, OptPass},
    validate_ir, IrFunction, IrInstr,
};
use sm_vm::{run_verified_semcode, RuntimeError};

#[test]
fn lowering_does_not_embed_crystalfold_logic() {
    let src =
        fs::read_to_string("crates/sm-ir/src/legacy_lowering.rs").expect("read legacy_lowering.rs");

    assert!(
        src.contains("run_default_opt_passes"),
        "lowering pipeline must invoke IR opt passes"
    );
    assert!(
        !src.contains("fold_constants_and_identities"),
        "constant fold implementation must live in sm-ir passes"
    );
    assert!(
        !src.contains("enum ConstVal"),
        "const-fold state machine must not live in legacy_lowering"
    );
    assert!(
        !src.contains("remove_redundant_consecutive_loads"),
        "structural cleanup helpers must live in sm-ir passes"
    );
    assert!(
        !src.contains("remove_noop_jumps"),
        "jump cleanup helpers must live in sm-ir passes"
    );
    assert!(
        !src.contains("remove_unreachable_until_label"),
        "reachability cleanup helpers must live in sm-ir passes"
    );
}

/// Runs `instrs` as `main` twice through the real pipeline — once unoptimized
/// (O0) and once through the real `CrystalFoldPass` (O1) — via the same
/// emit -> verify -> execute path production code uses, and returns
/// (o0_result, o1_result) so callers can assert they agree.
fn run_under_o0_and_o1(
    instrs: Vec<IrInstr>,
) -> (Result<(), RuntimeError>, Result<(), RuntimeError>) {
    let function = IrFunction {
        name: "main".to_string(),
        instrs,
        ownership_events: Vec::new(),
        params: Vec::new(),
    };
    validate_ir(&function).expect("raw IR must remain structurally admitted");

    let o0_bytes = emit_ir_to_semcode(std::slice::from_ref(&function), false).expect("emit O0");
    sm_verify::verify_semcode_token(&o0_bytes).expect("O0 must pass verifier admission");
    let o0_result = run_verified_semcode(&o0_bytes);

    let mut o1_module = IrModule {
        functions: vec![function],
    };
    CrystalFoldPass.run(&mut o1_module);
    let o1_bytes = emit_ir_to_semcode(&o1_module.functions, false).expect("emit O1");
    sm_verify::verify_semcode_token(&o1_bytes).expect("O1 must pass verifier admission");
    let o1_result = run_verified_semcode(&o1_bytes);

    (o0_result, o1_result)
}

/// FA-04-024 / #1730: CrystalFold's `const_eq` omits a `(ConstVal::U32,
/// ConstVal::U32)` arm and falls through to `_ => false`, so `CmpEq`/`CmpNe`
/// folding on tracked-constant u32 operands can silently disagree with
/// unoptimized (O0) execution. Each case below asserts a u32 comparison via
/// `assert(...)`, so a wrong fold flips a passing program into a runtime
/// `AssertionFailed` trap under O1 while O0 (which evaluates `CmpEq`/`CmpNe`
/// at runtime, not compile time) stays correct — a real, observable
/// divergence through the actual optimizer/verifier/VM pipeline, not just a
/// private-helper unit check.
#[test]
fn crystalfold_u32_equality_matches_unoptimized_execution() {
    let cases: Vec<(&str, Vec<IrInstr>)> = vec![
        (
            "equal_zero_cmpeq",
            vec![
                IrInstr::LoadU32 { dst: 0, val: 0 },
                IrInstr::LoadU32 { dst: 1, val: 0 },
                IrInstr::CmpEq {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::Assert { cond: 2 },
                IrInstr::Ret { src: None },
            ],
        ),
        (
            "equal_seven_cmpeq",
            vec![
                IrInstr::LoadU32 { dst: 0, val: 7 },
                IrInstr::LoadU32 { dst: 1, val: 7 },
                IrInstr::CmpEq {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::Assert { cond: 2 },
                IrInstr::Ret { src: None },
            ],
        ),
        (
            "equal_u32_max_cmpeq",
            vec![
                IrInstr::LoadU32 {
                    dst: 0,
                    val: u32::MAX,
                },
                IrInstr::LoadU32 {
                    dst: 1,
                    val: u32::MAX,
                },
                IrInstr::CmpEq {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::Assert { cond: 2 },
                IrInstr::Ret { src: None },
            ],
        ),
        (
            "equal_seven_cmpne_negated",
            vec![
                IrInstr::LoadU32 { dst: 0, val: 7 },
                IrInstr::LoadU32 { dst: 1, val: 7 },
                IrInstr::CmpNe {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::BoolNot { dst: 3, src: 2 },
                IrInstr::Assert { cond: 3 },
                IrInstr::Ret { src: None },
            ],
        ),
        (
            "unequal_seven_eight_cmpeq_negated",
            vec![
                IrInstr::LoadU32 { dst: 0, val: 7 },
                IrInstr::LoadU32 { dst: 1, val: 8 },
                IrInstr::CmpEq {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::BoolNot { dst: 3, src: 2 },
                IrInstr::Assert { cond: 3 },
                IrInstr::Ret { src: None },
            ],
        ),
        (
            "unequal_seven_eight_cmpne",
            vec![
                IrInstr::LoadU32 { dst: 0, val: 7 },
                IrInstr::LoadU32 { dst: 1, val: 8 },
                IrInstr::CmpNe {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::Assert { cond: 2 },
                IrInstr::Ret { src: None },
            ],
        ),
    ];

    for (name, instrs) in cases {
        let (o0_result, o1_result) = run_under_o0_and_o1(instrs);

        assert!(
            o0_result.is_ok(),
            "case {name}: unoptimized (O0) execution must pass a correct u32 comparison, got {o0_result:?}"
        );
        assert!(
            o1_result.is_ok(),
            "case {name}: CrystalFold (O1) must not change the observable result of a valid u32 comparison, got {o1_result:?}"
        );
        assert_eq!(
            o0_result.is_ok(),
            o1_result.is_ok(),
            "case {name}: O0 and O1 disagree on pass/trap outcome (O0={o0_result:?}, O1={o1_result:?})"
        );
    }
}
