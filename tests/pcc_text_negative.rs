use std::path::PathBuf;

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn cli_err(args: Vec<String>, context: &str) -> String {
    smc_cli::run(args).expect_err(&format!("{context} unexpectedly passed"))
}

struct NegativeFixture {
    path: &'static str,
    expected_markers: &'static [&'static str],
}

#[test]
fn pcc_text_negative_fixtures_fail_with_expected_markers() {
    let fixtures = [
        NegativeFixture {
            path: "tests/fixtures/pcc/text/fail/text_plus_i32.sm",
            expected_markers: &[
                "E0201",
                "text concatenation currently admits only text + text operands",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/text/fail/text_plus_bool.sm",
            expected_markers: &[
                "E0201",
                "text concatenation currently admits only text + text operands",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/text/fail/text_plus_quad.sm",
            expected_markers: &[
                "E0201",
                "text concatenation currently admits only text + text operands",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/text/fail/to_text_record.sm",
            expected_markers: &[
                "E0201",
                "builtin 'to_text' does not yet support record type 'Sensor'",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/text/fail/multiline_text.sm",
            expected_markers: &["E0000", "unterminated string literal"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/text/fail/text_ordering.sm",
            expected_markers: &[
                "E0201",
                "relational operators are currently admitted only for same-family i32 operands",
            ],
        },
    ];

    for fixture in fixtures {
        let input = repo_path(fixture.path);
        let err = cli_err(
            vec!["check".to_string(), input.clone()],
            &format!("smc check for {input}"),
        );

        assert!(
            !err.contains("panicked"),
            "negative fixture panicked instead of reporting diagnostic: {}\n{}",
            fixture.path,
            err
        );

        for marker in fixture.expected_markers {
            assert!(
                err.contains(marker),
                "negative fixture {} did not contain expected marker `{}`.\nOutput:\n{}",
                fixture.path,
                marker,
                err
            );
        }
    }
}
