use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn fixture_path(rel: &str) -> String {
    repo_path(&format!(
        "tests/fixtures/pcc7_collections_diagnostics/{rel}"
    ))
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

fn assert_collection_check_error(rel: &str, code: &str, needle: &str) {
    let input = fixture_path(rel);
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

fn assert_invalid_collection_source_does_not_verify(rel: &str, code: &str, needle: &str) {
    assert_collection_check_error(rel, code, needle);

    let input = fixture_path(rel);
    let dir = mk_temp_dir("pcc7_collections_diagnostics_invalid");
    let out = dir.join("out.smc");
    let out_arg = out.to_string_lossy().replace('\\', "/");
    let compile_res = smc_cli::run(vec![
        "compile".to_string(),
        input.clone(),
        "-o".to_string(),
        out_arg.clone(),
    ]);

    match compile_res {
        Ok(()) => {
            let bytes = std::fs::read(&out).unwrap_or_else(|err| {
                panic!("compile succeeded but emitted artifact for {input} was not readable: {err}")
            });
            assert!(
                !bytes.is_empty(),
                "compile succeeded but emitted artifact for {input} was empty"
            );
            let verify_err = cli_err(
                vec!["verify".to_string(), out_arg.clone()],
                &format!("smc verify for {out_arg}"),
            );
            assert!(
                verify_err.contains("Error ["),
                "expected verify to reject invalid collections artifact for {rel}, got: {verify_err}"
            );
        }
        Err(err) => {
            assert!(
                err.contains(&format!("Error [{code}]")) || err.contains(needle),
                "expected compile failure to mention {code} or '{needle}' for {rel}, got: {err}"
            );
        }
    }

    let _ = std::fs::remove_dir_all(&dir);
}

fn assert_collection_runtime_trap(rel: &str, needle: &str) {
    let input = fixture_path(rel);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );

    let dir = mk_temp_dir("pcc7_collections_diagnostics_runtime");
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
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );

    let err = cli_err(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for {input}"),
    );
    assert!(
        err.contains(needle),
        "expected runtime trap containing '{needle}' for {rel}, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn assert_sequence_index_type_error(index_expr: &str, needle: &str) {
    let dir = mk_temp_dir("pcc7_sequence_index_type_error");
    let source_path = dir.join("probe.sm");
    std::fs::write(
        &source_path,
        format!(
            r#"
fn main() {{
    let values: Sequence(i32) = [1, 2, 3];
    let x = values[{index_expr}];
    return;
}}
"#
        ),
    )
    .expect("write probe source");

    let err = cli_err(
        vec![
            "check".to_string(),
            source_path.to_string_lossy().replace('\\', "/"),
        ],
        "smc check for sequence index type error",
    );
    assert!(
        err.contains("Error [E0201]"),
        "expected sequence index type error code E0201, got: {err}"
    );
    assert!(
        err.contains(needle),
        "expected sequence index type error containing '{needle}', got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc7_map_empty_requires_context_rejects_and_does_not_verify() {
    assert_invalid_collection_source_does_not_verify(
        "negative_map_empty_no_type_annotation.sm",
        "E0201",
        "map_empty() requires a contextual Map(K, V) type",
    );
}

#[test]
fn pcc7_map_empty_statement_rejects_and_does_not_verify() {
    assert_invalid_collection_source_does_not_verify(
        "negative_map_empty_as_statement.sm",
        "E0201",
        "map_empty() requires a contextual Map(K, V) type",
    );
}

#[test]
fn pcc7_sequence_element_type_mismatch_rejects_and_does_not_verify() {
    assert_invalid_collection_source_does_not_verify(
        "negative_sequence_element_type_mismatch.sm",
        "E0201",
        "type mismatch in ordered sequence item 1: I32 vs Bool",
    );
}

#[test]
fn pcc7_map_key_type_mismatch_rejects_and_does_not_verify() {
    assert_invalid_collection_source_does_not_verify(
        "negative_map_key_type_mismatch.sm",
        "E0201",
        "builtin 'map_set' key type Bool does not match map key type I32",
    );
}

#[test]
fn pcc7_map_value_type_mismatch_rejects_and_does_not_verify() {
    assert_invalid_collection_source_does_not_verify(
        "negative_map_value_type_mismatch.sm",
        "E0201",
        "builtin 'map_set' value type I32 does not match map value type Bool",
    );
}

#[test]
fn pcc7_sequence_index_out_of_bounds_traps_deterministically() {
    assert_collection_runtime_trap(
        "negative_sequence_index_out_of_bounds.sm",
        "SEQUENCE_GET index out of bounds",
    );
}

#[test]
fn pcc7_sequence_pop_empty_traps_deterministically() {
    assert_collection_runtime_trap(
        "negative_sequence_pop_empty.sm",
        "SEQUENCE_POP source must be non-empty",
    );
}

#[test]
fn pcc7_sequence_index_requires_i32_index_rejects() {
    assert_sequence_index_type_error("true", "sequence indexing currently requires i32 index");
}

#[test]
fn pcc7_sequence_index_type_mismatch_fixture_rejects_and_does_not_verify() {
    assert_invalid_collection_source_does_not_verify(
        "negative_sequence_index_type_mismatch.sm",
        "E0201",
        "sequence indexing currently requires i32 index",
    );
}
