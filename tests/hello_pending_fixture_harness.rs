use std::path::PathBuf;

use sm_front::hello_parser::parse_hello_file;
use sm_front::hello_sema::validate_hello_file;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingHelloFixtureClass {
    ParseAndSemaCanonical,
    ParseAndSemaSecondary,
    ParserReject,
    ParserRejectOrSemaReject,
}

#[derive(Debug, Clone, Copy)]
struct PendingHelloFixtureCase {
    rel: &'static str,
    expected: PendingHelloFixtureClass,
}

const CASES: &[PendingHelloFixtureCase] = &[
    PendingHelloFixtureCase {
        rel: "tests/fixtures/pending/hello/positive_hello_verbose_directional.sm",
        expected: PendingHelloFixtureClass::ParseAndSemaCanonical,
    },
    PendingHelloFixtureCase {
        rel: "tests/fixtures/pending/hello/positive_hello_minimal_observe_directional.sm",
        expected: PendingHelloFixtureClass::ParseAndSemaSecondary,
    },
    PendingHelloFixtureCase {
        rel: "tests/fixtures/pending/hello/negative_hello_print_legacy_canonical.sm",
        expected: PendingHelloFixtureClass::ParserReject,
    },
    PendingHelloFixtureCase {
        rel: "tests/fixtures/pending/hello/negative_hello_observe_non_text_payload.sm",
        expected: PendingHelloFixtureClass::ParserReject,
    },
    PendingHelloFixtureCase {
        rel: "tests/fixtures/pending/hello/negative_hello_require_side_effect_shape.sm",
        expected: PendingHelloFixtureClass::ParserReject,
    },
    PendingHelloFixtureCase {
        rel: "tests/fixtures/pending/hello/negative_hello_general_io_shape.sm",
        expected: PendingHelloFixtureClass::ParserReject,
    },
];

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn fixture_text(rel: &str) -> String {
    std::fs::read_to_string(repo_path(rel))
        .unwrap_or_else(|err| panic!("failed to read fixture {rel}: {err}"))
}

fn registry_text() -> String {
    fixture_text("docs/language/semantic_hello_fixtures.md")
}

fn classify_fixture(rel: &str) -> PendingHelloFixtureClass {
    let input = fixture_text(rel);
    match parse_hello_file(&input) {
        Err(_) => PendingHelloFixtureClass::ParserReject,
        Ok(parsed) => match validate_hello_file(parsed) {
            Ok(checked) if checked.report.canonical_shape => {
                PendingHelloFixtureClass::ParseAndSemaCanonical
            }
            Ok(checked) if checked.report.secondary_shape => {
                PendingHelloFixtureClass::ParseAndSemaSecondary
            }
            Ok(_) => panic!("unexpected Hello sema classification for {rel}"),
            Err(_) => PendingHelloFixtureClass::ParserRejectOrSemaReject,
        },
    }
}

#[test]
fn hello_pending_fixture_harness_reads_registry_and_classifies_all_fixtures() {
    let registry = registry_text();

    for case in CASES {
        assert!(
            registry.contains(case.rel),
            "fixture registry should mention {}",
            case.rel
        );

        let actual = classify_fixture(case.rel);
        assert_eq!(
            actual, case.expected,
            "unexpected pending Hello classification for {}",
            case.rel
        );
    }
}
