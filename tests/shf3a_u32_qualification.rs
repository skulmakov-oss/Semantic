//! SHF-3A2 (#2008): full-pipeline qualification of `semantic.compiler.u32/0.1`
//! (`docs/spec/compiler_u32_v0.md`).
//!
//! Every source vector is compiled from Semantic source at both `O0` and `O1`,
//! admitted by `sm-verify`, executed by `sm-vm` through the verified-token route,
//! and observed as the returned value or the runtime trap. The two optimisation
//! levels must agree exactly (contract section 7).

use sm_emit::{compile_program_to_semcode_with_options, CompileProfile, OptLevel};
use sm_format::semcode_format::{header_spec_from_magic, Opcode, MAGIC22, MAGIC23};
use sm_ir::{emit_ir_to_semcode, IrFunction, IrInstr};
use sm_runtime_core::RuntimeTrap;
use sm_verify::{verify_semcode, verify_semcode_token};
use sm_vm::{run_verified_function_semcode_with_args, RuntimeError, Value};

const MAX: &str = "4294967295u32";
const I32_MAX: &str = "2147483647u32";
const I32_MAX_PLUS_1: &str = "2147483648u32";

fn source(ret: &str, expr: &str) -> String {
    format!("fn probe() -> {ret} {{\n    return {expr};\n}}\nfn main() {{\n    return;\n}}\n")
}

fn compile(src: &str, opt: OptLevel) -> Vec<u8> {
    compile_program_to_semcode_with_options(src, CompileProfile::RustLike, opt)
        .unwrap_or_else(|e| panic!("compile ({opt:?}):\n{src}\n{e}"))
}

fn run(bytes: &[u8]) -> Result<Value, RuntimeError> {
    let token = verify_semcode_token(bytes).unwrap_or_else(|e| panic!("verify: {e}"));
    let entry = token.require_entry("probe").expect("probe entry");
    run_verified_function_semcode_with_args(&entry, vec![])
}

/// Outcome under O0 and O1; panics unless both levels agree exactly.
fn outcome(ret: &str, expr: &str) -> Result<Value, RuntimeError> {
    let src = source(ret, expr);
    let o0 = run(&compile(&src, OptLevel::O0));
    let o1 = run(&compile(&src, OptLevel::O1));
    assert_eq!(
        format!("{o0:?}"),
        format!("{o1:?}"),
        "O0/O1 divergence for {expr}"
    );
    o0
}

#[track_caller]
fn check_u32(expr: &str, expected: u32) {
    let got = outcome("u32", expr).unwrap_or_else(|e| panic!("{expr}: unexpected {e:?}"));
    assert_eq!(
        format!("{got:?}"),
        format!("{:?}", Value::U32(expected)),
        "{expr}"
    );
}

#[track_caller]
fn check_bool(expr: &str, expected: bool) {
    let got = outcome("bool", expr).unwrap_or_else(|e| panic!("{expr}: unexpected {e:?}"));
    assert_eq!(
        format!("{got:?}"),
        format!("{:?}", Value::Bool(expected)),
        "{expr}"
    );
}

#[track_caller]
fn check_trap(expr: &str, expected: RuntimeTrap) {
    match outcome("u32", expr) {
        Err(RuntimeError::Trap(trap)) => assert_eq!(trap, expected, "{expr}"),
        other => panic!("{expr}: expected {expected:?}, got {other:?}"),
    }
}

#[test]
fn checked_arithmetic_positive_vectors() {
    check_u32("0u32 + 0u32", 0);
    check_u32("1u32 + 1u32", 2);
    check_u32(&format!("{MAX} + 0u32"), u32::MAX);
    check_u32("1u32 - 1u32", 0);
    check_u32(&format!("{MAX} - 1u32"), u32::MAX - 1);
    check_u32(&format!("0u32 * {MAX}"), 0);
    check_u32(&format!("1u32 * {MAX}"), u32::MAX);
    check_u32("65536u32 * 65535u32", 4_294_901_760);
    check_u32("0u32 / 1u32", 0);
    check_u32("7u32 / 3u32", 2);
    check_u32(&format!("{MAX} / 1u32"), u32::MAX);
    check_u32(&format!("{MAX} / {MAX}"), 1);
    check_u32("0u32 % 1u32", 0);
    check_u32("7u32 % 3u32", 1);
    check_u32(&format!("{MAX} % 2u32"), 1);
    check_u32(&format!("{I32_MAX_PLUS_1} + {I32_MAX}"), u32::MAX);
}

#[test]
fn overflow_underflow_and_zero_divisor_trap_with_frozen_classes() {
    check_trap(&format!("{MAX} + 1u32"), RuntimeTrap::ArithmeticOverflow);
    check_trap("0u32 - 1u32", RuntimeTrap::ArithmeticOverflow);
    check_trap(&format!("{MAX} * 2u32"), RuntimeTrap::ArithmeticOverflow);
    check_trap("65536u32 * 65536u32", RuntimeTrap::ArithmeticOverflow);
    check_trap("1u32 / 0u32", RuntimeTrap::DivisionByZero);
    check_trap("1u32 % 0u32", RuntimeTrap::DivisionByZero);
    check_trap("0u32 / 0u32", RuntimeTrap::DivisionByZero);
    check_trap("0u32 % 0u32", RuntimeTrap::DivisionByZero);
}

#[test]
fn ordering_is_unsigned_across_the_whole_domain() {
    let points = ["0u32", "1u32", I32_MAX, I32_MAX_PLUS_1, MAX];
    let value = |s: &str| -> u64 { s.trim_end_matches("u32").parse().unwrap() };
    for a in points {
        for b in points {
            let (x, y) = (value(a), value(b));
            check_bool(&format!("{a} < {b}"), x < y);
            check_bool(&format!("{a} <= {b}"), x <= y);
            check_bool(&format!("{a} > {b}"), x > y);
            check_bool(&format!("{a} >= {b}"), x >= y);
        }
    }
    check_bool(&format!("{I32_MAX_PLUS_1} > {I32_MAX}"), true);
    check_bool(&format!("{MAX} > {I32_MAX}"), true);
    check_bool(&format!("{MAX} >= {MAX}"), true);
}

#[test]
fn arithmetic_and_ordering_compose_through_bindings_and_loops() {
    // Non-constant operands: values flow through locals and a loop, so the
    // runtime opcodes (not only constant folding) are exercised at O1 too.
    let src = "fn probe() -> u32 {\n    let n: u32 = 10u32;\n    let mut i: u32 = 0u32;\n    let mut acc: u32 = 0u32;\n    while i < n {\n        acc = acc + i * 3u32;\n        i = i + 1u32;\n    }\n    assert(i >= n);\n    assert(acc / 9u32 == 15u32);\n    assert(acc % 9u32 == 0u32);\n    return acc - 1u32;\n}\nfn main() {\n    return;\n}\n";
    for opt in [OptLevel::O0, OptLevel::O1] {
        let got = run(&compile(src, opt)).expect("run");
        assert_eq!(
            format!("{got:?}"),
            format!("{:?}", Value::U32(134)),
            "{opt:?}"
        );
    }
}

#[test]
fn runtime_overflow_from_non_constant_operands_traps() {
    let src = format!("fn probe() -> u32 {{\n    let mut x: u32 = {MAX};\n    x = x - 1u32;\n    x = x + 1u32;\n    return x + 1u32;\n}}\nfn main() {{\n    return;\n}}\n");
    for opt in [OptLevel::O0, OptLevel::O1] {
        let err = run(&compile(&src, opt)).expect_err("must trap");
        assert!(
            matches!(err, RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow)),
            "{opt:?}: {err:?}"
        );
    }
}

fn compile_err(src: &str) -> String {
    compile_program_to_semcode_with_options(src, CompileProfile::RustLike, OptLevel::O0)
        .map(|_| ())
        .expect_err(&format!("must not compile:\n{src}"))
        .to_string()
}

#[test]
fn mixed_families_unary_minus_and_unsuffixed_literals_are_rejected() {
    let cases: &[(&str, &str)] = &[
        ("u32", "1u32 + 1"),
        ("u32", "1 + 1u32"),
        ("u32", "1u32 - 1"),
        ("u32", "1u32 * 1"),
        ("u32", "1u32 / 1"),
        ("u32", "1u32 % 1"),
        ("bool", "1u32 < 1"),
        ("bool", "1 < 1u32"),
        ("bool", "1u32 >= 1"),
        ("bool", "1u32 == 1"),
        ("u32", "-1u32"),
        ("u32", "1u32 + 1.0"),
    ];
    for (ret, expr) in cases {
        let src = source(ret, expr);
        let first = compile_err(&src);
        assert_eq!(
            first,
            compile_err(&src),
            "{expr}: diagnostic must be deterministic"
        );
    }
    // A u32 binding plus an unsuffixed literal stays a type error: no
    // contextual literal coercion.
    compile_err("fn probe() -> u32 {\n    let x: u32 = 1u32;\n    return x + 1;\n}\nfn main() {\n    return;\n}\n");
}

#[test]
fn measured_u32_arithmetic_stays_rejected() {
    for expr in ["a + b", "a - b", "a * b", "a / b", "a % b", "a < b"] {
        let src = format!(
            "fn probe(a: u32[ms], b: u32[ms]) -> bool {{\n    let c = {expr};\n    return true;\n}}\nfn main() {{\n    return;\n}}\n"
        );
        compile_err(&src);
    }
    // The ordering diagnostic names the admitted set honestly: plain u32 is
    // admitted, measured u32 is not.
    let err = compile_err(
        "fn probe(a: u32[ms], b: u32[ms]) -> bool {
    return a < b;
}
fn main() {
    return;
}
",
    );
    assert!(
        err.contains("same-family i32 or plain (unmeasured) u32 operands"),
        "{err}"
    );
}

#[test]
fn only_programs_using_the_u32_family_require_semcod23() {
    let plain = compile(&source("i32", "1 + 2"), OptLevel::O0);
    assert_eq!(
        &plain[0..8],
        &MAGIC22,
        "unrelated programs keep the V22 floor"
    );
    verify_semcode(&plain).expect("ordinary V22 artifact still admitted");
    let eq_only = compile(&source("bool", "1u32 == 2u32"), OptLevel::O0);
    assert_eq!(&eq_only[0..8], &MAGIC22, "u32 equality needs no new opcode");

    let arith = compile(&source("u32", "7u32 + 8u32"), OptLevel::O0);
    assert_eq!(&arith[0..8], &MAGIC23);
    let spec = header_spec_from_magic(&MAGIC23).expect("V23 supported");
    assert_eq!(spec.rev, 24);
    verify_semcode(&arith).expect("V23 artifact admitted");
}

#[test]
fn relabelling_a_v23_artifact_as_v22_is_rejected() {
    for expr in [
        "1u32 + 2u32",
        "3u32 - 2u32",
        "2u32 * 2u32",
        "4u32 / 2u32",
        "4u32 % 3u32",
        "1u32 < 2u32",
        "1u32 <= 2u32",
    ] {
        let ret = if expr.contains('<') { "bool" } else { "u32" };
        let mut bytes = compile(&source(ret, expr), OptLevel::O0);
        assert_eq!(&bytes[0..8], &MAGIC23, "{expr}");
        bytes[0..8].copy_from_slice(&MAGIC22);
        assert!(
            verify_semcode(&bytes).is_err(),
            "{expr}: a SEMCOD22 header must not admit the u32 opcode family"
        );
    }
}

#[test]
fn opcode_bytes_are_frozen() {
    let table = [
        (Opcode::CmpU32Lt, 0x24u8),
        (Opcode::CmpU32Le, 0x25),
        (Opcode::AddU32, 0x26),
        (Opcode::SubU32, 0x27),
        (Opcode::MulU32, 0x28),
        (Opcode::DivU32, 0x29),
        (Opcode::ModU32, 0x2A),
    ];
    for (op, byte) in table {
        assert_eq!(op as u8, byte, "{op:?}");
        assert_eq!(Opcode::from_byte(byte).expect("decodes"), op);
        assert_eq!(op.minimum_semcode_revision(), 24, "{op:?}");
    }
    assert!(Opcode::from_byte(0x2B).is_err(), "0x2B stays unassigned");
}

/// A hand-built V23 function `probe` whose last instruction before `Ret` is
/// `AddU32 r7, r5, r6` (distinctive operands so the bytes are unambiguous).
fn hand_built_add_u32() -> Vec<u8> {
    emit_ir_to_semcode(
        &[IrFunction {
            name: "probe".to_string(),
            instrs: vec![
                IrInstr::LoadU32 { dst: 5, val: 1 },
                IrInstr::LoadU32 { dst: 6, val: 2 },
                IrInstr::AddU32 {
                    dst: 7,
                    lhs: 5,
                    rhs: 6,
                },
                IrInstr::Ret { src: Some(7) },
            ],
            ownership_events: Vec::new(),
            params: Vec::new(),
        }],
        false,
    )
    .expect("emit")
}

const ADD_U32_R7_R5_R6: [u8; 7] = [0x26, 7, 0, 5, 0, 6, 0];

#[test]
fn hand_built_u32_instruction_round_trips_and_malformed_forms_fail_closed() {
    let bytes = hand_built_add_u32();
    assert_eq!(&bytes[0..8], &MAGIC23);
    let token = verify_semcode_token(&bytes).expect("admitted");
    let entry = token.require_entry("probe").expect("entry");
    let got = run_verified_function_semcode_with_args(&entry, vec![]).expect("run");
    assert_eq!(format!("{got:?}"), format!("{:?}", Value::U32(3)));

    let at = bytes
        .windows(ADD_U32_R7_R5_R6.len())
        .position(|w| w == ADD_U32_R7_R5_R6)
        .expect("instruction bytes present");

    // Unknown opcode in place of AddU32.
    let mut unknown = bytes.clone();
    unknown[at] = 0x2B;
    assert!(
        verify_semcode(&unknown).is_err(),
        "unknown opcode must be rejected"
    );

    // Truncated AddU32: drop Ret and the rhs register, shrinking the code
    // length of the (only, last) function to match.
    let tail = bytes.len() - (at + ADD_U32_R7_R5_R6.len()); // Ret bytes
    let drop = tail + 2;
    let mut truncated = bytes[..bytes.len() - drop].to_vec();
    let name_at = truncated
        .windows(b"probe".len())
        .rposition(|w| w == b"probe")
        .expect("function name");
    let len_at = name_at + b"probe".len();
    let len = u32::from_le_bytes(truncated[len_at..len_at + 4].try_into().unwrap());
    truncated[len_at..len_at + 4].copy_from_slice(&(len - drop as u32).to_le_bytes());
    assert!(
        verify_semcode(&truncated).is_err(),
        "truncated instruction must be rejected"
    );
}

#[test]
fn compilation_is_byte_identical_and_execution_deterministic() {
    let src = format!("fn probe() -> u32 {{\n    let a: u32 = {I32_MAX_PLUS_1};\n    let b: u32 = 3u32;\n    let c: u32 = a / b + a % b * 2u32 - 1u32;\n    assert(c > b);\n    return c;\n}}\nfn main() {{\n    return;\n}}\n");
    for opt in [OptLevel::O0, OptLevel::O1] {
        assert_eq!(
            compile(&src, opt),
            compile(&src, opt),
            "{opt:?}: SemCode must be byte-identical"
        );
    }
    let o0 = format!("{:?}", run(&compile(&src, OptLevel::O0)));
    let o1 = format!("{:?}", run(&compile(&src, OptLevel::O1)));
    assert_eq!(o0, o1);
    assert_eq!(
        o0,
        format!("{:?}", Ok::<Value, RuntimeError>(Value::U32(715_827_885)))
    );
}

#[test]
fn existing_behaviour_is_unchanged() {
    // i32 still wraps, i32 division by zero still traps.
    let i32_wrap = source("i32", "2147483647 + 1");
    for opt in [OptLevel::O0, OptLevel::O1] {
        let got = run(&compile(&i32_wrap, opt)).expect("i32 wraps");
        assert_eq!(
            format!("{got:?}"),
            format!("{:?}", Value::I32(i32::MIN)),
            "{opt:?}"
        );
        let err = run(&compile(&source("i32", "1 / 0"), opt)).expect_err("i32 div zero");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::DivisionByZero)
        ));
    }
    // u32 equality and match are unchanged.
    check_bool("4294967295u32 == 4294967295u32", true);
    check_bool("0u32 != 4294967295u32", true);
    let m = "fn probe() -> i32 {\n    let x: u32 = 4294967295u32;\n    let r: i32 = match x {\n        0 => { 1 }\n        4294967295 => { 2 }\n        _ => { 3 }\n    };\n    return r;\n}\nfn main() {\n    return;\n}\n";
    let got = run(&compile(m, OptLevel::O0)).expect("u32 match");
    assert_eq!(format!("{got:?}"), format!("{:?}", Value::I32(2)));
}
