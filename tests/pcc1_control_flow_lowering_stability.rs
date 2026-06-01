use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

fn cli_err(args: Vec<String>, context: &str) -> String {
    smc_cli::run(args).expect_err(&format!("{context} unexpectedly passed"))
}

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
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

fn compile_semcode_bytes(source: &str, out_name: &str) -> Vec<u8> {
    let dir = mk_temp_dir("smc_pcc1_control_flow_lowering_stability");
    let input = write_temp_source(&dir, "input.sm", source);
    let out = dir.join(out_name);
    let input_arg = input.to_string_lossy().replace('\\', "/");
    let out_arg = out.to_string_lossy().replace('\\', "/");

    cli_ok(
        vec!["check".to_string(), input_arg.clone()],
        &format!("smc check for {input_arg}"),
    );
    cli_ok(
        vec!["run".to_string(), input_arg.clone()],
        &format!("smc run for {input_arg}"),
    );
    cli_ok(
        vec![
            "compile".to_string(),
            input_arg.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {input_arg}"),
    );
    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );
    cli_ok(
        vec!["run-smc".to_string(), out_arg.clone()],
        &format!("smc run-smc for {out_arg}"),
    );

    let bytes = std::fs::read(&out)
        .unwrap_or_else(|err| panic!("read emitted semcode failed for {out_arg}: {err}"));
    let _ = std::fs::remove_dir_all(&dir);
    bytes
}

fn assert_positive_semcode_stable(source: &str, out_prefix: &str) {
    let first = compile_semcode_bytes(source, &format!("{out_prefix}_first.smc"));
    let second = compile_semcode_bytes(source, &format!("{out_prefix}_second.smc"));
    assert_eq!(first, second, "SemCode bytes drifted for {out_prefix}");
}

fn assert_negative_control_flow_does_not_emit_semcode(source: &str, label: &str) {
    let dir = mk_temp_dir("smc_pcc1_control_flow_lowering_negative");
    let input = write_temp_source(&dir, "input.sm", source);
    let input_arg = input.to_string_lossy().replace('\\', "/");

    let check_err = cli_err(
        vec!["check".to_string(), input_arg.clone()],
        &format!("smc check for {input_arg}"),
    );
    assert!(
        check_err.contains("Error [E0201]"),
        "expected stable control-flow diagnostic code for {label}, got: {check_err}"
    );

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
            let verify_err = smc_cli::run(vec!["verify".to_string(), out_arg.clone()]);
            assert!(
                verify_err.is_err(),
                "negative control-flow fixture {label} unexpectedly produced a verifiable artifact ({} bytes)",
                bytes.len()
            );
        }
        Err(_) => {}
    }

    let _ = std::fs::remove_dir_all(&dir);
}

// PCC-1 lowering gate mapping:
// - Syntax Hell: parser acceptance of if / while / loop surfaces
// - Type Hell: bool loop conditions and loop-control diagnostics
// - Lowering Hell: stable compile-to-SemCode paths
// - Verifier Hell: emitted SemCode must verify before execution
// - VM Hell: emitted SemCode must run deterministically
// - Practical Hell: public CLI pipeline coverage
// - User Pain / Diagnostics Hell: stable rejection messages

#[test]
fn pcc1_if_else_emits_stable_semcode() {
    assert_positive_semcode_stable(
        r#"
fn main() {
    let value: i32 = if true { 1 } else { 2 };
    assert(value == 1);
    return;
}
"#,
        "if_else_stable",
    );
}

#[test]
fn pcc1_while_emits_stable_semcode() {
    assert_positive_semcode_stable(
        r#"
fn main() {
    let mut i: i32 = 0;
    while i < 3 {
        i = i + 1;
    }
    assert(i == 3);
    return;
}
"#,
        "while_stable",
    );
}

#[test]
fn pcc1_loop_break_continue_emits_stable_semcode() {
    assert_positive_semcode_stable(
        r#"
fn main() {
    let mut i: i32 = 0;
    loop {
        i = i + 1;
        if i < 3 {
            continue;
        }
        break;
    }
    assert(i == 3);
    return;
}
"#,
        "loop_break_continue_stable",
    );
}

#[test]
fn pcc1_nested_while_break_emits_stable_semcode() {
    assert_positive_semcode_stable(
        r#"
fn main() {
    let mut outer: i32 = 0;
    let mut inner_breaks: i32 = 0;
    while outer < 3 {
        outer = outer + 1;
        let mut inner: i32 = 0;
        while inner < 4 {
            inner = inner + 1;
            inner_breaks = inner_breaks + 1;
            break;
        }
    }
    assert(outer == 3);
    assert(inner_breaks == 3);
    return;
}
"#,
        "nested_while_break_stable",
    );
}

#[test]
fn pcc1_nested_loop_continue_emits_stable_semcode() {
    assert_positive_semcode_stable(
        r#"
fn main() {
    let mut outer: i32 = 0;
    let mut inner_steps: i32 = 0;
    while outer < 2 {
        outer = outer + 1;
        let mut inner: i32 = 0;
        loop {
            inner = inner + 1;
            inner_steps = inner_steps + 1;
            if inner < 2 {
                continue;
            }
            break;
        }
    }
    assert(outer == 2);
    assert(inner_steps == 4);
    return;
}
"#,
        "nested_loop_continue_stable",
    );
}

#[test]
fn pcc1_nested_loop_break_then_outer_continue_emits_stable_semcode() {
    assert_positive_semcode_stable(
        r#"
fn main() {
    let mut outer: i32 = 0;
    let mut inner_breaks: i32 = 0;
    let mut tail: i32 = 0;
    while outer < 3 {
        outer = outer + 1;
        loop {
            inner_breaks = inner_breaks + 1;
            break;
        }
        if outer < 3 {
            continue;
        }
        tail = tail + 1;
    }
    assert(outer == 3);
    assert(inner_breaks == 3);
    assert(tail == 1);
    return;
}
"#,
        "nested_loop_break_then_outer_continue_stable",
    );
}

#[test]
fn pcc1_invalid_control_flow_does_not_emit_semcode() {
    let cases = [
        (
            r#"
fn main() {
    break;
}
"#,
            "negative_break_outside_loop",
        ),
        (
            r#"
fn main() {
    continue;
}
"#,
            "negative_continue_outside_loop",
        ),
        (
            r#"
fn main() {
    while 1 {
        return;
    }
}
"#,
            "negative_while_non_bool_condition",
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
            "negative_break_after_nested_loop_outside_loop",
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
            "negative_continue_after_nested_loop_outside_loop",
        ),
    ];

    for (source, label) in cases {
        assert_negative_control_flow_does_not_emit_semcode(source, label);
    }
}
