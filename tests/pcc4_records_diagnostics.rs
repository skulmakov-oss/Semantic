use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn fixture_path(rel: &str) -> String {
    repo_path(&format!("tests/fixtures/pcc4_records/{rel}"))
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

fn assert_record_check_error(rel: &str, code: &str, needle: &str) {
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

fn assert_invalid_record_source_does_not_verify(rel: &str, code: &str, needle: &str) {
    assert_record_check_error(rel, code, needle);

    let input = fixture_path(rel);
    let dir = mk_temp_dir("pcc4_records_diagnostics_invalid");
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
                "expected verify to reject invalid record artifact for {rel}, got: {verify_err}"
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

#[test]
fn pcc4_records_unknown_type_rejects_and_does_not_verify() {
    assert_invalid_record_source_does_not_verify(
        "negative_unknown_record_type.sm",
        "E0201",
        "unknown record type 'MissingRecord'",
    );
}

#[test]
fn pcc4_records_missing_required_field_rejects_and_does_not_verify() {
    assert_invalid_record_source_does_not_verify(
        "negative_missing_record_field.sm",
        "E0201",
        "record literal 'Pair' is missing field 'right'",
    );
}

#[test]
fn pcc4_records_duplicate_field_rejects_and_does_not_verify() {
    assert_invalid_record_source_does_not_verify(
        "negative_duplicate_record_field.sm",
        "E0201",
        "record literal 'Pair' cannot repeat field 'left'",
    );
}

#[test]
fn pcc4_records_unknown_field_access_rejects_and_does_not_verify() {
    assert_invalid_record_source_does_not_verify(
        "negative_unknown_record_field_access.sm",
        "E0201",
        "record type 'Pair' has no field named 'middle'",
    );
}

#[test]
fn pcc4_records_field_type_mismatch_rejects_and_does_not_verify() {
    assert_invalid_record_source_does_not_verify(
        "negative_record_field_type_mismatch.sm",
        "E0201",
        "type mismatch in record field 'Probe.value'",
    );
}
