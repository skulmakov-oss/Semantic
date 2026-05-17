use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn fixture_path(rel: &str) -> String {
    repo_path(&format!("tests/fixtures/pcc5_adt_diagnostics/{rel}"))
}

fn cli_err(args: Vec<String>, context: &str) -> String {
    smc_cli::run(args).expect_err(&format!("{context} unexpectedly passed"))
}

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!(
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

fn assert_adt_check_error(rel: &str, code: &str, needle: &str) {
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

fn assert_invalid_adt_source_does_not_verify(rel: &str, code: &str, needle: &str) {
    assert_adt_check_error(rel, code, needle);

    let input = fixture_path(rel);
    let dir = mk_temp_dir("pcc5_adt_diagnostics_invalid");
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
                "expected verify to reject invalid ADT artifact for {rel}, got: {verify_err}"
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
fn pcc5_adt_unknown_type_and_constructor_diagnostics_are_stable() {
    for (rel, code, needle) in [
        (
            "negative_unknown_adt_type.sm",
            "E0201",
            "unknown record type 'MissingDirection'",
        ),
        (
            "negative_unknown_constructor.sm",
            "E0201",
            "enum 'Direction' has no variant named 'Left'",
        ),
        (
            "negative_constructor_wrong_adt_type.sm",
            "E0201",
            "type mismatch in let 'signal'",
        ),
    ] {
        assert_invalid_adt_source_does_not_verify(rel, code, needle);
    }
}

#[test]
fn pcc5_adt_payload_constructor_diagnostics_are_stable() {
    for (rel, code, needle) in [
        (
            "negative_constructor_payload_type_mismatch.sm",
            "E0201",
            "type mismatch in enum constructor 'Probe::Value' payload item 0: I32 vs Bool",
        ),
        (
            "negative_constructor_payload_arity_mismatch.sm",
            "E0201",
            "enum constructor 'Probe::Pair' expects 2 payload items, got 1",
        ),
    ] {
        assert_invalid_adt_source_does_not_verify(rel, code, needle);
    }
}

#[test]
fn pcc5_match_unknown_variant_and_result_diagnostics_are_stable() {
    for (rel, code, needle) in [
        (
            "negative_match_unknown_variant.sm",
            "E0201",
            "enum 'Direction' has no variant named 'Left' in match pattern",
        ),
        (
            "negative_match_result_type_mismatch.sm",
            "E0201",
            "match expression branch type mismatch: expected I32, got Bool",
        ),
        (
            "negative_match_missing_variant.sm",
            "E0201",
            "non-exhaustive match expression for enum 'Direction'; missing variants: Down",
        ),
    ] {
        assert_invalid_adt_source_does_not_verify(rel, code, needle);
    }
}
