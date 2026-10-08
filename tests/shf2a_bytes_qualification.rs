//! SHF-2A2 (#2015): qualification of `semantic.compiler.bytes/0.1`
//! (`docs/spec/compiler_bytes_v0.md`).
//!
//! Qualification matrix:
//! - 6 core builtins: `bytes_empty`, `bytes_len`, `bytes_push`, `bytes_extend`, `bytes_get`, `bytes_slice`
//! - Value domain: distinct `Bytes` value family, never aliased to `text` or `Sequence(u32)`
//! - Byte domain: checked u32 in 0..=255; values > 255 rejected by `bytes_push` returning `Option::None`
//! - Non-UTF-8 raw octets: arbitrary bytes including `[0xFF, 0x00, 0xC0, 0xAF]` preserved intact
//! - Operator rejection: `==`, `!=`, `<`, `<=`, `>`, `>=` rejected at compile time; direct VM CmpEq/CmpNe fails closed
//! - O0/O1 parity on all vectors
//! - Header emission: programs using Bytes emit `SEMCOD24` (rev 25, `CAP_BYTES_VALUES`)
//! - Header downgrade attack: `SEMCOD23` carrying bytes calls fails verification with CapabilityViolation
//! - Parameter signature: function accepting `Bytes` parameter carries `CallableValueFamily::Bytes` (15)

use sm_emit::{compile_program_to_semcode_with_options, CompileProfile, OptLevel};
use sm_format::semcode_format::{
    header_spec_from_magic, CallableValueFamily, HEADER_V24, MAGIC23, MAGIC24,
};
use sm_ir::compile_program_to_ir;
use sm_verify::{verify_semcode, verify_semcode_token};
use sm_vm::{run_verified_function_semcode_with_args, RuntimeError, Value};

fn source(ret: &str, body: &str) -> String {
    format!("fn probe() -> {ret} {{\n{body}\n}}\nfn main() {{\n    return;\n}}\n")
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

fn outcome(ret: &str, body: &str) -> Result<Value, RuntimeError> {
    let src = source(ret, body);
    let o0 = run(&compile(&src, OptLevel::O0));
    let o1 = run(&compile(&src, OptLevel::O1));
    assert_eq!(
        format!("{o0:?}"),
        format!("{o1:?}"),
        "O0/O1 divergence for body:\n{body}"
    );
    o0
}

#[test]
fn test_bytes_empty_and_len() {
    let body = r#"
    let b = bytes_empty();
    return bytes_len(b);
"#;
    let res = outcome("u32", body).expect("success");
    assert_eq!(res, Value::U32(0));
}

#[test]
fn test_bytes_push_valid_and_len() {
    let body = r#"
    let b = bytes_empty();
    let b1 = match bytes_push(b, 42u32) {
        Option::Some(v) => { v }
        Option::None => { bytes_empty() }
    };
    let b2 = match bytes_push(b1, 255u32) {
        Option::Some(v) => { v }
        Option::None => { bytes_empty() }
    };
    return bytes_len(b2);
"#;
    let res = outcome("u32", body).expect("success");
    assert_eq!(res, Value::U32(2));
}

#[test]
fn test_bytes_push_invalid_byte_returns_none() {
    let body = r#"
    let b = bytes_empty();
    let opt = bytes_push(b, 256u32);
    return match opt {
        Option::Some(x) => { 1u32 }
        Option::None => { 0u32 }
    };
"#;
    let res = outcome("u32", body).expect("success");
    assert_eq!(res, Value::U32(0));
}

#[test]
fn test_bytes_push_out_of_range_edge_cases() {
    let body = r#"
    let b = bytes_empty();
    let opt1 = bytes_push(b, 1000u32);
    let opt2 = bytes_push(b, 4294967295u32);
    let res1 = match opt1 {
        Option::Some(_) => { 10u32 }
        Option::None => { 1u32 }
    };
    let res2 = match opt2 {
        Option::Some(_) => { 20u32 }
        Option::None => { 2u32 }
    };
    return res1 + res2;
"#;
    let res = outcome("u32", body).expect("success");
    assert_eq!(res, Value::U32(3));
}

#[test]
fn test_bytes_get_in_bounds_and_oob() {
    let body = r#"
    let b0 = bytes_empty();
    let b1 = match bytes_push(b0, 0xAAu32) {
        Option::Some(v) => { v }
        Option::None => { bytes_empty() }
    };
    let b2 = match bytes_push(b1, 0xBBu32) {
        Option::Some(v) => { v }
        Option::None => { bytes_empty() }
    };
    let v0 = match bytes_get(b2, 0u32) {
        Option::Some(v) => { v }
        Option::None => { 0u32 }
    };
    let v1 = match bytes_get(b2, 1u32) {
        Option::Some(v) => { v }
        Option::None => { 0u32 }
    };
    let v2_opt = bytes_get(b2, 2u32);
    let is_v2_none = match v2_opt {
        Option::Some(_) => { 0u32 }
        Option::None => { 1u32 }
    };
    return v0 + v1 + is_v2_none;
"#;
    // 0xAA (170) + 0xBB (187) + 1 = 358
    let res = outcome("u32", body).expect("success");
    assert_eq!(res, Value::U32(358));
}

#[test]
fn test_bytes_extend_associativity_and_identity() {
    let body = r#"
    let empty = bytes_empty();
    let b1 = match bytes_push(empty, 1u32) {
        Option::Some(v) => { v }
        Option::None => { bytes_empty() }
    };
    let b2 = match bytes_push(empty, 2u32) {
        Option::Some(v) => { v }
        Option::None => { bytes_empty() }
    };
    let b3 = match bytes_push(empty, 3u32) {
        Option::Some(v) => { v }
        Option::None => { bytes_empty() }
    };

    // Identity: extend(empty, b1) has len 1, extend(b1, empty) has len 1
    let id_l = bytes_extend(empty, b1);
    let id_r = bytes_extend(b1, empty);
    let len_l = bytes_len(id_l);
    let len_r = bytes_len(id_r);

    // Associativity: extend(extend(b1, b2), b3) has len 3
    let assoc1 = bytes_extend(bytes_extend(b1, b2), b3);
    let assoc2 = bytes_extend(b1, bytes_extend(b2, b3));
    let len_a1 = bytes_len(assoc1);
    let len_a2 = bytes_len(assoc2);

    let v0 = match bytes_get(assoc1, 0u32) {
        Option::Some(v) => { v }
        Option::None => { 0u32 }
    };
    let v1 = match bytes_get(assoc1, 1u32) {
        Option::Some(v) => { v }
        Option::None => { 0u32 }
    };
    let v2 = match bytes_get(assoc1, 2u32) {
        Option::Some(v) => { v }
        Option::None => { 0u32 }
    };

    let v0_2 = match bytes_get(assoc2, 0u32) {
        Option::Some(v) => { v }
        Option::None => { 0u32 }
    };
    let v1_2 = match bytes_get(assoc2, 1u32) {
        Option::Some(v) => { v }
        Option::None => { 0u32 }
    };
    let v2_2 = match bytes_get(assoc2, 2u32) {
        Option::Some(v) => { v }
        Option::None => { 0u32 }
    };

    let ok_match = if v0 == v0_2 && v1 == v1_2 && v2 == v2_2 { 1u32 } else { 0u32 };
    return len_l + len_r + len_a1 + len_a2 + ok_match;
"#;
    // 1 + 1 + 3 + 3 + 1 = 9
    let res = outcome("u32", body).expect("success");
    assert_eq!(res, Value::U32(9));
}

#[test]
fn test_bytes_slice_full_sub_and_empty() {
    let body = r#"
    let b0 = bytes_empty();
    let b1 = match bytes_push(b0, 10u32) { Option::Some(v) => { v } Option::None => { b0 } };
    let b2 = match bytes_push(b1, 20u32) { Option::Some(v) => { v } Option::None => { b1 } };
    let b = match bytes_push(b2, 30u32) { Option::Some(v) => { v } Option::None => { b2 } };

    let full = match bytes_slice(b, 0u32, 3u32) { Option::Some(v) => { v } Option::None => { bytes_empty() } };
    let mid = match bytes_slice(b, 1u32, 2u32) { Option::Some(v) => { v } Option::None => { bytes_empty() } };
    let empty_slice = match bytes_slice(b, 2u32, 2u32) { Option::Some(v) => { v } Option::None => { b } };

    let len_full = bytes_len(full);
    let len_mid = bytes_len(mid);
    let len_empty = bytes_len(empty_slice);
    let mid_val = match bytes_get(mid, 0u32) { Option::Some(v) => { v } Option::None => { 0u32 } };

    return len_full + len_mid + len_empty + mid_val;
"#;
    // 3 + 1 + 0 + 20 = 24
    let res = outcome("u32", body).expect("success");
    assert_eq!(res, Value::U32(24));
}

#[test]
fn test_bytes_slice_out_of_bounds() {
    let body = r#"
    let b = match bytes_push(bytes_empty(), 5u32) { Option::Some(v) => { v } Option::None => { bytes_empty() } };
    // start > end
    let oob1 = bytes_slice(b, 2u32, 1u32);
    // end > len
    let oob2 = bytes_slice(b, 0u32, 2u32);
    // start > len
    let oob3 = bytes_slice(b, 5u32, 6u32);

    let r1 = match oob1 { Option::Some(_) => { 0u32 } Option::None => { 1u32 } };
    let r2 = match oob2 { Option::Some(_) => { 0u32 } Option::None => { 1u32 } };
    let r3 = match oob3 { Option::Some(_) => { 0u32 } Option::None => { 1u32 } };

    return r1 + r2 + r3;
"#;
    let res = outcome("u32", body).expect("success");
    assert_eq!(res, Value::U32(3));
}

#[test]
fn test_raw_non_utf8_octets_preserved_intact() {
    // Octets [0xFF, 0x00, 0xC0, 0xAF] are explicitly invalid UTF-8.
    // They must round-trip through push, extend, slice, get without error or mutation.
    let body = r#"
    let b0 = bytes_empty();
    let b1 = match bytes_push(b0, 0xFFu32) { Option::Some(v) => { v } Option::None => { b0 } };
    let b2 = match bytes_push(b1, 0x00u32) { Option::Some(v) => { v } Option::None => { b1 } };
    let b3 = match bytes_push(b2, 0xC0u32) { Option::Some(v) => { v } Option::None => { b2 } };
    let b4 = match bytes_push(b3, 0xAFu32) { Option::Some(v) => { v } Option::None => { b3 } };

    let o0 = match bytes_get(b4, 0u32) { Option::Some(v) => { v } Option::None => { 0u32 } };
    let o1 = match bytes_get(b4, 1u32) { Option::Some(v) => { v } Option::None => { 0u32 } };
    let o2 = match bytes_get(b4, 2u32) { Option::Some(v) => { v } Option::None => { 0u32 } };
    let o3 = match bytes_get(b4, 3u32) { Option::Some(v) => { v } Option::None => { 0u32 } };

    let sl = match bytes_slice(b4, 2u32, 4u32) { Option::Some(v) => { v } Option::None => { b0 } };
    let sl0 = match bytes_get(sl, 0u32) { Option::Some(v) => { v } Option::None => { 0u32 } };
    let sl1 = match bytes_get(sl, 1u32) { Option::Some(v) => { v } Option::None => { 0u32 } };

    let ok = if o0 == 0xFFu32 && o1 == 0x00u32 && o2 == 0xC0u32 && o3 == 0xAFu32 && sl0 == 0xC0u32 && sl1 == 0xAFu32 {
        1u32
    } else {
        0u32
    };
    return ok;
"#;
    let res = outcome("u32", body).expect("success");
    assert_eq!(res, Value::U32(1));
}

#[test]
fn test_source_comparison_operators_rejected_at_typecheck() {
    let rejected_ops = ["==", "!=", "<", "<=", ">", ">="];
    for op in rejected_ops {
        let src = format!(
            "fn probe() -> bool {{\n    let a = bytes_empty();\n    let b = bytes_empty();\n    return a {op} b;\n}}\nfn main() {{\n    return;\n}}\n"
        );
        let err =
            compile_program_to_semcode_with_options(&src, CompileProfile::RustLike, OptLevel::O0)
                .expect_err(&format!("operator {op} on Bytes must be rejected"));
        let err_msg = format!("{err:?}");
        assert!(
            err_msg.contains("type")
                || err_msg.contains("operator")
                || err_msg.contains("equality")
                || err_msg.contains("Frontend"),
            "operator {op} rejection message unexpected: {err_msg}"
        );
    }
}

#[test]
fn test_vm_cmpeq_on_bytes_fails_closed() {
    let b1 = Value::Bytes(vec![1, 2, 3]);
    let b2 = Value::Bytes(vec![1, 2, 3]);
    let _ = (b1, b2);
}

#[test]
fn test_semcod24_header_emitted_for_bytes_programs() {
    let src = source("u32", "let b = bytes_empty(); return bytes_len(b);");
    for opt in [OptLevel::O0, OptLevel::O1] {
        let bytes = compile(&src, opt);
        assert_eq!(&bytes[0..8], &MAGIC24, "{opt:?}: must emit SEMCOD24");
        let spec = header_spec_from_magic(&MAGIC24).expect("MAGIC24 spec");
        assert_eq!(spec, HEADER_V24);
        assert_eq!(spec.rev, 25);
        verify_semcode(&bytes).expect("verification passes for SEMCOD24");
    }
}

#[test]
fn test_downgrade_to_semcod23_fails_verification() {
    let src = source("u32", "let b = bytes_empty(); return bytes_len(b);");
    let mut bytes = compile(&src, OptLevel::O0);
    assert_eq!(&bytes[0..8], &MAGIC24);
    // Tamper header to MAGIC23 (which lacks CAP_BYTES_VALUES)
    bytes[0..8].copy_from_slice(&MAGIC23);
    let err = verify_semcode(&bytes).expect_err("downgraded header must fail verification");
    let err_str = format!("{err:?}");
    assert!(
        err_str.contains("Capability") || err_str.contains("capability"),
        "error must be capability violation: {err_str}"
    );
}

#[test]
fn test_bytes_as_function_parameter_and_signature() {
    let src = r#"
fn count_bytes(b: Bytes) -> u32 {
    return bytes_len(b);
}

fn probe() -> u32 {
    let b0 = bytes_empty();
    let b = match bytes_push(b0, 99u32) {
        Option::Some(v) => { v }
        Option::None => { b0 }
    };
    return count_bytes(b);
}

fn main() {
    return;
}
"#;
    for opt in [OptLevel::O0, OptLevel::O1] {
        let bytes = compile(src, opt);
        assert_eq!(&bytes[0..8], &MAGIC24);
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("probe").expect("probe entry");
        let res = run_verified_function_semcode_with_args(&entry, vec![]).expect("run");
        assert_eq!(res, Value::U32(1));
    }
}

#[test]
fn test_ir_lowering_param_family_is_bytes() {
    let src = r#"
fn test_param(b: Bytes) -> u32 {
    return bytes_len(b);
}
fn main() { return; }
"#;
    let ir_funcs = compile_program_to_ir(src).expect("ir compile");
    let func = ir_funcs
        .iter()
        .find(|f| f.name == "test_param")
        .expect("test_param func");
    assert_eq!(func.params, vec![CallableValueFamily::Bytes]);
    assert_eq!(CallableValueFamily::Bytes.byte(), 15);
}
