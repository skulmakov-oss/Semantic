#![allow(clippy::ptr_arg, clippy::single_match)]
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn cli_err(args: Vec<String>, context: &str) -> String {
    smc_cli::run(args).expect_err(&format!("{context} unexpectedly passed"))
}

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

fn write_temp_source(dir: &PathBuf, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, source)
        .unwrap_or_else(|err| panic!("write source failed for {path:?}: {err}"));
    path
}

fn diagnostic_for(source: &str, label: &str) -> String {
    let dir = mk_temp_dir("smc_pcc1_control_flow_diagnostics");
    let input = write_temp_source(&dir, "input.sm", source);
    let input_arg = input.to_string_lossy().replace('\\', "/");
    let err = cli_err(
        vec!["check".to_string(), input_arg.clone()],
        &format!("smc check for {input_arg}"),
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        err.contains("Error [E0201]"),
        "expected stable control-flow diagnostic code for {label}, got: {err}"
    );
    err
}

fn compile_path_does_not_verify(source: &str, label: &str) {
    let dir = mk_temp_dir("smc_pcc1_control_flow_diagnostics_compile");
    let input = write_temp_source(&dir, "input.sm", source);
    let input_arg = input.to_string_lossy().replace('\\', "/");
    let out = dir.join("out.smc");
    let out_arg = out.to_string_lossy().replace('\\', "/");

    let compile_res = smc_cli::run(vec![
        "compile".to_string(),
        input_arg.clone(),
        "-o".to_string(),
        out_arg.clone(),
    ]);

    match compile_res {
        Ok(()) => {
            let bytes = std::fs::read(&out).unwrap_or_else(|err| {
                panic!("compile succeeded but artifact was not readable for {label}: {err}")
            });
            let verify_res = smc_cli::run(vec!["verify".to_string(), out_arg.clone()]);
            assert!(
                verify_res.is_err(),
                "invalid control-flow source {label} unexpectedly produced a verifiable artifact ({} bytes)",
                bytes.len()
            );
        }
        Err(_) => {}
    }

    let _ = std::fs::remove_dir_all(&dir);
}

fn check_path_passes(source: &str, label: &str) {
    let dir = mk_temp_dir("smc_pcc1_control_flow_diagnostics_check");
    let input = write_temp_source(&dir, "input.sm", source);
    let input_arg = input.to_string_lossy().replace('\\', "/");

    smc_cli::run(vec!["check".to_string(), input_arg.clone()]).unwrap_or_else(|err| {
        panic!("smc check unexpectedly failed for {label} ({input_arg}): {err}")
    });

    let _ = std::fs::remove_dir_all(&dir);
}

// PCC-1 diagnostic mapping:
// - Type Hell: invalid loop-control / non-bool condition rejection
// - User Pain / Diagnostics Hell: stable, specific CLI diagnostics
// - Lowering Hell guard: invalid sources must not emit valid SemCode

#[test]
fn pcc1_break_outside_loop_has_stable_diagnostic() {
    let err = diagnostic_for(
        r#"
fn main() {
    break;
}
"#,
        "break outside loop",
    );
    assert!(err.contains("bare break is allowed only inside while or statement loop"));
}

#[test]
fn pcc1_continue_outside_loop_has_stable_diagnostic() {
    let err = diagnostic_for(
        r#"
fn main() {
    continue;
}
"#,
        "continue outside loop",
    );
    assert!(err.contains("continue is allowed only inside while or statement loop"));
}

#[test]
fn pcc1_break_after_nested_loop_has_stable_diagnostic() {
    let err = diagnostic_for(
        r#"
fn main() {
    let mut outer: i32 = 0;
    while outer < 1 {
        outer = outer + 1;
        loop {
            break;
        }
    }
    break;
}
"#,
        "break after nested loop",
    );
    assert!(err.contains("bare break is allowed only inside while or statement loop"));
}

#[test]
fn pcc1_continue_after_nested_loop_has_stable_diagnostic() {
    let err = diagnostic_for(
        r#"
fn main() {
    let mut outer: i32 = 0;
    while outer < 1 {
        outer = outer + 1;
        loop {
            break;
        }
    }
    continue;
}
"#,
        "continue after nested loop",
    );
    assert!(err.contains("continue is allowed only inside while or statement loop"));
}

#[test]
fn pcc1_while_non_bool_condition_has_stable_diagnostic() {
    let err = diagnostic_for(
        r#"
fn main() {
    while 1 {
        return;
    }
}
"#,
        "while non-bool condition",
    );
    assert!(err.contains("while condition must be bool"));
    assert!(err.contains("bool"));
}

#[test]
fn pcc1_if_non_bool_condition_has_stable_diagnostic() {
    let err = diagnostic_for(
        r#"
fn main() {
    let total: f64 = if T { 1.0 } else { 2.0 };
    return;
}
"#,
        "if quad condition",
    );
    assert!(err.contains("if expression condition must be bool"));
    assert!(err.contains("quad"));
}

#[test]
fn pcc1_if_bare_quad_condition_has_stable_diagnostic() {
    let err = diagnostic_for(
        r#"
fn main() {
    let q: quad = T;
    if q {
        return;
    } else {
        return;
    }
}
"#,
        "bare quad condition",
    );
    assert!(err.contains("if condition must be bool"));
    assert!(err.contains("explicit compare is required for quad"));
}

#[test]
fn pcc1_if_explicit_true_quad_comparison_has_stable_admission() {
    check_path_passes(
        r#"
fn main() {
    let q: quad = T;
    if q == T {
        return;
    } else {
        return;
    }
}
"#,
        "explicit true quad comparison",
    );
}

#[test]
fn pcc1_if_explicit_false_quad_comparison_has_stable_admission() {
    check_path_passes(
        r#"
fn main() {
    let q: quad = F;
    if q == F {
        return;
    } else {
        return;
    }
}
"#,
        "explicit false quad comparison",
    );
}

#[test]
fn pcc1_invalid_control_flow_compile_path_does_not_verify() {
    let cases = [
        (
            r#"
fn main() {
    break;
}
"#,
            "break outside loop",
        ),
        (
            r#"
fn main() {
    continue;
}
"#,
            "continue outside loop",
        ),
        (
            r#"
fn main() {
    while 1 {
        return;
    }
}
"#,
            "while non-bool condition",
        ),
        (
            r#"
fn main() {
    let mut outer: i32 = 0;
    while outer < 1 {
        outer = outer + 1;
        loop {
            break;
        }
    }
    break;
}
"#,
            "break after nested loop",
        ),
        (
            r#"
fn main() {
    let mut outer: i32 = 0;
    while outer < 1 {
        outer = outer + 1;
        loop {
            break;
        }
    }
    continue;
}
"#,
            "continue after nested loop",
        ),
        (
            r#"
fn main() {
    let total: f64 = if T { 1.0 } else { 2.0 };
    return;
}
"#,
            "if quad condition",
        ),
    ];

    for (source, label) in cases {
        compile_path_does_not_verify(source, label);
    }
}
