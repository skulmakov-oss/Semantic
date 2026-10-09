//! SHF-2B U1: `semantic.compiler.endian/0.1`, source → verified token → VM.
//! Literal vectors are independent of the host encoding functions under test.

use sm_emit::{compile_program_to_semcode_with_options, CompileProfile, OptLevel};
use sm_format::semcode_format::{header_spec_from_magic, MAGIC22, MAGIC23, MAGIC24};
use sm_ir::{emit_ir_to_semcode, IrFunction, IrInstr};
use sm_runtime_core::{AdtCarrier, RuntimeTrap};
use sm_verify::{verify_semcode, verify_semcode_token, VerificationCode};
use sm_vm::{run_verified_function_semcode_with_args, RuntimeError, Value};

const GOLDEN: &str = include_str!("fixtures/shf2b/endian_golden.sm");
const HELPERS: [&str; 6] = [
    "write_u16_le",
    "write_u32_le",
    "write_i32_le",
    "read_u16_le",
    "read_u32_le",
    "read_i32_le",
];

fn source(ret: &str, body: &str) -> String {
    format!("fn probe() -> {ret} {{\n{body}\n}}\nfn main() {{ return; }}\n")
}

fn compile(src: &str, opt: OptLevel) -> Vec<u8> {
    compile_program_to_semcode_with_options(src, CompileProfile::RustLike, opt)
        .unwrap_or_else(|e| panic!("compile ({opt:?}):\n{src}\n{e}"))
}

fn run(bytes: &[u8]) -> Result<Value, RuntimeError> {
    let token = verify_semcode_token(bytes).expect("verifier admission");
    let entry = token.require_entry("probe").expect("probe entry");
    run_verified_function_semcode_with_args(&entry, vec![])
}

fn outcome(src: &str) -> Result<Value, RuntimeError> {
    let mut results = Vec::new();
    for opt in [OptLevel::O0, OptLevel::O1] {
        let bytes = compile(src, opt);
        assert_eq!(bytes, compile(src, opt), "artifact determinism ({opt:?})");
        let result = run(&bytes);
        assert_eq!(
            format!("{result:?}"),
            format!("{:?}", run(&bytes)),
            "execution determinism"
        );
        results.push(result);
    }
    assert_eq!(
        format!("{:?}", results[0]),
        format!("{:?}", results[1]),
        "O0/O1: {src}"
    );
    results.remove(0)
}

fn option(value: Option<Value>) -> Value {
    Value::Adt(AdtCarrier {
        type_name: "Option".to_string(),
        variant_name: if value.is_some() { "Some" } else { "None" }.to_string(),
        tag: u16::from(value.is_some()),
        payload: value.into_iter().collect(),
    })
}

fn buffer(octets: &[u8]) -> String {
    let mut expr = "bytes_empty()".to_string();
    for byte in octets {
        expr = format!("match bytes_push({expr}, {byte}u32) {{ Option::Some(v) => {{ v }} Option::None => {{ assert(false); bytes_empty() }} }}");
    }
    expr
}

#[test]
fn writes_exact_little_endian_vectors_without_truncation() {
    for (expr, expected) in [
        (
            "write_u16_le(0x1234u32)",
            option(Some(Value::Bytes(vec![0x34, 0x12]))),
        ),
        ("write_u16_le(0u32)", option(Some(Value::Bytes(vec![0, 0])))),
        (
            "write_u16_le(65535u32)",
            option(Some(Value::Bytes(vec![255, 255]))),
        ),
        ("write_u16_le(65536u32)", option(None)),
        ("write_u16_le(4294967295u32)", option(None)),
    ] {
        assert_eq!(
            outcome(&source("Option(Bytes)", &format!("return {expr};"))).unwrap(),
            expected,
            "{expr}"
        );
    }
    for (expr, octets) in [
        ("write_u32_le(0x01020304u32)", [4, 3, 2, 1]),
        ("write_u32_le(0u32)", [0, 0, 0, 0]),
        ("write_u32_le(4294967295u32)", [255, 255, 255, 255]),
        ("write_i32_le(-1)", [255, 255, 255, 255]),
        ("write_i32_le(-2147483647 - 1)", [0, 0, 0, 128]),
        ("write_i32_le(2147483647)", [255, 255, 255, 127]),
        ("write_i32_le(0)", [0, 0, 0, 0]),
        ("write_i32_le(-16909060)", [252, 252, 253, 254]),
    ] {
        assert_eq!(
            outcome(&source("Bytes", &format!("return {expr};"))).unwrap(),
            Value::Bytes(octets.to_vec()),
            "{expr}"
        );
    }
}

#[test]
fn reads_independent_octets_with_exact_signed_and_unsigned_values() {
    for (name, octets, expected) in [
        ("read_u16_le", vec![0x34, 0x12], Value::U32(0x1234)),
        ("read_u16_le", vec![0, 0], Value::U32(0)),
        ("read_u16_le", vec![255, 255], Value::U32(65535)),
        ("read_u32_le", vec![4, 3, 2, 1], Value::U32(0x01020304)),
        ("read_u32_le", vec![0, 0, 0, 0], Value::U32(0)),
        (
            "read_u32_le",
            vec![255, 255, 255, 255],
            Value::U32(u32::MAX),
        ),
        ("read_i32_le", vec![255, 255, 255, 255], Value::I32(-1)),
        ("read_i32_le", vec![0, 0, 0, 128], Value::I32(i32::MIN)),
        (
            "read_i32_le",
            vec![255, 255, 255, 127],
            Value::I32(i32::MAX),
        ),
        (
            "read_i32_le",
            vec![252, 252, 253, 254],
            Value::I32(-16909060),
        ),
    ] {
        let ret = if name == "read_i32_le" {
            "Option(i32)"
        } else {
            "Option(u32)"
        };
        // Exact width, then an unaligned offset and trailing bytes; reads cannot swap or consume the tail.
        for (bytes, offset) in [
            (octets.clone(), 0),
            ([vec![0x80], octets, vec![0xff, 0x7f]].concat(), 1),
        ] {
            let body = format!("return {name}({}, {offset}u32);", buffer(&bytes));
            assert_eq!(
                outcome(&source(ret, &body)).unwrap(),
                option(Some(expected.clone())),
                "{name} {bytes:?} at {offset}"
            );
        }
    }
}

#[test]
fn empty_truncated_out_of_bounds_and_overflow_reads_are_none() {
    for name in ["read_u16_le", "read_u32_le", "read_i32_le"] {
        let width = if name == "read_u16_le" { 2 } else { 4 };
        let ret = if name == "read_i32_le" {
            "Option(i32)"
        } else {
            "Option(u32)"
        };
        for len in 0..width {
            let body = format!("return {name}({}, 0u32);", buffer(&vec![0xff; len]));
            assert_eq!(
                outcome(&source(ret, &body)).unwrap(),
                option(None),
                "{name}, truncated {len}"
            );
        }
        for offset in [1, 4, 5, u32::MAX - 3, u32::MAX - 1, u32::MAX] {
            let body = format!(
                "return {name}({}, {offset}u32);",
                buffer(&vec![0xff; width])
            );
            assert_eq!(
                outcome(&source(ret, &body)).unwrap(),
                option(None),
                "{name}, offset {offset}"
            );
        }
    }
}

#[test]
fn golden_source_builds_raw_octets_and_round_trips_every_helper() {
    assert_eq!(
        outcome(GOLDEN).unwrap(),
        Value::Bytes(vec![
            0, 1, 0x7f, 0x80, 0xff, 0xc0, 0xaf, 0x34, 0x12, 4, 3, 2, 1, 255, 255, 255, 255, 0, 0,
            0, 128, 255, 255, 255, 127,
        ])
    );
}

#[test]
fn reads_and_concatenated_writes_preserve_original_buffer() {
    let src = source(
        "Bytes",
        &format!(
            r#"
        let original = {};
        let decoded = read_u32_le(original, 0u32);
        assert(match decoded {{ Option::Some(v) => {{ v == 4286611200u32 }} Option::None => {{ false }} }});
        let extended = bytes_extend(original, write_i32_le(-1));
        assert(bytes_len(extended) == 8u32);
        return original;
    "#,
            buffer(&[0, 0x7f, 0x80, 0xff])
        ),
    );
    assert_eq!(
        outcome(&src).unwrap(),
        Value::Bytes(vec![0, 0x7f, 0x80, 0xff])
    );
}

#[test]
fn existing_argument_traps_are_identical_at_o0_and_o1() {
    for body in [
        "return write_u16_le(4294967295u32 + 1u32);",
        "return read_u16_le(bytes_empty(), 1u32 / 0u32);",
    ] {
        let ret = if body.contains("write_") {
            "Option(Bytes)"
        } else {
            "Option(u32)"
        };
        let expected = if body.contains("+") {
            RuntimeTrap::ArithmeticOverflow
        } else {
            RuntimeTrap::DivisionByZero
        };
        assert!(
            matches!(outcome(&source(ret, body)), Err(RuntimeError::Trap(trap)) if trap == expected)
        );
    }
}

#[test]
fn source_rejects_wrong_families_arity_and_named_arguments() {
    for (ret, call, message) in [
        ("Option(Bytes)", "write_u16_le(-1)", "must be u32"),
        ("Bytes", "write_u32_le(1)", "must be u32"),
        ("Bytes", "write_i32_le(1u32)", "must be i32"),
        ("Option(u32)", "read_u16_le(\"raw\", 0u32)", "must be Bytes"),
        (
            "Option(u32)",
            "read_u32_le(bytes_empty(), 0)",
            "must be u32",
        ),
        (
            "Option(i32)",
            "read_i32_le(bytes_empty(), 0)",
            "must be u32",
        ),
        ("Option(Bytes)", "write_u16_le()", "positional argument"),
        ("Bytes", "write_u32_le(1u32, 2u32)", "positional argument"),
        ("Bytes", "write_i32_le(value = 1)", "positional argument"),
        (
            "Option(u32)",
            "read_u32_le(bytes_empty())",
            "positional argument",
        ),
    ] {
        for opt in [OptLevel::O0, OptLevel::O1] {
            let error = compile_program_to_semcode_with_options(
                &source(ret, &format!("return {call};")),
                CompileProfile::RustLike,
                opt,
            )
            .expect_err(call);
            assert!(error.to_string().contains(message), "{call}: {error}");
        }
    }
}

#[test]
fn endian_names_cannot_be_self_shadowed() {
    for name in HELPERS {
        let src = format!("fn {name}() {{ return; }} fn main() {{ return; }}");
        assert!(
            compile_program_to_semcode_with_options(&src, CompileProfile::RustLike, OptLevel::O0)
                .is_err(),
            "reserved {name}"
        );
    }
}

fn bare_call(name: &str, load: Option<IrInstr>, args: Vec<u16>) -> Vec<u8> {
    let mut instrs: Vec<IrInstr> = load.into_iter().collect();
    instrs.push(IrInstr::Call {
        dst: Some(1),
        name: name.to_string(),
        args,
    });
    instrs.push(IrInstr::Ret { src: Some(1) });
    emit_ir_to_semcode(
        &[IrFunction {
            name: "probe".to_string(),
            instrs,
            params: vec![],
            ownership_events: vec![],
        }],
        false,
    )
    .expect("emit bare call")
}

#[test]
fn every_bare_endian_call_requires_new_authority_before_execution() {
    for name in HELPERS {
        // No Bytes parameter or core Bytes call: admission must gate the builtin itself.
        let bytes = bare_call(name, None, vec![]);
        assert_eq!(&bytes[..8], b"SEMCOD25", "{name}");
        let magic: [u8; 8] = bytes[..8].try_into().unwrap();
        let spec = header_spec_from_magic(&magic).expect("new header");
        assert_eq!(spec.rev, 26);
        assert_ne!(spec.capabilities & (1 << 29), 0);
        verify_semcode(&bytes).expect("structural admission precedes runtime type checking");
        for old in [MAGIC22, MAGIC23, MAGIC24] {
            let mut downgraded = bytes.clone();
            downgraded[..8].copy_from_slice(&old);
            let report =
                verify_semcode_token(&downgraded).expect_err("downgraded call must not execute");
            assert!(
                report
                    .diagnostics
                    .iter()
                    .any(|d| d.code == VerificationCode::CapabilityViolation),
                "{name}: {report:?}"
            );
        }
    }
    let legacy = compile(&source("Bytes", "return bytes_empty();"), OptLevel::O0);
    assert_eq!(
        &legacy[..8],
        &MAGIC24,
        "existing Bytes contract stays fixed"
    );
    assert_eq!(
        header_spec_from_magic(&MAGIC24).unwrap().capabilities & (1 << 29),
        0
    );
}

#[test]
fn internal_functions_with_endian_spellings_keep_their_previous_header_floor() {
    for name in HELPERS {
        // Bare IR permits internal names that source reserves. Internal targets win.
        let bytes = emit_ir_to_semcode(
            &[
                IrFunction {
                    name: name.to_string(),
                    instrs: vec![
                        IrInstr::LoadI32 { dst: 0, val: 7 },
                        IrInstr::Ret { src: Some(0) },
                    ],
                    params: vec![],
                    ownership_events: vec![],
                },
                IrFunction {
                    name: "probe".to_string(),
                    instrs: vec![
                        IrInstr::Call {
                            dst: Some(0),
                            name: name.to_string(),
                            args: vec![],
                        },
                        IrInstr::Ret { src: Some(0) },
                    ],
                    params: vec![],
                    ownership_events: vec![],
                },
            ],
            false,
        )
        .expect("emit internal function collision");
        assert_eq!(
            &bytes[..8],
            &MAGIC22,
            "{name} is an internal function, not an endian primitive"
        );
        assert_eq!(run(&bytes).unwrap(), Value::I32(7));
    }
}

#[test]
fn admitted_malformed_calls_still_reject_arity_and_runtime_types() {
    for name in HELPERS {
        assert!(
            matches!(
                run(&bare_call(name, None, vec![])),
                Err(RuntimeError::TypeMismatchRuntime(_))
            ),
            "{name} arity"
        );
        let load = if name == "write_i32_le" {
            IrInstr::LoadU32 { dst: 0, val: 1 }
        } else {
            IrInstr::LoadI32 { dst: 0, val: 1 }
        };
        let args = if name.starts_with("read_") {
            vec![0, 0]
        } else {
            vec![0]
        };
        assert!(
            matches!(
                run(&bare_call(name, Some(load), args)),
                Err(RuntimeError::TypeMismatchRuntime(_))
            ),
            "{name} type"
        );
    }
}

#[test]
fn admitted_reads_reject_wrong_offset_family_after_a_valid_bytes_argument() {
    for name in ["read_u16_le", "read_u32_le", "read_i32_le"] {
        let bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "probe".to_string(),
                instrs: vec![
                    IrInstr::LoadU32 { dst: 0, val: 0 },
                    IrInstr::Call {
                        dst: Some(1),
                        name: "write_u32_le".to_string(),
                        args: vec![0],
                    },
                    IrInstr::LoadI32 { dst: 2, val: 0 },
                    IrInstr::Call {
                        dst: Some(3),
                        name: name.to_string(),
                        args: vec![1, 2],
                    },
                    IrInstr::Ret { src: Some(3) },
                ],
                params: vec![],
                ownership_events: vec![],
            }],
            false,
        )
        .expect("emit malformed offset call");
        let error = run(&bytes).expect_err("i32 offset is not u32 even if nonnegative");
        assert!(
            matches!(error, RuntimeError::TypeMismatchRuntime(ref message) if message.contains("argument 2 expects u32")),
            "{name}: {error:?}"
        );
    }
}
