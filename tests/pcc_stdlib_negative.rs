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
fn pcc_stdlib_negative_fixtures_fail_with_expected_markers() {
    let fixtures = [
        NegativeFixture {
            path: "tests/fixtures/pcc/stdlib/fail/print_i32.sm",
            expected_markers: &["E0201", "builtin 'print' expects text", "I32"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/stdlib/fail/print_bool.sm",
            expected_markers: &["E0201", "builtin 'print' expects text", "Bool"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/stdlib/fail/print_quad.sm",
            expected_markers: &["E0201", "builtin 'print' expects text", "Quad"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/stdlib/fail/print_sequence.sm",
            expected_markers: &["E0201", "builtin 'print' expects text", "Sequence"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/stdlib/fail/print_map.sm",
            expected_markers: &["E0201", "builtin 'print' expects text", "Map"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/stdlib/fail/to_text_record.sm",
            expected_markers: &[
                "E0201",
                "builtin 'to_text' does not yet support record type",
                "Sensor",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/stdlib/fail/to_text_sequence.sm",
            expected_markers: &[
                "E0201",
                "builtin 'to_text' currently supports text, bool, i32, u32, and quad",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/stdlib/fail/unknown_std_namespace.sm",
            expected_markers: &["E0201", "unknown enum type", "text"],
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
