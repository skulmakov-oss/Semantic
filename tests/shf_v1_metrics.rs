//! Actual VM counters for verified, pure SHF-v1 workloads; run with --nocapture.
use prom_abi::{AbiError, ApplicationHostAbi};
use prom_cap::{ApplicationCapabilityProfile, CapabilityManifest};
use sm_emit::{compile_program_to_semcode_with_options, CompileProfile, OptLevel};
use sm_format::sha256::sha256_hex;
use sm_runtime_core::{ExecutionConfig, ExecutionContext, QuotaKind, RuntimeQuotas};
use sm_verify::verify_semcode_token;
use sm_vm::arg_policy::{ArgBudgets, ArgPolicy, ArgResource};
use sm_vm::{
    run_verified_entry_semcode_with_application_host_and_capabilities_and_config_metered,
    run_verified_function_semcode_with_args_and_policy_events, ExecutionCounters, RuntimeError,
    Value,
};

// The workload is pure; any attempted host dispatch is an unexpected test failure.
struct NoHost;
impl ApplicationHostAbi for NoHost {
    fn args_read(&mut self, _: u32) -> Result<String, AbiError> {
        panic!("host call")
    }
    fn stdin_read_text(&mut self) -> Result<String, AbiError> {
        panic!("host call")
    }
    fn stdout_write(&mut self, _: &str) -> Result<(), AbiError> {
        panic!("host call")
    }
    fn stderr_write(&mut self, _: &str) -> Result<(), AbiError> {
        panic!("host call")
    }
    fn path_inspect(&mut self, _: &str) -> Result<bool, AbiError> {
        panic!("host call")
    }
    fn fs_read_text(&mut self, _: &str) -> Result<String, AbiError> {
        panic!("host call")
    }
    fn fs_write_text(&mut self, _: &str, _: &str) -> Result<(), AbiError> {
        panic!("host call")
    }
    fn time_duration_millis(&mut self) -> Result<u32, AbiError> {
        panic!("host call")
    }
}

const MAIN: &str = "fn main() { return; }";
const LEXER: &str = include_str!("fixtures/shf_v1/lexer.sm");
const FRONTEND: &str = include_str!("fixtures/shf_v1/frontend.sm");
const STRUCTURED: &str = include_str!("fixtures/shf_v1/structured.sm");
const EVIDENCE: &str = include_str!("fixtures/shf_v1/evidence.sm");

fn source_with_main(source: &str, body: &str) -> String {
    assert_eq!(source.matches(MAIN).count(), 1, "fixture main drift");
    source.replacen(MAIN, &format!("fn main() {{ {body} return; }}"), 1)
}

fn measure(source: &str, opt: OptLevel, label: &str) -> ExecutionCounters {
    let bytes = compile_program_to_semcode_with_options(source, CompileProfile::RustLike, opt)
        .expect("compile workload");
    let token = verify_semcode_token(&bytes).expect("verify workload");
    let entry = token.require_entry("main").expect("main");
    let caps = CapabilityManifest::for_application_profile(ApplicationCapabilityProfile::Pure);
    let config = ExecutionConfig::for_context(ExecutionContext::VerifiedLocal);
    let run = || {
        let result =
            run_verified_entry_semcode_with_application_host_and_capabilities_and_config_metered(
                &entry,
                &mut NoHost,
                &caps,
                config,
            );
        result.result.expect("guest assertions and execution");
        result.counters.expect("VM started")
    };
    let counters = run();
    assert_eq!(counters, run(), "{label} {opt:?}: repeat");
    assert!(counters.steps > 0 && counters.calls > 0 && counters.peak_frames > 0);
    assert_eq!(counters.effect_calls, 0);
    println!(
        "{label} {opt:?} steps={} calls={} peak_frames={} source_sha256={} semcode_sha256={}",
        counters.steps,
        counters.calls,
        counters.peak_frames,
        sha256_hex(source.as_bytes()),
        sha256_hex(&bytes)
    );
    counters
}

#[test]
fn measured_shf_v1_workloads() {
    let frontend = format!("{LEXER}\n{FRONTEND}");
    for opt in [OptLevel::O0, OptLevel::O1] {
        for size in [1, 16, 64] {
            let input = "a".repeat(size);
            let body = format!("let t: Token = next_token(\"{input}\", 0u32); assert(t.kind == 1u32); assert(t.start == 0u32); assert(t.end == {size}u32);");
            measure(
                &source_with_main(LEXER, &body),
                opt,
                &format!("scanner bytes={size}"),
            );
        }
        for terms in [1, 8, 30] {
            let input = format!("let x={}1;", "1+".repeat(terms - 1));
            let body = format!("assert(frontend_nodes(\"{input}\") == {});", terms * 2);
            measure(
                &source_with_main(&frontend, &body),
                opt,
                &format!("frontend terms={terms} bytes={}", input.len()),
            );
        }
        measure(
            &source_with_main(
                EVIDENCE,
                "assert(scenario(0u32) == 3u32); assert(quarantine_only() == 0u32);",
            ),
            opt,
            "evidence",
        );
        for (size, expected) in [(1, 0), (8, 84), (32, 1488), (64, 6048)] {
            let body = format!("assert(structured({size}) == {expected});");
            measure(
                &source_with_main(STRUCTURED, &body),
                opt,
                &format!("structured nodes={size}"),
            );
        }
    }
}

#[test]
fn malformed_artifact_and_denied_host_call_are_rejected() {
    for opt in [OptLevel::O0, OptLevel::O1] {
        let bytes = compile_program_to_semcode_with_options(LEXER, CompileProfile::RustLike, opt)
            .expect("compile");
        let mut damaged = bytes.clone();
        damaged[0] ^= 0xff;
        assert!(
            verify_semcode_token(&damaged).is_err(),
            "corrupt magic must be rejected"
        );

        let source = source_with_main(LEXER, "let x: text = args_read(0u32);");
        let effect =
            compile_program_to_semcode_with_options(&source, CompileProfile::RustLike, opt)
                .expect("compile effect control");
        let token = verify_semcode_token(&effect).expect("verify effect control");
        let entry = token.require_entry("main").expect("main");
        let caps = CapabilityManifest::for_application_profile(ApplicationCapabilityProfile::Pure);
        let run =
            run_verified_entry_semcode_with_application_host_and_capabilities_and_config_metered(
                &entry,
                &mut NoHost,
                &caps,
                ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
            );
        assert!(
            matches!(&run.result, Err(RuntimeError::CapabilityDenied(_))),
            "{:?}",
            run.result
        );
        assert_eq!(run.counters.expect("VM started").effect_calls, 0);
    }
}

#[test]
fn structured_arg_policies_are_deterministic_and_bounded() {
    let quotas = RuntimeQuotas::verified_local();
    let maximum = ArgBudgets {
        steps: quotas.max_steps as u64,
        calls: quotas.max_calls as u64,
        frames: quotas.max_frames as u64,
        stack_depth: quotas.max_stack_depth as u64,
    };
    let adaptive = ArgPolicy::custom(
        ArgBudgets {
            steps: 16,
            calls: 1,
            frames: 1,
            stack_depth: 1,
        },
        maximum,
        true,
        1,
    )
    .expect("adaptive policy");
    let insufficient = ArgBudgets {
        steps: 1,
        ..maximum
    };
    let fixed_small =
        ArgPolicy::custom(insufficient, insufficient, false, 0).expect("fixed policy");
    let adaptive_small = ArgPolicy::custom(
        ArgBudgets {
            steps: 16,
            ..maximum
        },
        ArgBudgets {
            steps: 32,
            ..maximum
        },
        true,
        1,
    )
    .expect("bounded adaptive policy");

    for opt in [OptLevel::O0, OptLevel::O1] {
        let bytes =
            compile_program_to_semcode_with_options(STRUCTURED, CompileProfile::RustLike, opt)
                .expect("compile structured");
        let token = verify_semcode_token(&bytes).expect("verify structured");
        let entry = token.require_entry("structured").expect("structured entry");
        let run = |policy| {
            run_verified_function_semcode_with_args_and_policy_events(
                &entry,
                vec![Value::I32(8)],
                policy,
            )
        };
        let (fixed_result, fixed_events) = run(ArgPolicy::god()).expect("god policy");
        assert_eq!(fixed_result, Value::I32(84));
        assert!(fixed_events.is_empty());
        let (result, events) = run(adaptive).expect("adaptive policy execution");
        let (repeated_result, repeated_events) = run(adaptive).expect("adaptive repeat");
        assert_eq!(result, fixed_result);
        assert_eq!(repeated_result, fixed_result);
        assert_eq!(events, repeated_events, "{opt:?}: adaptive decisions");
        assert!(events.len() >= 2, "{opt:?}: {events:?}");
        for event in &events {
            let hard_max = match event.resource {
                ArgResource::Steps => maximum.steps,
                ArgResource::Calls => maximum.calls,
                ArgResource::Frames => maximum.frames,
                ArgResource::StackDepth => maximum.stack_depth,
                ArgResource::EffectCalls => quotas.max_effect_calls as u64,
            };
            assert!(event.budget_after <= hard_max, "{opt:?}: {event:?}");
        }
        let adaptive_error = run(adaptive_small).expect_err("adaptive steps must exhaust");
        let repeated_error = run(adaptive_small).expect_err("adaptive steps must exhaust again");
        assert!(
            matches!(&adaptive_error, RuntimeError::QuotaExceeded(exceeded) if exceeded.kind == QuotaKind::Steps),
            "{opt:?}: {adaptive_error:?}"
        );
        assert_eq!(
            adaptive_error, repeated_error,
            "{opt:?}: adaptive exhaustion"
        );
        let error = run(fixed_small).expect_err("one step must exhaust");
        assert!(
            matches!(&error, RuntimeError::QuotaExceeded(exceeded) if exceeded.kind == QuotaKind::Steps),
            "{opt:?}: {error:?}"
        );
    }
}
