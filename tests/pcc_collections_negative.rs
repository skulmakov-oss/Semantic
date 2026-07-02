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
fn pcc_collections_negative_fixtures_fail_with_expected_markers() {
    let fixtures = [
        NegativeFixture {
            path: "tests/fixtures/pcc/collections/fail/map_remove_unsupported.sm",
            expected_markers: &["E0201", "unknown function", "map_remove"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/collections/fail/map_iteration_unsupported.sm",
            expected_markers: &["E0201", "Iterable", "Sequence(type)"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/collections/fail/to_text_sequence.sm",
            expected_markers: &[
                "E0201",
                "builtin 'to_text' currently supports",
                "got Sequence",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/collections/fail/to_text_map.sm",
            expected_markers: &["E0201", "builtin 'to_text' currently supports", "got Map"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/collections/fail/sequence_contains_wrong_type.sm",
            expected_markers: &["E0201", "contains", "sequence element type I32"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/collections/fail/map_set_wrong_key_type.sm",
            expected_markers: &["E0201", "map_set", "key type Bool"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/collections/fail/map_set_wrong_value_type.sm",
            expected_markers: &["E0201", "map_set", "value type I32"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/collections/fail/sequence_index_wrong_type.sm",
            expected_markers: &["E0201", "sequence indexing", "got Bool"],
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
