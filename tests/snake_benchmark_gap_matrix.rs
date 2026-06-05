use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

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

fn check_run_compile_verify(rel: &str) {
    let input = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    cli_ok(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for {input}"),
    );

    let dir = mk_temp_dir("smc_snake_benchmark_gap_matrix");
    let out = dir.join("out.smc");
    let out_arg = out.to_string_lossy().replace('\\', "/");
    cli_ok(
        vec![
            "compile".to_string(),
            input.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {input}"),
    );
    cli_ok(
        vec!["verify".to_string(), out_arg],
        &format!("smc verify for {input}"),
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn snake_benchmark_positive_surface_passes_end_to_end() {
    for rel in [
        "tests/fixtures/snake_benchmark/positive_text_equality.sm",
        "tests/fixtures/snake_benchmark/positive_enum_match.sm",
        "tests/fixtures/snake_benchmark/positive_i32_relational.sm",
        "tests/fixtures/snake_benchmark/positive_i32_arithmetic.sm",
        "tests/fixtures/snake_benchmark/positive_i32_div_mod.sm",
        "tests/fixtures/snake_benchmark/positive_text_concat.sm",
        "tests/fixtures/snake_benchmark/positive_text_to_text.sm",
        "tests/fixtures/snake_benchmark/positive_print.sm",
        "tests/fixtures/snake_benchmark/positive_let_mut.sm",
        "tests/fixtures/snake_benchmark/positive_reassignment.sm",
        "tests/fixtures/snake_benchmark/positive_while_loop.sm",
        "tests/fixtures/snake_benchmark/positive_loop_control.sm",
        "tests/fixtures/snake_benchmark/positive_sequence_indexing.sm",
        "tests/fixtures/snake_benchmark/positive_sequence_iteration.sm",
        "tests/fixtures/snake_benchmark/positive_sequence_len.sm",
        "tests/fixtures/snake_benchmark/positive_is_empty.sm",
        "tests/fixtures/snake_benchmark/positive_contains.sm",
        "tests/fixtures/snake_benchmark/positive_push_prepend.sm",
        "tests/fixtures/snake_benchmark/positive_closure_capture.sm",
        "tests/fixtures/snake_benchmark/positive_pop.sm",
        "tests/fixtures/snake_benchmark/positive_map_basic.sm",
        "tests/fixtures/snake_benchmark/positive_random_seeded.sm",
    ] {
        check_run_compile_verify(rel);
    }
}

#[test]
fn snake_benchmark_runtime_negative_reports_contract_violations() {
    // These fixtures pass smc check but fail at smc run with a specific error message.
    let cases = [
        (
            "tests/fixtures/snake_benchmark/negative_random_invalid_range.sm",
            "lo (10) must be strictly less than hi (10)",
        ),
        (
            "tests/fixtures/snake_benchmark/negative_i32_div_zero.sm",
            "DivisionByZero",
        ),
        (
            "tests/fixtures/snake_benchmark/negative_i32_mod_zero.sm",
            "DivisionByZero",
        ),
    ];

    for (rel, needle) in cases {
        let input = repo_path(rel);
        let err = cli_err(
            vec!["run".to_string(), input.clone()],
            &format!("smc run for {input}"),
        );
        assert!(
            err.contains(needle),
            "expected runtime error containing '{needle}' for {rel}, got: {err}"
        );
    }
}

#[test]
fn snake_benchmark_negative_gap_suite_reports_current_blockers() {
    let cases = [
        (
            "tests/fixtures/snake_benchmark/negative_map_no_type_annotation.sm",
            "E0201",
            "map_empty() requires a contextual Map(K, V) type",
        ),
        (
            "tests/fixtures/snake_benchmark/negative_map_empty_as_statement.sm",
            "E0201",
            "map_empty() requires a contextual Map(K, V) type",
        ),
        (
            "tests/fixtures/snake_benchmark/negative_text_plus_scalar.sm",
            "E0201",
            "text concatenation currently admits only text + text operands",
        ),
        (
            "tests/fixtures/snake_benchmark/negative_to_text_record.sm",
            "E0201",
            "builtin 'to_text' does not yet support record type 'Probe'",
        ),
        (
            "tests/fixtures/snake_benchmark/negative_loop_break.sm",
            "E0201",
            "bare break is allowed only inside while or statement loop",
        ),
        (
            "tests/fixtures/snake_benchmark/negative_continue_statement.sm",
            "E0201",
            "continue is allowed only inside while or statement loop",
        ),
        (
            "tests/fixtures/snake_benchmark/negative_print_non_text.sm",
            "E0201",
            "builtin 'print' expects text",
        ),
    ];

    for (rel, code, needle) in cases {
        let input = repo_path(rel);
        let err = cli_err(
            vec!["check".to_string(), input.clone()],
            &format!("smc check for {input}"),
        );
        assert!(
            err.contains(&format!("Error [{code}]")),
            "expected diagnostic code {code} for {rel}, got: {err}"
        );
        assert!(
            err.contains(needle),
            "expected diagnostic containing '{needle}' for {rel}, got: {err}"
        );
    }
}
