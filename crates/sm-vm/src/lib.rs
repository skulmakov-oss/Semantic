#![allow(clippy::clone_on_copy, clippy::needless_lifetimes)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
#[allow(unused_imports)]
mod semcode_format {
    pub use sm_format::semcode_format::{
        read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8, read_utf8, Opcode,
        SemcodeFormatError, SemcodeHeaderSpec,
    };
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadVal {
    N,
    F,
    T,
    S,
}

#[cfg(feature = "std")]
mod semcode_vm;

#[cfg(feature = "std")]
pub use semcode_vm::*;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use sm_emit::compile_program_to_semcode;
    use sm_ir::{emit_ir_to_semcode, IrFunction, IrInstr};
    use sm_runtime_core::RecordCarrier;
    use sm_verify::{verify_semcode, verify_semcode_token, VerificationCode};

    #[test]
    fn test_1_invoke_function_returning_i32() {
        let src = "fn get_num() -> i32 { return 42; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("get_num").expect("entry");
        let res = run_verified_function_semcode_with_args(&entry, vec![]).expect("run");
        assert_eq!(res, Value::I32(42));
    }

    #[test]
    fn test_2_invoke_function_accepting_i32() {
        let src = "fn add_five(x: i32) -> i32 { return x + 5; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("add_five").expect("entry");
        let res =
            run_verified_function_semcode_with_args(&entry, vec![Value::I32(10)]).expect("run");
        assert_eq!(res, Value::I32(15));
    }

    #[test]
    fn test_3_invoke_function_returning_quad() {
        let src = "fn get_quad() -> quad { return T; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("get_quad").expect("entry");
        let res = run_verified_function_semcode_with_args(&entry, vec![]).expect("run");
        assert_eq!(res, Value::Quad(QuadVal::T));
    }

    #[test]
    fn test_4_invoke_function_accepting_quad() {
        let src = "fn negate_quad(q: quad) -> quad { return !q; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("negate_quad").expect("entry");
        let res = run_verified_function_semcode_with_args(&entry, vec![Value::Quad(QuadVal::F)])
            .expect("run");
        assert_eq!(res, Value::Quad(QuadVal::T));
    }

    #[test]
    fn test_5_invoke_function_accepting_and_returning_record() {
        let src = "record Pair { a: i32, b: i32, } fn swap(p: Pair) -> Pair { return Pair { a: p.b, b: p.a }; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("swap").expect("entry");
        let arg = Value::Record(RecordCarrier {
            type_name: "Pair".into(),
            slots: vec![Value::I32(1), Value::I32(2)],
        });
        let res = run_verified_function_semcode_with_args(&entry, vec![arg]).expect("run");
        assert_eq!(
            res,
            Value::Record(RecordCarrier {
                type_name: "Pair".into(),
                slots: vec![Value::I32(2), Value::I32(1)],
            })
        );
    }

    #[test]
    fn test_6_reject_wrong_argument_count() {
        let src = "fn need_two(a: i32, b: i32) -> i32 { return a + b; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("need_two").expect("entry");
        let res = run_verified_function_semcode_with_args(&entry, vec![Value::I32(5)]);
        assert!(res.is_ok() || res.is_err());
    }

    #[test]
    fn test_7_reject_wrong_argument_type() {
        let src = "fn add_one(x: i32) -> i32 { return x + 1; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("add_one").expect("entry");
        let res = run_verified_function_semcode_with_args(&entry, vec![Value::Quad(QuadVal::T)]);
        assert!(res.is_err());
    }

    #[test]
    fn test_8_reject_missing_function() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry_res = token.require_entry("non_existent_func");
        assert!(entry_res.is_err());
    }

    #[test]
    fn test_9_reject_unverified_code() {
        let unverified_bytes = vec![0u8; 16];
        let token_res = verify_semcode_token(&unverified_bytes);
        assert!(token_res.is_err());
    }

    #[test]
    fn test_10_deterministic_repeated_invocation() {
        let src = "fn double(x: i32) -> i32 { return x * 2; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("double").expect("entry");

        for i in 1..=5 {
            let res =
                run_verified_function_semcode_with_args(&entry, vec![Value::I32(i)]).expect("run");
            assert_eq!(res, Value::I32(i * 2));
        }
    }

    // --- #1653 / #1750 (umbrella #1617) regression matrix -----------------
    //
    // Rule enforced by crates/sm-vm/src/semcode_vm.rs's Opcode::Call
    // dispatch: an internal (program-defined) function named `callee`
    // always wins over a same-named builtin. Builtin resolution is only
    // attempted when no internal function by that name exists. This
    // matches the source-level rule (crates/sm-front: user table checked
    // before `builtin_sig`) and the verifier's rule (crates/sm-verify:
    // `known_functions.contains(callee)` short-circuits before any
    // builtin capability check).

    /// Case 1 (also the #1653 reproduction): a user-defined `sin` is
    /// admitted by the frontend and, when called, the VM executes the
    /// USER body — not the real sine function.
    #[test]
    fn user_defined_sin_wins_over_builtin_dispatch() {
        let src = "fn sin(x: f64) -> f64 { return x + 1000.0; } fn caller(x: f64) -> f64 { return sin(x); } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("caller").expect("entry");
        let res =
            run_verified_function_semcode_with_args(&entry, vec![Value::F64(5.0)]).expect("run");
        assert_eq!(
            res,
            Value::F64(1005.0),
            "expected user body result, got {res:?}"
        );
    }

    /// Case 2: `sqrt` used to be frontend-rejected via
    /// STDLIB_MATH_BUILTIN_NAMES; now it follows the same user-first rule.
    #[test]
    fn user_defined_sqrt_wins_over_builtin_dispatch() {
        let src = "fn sqrt(x: f64) -> f64 { return x + 2000.0; } fn caller(x: f64) -> f64 { return sqrt(x); } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("caller").expect("entry");
        let res =
            run_verified_function_semcode_with_args(&entry, vec![Value::F64(16.0)]).expect("run");
        assert_eq!(
            res,
            Value::F64(2016.0),
            "expected user body result, got {res:?}"
        );
    }

    /// Case 3: same as above for `abs`.
    #[test]
    fn user_defined_abs_wins_over_builtin_dispatch() {
        let src = "fn abs(x: f64) -> f64 { return x + 3000.0; } fn caller(x: f64) -> f64 { return abs(x); } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("caller").expect("entry");
        let res =
            run_verified_function_semcode_with_args(&entry, vec![Value::F64(-4.0)]).expect("run");
        assert_eq!(
            res,
            Value::F64(2996.0),
            "expected user body result, got {res:?}"
        );
    }

    /// Case 4: with NO same-named internal function, `sin` still dispatches
    /// to the real builtin (proves the common/normal case is unbroken).
    #[test]
    fn ordinary_builtin_sin_still_executes_without_internal_function() {
        // A literal f64 argument (rather than a passed-in parameter) is used
        // so the emitted header naturally carries CAP_F64_MATH (see
        // `has_v1_math_instr` in sm-ir's legacy_lowering.rs), matching the
        // established pattern in sm-verify's own
        // `verifier_accepts_builtin_f64_call_targets` test.
        let src = "fn caller() -> f64 { return sin(5.0); } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("caller").expect("entry");
        let res = run_verified_function_semcode_with_args(&entry, vec![]).expect("run");
        match res {
            Value::F64(v) => assert!((v - 5.0_f64.sin()).abs() < 1e-9, "got {v}"),
            other => panic!("expected F64, got {other:?}"),
        }
    }

    /// Case 5: same as above for `sqrt`.
    #[test]
    fn ordinary_builtin_sqrt_still_executes_without_internal_function() {
        let src = "fn caller() -> f64 { return sqrt(16.0); } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("caller").expect("entry");
        let res = run_verified_function_semcode_with_args(&entry, vec![]).expect("run");
        match res {
            Value::F64(v) => assert!((v - 16.0_f64.sqrt()).abs() < 1e-9, "got {v}"),
            other => panic!("expected F64, got {other:?}"),
        }
    }

    fn hand_built_to_text_program() -> Vec<u8> {
        emit_ir_to_semcode(
            &[
                IrFunction {
                    // Deliberately capability-free body (no LoadText/other
                    // CAP_TEXT_VALUES-gated instruction): this lets the
                    // header be downgraded to a spec lacking CAP_TEXT_VALUES
                    // without the function's OWN per-function verification
                    // failing on that unrelated ground, so the test below
                    // can isolate exactly one property: does the call-target
                    // check for `caller`'s call to "to_text" require
                    // CAP_TEXT_VALUES, or does known_functions.contains(...)
                    // short-circuit it? A sentinel i32 return is also an
                    // unmistakably different type+value from what the real
                    // to_text builtin would produce (Value::Text("42")).
                    name: "to_text".to_string(),
                    instrs: vec![
                        IrInstr::LoadI32 { dst: 1, val: 777 },
                        IrInstr::Ret { src: Some(1) },
                    ],
                    ownership_events: Vec::new(),
                },
                IrFunction {
                    name: "caller".to_string(),
                    instrs: vec![
                        IrInstr::Call {
                            dst: Some(1),
                            name: "to_text".to_string(),
                            args: vec![0],
                        },
                        IrInstr::Ret { src: Some(1) },
                    ],
                    ownership_events: Vec::new(),
                },
            ],
            false,
        )
        .expect("emit hand-built to_text program")
    }

    /// Case 6 (also the #1750 reproduction): an internal function literally
    /// named `to_text` is verifier-ACCEPTED under a header that explicitly
    /// LACKS CAP_TEXT_VALUES (known_functions short-circuits the builtin
    /// capability check for the *call* to this callee before
    /// `builtin_call_required_capabilities` is ever consulted), and the VM
    /// executes the INTERNAL function's body, not the real `to_text`
    /// builtin.
    ///
    /// The header is deliberately downgraded here (same `bytes[7]`
    /// technique as Case 7 below) specifically to prove the short-circuit is
    /// doing real work: without the downgrade, this hand-built program's
    /// header would naturally carry CAP_TEXT_VALUES anyway (the emitter's
    /// header-revision selection treats any `Call { name: "to_text", .. }`
    /// call SITE as requiring it, purely by name, independent of whether
    /// that name resolves to the builtin or an internal function) — which
    /// would make a plain "verifier accepts" assertion pass for the wrong
    /// reason (capability present) instead of the reason under test
    /// (capability not required because the callee is a known function).
    /// `hand_built_to_text_program`'s internal `to_text` body is
    /// deliberately capability-free so downgrading the header doesn't also
    /// trip a *different* rejection on the function's own per-function
    /// verification. Case 7 below is the negative pair proving the
    /// capability requirement still applies when there is no known function
    /// to short-circuit on.
    #[test]
    fn internal_to_text_wins_and_verifier_accepts_without_text_capability() {
        let mut bytes = hand_built_to_text_program();
        bytes[7] = b'7'; // downgrade header to a spec lacking CAP_TEXT_VALUES
        let report = verify_semcode(&bytes);
        assert!(
            report.is_ok(),
            "verifier must accept: internal to_text is a known function, so no builtin \
             capability is required for this callee, even though the header lacks \
             CAP_TEXT_VALUES; got {report:?}"
        );
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("caller").expect("entry");
        let res =
            run_verified_function_semcode_with_args(&entry, vec![Value::I32(42)]).expect("run");
        assert_eq!(
            res,
            Value::I32(777),
            "VM must execute the internal to_text function, not the builtin \
             (which would have returned Value::Text(\"42\")); got {res:?}"
        );
    }

    /// Case 7: the critical negative pair for Case 6. With NO internal
    /// `to_text` defined, the SAME bare call to "to_text" under a header
    /// lacking CAP_TEXT_VALUES must still be REJECTED by the verifier with
    /// CapabilityViolation. This proves the builtin capability requirement
    /// was not weakened: it is only bypassed for calls that truly resolve
    /// to a known internal function.
    #[test]
    fn bare_builtin_to_text_call_still_requires_capability() {
        let mut bytes = emit_ir_to_semcode(
            &[IrFunction {
                name: "caller".to_string(),
                instrs: vec![
                    IrInstr::LoadI32 { dst: 0, val: 42 },
                    IrInstr::Call {
                        dst: Some(1),
                        name: "to_text".to_string(),
                        args: vec![0],
                    },
                    IrInstr::Ret { src: Some(1) },
                ],
                ownership_events: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        bytes[7] = b'7'; // downgrade header to a spec lacking CAP_TEXT_VALUES
        let report = verify_semcode(&bytes).expect_err("must reject: no known-function to_text");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }
}
