//! End-to-end tests for `smc look ui frame` (Issue #1365).
//!
//! Two invocation styles, matching existing CLI test conventions
//! (`tests/cli_command_shape_diagnostics.rs`, `tests/work_layer_guard_tests.rs`):
//! in-process `smc_cli::run` for argument-shape diagnostics, and a real
//! `Command::new(env!("CARGO_BIN_EXE_smc"))` subprocess for true end-to-end
//! behavior (file I/O, exit codes, determinism).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn run_in_process(args: &[&str]) -> Result<(), String> {
    smc_cli::run(args.iter().map(|s| s.to_string()).collect())
}

fn smc(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc")
}

fn fixture(name: &str) -> String {
    format!("tests/fixtures/ui_frame_inspection/{name}")
}

fn temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{prefix}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("mkdir temp dir");
    dir
}

// ---- CLI parsing / argument-shape diagnostics (in-process) ------------

#[test]
fn rejects_neither_input_mode() {
    let err = run_in_process(&["look", "ui", "frame"]).expect_err("must fail");
    assert!(err.starts_with("InvalidArguments"), "got: {err}");
}

#[test]
fn rejects_both_input_modes() {
    let err = run_in_process(&[
        "look",
        "ui",
        "frame",
        "--from",
        &fixture("snapshot_valid.json"),
        &fixture("valid_source.txt"),
    ])
    .expect_err("must fail");
    assert!(err.starts_with("InvalidArguments"), "got: {err}");
}

#[test]
fn rejects_events_combined_with_from() {
    let err = run_in_process(&[
        "look",
        "ui",
        "frame",
        "--from",
        &fixture("snapshot_valid.json"),
        "--events",
        &fixture("events_admit_then_deny.json"),
    ])
    .expect_err("must fail");
    assert!(err.starts_with("InvalidArguments"), "got: {err}");
}

#[test]
fn rejects_missing_input_file() {
    let err = run_in_process(&["look", "ui", "frame", "definitely-does-not-exist.txt"])
        .expect_err("must fail");
    assert!(err.starts_with("InvalidSource"), "got: {err}");
}

#[test]
fn rejects_unsupported_format() {
    let err = run_in_process(&[
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--format",
        "svg",
    ])
    .expect_err("must fail");
    assert!(err.starts_with("UnsupportedFormat"), "got: {err}");
}

#[test]
fn rejects_malformed_look_subcommand_shape() {
    assert!(run_in_process(&["look"]).is_err());
    assert!(run_in_process(&["look", "ui"]).is_err());
    assert!(run_in_process(&["look", "not-ui", "frame"]).is_err());
}

// ---- snapshot mode (subprocess, real end-to-end) -----------------------

#[test]
fn snapshot_mode_loads_and_reports_selected_frame() {
    let out = smc(&[
        "look",
        "ui",
        "frame",
        "--from",
        &fixture("snapshot_valid.json"),
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("mode: snapshot"));
    assert!(stdout.contains("status: Loaded"));
    assert!(stdout.contains("captured-frames: 1"));
    assert!(stdout.contains(
        "NOTICE: inspection output only; not live runtime authority; does not authorize action or committed effect"
    ));
}

#[test]
fn snapshot_mode_never_compiles_source_even_when_frame_is_source_shaped() {
    // A snapshot document is never treated as source, regardless of
    // content -- confirmed by using a genuinely valid snapshot and
    // asserting the report never mentions compilation/activation at all.
    let out = smc(&[
        "look",
        "ui",
        "frame",
        "--from",
        &fixture("snapshot_valid.json"),
    ]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("source-profile: projection-source-v0"));
    assert!(stdout.contains("source-profile: n/a (snapshot mode)"));
}

#[test]
fn snapshot_mode_rejects_malformed_input_distinctly() {
    let out = smc(&[
        "look",
        "ui",
        "frame",
        "--from",
        &fixture("snapshot_malformed_json.json"),
    ]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with("InvalidSnapshot"), "got: {stderr}");
}

#[test]
fn snapshot_mode_rejects_unsupported_version_distinctly_from_malformed() {
    let out = smc(&[
        "look",
        "ui",
        "frame",
        "--from",
        &fixture("snapshot_unsupported_version.json"),
    ]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with("UnsupportedVersion"), "got: {stderr}");
}

// ---- source mode + events + frame selection (subprocess) --------------

#[test]
fn source_mode_with_events_and_frame_selects_expected_frame() {
    let out = smc(&[
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--events",
        &fixture("events_admit_then_deny.json"),
        "--frame",
        "2",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("mode: source"));
    assert!(stdout.contains("source-profile: projection-source-v0"));
    assert!(stdout.contains("requested-frame: 2"));
    assert!(stdout.contains("captured-frames: 5"));
    // `status`/`admission` describe the state as of the *selected* frame,
    // not the script's later steps: frame 2 is right after the admitted
    // click, so it reports Captured/not-denied even though this same
    // script later (frame 4) denies a click on LABEL_NODE.
    assert!(stdout.contains("status: Captured"));
    assert!(stdout.contains("admission: last-denied=false"));
    assert!(stdout.contains("availability: Some(Available)"));
}

#[test]
fn source_mode_final_denied_action_reports_denied_status() {
    let out = smc(&[
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--events",
        &fixture("events_admit_then_deny.json"),
        "--frame",
        "4",
    ]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("status: Denied"));
    assert!(stdout.contains("admission: last-denied=true"));
    // Denied != Failed: the process still exits 0 and produces a full report.
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn source_mode_frame_not_found_is_distinct_from_empty_success() {
    let out = smc(&[
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--frame",
        "999",
    ]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with("FrameNotFound"), "got: {stderr}");
}

#[test]
fn source_mode_invalid_source_fails_before_frame_production() {
    let out = smc(&["look", "ui", "frame", &fixture("invalid_source.txt")]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with("CompileFailed"), "got: {stderr}");
}

#[test]
fn source_mode_invalid_event_script_is_rejected_distinctly() {
    let out = smc(&[
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--events",
        &fixture("events_invalid_kind.json"),
    ]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with("InvalidEventScript"), "got: {stderr}");
}

#[test]
fn source_mode_completes_headlessly_without_a_native_backend_or_display() {
    // This process/CI environment has no display server or GPU device. If
    // `run_source_mode` ever constructed `NativeBackend` (winit/wgpu)
    // instead of `InMemoryBackend`, window/surface creation would fail or
    // hang here. Clean, fast success is itself the evidence that only the
    // headless in-memory backend was used.
    let out = smc(&[
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--events",
        &fixture("events_tab_and_activate.json"),
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ---- --out handling -----------------------------------------------------

#[test]
fn out_flag_writes_atomically_and_matches_stdout_byte_for_byte() {
    let dir = temp_dir("smc_look_out_ok");
    let out_path = dir.join("frame.json");

    let stdout_run = smc(&[
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--format",
        "draw-json",
    ]);
    assert!(stdout_run.status.success());

    let file_run = smc(&[
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--format",
        "draw-json",
        "--out",
        out_path.to_str().unwrap(),
    ]);
    assert!(file_run.status.success());
    assert!(file_run.stdout.is_empty(), "--out must suppress stdout");

    let written = fs::read(&out_path).expect("read --out file");
    assert_eq!(
        written, stdout_run.stdout,
        "--out content must match stdout content exactly"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn out_flag_reports_write_failure_distinctly_and_does_not_touch_destination() {
    let dir = temp_dir("smc_look_out_fail");
    // A directory can never be opened for writing as a regular file --
    // this deterministically forces the write step to fail.
    let unwritable_dest = dir.join("subdir");
    fs::create_dir_all(&unwritable_dest).expect("mkdir");

    let out = smc(&[
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--out",
        unwritable_dest.to_str().unwrap(),
    ]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with("OutputWriteFailed"), "got: {stderr}");
    // The destination is a pre-existing directory; the failed write must
    // not have replaced or removed it.
    assert!(unwritable_dest.is_dir());

    let _ = fs::remove_dir_all(&dir);
}

// ---- determinism ----------------------------------------------------------

#[test]
fn same_inputs_produce_byte_identical_text_output_twice() {
    let args = [
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--events",
        &fixture("events_admit_then_deny.json"),
        "--frame",
        "3",
    ];
    let a = smc(&args);
    let b = smc(&args);
    assert!(a.status.success() && b.status.success());
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn same_inputs_produce_byte_identical_draw_json_output_twice() {
    let args = [
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--events",
        &fixture("events_admit_then_deny.json"),
        "--format",
        "draw-json",
    ];
    let a = smc(&args);
    let b = smc(&args);
    assert!(a.status.success() && b.status.success());
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn snapshot_written_by_source_mode_reads_back_identically_via_from() {
    let dir = temp_dir("smc_look_roundtrip");
    let snap_path = dir.join("snap.json");

    let write = smc(&[
        "look",
        "ui",
        "frame",
        &fixture("valid_source.txt"),
        "--events",
        &fixture("events_admit_then_deny.json"),
        "--format",
        "draw-json",
        "--out",
        snap_path.to_str().unwrap(),
    ]);
    assert!(write.status.success());
    let original = fs::read(&snap_path).expect("read snapshot");

    let reread = smc(&[
        "look",
        "ui",
        "frame",
        "--from",
        snap_path.to_str().unwrap(),
        "--format",
        "draw-json",
    ]);
    assert!(reread.status.success());
    assert_eq!(
        reread.stdout, original,
        "re-read draw-json must be byte-identical to the written snapshot"
    );

    let _ = fs::remove_dir_all(&dir);
}
