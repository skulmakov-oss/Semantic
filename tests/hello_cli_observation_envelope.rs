use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use std::sync::atomic::{AtomicUsize, Ordering};

static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        DIR_COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn smc_output(args: &[&str]) -> Output {
    let exe = env!("CARGO_BIN_EXE_smc");
    Command::new(exe).args(args).output().expect("run smc")
}

fn assert_stdout_exact(output: &Output, expected: &str) {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected,
        "unexpected stdout"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_no_stdout_payload(output: &Output) {
    assert!(
        !output.status.success(),
        "expected failure, got stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).is_empty(),
        "expected no stdout payload, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn smc_run_renders_controlled_observation_payload_once() {
    let temp = mk_temp_dir("smc_cli_source_run");
    let source = temp.join("hello.sm");
    std::fs::write(
        &source,
        r#"
fn main() {
    print("Hello, World!");
}
"#,
    )
    .expect("write source fixture");
    let source_arg = source.to_string_lossy().replace('\\', "/");
    let output = smc_output(&["run", &source_arg]);
    assert_stdout_exact(&output, "Hello, World!\n");
    let _ = std::fs::remove_dir_all(&temp);
}

#[test]
fn smc_run_smc_renders_controlled_observation_payload_once() {
    let temp = mk_temp_dir("smc_cli_observation_envelope");
    let source = temp.join("hello.sm");
    std::fs::write(
        &source,
        r#"
fn main() {
    print("Hello, World!");
}
"#,
    )
    .expect("write source fixture");
    let artifact = temp.join("hello_world.smc");
    let source_arg = source.to_string_lossy().replace('\\', "/");
    let artifact_arg = artifact.to_string_lossy().replace('\\', "/");

    let compile_output = smc_output(&["compile", &source_arg, "-o", &artifact_arg]);
    assert!(
        compile_output.status.success(),
        "compile failed: {}",
        String::from_utf8_lossy(&compile_output.stderr)
    );

    let verify_output = smc_output(&["verify", &artifact_arg]);
    assert!(
        verify_output.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&verify_output.stderr)
    );

    let run_output = smc_output(&["run-smc", &artifact_arg]);
    assert_stdout_exact(&run_output, "Hello, World!\n");

    let _ = std::fs::remove_dir_all(&temp);
}

#[test]
fn smc_run_rejects_non_text_and_forbidden_host_output_markers_without_stdout_payload() {
    let cases = [
        ("print_10", "fn main() { print(10); }"),
        ("print_t", "fn main() { print(T); }"),
        ("print_stdout", "fn main() { print(\"stdout\"); }"),
        ("print_print", "fn main() { print(\"print\"); }"),
        ("print_io_write", "fn main() { print(\"io.write\"); }"),
        ("print_file", "fn main() { print(\"file\"); }"),
        ("print_network", "fn main() { print(\"network\"); }"),
        ("print_stdin", "fn main() { print(\"stdin\"); }"),
    ];

    for (name, source) in cases {
        let temp = mk_temp_dir(&format!("smc_cli_negative_{name}"));
        let input = temp.join(format!("{name}.sm"));
        std::fs::write(&input, source).expect("write temp source");
        let output = smc_output(&["run", &input.to_string_lossy()]);
        assert_no_stdout_payload(&output);
        let _ = std::fs::remove_dir_all(&temp);
    }
}

#[test]
fn smc_run_smc_rejects_verify_failure_without_stdout_payload() {
    let temp = mk_temp_dir("smc_cli_verify_failure");
    let artifact = temp.join("invalid.smc");
    std::fs::write(&artifact, b"not a semcode artifact").expect("write invalid artifact");
    let output = smc_output(&["run-smc", &artifact.to_string_lossy()]);
    assert_no_stdout_payload(&output);
    let _ = std::fs::remove_dir_all(&temp);
}
