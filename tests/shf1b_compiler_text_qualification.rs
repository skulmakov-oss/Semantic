//! SHF-1B (#2004): full-pipeline qualification of the compiler-grade UTF-8
//! text primitives frozen by `docs/spec/compiler_text_v0.md`
//! (`semantic.compiler.text/0.1`).
//!
//! Every positive vector is compiled from Semantic source, admitted by
//! `sm-verify`, executed by `sm-vm` through the verified-token route, and
//! observed as the actual returned runtime `Value`. No Rust helper is used as
//! a substitute for the pipeline.

use sm_emit::compile_program_to_semcode;
use sm_ir::{emit_ir_to_semcode, IrFunction, IrInstr};
use sm_format::semcode_format::MAGIC7;
use sm_runtime_core::AdtCarrier;
use sm_verify::{verify_semcode, verify_semcode_token, VerificationCode};
use sm_vm::{run_verified_function_semcode_with_args, Value};

/// Compile `fn probe() -> <ret> { return <expr>; }`, verify it, run the
/// verified `probe` entry, and return the observed value.
fn eval(ret: &str, expr: &str) -> Value {
    let src = format!("fn probe() -> {ret} {{\n    return {expr};\n}}\nfn main() {{\n    return;\n}}\n");
    eval_source(&src)
}

fn eval_source(src: &str) -> Value {
    let bytes = compile_program_to_semcode(src).unwrap_or_else(|e| panic!("compile:\n{src}\n{e}"));
    let token = verify_semcode_token(&bytes).unwrap_or_else(|e| panic!("verify:\n{src}\n{e}"));
    let entry = token.require_entry("probe").expect("probe entry");
    run_verified_function_semcode_with_args(&entry, vec![])
        .unwrap_or_else(|e| panic!("run:\n{src}\n{e}"))
}

fn compile_err(src: &str) -> String {
    compile_program_to_semcode(src)
        .map(|_| ())
        .expect_err(&format!("must not compile:\n{src}"))
        .to_string()
}

fn some(v: Value) -> Value {
    Value::Adt(AdtCarrier {
        type_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        tag: 1,
        payload: vec![v],
    })
}

fn none() -> Value {
    Value::Adt(AdtCarrier {
        type_name: "Option".to_string(),
        variant_name: "None".to_string(),
        tag: 0,
        payload: Vec::new(),
    })
}

fn u(v: u32) -> Value {
    Value::U32(v)
}

fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}

fn b(v: bool) -> Value {
    Value::Bool(v)
}

#[track_caller]
fn check(ret: &str, expr: &str, expected: Value) {
    let got = eval(ret, expr);
    assert_eq!(format!("{got:?}"), format!("{expected:?}"), "{expr}");
}

// Mixed text: "aé€🙂z" = 61 | C3 A9 | E2 82 AC | F0 9F 99 82 | 7A (11 bytes).
const MIXED: &str = "\"a\u{e9}\u{20ac}\u{1f642}z\"";

#[test]
fn text_len_counts_utf8_bytes_not_scalars_or_graphemes() {
    check("u32", "text_len(\"\")", u(0));
    check("u32", "text_len(\"A\")", u(1));
    check("u32", "text_len(\"\u{e9}\")", u(2));
    check("u32", "text_len(\"\u{20ac}\")", u(3));
    check("u32", "text_len(\"\u{1f642}\")", u(4));
    check("u32", &format!("text_len({MIXED})"), u(11));
    check("u32", "text_len(\"hello world\")", u(11));
}

#[test]
fn text_byte_at_returns_raw_bytes_including_continuation_bytes() {
    let r = "Option(u32)";
    check(r, "text_byte_at(\"A\", 0u32)", some(u(0x41)));
    check(r, "text_byte_at(\"\u{e9}\", 0u32)", some(u(0xC3)));
    check(r, "text_byte_at(\"\u{e9}\", 1u32)", some(u(0xA9)));
    check(r, "text_byte_at(\"\u{e9}\", 2u32)", none());
    for (i, byte) in [0xE2u32, 0x82, 0xAC].into_iter().enumerate() {
        check(r, &format!("text_byte_at(\"\u{20ac}\", {i}u32)"), some(u(byte)));
    }
    for (i, byte) in [0xF0u32, 0x9F, 0x99, 0x82].into_iter().enumerate() {
        check(r, &format!("text_byte_at(\"\u{1f642}\", {i}u32)"), some(u(byte)));
    }
    check(r, &format!("text_byte_at({MIXED}, 0u32)"), some(u(0x61)));
    check(r, &format!("text_byte_at({MIXED}, 10u32)"), some(u(0x7A)));
    check(r, &format!("text_byte_at({MIXED}, 11u32)"), none());
    check(r, &format!("text_byte_at({MIXED}, 4000000000u32)"), none());
    check(r, "text_byte_at(\"\", 0u32)", none());
}

#[test]
fn text_slice_is_half_open_and_rejects_non_scalar_boundaries() {
    let r = "Option(text)";
    check(r, &format!("text_slice({MIXED}, 3u32, 3u32)"), some(t("")));
    check(r, &format!("text_slice({MIXED}, 0u32, 11u32)"), some(t("a\u{e9}\u{20ac}\u{1f642}z")));
    check(r, &format!("text_slice({MIXED}, 1u32, 6u32)"), some(t("\u{e9}\u{20ac}")));
    check(r, &format!("text_slice({MIXED}, 6u32, 10u32)"), some(t("\u{1f642}")));
    check(r, &format!("text_slice({MIXED}, 11u32, 11u32)"), some(t("")));
    // start / end inside a multi-byte scalar
    check(r, &format!("text_slice({MIXED}, 2u32, 6u32)"), none());
    check(r, &format!("text_slice({MIXED}, 1u32, 2u32)"), none());
    check(r, &format!("text_slice({MIXED}, 7u32, 7u32)"), none());
    // start > end, end > L, start > L
    check(r, &format!("text_slice({MIXED}, 3u32, 1u32)"), none());
    check(r, &format!("text_slice({MIXED}, 0u32, 12u32)"), none());
    check(r, &format!("text_slice({MIXED}, 12u32, 12u32)"), none());
    check(r, "text_slice(\"\", 0u32, 0u32)", some(t("")));
    check(r, "text_slice(\"\", 0u32, 1u32)", none());
}

#[test]
fn text_starts_with_and_ends_with_compare_exact_bytes() {
    check("bool", "text_starts_with(\"abc\", \"\")", b(true));
    check("bool", "text_starts_with(\"\", \"\")", b(true));
    check("bool", "text_starts_with(\"abc\", \"ab\")", b(true));
    check("bool", "text_starts_with(\"abc\", \"bc\")", b(false));
    check("bool", "text_starts_with(\"abc\", \"Ab\")", b(false));
    check("bool", "text_starts_with(\"Abc\", \"ab\")", b(false));
    check("bool", "text_ends_with(\"abC\", \"bc\")", b(false));
    check("bool", "text_starts_with(\"ab\", \"abc\")", b(false));
    check("bool", &format!("text_starts_with({MIXED}, \"a\u{e9}\")"), b(true));
    check("bool", "text_ends_with(\"abc\", \"\")", b(true));
    check("bool", "text_ends_with(\"\", \"\")", b(true));
    check("bool", "text_ends_with(\"abc\", \"bc\")", b(true));
    check("bool", "text_ends_with(\"abc\", \"ab\")", b(false));
    check("bool", "text_ends_with(\"bc\", \"abc\")", b(false));
    check("bool", &format!("text_ends_with({MIXED}, \"\u{1f642}z\")"), b(true));
    // decomposed e + combining acute is not equal to precomposed U+00E9
    check("bool", "text_ends_with(\"caf\u{e9}\", \"e\u{301}\")", b(false));
}

#[test]
fn text_find_returns_smallest_byte_offset() {
    let r = "Option(u32)";
    check(r, "text_find(\"abcabc\", \"abc\")", some(u(0)));
    check(r, "text_find(\"xxabcabc\", \"abc\")", some(u(2)));
    check(r, "text_find(\"xyzab\", \"ab\")", some(u(3)));
    check(r, "text_find(\"abc\", \"d\")", none());
    check(r, &format!("text_find({MIXED}, \"\u{20ac}\")"), some(u(3)));
    check(r, &format!("text_find({MIXED}, \"z\")"), some(u(10)));
    check(r, "text_find(\"abc\", \"\")", some(u(0)));
    check(r, "text_find(\"\", \"\")", some(u(0)));
    check(r, "text_find(\"ab\", \"abc\")", none());
    check(r, "text_find(\"\", \"a\")", none());
}

#[test]
fn text_is_empty_is_zero_byte_length_only() {
    check("bool", "text_is_empty(\"\")", b(true));
    check("bool", "text_is_empty(\" \")", b(false));
    check("bool", "text_is_empty(\"\u{e9}\")", b(false));
}

#[test]
fn option_results_are_canonical_semantic_options_through_match() {
    let src = "fn probe() -> u32 {\n    let t: text = \"x\u{e9}y\";\n    let a: u32 = match text_byte_at(t, 1u32) {\n        Option::Some(v) => { v }\n        Option::None => { 0u32 }\n    };\n    let found: bool = match text_find(t, \"y\") {\n        Option::Some(i) => { i == 3u32 }\n        Option::None => { false }\n    };\n    let sliced: text = match text_slice(t, 1u32, 3u32) {\n        Option::Some(s) => { s }\n        Option::None => { \"\" }\n    };\n    assert(found);\n    assert(sliced == \"\u{e9}\");\n    assert(text_is_empty(\"\"));\n    return a;\n}\nfn main() {\n    return;\n}\n";
    assert_eq!(format!("{:?}", eval_source(src)), format!("{:?}", u(0xC3)));
}

#[test]
fn text_builtin_on_a_parameter_runs_without_any_text_literal() {
    // No LoadText/ConcatText anywhere: the text value arrives as an argument
    // and is inspected only by the builtin, through the verified entry.
    let src = "fn probe(t: text) -> u32 {\n    return text_len(t);\n}\nfn main() {\n    return;\n}\n";
    let bytes = compile_program_to_semcode(src).expect("compile");
    let token = verify_semcode_token(&bytes).expect("verify");
    let entry = token.require_entry("probe").expect("entry");
    let got = run_verified_function_semcode_with_args(&entry, vec![t("\u{20ac}")]).expect("run");
    assert_eq!(format!("{got:?}"), format!("{:?}", u(3)));
}

#[test]
fn bare_text_builtin_calls_require_text_capability_at_admission() {
    for name in [
        "text_len",
        "text_byte_at",
        "text_slice",
        "text_starts_with",
        "text_ends_with",
        "text_find",
        "text_is_empty",
    ] {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "caller".to_string(),
                instrs: vec![
                    IrInstr::LoadI32 { dst: 0, val: 1 },
                    IrInstr::Call {
                        dst: Some(1),
                        name: name.to_string(),
                        args: vec![0],
                    },
                    IrInstr::Ret { src: Some(1) },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let bytes = downgrade_header(&bytes, MAGIC7); // SEMCODE7 lacks CAP_TEXT_VALUES
        let report = verify_semcode(&bytes).expect_err(name);
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation,
            "{name}"
        );
    }
}

/// Re-encode a current artifact under an older header (same technique as the
/// existing `to_text` capability tests in `sm-vm`).
fn downgrade_header(bytes: &[u8], target_magic: [u8; 8]) -> Vec<u8> {
    let (_, functions) =
        sm_format::semcode_decode::decode_semcode_envelope(bytes).expect("decode");
    let mut out = Vec::new();
    out.extend_from_slice(&target_magic);
    for f in &functions {
        let mut code = Vec::new();
        code.extend_from_slice(&f.code_slice[..f.string_table_end_offset]);
        code.extend_from_slice(&f.code_slice[f.instr_start_offset..]);
        out.extend_from_slice(&(f.name.len() as u16).to_le_bytes());
        out.extend_from_slice(f.name.as_bytes());
        out.extend_from_slice(&(code.len() as u32).to_le_bytes());
        out.extend_from_slice(&code);
    }
    out
}

#[test]
fn text_builtin_names_are_reserved_against_user_definitions() {
    for name in [
        "text_len",
        "text_byte_at",
        "text_slice",
        "text_starts_with",
        "text_ends_with",
        "text_find",
        "text_is_empty",
    ] {
        let err = compile_err(&format!(
            "fn {name}(x: i32) -> i32 {{\n    return x;\n}}\nfn main() {{\n    return;\n}}\n"
        ));
        assert!(err.contains(name) && err.contains("reserved"), "{name}: {err}");
    }
}

#[test]
fn text_builtins_reject_wrong_arity_and_argument_families() {
    let cases: &[(&str, &str, &str)] = &[
        // (call, return type, required message fragment)
        ("text_len()", "u32", "builtin 'text_len' takes exactly 1 positional argument"),
        ("text_len(\"a\", \"b\")", "u32", "builtin 'text_len' takes exactly 1 positional argument"),
        ("text_len(7u32)", "u32", "builtin 'text_len' argument 1 must be text"),
        ("text_byte_at(\"a\")", "Option(u32)", "builtin 'text_byte_at' takes exactly 2 positional arguments"),
        ("text_byte_at(7u32, 0u32)", "Option(u32)", "builtin 'text_byte_at' argument 1 must be text"),
        ("text_byte_at(\"a\", 0)", "Option(u32)", "builtin 'text_byte_at' argument 2 must be u32"),
        ("text_byte_at(\"a\", \"0\")", "Option(u32)", "builtin 'text_byte_at' argument 2 must be u32"),
        ("text_slice(\"a\", 0u32)", "Option(text)", "builtin 'text_slice' takes exactly 3 positional arguments"),
        ("text_slice(true, 0u32, 0u32)", "Option(text)", "builtin 'text_slice' argument 1 must be text"),
        ("text_slice(\"a\", 0, 1u32)", "Option(text)", "builtin 'text_slice' argument 2 must be u32"),
        ("text_slice(\"a\", 0u32, true)", "Option(text)", "builtin 'text_slice' argument 3 must be u32"),
        ("text_starts_with(\"a\")", "bool", "builtin 'text_starts_with' takes exactly 2 positional arguments"),
        ("text_starts_with(1u32, \"a\")", "bool", "builtin 'text_starts_with' argument 1 must be text"),
        ("text_starts_with(\"a\", 1u32)", "bool", "builtin 'text_starts_with' argument 2 must be text"),
        ("text_ends_with(\"a\", \"b\", \"c\")", "bool", "builtin 'text_ends_with' takes exactly 2 positional arguments"),
        ("text_ends_with(\"a\", 1)", "bool", "builtin 'text_ends_with' argument 2 must be text"),
        ("text_find(\"a\")", "Option(u32)", "builtin 'text_find' takes exactly 2 positional arguments"),
        ("text_find(1u32, \"a\")", "Option(u32)", "builtin 'text_find' argument 1 must be text"),
        ("text_find(\"a\", false)", "Option(u32)", "builtin 'text_find' argument 2 must be text"),
        ("text_is_empty()", "bool", "builtin 'text_is_empty' takes exactly 1 positional argument"),
        ("text_is_empty(0u32)", "bool", "builtin 'text_is_empty' argument 1 must be text"),
    ];
    for (call, ret, fragment) in cases {
        let src = format!("fn probe() -> {ret} {{\n    return {call};\n}}\nfn main() {{\n    return;\n}}\n");
        let first = compile_err(&src);
        assert!(first.contains(fragment), "{call}: {first}");
        assert_eq!(first, compile_err(&src), "{call}: diagnostic must be deterministic");
    }
}

#[test]
fn text_builtin_results_have_the_frozen_types() {
    // Result types are exactly the frozen signatures: a mismatching
    // declaration must be rejected, not coerced.
    for (call, wrong) in [
        ("text_len(\"a\")", "i32"),
        ("text_byte_at(\"a\", 0u32)", "u32"),
        ("text_slice(\"a\", 0u32, 1u32)", "text"),
        ("text_starts_with(\"a\", \"a\")", "u32"),
        ("text_find(\"a\", \"a\")", "Option(i32)"),
        ("text_is_empty(\"a\")", "u32"),
    ] {
        compile_err(&format!(
            "fn probe() -> {wrong} {{\n    return {call};\n}}\nfn main() {{\n    return;\n}}\n"
        ));
    }
}

#[test]
fn compilation_and_execution_are_deterministic() {
    let src = format!("fn probe() -> Option(text) {{\n    let x: u32 = text_len({MIXED});\n    assert(x == 11u32);\n    assert(text_starts_with({MIXED}, \"a\"));\n    assert(text_ends_with({MIXED}, \"z\"));\n    assert(!text_is_empty({MIXED}));\n    let f: Option(u32) = text_find({MIXED}, \"\u{20ac}\");\n    let b: Option(u32) = text_byte_at({MIXED}, 4u32);\n    return text_slice({MIXED}, 1u32, 6u32);\n}}\nfn main() {{\n    return;\n}}\n");
    let a = compile_program_to_semcode(&src).expect("compile a");
    let b = compile_program_to_semcode(&src).expect("compile b");
    assert_eq!(a, b, "SemCode must be byte-identical across compilations");
    let first = format!("{:?}", eval_source(&src));
    let second = format!("{:?}", eval_source(&src));
    assert_eq!(first, second);
    assert_eq!(first, format!("{:?}", some(t("\u{e9}\u{20ac}"))));
}

#[test]
fn invalid_utf8_in_string_table_is_rejected_before_execution() {
    // SHF-1B adds no new ingress: a corrupted text literal must still be
    // rejected by admission, never reach the new builtins.
    let src = "fn probe() -> u32 {\n    return text_len(\"QZQZQZ\");\n}\nfn main() {\n    return;\n}\n";
    let mut bytes = compile_program_to_semcode(src).expect("compile");
    let at = bytes
        .windows(6)
        .position(|w| w == b"QZQZQZ")
        .expect("literal bytes present");
    bytes[at] = 0xFF;
    assert!(verify_semcode_token(&bytes).is_err(), "invalid UTF-8 must not be admitted");
}
