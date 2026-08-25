#![allow(clippy::clone_on_copy, clippy::needless_lifetimes)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
#[allow(unused_imports)]
mod semcode_format {
    pub use sm_format::semcode_format::{
        read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8, read_utf8, CallableSignature,
        CallableValueFamily, Opcode, SemcodeFormatError, SemcodeHeaderSpec, MAGIC7,
        SIGNATURE_SECTION_TAG,
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
    use crate::semcode_format::{CallableValueFamily, MAGIC7};
    use sm_emit::compile_program_to_semcode;
    use sm_ir::{emit_ir_to_semcode, IrFunction, IrInstr};
    use sm_runtime_core::RecordCarrier;
    use sm_verify::{verify_semcode, verify_semcode_token, VerificationCode};

    /// #1773 (FA-09-005): rebuilds `bytes` under `target_magic` (always a
    /// pre-V11 header in this file's use, below the mandatory-OWN0 floor),
    /// dropping every function's OWN0/SIG0 sections entirely - not just
    /// SIG0 - since a genuine artifact under a pre-V11 header carries
    /// neither. `sm-verify`'s own test module has an analogous helper that
    /// strips only SIG0, because its downgrade targets are all V11+ (where
    /// OWN0 is still expected); this file's downgrade targets are not, so
    /// keeping a real OWN0 here would trip the per-function
    /// CAP_OWNERSHIP_PATHS check for an unrelated reason. Assumes no DBG0
    /// section (none of this file's fixtures enable debug symbols).
    fn downgrade_header_stripping_signature(bytes: &[u8], target_magic: [u8; 8]) -> Vec<u8> {
        let (_, functions) = sm_format::semcode_decode::decode_semcode_envelope(bytes)
            .expect("decode current bytes");
        let mut out = Vec::new();
        out.extend_from_slice(&target_magic);
        for f in &functions {
            debug_assert!(!f.has_debug_section, "fixture must not use debug symbols");
            let mut code = Vec::new();
            code.extend_from_slice(&f.code_slice[..f.string_table_end_offset]);
            code.extend_from_slice(&f.code_slice[f.instr_start_offset..]);
            let name_bytes = f.name.as_bytes();
            out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            out.extend_from_slice(name_bytes);
            out.extend_from_slice(&(code.len() as u32).to_le_bytes());
            out.extend_from_slice(&code);
        }
        out
    }

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

    /// #1774 (FA-09-006): the original `test_6_reject_wrong_argument_count`
    /// ended with `assert!(res.is_ok() || res.is_err())`, which is true for
    /// every possible `Result` and therefore proves nothing about whether
    /// the wrong count was actually rejected. `b` is unused by the body on
    /// purpose: since #1773, `validate_call_arguments` rejects an argument-
    /// count mismatch inside `push_frame`, before the frame -- and therefore
    /// the body -- ever runs, with a distinct `TypeMismatchRuntime`.
    /// Mutation-tested by temporarily disabling that boundary check: `b`'s
    /// parameter binding still gets one unconditional `StoreVar` read of its
    /// argument register during lowering regardless of whether the body
    /// later references `b`, so execution does not silently succeed --
    /// instead it falls through to a materially worse, VM-internals-leaking
    /// `RuntimeError::BadFormat("read uninitialized reg r1")`. Asserting the
    /// specific `TypeMismatchRuntime` variant (not just `is_err()`) is what
    /// distinguishes the clean boundary rejection this test protects from
    /// that fallback failure mode.
    #[test]
    fn rejects_missing_unused_argument_at_invocation_boundary() {
        let src = "fn need_two(a: i32, b: i32) -> i32 { return a; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("need_two").expect("entry");
        let err = run_verified_function_semcode_with_args(&entry, vec![Value::I32(5)])
            .expect_err("a missing argument for an unused parameter must still be rejected");
        assert!(
            matches!(err, RuntimeError::TypeMismatchRuntime(_)),
            "expected a boundary argument-count rejection (TypeMismatchRuntime) from \
             validate_call_arguments, got {err:?}"
        );
    }

    /// #1774 (FA-09-006): the original `test_7_reject_wrong_argument_type`
    /// invoked `add_one(x: i32) { x + 1 }` with a `Quad` argument. `x` is
    /// used by `AddI32`, so that rejection only proves a downstream
    /// arithmetic opcode can reject a bad runtime shape -- not that the
    /// call boundary itself checks the declared parameter type. Here `x` is
    /// supplied correctly and `unused` is wrong-typed but never read by the
    /// body: if `validate_call_arguments`'s family check were removed, this
    /// call would never read the mistyped `unused` register and would
    /// silently *succeed* with `Value::I32(8)` (`x + 1`), so a failure here
    /// can only come from the invocation boundary, never from body
    /// semantics.
    #[test]
    fn rejects_wrong_unused_argument_family_at_invocation_boundary() {
        let src = "fn add_one(x: i32, unused: bool) -> i32 { return x + 1; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("add_one").expect("entry");
        let err = run_verified_function_semcode_with_args(
            &entry,
            vec![Value::I32(7), Value::Text("wrong".into())],
        )
        .expect_err(
            "a wrong-family argument in a parameter the body never reads must still be rejected",
        );
        assert!(
            matches!(err, RuntimeError::TypeMismatchRuntime(_)),
            "expected a boundary family rejection (TypeMismatchRuntime) from \
             validate_call_arguments, got {err:?}"
        );
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

    /// #1774 (FA-09-006): the original `test_10_deterministic_repeated_
    /// invocation` called `double(i)` for five *different* values of `i`,
    /// each exactly once. That proves `double(i) == 2*i` holds for five
    /// distinct inputs, not that repeated execution of the *same* verified
    /// SemCode/entry/argument state yields the same result every time --
    /// the actual claim `docs/spec/vm.md`'s Determinism Rule makes (same
    /// verified SemCode input, execution config, and entry function). This
    /// version holds the SemCode bytes, the resolved entry token, and the
    /// argument vector fixed and invokes the identical call ten times.
    /// `double` is a pure, host-effect-free function, matching the
    /// Determinism Rule's scope: it makes no claim about host-backed
    /// effects, wall-clock timing, or external scheduling.
    #[test]
    fn repeated_identical_invocation_is_deterministic() {
        let src = "fn double(x: i32) -> i32 { return x * 2; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("double").expect("entry");

        let results: Vec<Value> = (0..10)
            .map(|_| {
                run_verified_function_semcode_with_args(&entry, vec![Value::I32(21)]).expect("run")
            })
            .collect();
        assert!(
            results.iter().all(|r| *r == results[0]),
            "repeated invocation of identical SemCode/entry/argument state must yield an \
             identical result every time, got {results:?}"
        );
        assert_eq!(results[0], Value::I32(42));
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
                    params: vec![CallableValueFamily::I32],
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
                    params: vec![CallableValueFamily::I32],
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
        let bytes = hand_built_to_text_program();
        let bytes = downgrade_header_stripping_signature(&bytes, MAGIC7); // spec lacking CAP_TEXT_VALUES
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
        let bytes = emit_ir_to_semcode(
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
                params: Vec::new(),
            }],
            false,
        )
        .expect("emit");
        let bytes = downgrade_header_stripping_signature(&bytes, MAGIC7); // spec lacking CAP_TEXT_VALUES
        let report = verify_semcode(&bytes).expect_err("must reject: no known-function to_text");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::CapabilityViolation
        );
    }
}
