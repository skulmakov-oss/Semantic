//! SSF-07 (issue #1578) frozen compatibility contract for `std.text`.
//!
//! This is the single canonical evidence pointer for std.text's Selected
//! surface: `+`/`==` restricted to text-text operands, and `to_text`
//! restricted to {text, bool, i32, u32, quad}. It reuses the same `.sm`
//! fixtures already exercised by tests/pcc_text_negative.rs,
//! tests/pcc8_stdlib_diagnostics.rs, and tests/pcc3_text_core_gate.rs
//! (those files keep serving their own PCC-track qualification purpose),
//! plus new fixtures closing the marginal gap this slice found: `+`
//! against u32/map/record operands had no coverage anywhere in the repo.

use std::path::PathBuf;

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|e| panic!("{context} unexpectedly failed: {e}"));
}

fn cli_err(args: Vec<String>, context: &str) -> String {
    smc_cli::run(args).expect_err(&format!("{context} unexpectedly passed"))
}

struct NegativeFixture {
    path: &'static str,
    expected_markers: &'static [&'static str],
}

#[test]
fn std_text_plus_operator_rejects_every_non_text_operand() {
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
        // Closes the marginal gap this slice found: u32/map/record operands
        // against `+` had no test anywhere in the repo before this file.
        NegativeFixture {
            path: "tests/fixtures/pcc/text/fail/text_plus_u32.sm",
            expected_markers: &[
                "E0201",
                "text concatenation currently admits only text + text operands",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/text/fail/text_plus_map.sm",
            expected_markers: &[
                "E0201",
                "text concatenation currently admits only text + text operands",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/text/fail/text_plus_record.sm",
            expected_markers: &[
                "E0201",
                "text concatenation currently admits only text + text operands",
            ],
        },
        // Confirmed empirically that a Sequence operand takes a different
        // code path (the ordered-sequence operator gate fires before the
        // text-specific check ever runs) and therefore reports a different
        // message. Pinned as its own distinct case rather than assumed to
        // match the others.
        NegativeFixture {
            path: "tests/fixtures/pcc/text/fail/text_plus_sequence.sm",
            expected_markers: &[
                "E0201",
                "ordered sequence values are not part of the current M8.3 Wave 1 operator surface",
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

#[test]
fn std_text_to_text_rejects_every_unadmitted_input() {
    let fixtures = [
        NegativeFixture {
            path: "tests/fixtures/pcc8_stdlib_diagnostics/negative_to_text_record.sm",
            expected_markers: &[
                "E0201",
                "builtin 'to_text' does not yet support record type",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc8_stdlib_diagnostics/negative_to_text_map.sm",
            expected_markers: &["E0201", "builtin 'to_text' currently supports"],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc/text/fail/to_text_sequence.sm",
            expected_markers: &[
                "E0201",
                "builtin 'to_text' currently supports",
                "got Sequence",
            ],
        },
        NegativeFixture {
            path: "tests/fixtures/pcc8_stdlib_diagnostics/negative_to_text_wrong_arity.sm",
            expected_markers: &[
                "E0201",
                "builtin 'to_text' takes exactly one positional argument",
            ],
        },
    ];

    for fixture in fixtures {
        let input = repo_path(fixture.path);
        let err = cli_err(
            vec!["check".to_string(), input.clone()],
            &format!("smc check for {input}"),
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

#[test]
fn std_text_positive_surface_typechecks_and_runs() {
    for path in [
        "tests/fixtures/pcc3_text/positive_text_concat.sm",
        "tests/fixtures/pcc3_text/positive_text_equality.sm",
        "tests/fixtures/pcc8_stdlib/positive_to_text_basic_types.sm",
    ] {
        let input = repo_path(path);
        cli_ok(
            vec!["check".to_string(), input.clone()],
            &format!("smc check for {input}"),
        );
        cli_ok(
            vec!["run".to_string(), input.clone()],
            &format!("smc run for {input}"),
        );
    }
}
