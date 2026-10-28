use sm_emit::{compile_program_to_semcode_with_options, CompileProfile, OptLevel};
use sm_runtime_core::{ExecutionConfig, ExecutionContext, QuotaKind, RuntimeQuotas};
use sm_verify::{verify_semcode_token, verify_semcode_token_with_quotas};
use sm_vm::{
    run_verified_function_semcode_with_args, run_verified_function_semcode_with_args_and_config,
    RuntimeError, Value,
};

#[test]
fn structured_and_binary_roundtrips() {
    let source = include_str!("fixtures/shf_v1/structured.sm");
    for opt in [OptLevel::O0, OptLevel::O1] {
        let compile = || {
            compile_program_to_semcode_with_options(source, CompileProfile::RustLike, opt)
                .expect("compile")
        };
        let bytes = compile();
        assert_eq!(bytes, compile());
        let verified = verify_semcode_token(&bytes).expect("verify");
        let run = |name: &str, args: Vec<Value>| {
            run_verified_function_semcode_with_args(
                &verified.require_entry(name).expect("entry"),
                args,
            )
            .expect("run")
        };
        for (n, expected) in [
            (-1, -1),
            (0, 0),
            (1, 0),
            (2, 3),
            (8, 84),
            (32, 1488),
            (64, 6048),
            (65, -1),
            (i32::MAX, -1),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    run("structured", vec![Value::I32(n)]),
                    Value::I32(expected),
                    "n={n}, {opt:?}"
                );
            }
        }
        for v in [0, 1, 0x01020304, u32::MAX] {
            for _ in 0..2 {
                assert_eq!(run("byte_roundtrip", vec![Value::U32(v)]), Value::U32(v));
            }
        }
        assert_eq!(run("byte_checks", vec![]), Value::U32(1));
    }
}

#[test]
fn structured_stops_at_step_budget() {
    let source = include_str!("fixtures/shf_v1/structured.sm");
    for opt in [OptLevel::O0, OptLevel::O1] {
        let bytes = compile_program_to_semcode_with_options(source, CompileProfile::RustLike, opt)
            .expect("compile");
        let quotas = RuntimeQuotas {
            max_steps: 1,
            ..RuntimeQuotas::verified_local()
        };
        let token = verify_semcode_token_with_quotas(&bytes, quotas).expect("verify");
        let entry = token.require_entry("structured").expect("entry");
        for _ in 0..2 {
            let err = run_verified_function_semcode_with_args_and_config(
                &entry,
                vec![Value::I32(64)],
                ExecutionConfig::new(ExecutionContext::VerifiedLocal, quotas),
            )
            .expect_err("step budget must stop execution");
            assert!(
                matches!(err, RuntimeError::QuotaExceeded(exceeded) if exceeded.kind == QuotaKind::Steps),
                "{opt:?}: {err:?}"
            );
        }
    }
}
