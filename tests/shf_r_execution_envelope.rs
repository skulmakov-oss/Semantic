//! SHF-R (#2030): application-mode `smc run` execution envelope and
//! `--metrics json` qualification (contract R1 v2.1, matrix Q-01..Q-41).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

const METRICS_PREFIX: &str = "{\"schema\":\"semantic.run.metrics/0.1\"";

fn mk_temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "shf_r_envelope_{}_{}_{}",
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

fn write_program(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, source).expect("write fixture");
    path
}

fn smc_run(dir: &Path, program: &Path, profile: &str, extra: &[&str], app_args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_smc"));
    command
        .arg("run")
        .arg(program)
        .arg("--profile")
        .arg(profile)
        .arg("--root")
        .arg(dir)
        .args(extra)
        .current_dir(dir);
    if !app_args.is_empty() {
        command.arg("--").args(app_args);
    }
    command.output().expect("run smc")
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn metrics_lines(output: &Output) -> Vec<String> {
    stderr_of(output)
        .lines()
        .filter(|line| line.starts_with(METRICS_PREFIX))
        .map(str::to_string)
        .collect()
}

/// Exactly one metrics line, parsed with a real JSON parser.
fn metrics(output: &Output) -> Value {
    let lines = metrics_lines(output);
    assert_eq!(
        lines.len(),
        1,
        "expected one metrics line, stderr:\n{}",
        stderr_of(output)
    );
    serde_json::from_str(&lines[0]).expect("metrics line is valid JSON")
}

fn without_nondeterministic(mut value: Value) -> Value {
    value
        .as_object_mut()
        .expect("object")
        .remove("nondeterministic");
    value
}

fn assert_exit(output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "unexpected exit status; stderr:\n{}",
        stderr_of(output)
    );
}

const PURE_OK: &str = "fn main() {\n    let x: i32 = 1 + 2;\n    assert(x == 3);\n    return;\n}\n";

fn down_program(depth: u32) -> String {
    format!(
        "fn down(n: i32) -> i32 {{ if n == 0 {{ return 0; }} return 1 + down(n - 1); }}\n\
         fn main() {{ assert(down({depth}) == {depth}); return; }}\n"
    )
}

/// `h()` makes 128 calls to `f()`; `main` calls `h()` `outer` times and then
/// `f()` `tail` times. Non-root calls = outer * 129 + tail. Cheap enough to
/// reach the Calls boundary well inside the default 100 000 steps.
fn calls_program(outer: u32, tail: u32) -> String {
    let body: String = (0..128).map(|_| "    f();\n").collect();
    let tail_calls: String = (0..tail).map(|_| "    f();\n").collect();
    format!(
        "fn f() {{ return; }}\nfn h() {{\n{body}    return;\n}}\n\
         fn main() {{\n    let mut i: i32 = 0;\n    while i < {outer} {{\n        h();\n        i = i + 1;\n    }}\n{tail_calls}    return;\n}}\n"
    )
}

// ---------- R1: defaults and envelope ----------

#[test]
fn q01_default_run_emits_no_metrics_and_keeps_verified_local() {
    let dir = mk_temp_dir();
    let program = write_program(&dir, "ok.sm", PURE_OK);
    let output = smc_run(&dir, &program, "pure", &[], &[]);
    assert_exit(&output, 0);
    assert!(metrics_lines(&output).is_empty());
    // the unchanged default still stops at 100 000 steps
    let busy = write_program(
        &dir,
        "busy.sm",
        "fn main() {\n    let mut i: i32 = 0;\n    while i < 100000 { i = i + 1; }\n    return;\n}\n",
    );
    // Q-03: the trusted-compiler envelope without overrides behaves the same
    for extra in [&[][..], &["--envelope", "trusted-compiler"][..]] {
        let output = smc_run(&dir, &busy, "pure", extra, &[]);
        assert_exit(&output, 1);
        assert!(stderr_of(&output).contains("quota exceeded: Steps limit=100000 used=100001"));
    }
}

#[test]
fn q02_q36_envelopes_without_overrides_are_exactly_verified_local() {
    let dir = mk_temp_dir();
    let program = write_program(&dir, "ok.sm", PURE_OK);
    let expected = serde_json::json!({
        "max_steps": 100000, "max_calls": 16384, "max_frames": 256,
        "max_stack_depth": 256, "max_effect_calls": 1024
    });
    for extra in [
        vec!["--metrics", "json"],
        vec!["--envelope", "verified-local", "--metrics", "json"],
        vec!["--envelope", "trusted-compiler", "--metrics", "json"],
    ] {
        let output = smc_run(&dir, &program, "pure", &extra, &[]);
        assert_exit(&output, 0);
        assert_eq!(metrics(&output)["effective_quotas"], expected, "{extra:?}");
    }
}

#[test]
fn q15_q37_steps_override_never_changes_calls_and_frames_couple_stack_depth() {
    let dir = mk_temp_dir();
    let program = write_program(&dir, "ok.sm", PURE_OK);
    let output = smc_run(
        &dir,
        &program,
        "pure",
        &[
            "--envelope",
            "trusted-compiler",
            "--max-steps",
            "200000000",
            "--max-frames",
            "1024",
            "--metrics",
            "json",
        ],
        &[],
    );
    assert_exit(&output, 0);
    let quotas = &metrics(&output)["effective_quotas"];
    assert_eq!(quotas["max_steps"], 200000000);
    assert_eq!(quotas["max_calls"], 16384);
    assert_eq!(quotas["max_frames"], 1024);
    assert_eq!(quotas["max_stack_depth"], 1024);
    assert_eq!(quotas["max_effect_calls"], 1024);
}

#[test]
fn q04_q05_exact_steps_boundary_and_boundary_plus_one() {
    let dir = mk_temp_dir();
    let program = write_program(&dir, "ok.sm", PURE_OK);
    let probe = smc_run(&dir, &program, "pure", &["--metrics", "json"], &[]);
    let steps = metrics(&probe)["counters"]["steps"]
        .as_u64()
        .expect("steps");
    assert!(steps > 1);

    let exact = steps.to_string();
    let ok = smc_run(
        &dir,
        &program,
        "pure",
        &[
            "--envelope",
            "trusted-compiler",
            "--max-steps",
            &exact,
            "--metrics",
            "json",
        ],
        &[],
    );
    assert_exit(&ok, 0);
    assert_eq!(metrics(&ok)["counters"]["steps"], steps);

    let below = (steps - 1).to_string();
    let fail = smc_run(
        &dir,
        &program,
        "pure",
        &[
            "--envelope",
            "trusted-compiler",
            "--max-steps",
            &below,
            "--metrics",
            "json",
        ],
        &[],
    );
    assert_exit(&fail, 1);
    let m = metrics(&fail);
    assert_eq!(m["outcome"], "quota_exceeded");
    assert_eq!(m["execution_started"], true);
    assert_eq!(
        m["quota_exceeded"],
        serde_json::json!({"kind": "Steps", "limit": steps - 1, "used": steps})
    );
    assert_eq!(m["counters"]["steps"], steps - 1);
}

#[test]
fn q06_q07_exact_frames_boundary_under_trusted_compiler() {
    let dir = mk_temp_dir();
    // down(d) needs d + 2 frames (main, down(d) .. down(0))
    let ok_program = write_program(&dir, "ok.sm", &down_program(1022));
    let ok = smc_run(
        &dir,
        &ok_program,
        "pure",
        &[
            "--envelope",
            "trusted-compiler",
            "--max-frames",
            "1024",
            "--metrics",
            "json",
        ],
        &[],
    );
    assert_exit(&ok, 0);
    assert_eq!(metrics(&ok)["counters"]["peak_frames"], 1024);

    let deep_program = write_program(&dir, "deep.sm", &down_program(1023));
    let fail = smc_run(
        &dir,
        &deep_program,
        "pure",
        &[
            "--envelope",
            "trusted-compiler",
            "--max-frames",
            "1024",
            "--metrics",
            "json",
        ],
        &[],
    );
    assert_exit(&fail, 1);
    let m = metrics(&fail);
    assert_eq!(
        m["quota_exceeded"],
        serde_json::json!({"kind": "Frames", "limit": 1024, "used": 1025})
    );
    assert_eq!(m["counters"]["peak_frames"], 1024);
}

// ---------- R1: calls boundary (max_calls stays 16 384) ----------

#[test]
fn q38_exactly_16384_calls_succeed_under_raised_steps() {
    let dir = mk_temp_dir();
    let program = write_program(&dir, "calls.sm", &calls_program(127, 1));
    let output = smc_run(
        &dir,
        &program,
        "pure",
        &[
            "--envelope",
            "trusted-compiler",
            "--max-steps",
            "200000000",
            "--metrics",
            "json",
        ],
        &[],
    );
    assert_exit(&output, 0);
    assert_eq!(metrics(&output)["counters"]["calls"], 16384);
}

#[test]
fn q39_q40_call_16385_fails_identically_with_and_without_the_envelope() {
    let dir = mk_temp_dir();
    let program = write_program(&dir, "calls.sm", &calls_program(128, 0));
    let trusted = smc_run(
        &dir,
        &program,
        "pure",
        &[
            "--envelope",
            "trusted-compiler",
            "--max-steps",
            "200000000",
            "--metrics",
            "json",
        ],
        &[],
    );
    // Q-40: the fixture reaches the Calls boundary well inside the default
    // 100 000 steps, so the default path observes the same Calls failure.
    let default = smc_run(&dir, &program, "pure", &["--metrics", "json"], &[]);
    for output in [&trusted, &default] {
        assert_exit(output, 1);
        let m = metrics(output);
        assert_eq!(
            m["quota_exceeded"],
            serde_json::json!({"kind": "Calls", "limit": 16384, "used": 16385})
        );
        assert_eq!(m["counters"]["calls"], 16384);
        assert!(m["counters"]["steps"].as_u64().unwrap() < 100000);
    }
    assert_eq!(
        metrics(&trusted)["counters"],
        metrics(&default)["counters"],
        "the envelope must not alter Calls enforcement"
    );
}

// ---------- R1: argument validation ----------

#[test]
fn q08_to_q14_q41_invalid_arguments_fail_before_compilation() {
    let dir = mk_temp_dir();
    // a program that would fail to compile: validation must win first
    let program = write_program(&dir, "bad.sm", "fn main() { let x: i32 = ; }\n");
    let cases: &[(&[&str], &str)] = &[
        (
            &["--max-steps", "5"],
            "--max-steps requires --envelope trusted-compiler",
        ),
        (
            &["--max-frames", "5"],
            "--max-frames requires --envelope trusted-compiler",
        ),
        (
            &["--envelope", "trusted-compiler", "--max-steps", "0"],
            "--max-steps must be between 1 and 200000000",
        ),
        (
            &["--envelope", "trusted-compiler", "--max-steps", "200000001"],
            "--max-steps must be between 1 and 200000000",
        ),
        (
            &[
                "--envelope",
                "trusted-compiler",
                "--max-steps",
                "18446744073709551616",
            ],
            "--max-steps must be between 1 and 200000000",
        ),
        (
            &[
                "--envelope",
                "trusted-compiler",
                "--max-steps",
                "99999999999999999999999",
            ],
            "--max-steps must be between 1 and 200000000",
        ),
        (
            &["--envelope", "trusted-compiler", "--max-frames", "1025"],
            "--max-frames must be between 1 and 1024",
        ),
        (
            &["--envelope", "trusted-compiler", "--max-frames", "0"],
            "--max-frames must be between 1 and 1024",
        ),
        (
            &["--envelope", "trusted-compiler", "--max-steps", "-1"],
            "--max-steps must be a decimal integer",
        ),
        (
            &["--envelope", "trusted-compiler", "--max-steps", "+5"],
            "--max-steps must be a decimal integer",
        ),
        (
            &["--envelope", "trusted-compiler", "--max-steps", " 5"],
            "--max-steps must be a decimal integer",
        ),
        (
            &["--envelope", "trusted-compiler", "--max-steps", "0x10"],
            "--max-steps must be a decimal integer",
        ),
        (
            &["--envelope", "trusted-compiler", "--max-steps", ""],
            "--max-steps must be a decimal integer",
        ),
        (
            &["--envelope", "trusted-compiler", "--max-steps", "1e6"],
            "--max-steps must be a decimal integer",
        ),
        (
            &["--envelope", "trusted-compiler", "--max-steps", "\u{0665}"],
            "--max-steps must be a decimal integer",
        ),
        (
            &["--envelope", "root"],
            "unknown execution envelope 'root'; expected verified-local or trusted-compiler",
        ),
        (
            &["--metrics", "text"],
            "unsupported metrics format 'text'; expected json",
        ),
        (
            &[
                "--envelope",
                "trusted-compiler",
                "--max-steps",
                "5",
                "--max-steps",
                "6",
            ],
            "duplicate run option '--max-steps'",
        ),
        (
            &["--metrics", "json", "--metrics", "json"],
            "duplicate run option '--metrics'",
        ),
        (&["--max-calls", "100"], "unknown run option '--max-calls'"),
    ];
    for (extra, message) in cases {
        let mut args = extra.to_vec();
        args.extend(
            ["--metrics", "json"]
                .iter()
                .filter(|_| !extra.contains(&"--metrics")),
        );
        let output = smc_run(&dir, &program, "pure", &args, &[]);
        assert_exit(&output, 1);
        let stderr = stderr_of(&output);
        assert!(
            stderr.contains(message),
            "{extra:?}: expected {message:?}, got:\n{stderr}"
        );
        assert!(
            metrics_lines(&output).is_empty(),
            "{extra:?}: no metrics on argument errors"
        );
        assert!(
            !stderr.contains("expected primary expression"),
            "{extra:?}: compiled"
        );
    }
}

// ---------- R2: metrics outcomes ----------

#[test]
fn q21_compile_error_has_null_counters() {
    let dir = mk_temp_dir();
    let program = write_program(&dir, "bad.sm", "fn main() { let x: i32 = ; }\n");
    let output = smc_run(&dir, &program, "pure", &["--metrics", "json"], &[]);
    assert_exit(&output, 1);
    let m = metrics(&output);
    assert_eq!(m["outcome"], "compile_error");
    assert_eq!(m["phase_reached"], "compile");
    assert_eq!(m["execution_started"], false);
    assert!(m["counters"].is_null());
    assert!(m["quota_exceeded"].is_null());
}

#[test]
fn q22_verifier_rejection_through_the_register_budget_has_null_counters() {
    // Legitimate seam: the verifier checks the register budget from the same
    // effective RuntimeQuotas; a balanced 4 224-term expression exceeds 4 096.
    let group = format!("({})", vec!["1"; 66].join(" + "));
    let expression = vec![group.as_str(); 64].join(" + ");
    let dir = mk_temp_dir();
    let program = write_program(
        &dir,
        "regs.sm",
        &format!("fn main() {{\n    let v: i32 = {expression};\n    return;\n}}\n"),
    );
    // Q-16: admission is identical under every envelope, because the
    // admission-relevant quota fields are never overridden.
    let mut errors = Vec::new();
    for extra in [
        &["--metrics", "json"][..],
        &[
            "--envelope",
            "trusted-compiler",
            "--max-steps",
            "200000000",
            "--max-frames",
            "1024",
            "--metrics",
            "json",
        ][..],
    ] {
        let output = smc_run(&dir, &program, "pure", extra, &[]);
        assert_exit(&output, 1);
        let m = metrics(&output);
        assert_eq!(m["outcome"], "verify_rejected");
        assert_eq!(m["phase_reached"], "verify");
        assert_eq!(m["execution_started"], false);
        assert!(m["counters"].is_null());
        assert!(m["error"]
            .as_str()
            .unwrap()
            .contains("register budget of 4096"));
        errors.push(m["error"].clone());
    }
    assert_eq!(errors[0], errors[1]);
}

#[test]
fn host_init_error_has_null_counters_and_escapes_the_path() {
    let dir = mk_temp_dir();
    let program = write_program(&dir, "ok.sm", PURE_OK);
    let missing_root = "C:\\no such \"root\"\\dir";
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(["run"])
        .arg(&program)
        .args([
            "--profile",
            "pure",
            "--root",
            missing_root,
            "--metrics",
            "json",
        ])
        .current_dir(&dir)
        .output()
        .expect("run smc");
    assert_exit(&output, 1);
    let m = metrics(&output);
    assert_eq!(m["outcome"], "host_init_error");
    assert_eq!(m["execution_started"], false);
    assert!(m["counters"].is_null());
    assert!(m["error"].is_string());
}

#[test]
fn q18_capability_denial_is_reported_with_counters_and_audit() {
    let dir = mk_temp_dir();
    let program = write_program(
        &dir,
        "args.sm",
        "fn main() {\n    let x: text = args_read(0u32);\n    return;\n}\n",
    );
    let output = smc_run(
        &dir,
        &program,
        "pure",
        &[
            "--envelope",
            "trusted-compiler",
            "--max-steps",
            "200000000",
            "--metrics",
            "json",
        ],
        &["x"],
    );
    assert_exit(&output, 1);
    let m = metrics(&output);
    assert_eq!(m["outcome"], "capability_denied");
    assert_eq!(m["execution_started"], true);
    assert_eq!(m["counters"]["effect_calls"], 0);
    assert!(stderr_of(&output).contains("decision=deny"));
}

#[test]
fn q19_effect_quota_is_unchanged_by_the_envelope() {
    let dir = mk_temp_dir();
    let program = write_program(
        &dir,
        "effects.sm",
        "fn main() {\n    let a: text = args_read(0u32);\n    let mut i: i32 = 0;\n    while i < 1024 {\n        stdout_write(\"\");\n        i = i + 1;\n    }\n    return;\n}\n",
    );
    let output = smc_run(
        &dir,
        &program,
        "cli-read-only",
        &[
            "--envelope",
            "trusted-compiler",
            "--max-steps",
            "200000000",
            "--metrics",
            "json",
        ],
        &["x"],
    );
    assert_exit(&output, 1);
    let m = metrics(&output);
    assert_eq!(
        m["quota_exceeded"],
        serde_json::json!({"kind": "EffectCalls", "limit": 1024, "used": 1025})
    );
    assert_eq!(m["counters"]["effect_calls"], 1024);
}

#[test]
fn q24_trap_keeps_counters_and_exit_status() {
    let dir = mk_temp_dir();
    let program = write_program(
        &dir,
        "trap.sm",
        "fn main() {\n    assert(false);\n    return;\n}\n",
    );
    let output = smc_run(&dir, &program, "pure", &["--metrics", "json"], &[]);
    assert_exit(&output, 1);
    let m = metrics(&output);
    assert_eq!(m["outcome"], "trap");
    assert_eq!(m["execution_started"], true);
    assert!(m["counters"]["steps"].as_u64().unwrap() > 0);
}

#[test]
fn q20_success_metrics_are_complete() {
    let dir = mk_temp_dir();
    let program = write_program(&dir, "deep.sm", &down_program(10));
    let output = smc_run(&dir, &program, "pure", &["--metrics", "json"], &[]);
    assert_exit(&output, 0);
    let m = metrics(&output);
    assert_eq!(m["schema"], "semantic.run.metrics/0.1");
    assert_eq!(m["envelope"], "verified-local");
    assert_eq!(m["outcome"], "ok");
    assert_eq!(m["phase_reached"], "execute");
    assert_eq!(m["counters"]["calls"], 11);
    assert_eq!(m["counters"]["peak_frames"], 12);
    assert!(m["nondeterministic"]["wall_ms"].as_f64().unwrap() >= 0.0);
}

#[test]
fn q26_deterministic_fields_repeat_exactly() {
    let dir = mk_temp_dir();
    let program = write_program(&dir, "calls.sm", &calls_program(10, 3));
    let runs: Vec<Value> = (0..3)
        .map(|_| {
            without_nondeterministic(metrics(&smc_run(
                &dir,
                &program,
                "pure",
                &["--metrics", "json"],
                &[],
            )))
        })
        .collect();
    assert_eq!(runs[0], runs[1]);
    assert_eq!(runs[1], runs[2]);
}

#[test]
fn q30_q31_q32_streams_are_separated_and_json_is_escaped() {
    let dir = mk_temp_dir();
    let program = write_program(
        &dir,
        "echo.sm",
        "fn main() {\n    let x: text = args_read(0u32);\n    stdout_write(x);\n    return;\n}\n",
    );
    let arg = "C:\\path with \"quotes\"\tand\\backslash";
    let output = smc_run(
        &dir,
        &program,
        "cli-read-only",
        &["--metrics", "json"],
        &[arg],
    );
    assert_exit(&output, 0);
    // stdout carries only application output
    assert_eq!(String::from_utf8_lossy(&output.stdout), arg);
    // stderr: audit records, then exactly one metrics line, in that order
    let stderr = stderr_of(&output);
    let lines: Vec<&str> = stderr.lines().collect();
    let metrics_at = lines
        .iter()
        .position(|l| l.starts_with(METRICS_PREFIX))
        .expect("metrics");
    assert!(lines[..metrics_at]
        .iter()
        .all(|l| l.starts_with("schema=semantic.foundation.application.audit/")));
    assert_eq!(metrics_at, lines.len() - 1);

    // a diagnostic containing a newline round-trips through JSON
    let group = format!("({})", vec!["1"; 66].join(" + "));
    let expression = vec![group.as_str(); 64].join(" + ");
    let regs = write_program(
        &dir,
        "regs.sm",
        &format!("fn main() {{\n    let v: i32 = {expression};\n    return;\n}}\n"),
    );
    let rejected = smc_run(&dir, &regs, "pure", &["--metrics", "json"], &[]);
    let line = metrics_lines(&rejected).pop().expect("metrics");
    assert!(!line.contains('\n'));
    let error = serde_json::from_str::<Value>(&line).unwrap()["error"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        error.contains('\n'),
        "multi-line diagnostic preserved: {error:?}"
    );
}

// ---------- unchanged paths ----------

#[test]
fn q29_run_smc_and_single_argument_run_do_not_accept_the_new_flags() {
    let dir = mk_temp_dir();
    let program = write_program(&dir, "ok.sm", PURE_OK);
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(["run-smc"])
        .arg(dir.join("ok.smc"))
        .args(["--metrics", "json"])
        .output()
        .expect("run smc");
    assert_exit(&output, 1);
    assert!(stderr_of(&output).contains("usage: smc run-smc <input.smc>"));
    assert!(metrics_lines(&output).is_empty());
    let single = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("run")
        .arg(&program)
        .output()
        .expect("run smc");
    assert!(metrics_lines(&single).is_empty());
}

#[test]
fn q28_smc_test_output_is_unchanged() {
    let dir = mk_temp_dir();
    std::fs::write(
        dir.join("Semantic.package"),
        "format 1\npackage shfr_probe\nmanifest_dir .\nmodule_root src\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("src/main.sm"), PURE_OK).unwrap();
    std::fs::write(dir.join("tests/one.sm"), PURE_OK).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("test")
        .arg(&dir)
        .output()
        .expect("run smc test");
    assert_exit(&output, 0);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "ok tests/one.sm\ntest result: ok. 1 passed\n"
    );
}

#[test]
fn option_in_the_input_position_is_an_argument_error_without_metrics() {
    let dir = mk_temp_dir();
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(["run", "--foo", "--profile", "pure", "--root"])
        .arg(&dir)
        .args(["--metrics", "json"])
        .output()
        .expect("run smc");
    assert_exit(&output, 1);
    assert!(stderr_of(&output).contains("unknown flag '--foo'"));
    assert!(metrics_lines(&output).is_empty());
}
