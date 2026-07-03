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
fn pcc_control_flow_negative_fixtures_fail_with_expected_markers() {
    let fixtures = [
        NegativeFixture {
            path: "tests/fixtures/pcc/control_flow/fail/if_quad_condition.sm",
            expected_markers: &["E0201", "if condition must be bool"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/control_flow/fail/while_quad_condition.sm",
            expected_markers: &["E0201", "while condition must be bool"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/control_flow/fail/break_outside_loop.sm",
            expected_markers: &[
                "E0201",
                "bare break is allowed only inside while or statement loop",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/control_flow/fail/continue_outside_loop.sm",
            expected_markers: &[
                "E0201",
                "continue is allowed only inside while or statement loop",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/control_flow/fail/match_missing_fallback.sm",
            expected_markers: &["E0000", "expected '{'"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/control_flow/fail/missing_return_path.sm",
            expected_markers: &["E0201", "return type mismatch"],
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
