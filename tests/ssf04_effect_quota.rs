use prom_abi::{AbiError, AbiFailureKind, ApplicationHostAbi, HostCallId};
use prom_cap::{ApplicationCapabilityProfile, CapabilityManifest};
use sm_emit::compile_program_to_semcode;
use sm_runtime_core::{ExecutionConfig, ExecutionContext, QuotaKind, RuntimeQuotas};
use sm_vm::{run_verified_semcode_with_application_host_and_capabilities_and_config, RuntimeError};

const FLOW: &str = r#"
fn main() {
    let input_path: text = args_read(0u32);
    let output_path: text = args_read(1u32);
    let input: text = fs_read_text(input_path);
    fs_write_text(output_path, input + "!");
    return;
}
"#;

// Exercises all eight application-boundary builtins exactly once each, in the
// canonical read -> observation -> write -> time order.
const ALL_EIGHT_BUILTINS_FLOW: &str = r#"
fn main() {
    let name: text = args_read(0u32);
    let piped: text = stdin_read_text();
    stdout_write(name + piped);
    stderr_write(name);
    let exists: bool = path_inspect(name);
    let content: text = fs_read_text(name);
    fs_write_text(name, content);
    let millis: u32 = time_duration_ms();
    assert(exists == true);
    assert(millis == 7u32);
    return;
}
"#;

#[derive(Default)]
struct MemoryApplicationHost {
    args: Vec<String>,
    input: String,
    stdin: String,
    duration_millis: u32,
    calls: Vec<HostCallId>,
    writes: Vec<(String, String)>,
}

impl MemoryApplicationHost {
    fn unavailable(call: HostCallId) -> AbiError {
        AbiError::new(call, AbiFailureKind::Unavailable, "not configured")
    }
}

impl ApplicationHostAbi for MemoryApplicationHost {
    fn args_read(&mut self, index: u32) -> Result<String, AbiError> {
        self.calls.push(HostCallId::ArgsRead);
        self.args
            .get(index as usize)
            .cloned()
            .ok_or_else(|| Self::unavailable(HostCallId::ArgsRead))
    }

    fn stdin_read_text(&mut self) -> Result<String, AbiError> {
        self.calls.push(HostCallId::StdinReadText);
        Ok(self.stdin.clone())
    }

    fn stdout_write(&mut self, _text: &str) -> Result<(), AbiError> {
        self.calls.push(HostCallId::StdoutWrite);
        Ok(())
    }

    fn stderr_write(&mut self, _text: &str) -> Result<(), AbiError> {
        self.calls.push(HostCallId::StderrWrite);
        Ok(())
    }

    fn path_inspect(&mut self, _path: &str) -> Result<bool, AbiError> {
        self.calls.push(HostCallId::PathInspect);
        Ok(true)
    }

    fn fs_read_text(&mut self, _path: &str) -> Result<String, AbiError> {
        self.calls.push(HostCallId::FsRead);
        Ok(self.input.clone())
    }

    fn fs_write_text(&mut self, path: &str, text: &str) -> Result<(), AbiError> {
        self.calls.push(HostCallId::FsWrite);
        self.writes.push((path.to_string(), text.to_string()));
        Ok(())
    }

    fn time_duration_millis(&mut self) -> Result<u32, AbiError> {
        self.calls.push(HostCallId::TimeDuration);
        Ok(self.duration_millis)
    }
}

fn build_host() -> MemoryApplicationHost {
    MemoryApplicationHost {
        args: vec!["input.txt".to_string(), "output.txt".to_string()],
        input: "payload".to_string(),
        ..MemoryApplicationHost::default()
    }
}

fn run_flow_with_quota(
    host: &mut MemoryApplicationHost,
    max_effect_calls: usize,
) -> Result<(), RuntimeError> {
    let bytes = compile_program_to_semcode(FLOW).expect("compile");
    let quotas = RuntimeQuotas {
        max_effect_calls,
        ..RuntimeQuotas::verified_local()
    };
    run_verified_semcode_with_application_host_and_capabilities_and_config(
        &bytes,
        "main",
        host,
        &CapabilityManifest::for_application_profile(
            ApplicationCapabilityProfile::CliFileTransform,
        ),
        ExecutionConfig::new(ExecutionContext::VerifiedLocal, quotas),
    )
}

// FLOW performs exactly 4 admitted application host effects:
// args_read, args_read, fs_read_text, fs_write_text.

#[test]
fn admitted_application_host_effects_up_to_the_quota_all_pass() {
    let mut host = build_host();
    run_flow_with_quota(&mut host, 4).expect("4 admitted effects must fit a quota of 4");
    assert_eq!(
        host.writes,
        vec![("output.txt".to_string(), "payload!".to_string())]
    );
}

#[test]
fn the_fifth_admitted_effect_would_be_denied_but_the_fourth_stays_under_quota_of_four() {
    // Sanity boundary: one below the required count must already fail deterministically,
    // proving the quota is actually being enforced per-effect and not just at completion.
    let mut host = build_host();
    let error = run_flow_with_quota(&mut host, 3)
        .expect_err("3 admitted effects must exceed a 4-effect flow");
    let RuntimeError::QuotaExceeded(exceeded) = error else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(exceeded.kind, QuotaKind::EffectCalls);
    assert_eq!(exceeded.limit, 3);
    assert_eq!(exceeded.used, 4);
    // The quota must block the fourth host effect (fs_write_text) itself: no write observed.
    assert!(host.writes.is_empty());
    // Only the first three admitted effects (args_read, args_read, fs_read_text) executed.
    assert_eq!(
        host.calls,
        vec![
            HostCallId::ArgsRead,
            HostCallId::ArgsRead,
            HostCallId::FsRead
        ]
    );
}

#[test]
fn a_single_application_host_effect_exceeds_a_zero_quota_deterministically() {
    let mut host = build_host();
    let error =
        run_flow_with_quota(&mut host, 0).expect_err("zero quota must deny the first effect");
    let RuntimeError::QuotaExceeded(exceeded) = error else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(exceeded.kind, QuotaKind::EffectCalls);
    assert_eq!(exceeded.limit, 0);
    assert_eq!(exceeded.used, 1);
    assert!(host.calls.is_empty());
}

#[test]
fn capability_denied_application_builtins_are_not_charged_against_the_effect_quota() {
    // Pure denies every application builtin on the capability check, which runs
    // before the quota charge. A denied attempt must not consume any quota, and
    // the failure must surface as CapabilityDenied rather than QuotaExceeded even
    // though the configured quota (1) is otherwise tight enough to matter.
    let bytes = compile_program_to_semcode(FLOW).expect("compile");
    let mut host = build_host();
    let quotas = RuntimeQuotas {
        max_effect_calls: 1,
        ..RuntimeQuotas::verified_local()
    };
    let error = run_verified_semcode_with_application_host_and_capabilities_and_config(
        &bytes,
        "main",
        &mut host,
        &CapabilityManifest::for_application_profile(ApplicationCapabilityProfile::Pure),
        ExecutionConfig::new(ExecutionContext::VerifiedLocal, quotas),
    )
    .expect_err("Pure must deny the first application builtin on capability, not quota");
    assert!(
        matches!(error, RuntimeError::CapabilityDenied(_)),
        "unexpected error: {error:?}"
    );
    assert!(host.calls.is_empty());
}

fn run_all_eight_with_quota(
    host: &mut MemoryApplicationHost,
    max_effect_calls: usize,
) -> Result<(), RuntimeError> {
    let bytes = compile_program_to_semcode(ALL_EIGHT_BUILTINS_FLOW).expect("compile");
    let quotas = RuntimeQuotas {
        max_effect_calls,
        ..RuntimeQuotas::verified_local()
    };
    run_verified_semcode_with_application_host_and_capabilities_and_config(
        &bytes,
        "main",
        host,
        &CapabilityManifest::for_application_profile(
            ApplicationCapabilityProfile::CliFileTransform,
        ),
        ExecutionConfig::new(ExecutionContext::VerifiedLocal, quotas),
    )
}

#[test]
fn every_application_builtin_kind_is_charged_against_the_effect_quota() {
    // Covers read (args_read/fs_read_text), write (fs_write_text), stdout, stderr,
    // observation (path_inspect/stdin_read_text) and time paths in one flow: all
    // eight application builtins, one call each, exactly filling a quota of 8.
    let mut host = MemoryApplicationHost {
        args: vec!["input.txt".to_string()],
        input: "payload".to_string(),
        stdin: "piped".to_string(),
        duration_millis: 7,
        ..MemoryApplicationHost::default()
    };
    run_all_eight_with_quota(&mut host, 8).expect("8 admitted effects must fit a quota of 8");
    assert_eq!(
        host.calls,
        vec![
            HostCallId::ArgsRead,
            HostCallId::StdinReadText,
            HostCallId::StdoutWrite,
            HostCallId::StderrWrite,
            HostCallId::PathInspect,
            HostCallId::FsRead,
            HostCallId::FsWrite,
            HostCallId::TimeDuration,
        ]
    );
}

#[test]
fn the_eighth_distinct_builtin_kind_is_blocked_by_a_seven_call_quota() {
    let mut host = MemoryApplicationHost {
        args: vec!["input.txt".to_string()],
        input: "payload".to_string(),
        stdin: "piped".to_string(),
        duration_millis: 7,
        ..MemoryApplicationHost::default()
    };
    let error = run_all_eight_with_quota(&mut host, 7)
        .expect_err("7 admitted effects must exceed an 8-builtin flow");
    let RuntimeError::QuotaExceeded(exceeded) = error else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(exceeded.kind, QuotaKind::EffectCalls);
    assert_eq!(exceeded.limit, 7);
    assert_eq!(exceeded.used, 8);
    // The quota must block time_duration_ms (the eighth effect) itself.
    assert_eq!(host.calls.len(), 7);
    assert!(!host.calls.contains(&HostCallId::TimeDuration));
}
