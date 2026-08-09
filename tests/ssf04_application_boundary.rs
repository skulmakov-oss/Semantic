use prom_abi::{AbiError, AbiFailureKind, ApplicationHostAbi, HostCallId};
use prom_cap::{ApplicationCapabilityProfile, CapabilityKind, CapabilityManifest};
use sm_emit::compile_program_to_semcode;
use sm_runtime_core::{ExecutionConfig, ExecutionContext};
use sm_vm::{run_verified_semcode_with_application_host_and_capabilities_and_config, RuntimeError};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const FLOW: &str = r#"
fn main() {
    let input_path: text = args_read(0u32);
    let output_path: text = args_read(1u32);
    let input: text = fs_read_text(input_path);
    fs_write_text(output_path, input + "!");
    return;
}
"#;

const WRITE_WITHOUT_OBSERVATION: &str = r#"
fn main() {
    fs_write_text("output.txt", "payload");
    return;
}
"#;

#[test]
fn published_contract_and_example_name_the_exact_boundary() {
    let contract = include_str!("../docs/spec/controlled_application_boundary_v0.md");
    let example = include_str!("../examples/qualification/ssf04_file_transform.sm");
    for capability in [
        "args.read",
        "stdin.read_text",
        "stdout.write",
        "stderr.write",
        "path.inspect",
        "fs.read",
        "fs.write",
        "time.duration",
    ] {
        assert!(contract.contains(capability), "missing {capability}");
    }
    assert!(contract.contains("semantic.foundation.application/0.1"));
    assert!(contract.contains("Network, child process, ambient environment"));
    assert!(example.contains("args_read"));
    assert!(example.contains("fs_read_text"));
    assert!(example.contains("fs_write_text"));
}

#[derive(Default)]
struct MemoryApplicationHost {
    args: Vec<String>,
    input: String,
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
        Err(Self::unavailable(HostCallId::StdinReadText))
    }

    fn stdout_write(&mut self, _text: &str) -> Result<(), AbiError> {
        Err(Self::unavailable(HostCallId::StdoutWrite))
    }

    fn stderr_write(&mut self, _text: &str) -> Result<(), AbiError> {
        Err(Self::unavailable(HostCallId::StderrWrite))
    }

    fn path_inspect(&mut self, _path: &str) -> Result<bool, AbiError> {
        Err(Self::unavailable(HostCallId::PathInspect))
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
        Err(Self::unavailable(HostCallId::TimeDuration))
    }
}

fn run_flow(
    host: &mut MemoryApplicationHost,
    profile: ApplicationCapabilityProfile,
) -> Result<(), RuntimeError> {
    let bytes = compile_program_to_semcode(FLOW).expect("compile");
    assert_eq!(&bytes[..8], b"SEMCOD17");
    run_verified_semcode_with_application_host_and_capabilities_and_config(
        &bytes,
        "main",
        host,
        &CapabilityManifest::for_application_profile(profile),
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
}

#[test]
fn args_read_transform_write_is_exact_and_replay_deterministic() {
    let build_host = || MemoryApplicationHost {
        args: vec!["input.txt".to_string(), "output.txt".to_string()],
        input: "payload".to_string(),
        ..MemoryApplicationHost::default()
    };
    let mut first = build_host();
    run_flow(&mut first, ApplicationCapabilityProfile::CliFileTransform).expect("run");
    let mut replay = build_host();
    run_flow(&mut replay, ApplicationCapabilityProfile::CliFileTransform).expect("replay");

    assert_eq!(first.calls, replay.calls);
    assert_eq!(first.writes, replay.writes);
    assert_eq!(
        first.writes,
        vec![("output.txt".to_string(), "payload!".to_string())]
    );
}

#[test]
fn read_only_profile_denies_before_host_write() {
    let mut host = MemoryApplicationHost {
        args: vec!["input.txt".to_string(), "output.txt".to_string()],
        input: "payload".to_string(),
        ..MemoryApplicationHost::default()
    };
    let error = run_flow(&mut host, ApplicationCapabilityProfile::CliReadOnly)
        .expect_err("write capability must be denied");
    let RuntimeError::CapabilityDenied(denied) = error else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(denied.capability, CapabilityKind::FsWrite);
    assert_eq!(denied.call, Some(HostCallId::FsWrite));
    assert!(host.writes.is_empty());
}

#[test]
fn runtime_denies_write_before_observation_without_dispatching_host() {
    let bytes = compile_program_to_semcode(WRITE_WITHOUT_OBSERVATION).expect("compile");
    let mut host = MemoryApplicationHost::default();
    let error = run_verified_semcode_with_application_host_and_capabilities_and_config(
        &bytes,
        "main",
        &mut host,
        &CapabilityManifest::for_application_profile(
            ApplicationCapabilityProfile::CliFileTransform,
        ),
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
    .expect_err("write before observation must fail");

    let RuntimeError::HostAbi(denied) = error else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(denied.call, HostCallId::FsWrite);
    assert_eq!(denied.kind, AbiFailureKind::InvalidInput);
    assert!(host.writes.is_empty());
}

fn temp_dir(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "semantic_ssf04_{label}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).expect("mkdir");
    path
}

#[test]
fn cli_file_transform_stays_inside_declared_root() {
    let root = temp_dir("cli_flow");
    let source = root.join("transform.sm");
    std::fs::write(&source, FLOW).expect("source");
    std::fs::write(root.join("input.txt"), "payload").expect("input");

    smc_cli::run(vec![
        "run".to_string(),
        source.display().to_string(),
        "--profile".to_string(),
        "cli-file-transform".to_string(),
        "--root".to_string(),
        root.display().to_string(),
        "--".to_string(),
        "input.txt".to_string(),
        "output.txt".to_string(),
    ])
    .expect("cli run");

    assert_eq!(
        std::fs::read_to_string(root.join("output.txt")).expect("output"),
        "payload!"
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn cli_rejects_parent_traversal_without_creating_output() {
    let root = temp_dir("cli_traversal");
    let source = root.join("transform.sm");
    std::fs::write(&source, FLOW).expect("source");
    std::fs::write(root.join("input.txt"), "payload").expect("input");
    let escaped = root.parent().expect("parent").join("ssf04-escaped.txt");
    let _ = std::fs::remove_file(&escaped);

    let error = smc_cli::run(vec![
        "run".to_string(),
        source.display().to_string(),
        "--profile".to_string(),
        "cli-file-transform".to_string(),
        "--root".to_string(),
        root.display().to_string(),
        "--".to_string(),
        "input.txt".to_string(),
        "../ssf04-escaped.txt".to_string(),
    ])
    .expect_err("traversal must fail");

    assert!(error.contains("parent traversal"));
    assert!(!escaped.exists());
    std::fs::remove_dir_all(root).expect("cleanup");
}
