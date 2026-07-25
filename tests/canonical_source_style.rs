use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_path(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
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

/// The three examples `docs/spec/source_style.md` names as its canonical
/// demonstrations: two Rust-like executable programs and one Logos profile
/// program.
const STYLE_CONTRACT_EXAMPLES: [&str; 3] = [
    "examples/canonical/match_control_flow/src/main.sm",
    "examples/canonical/rule_state_decision/src/main.sm",
    "examples/canonical/quad_cycle_logos/src/main.sm",
];

const RUSTLIKE_STYLE_CONTRACT_EXAMPLES: [&str; 2] = [
    "examples/canonical/match_control_flow/src/main.sm",
    "examples/canonical/rule_state_decision/src/main.sm",
];

const LOGOS_STYLE_CONTRACT_EXAMPLE: &str = "examples/canonical/quad_cycle_logos/src/main.sm";

#[test]
fn style_contract_examples_pass_fmt_check() {
    for rel in STYLE_CONTRACT_EXAMPLES {
        let original = read(rel);
        let formatted = smc_cli::format_source_text(&original);
        assert_eq!(
            formatted, original,
            "{rel} is not already `smc fmt`-clean (Section A invariants)"
        );
    }
}

#[test]
fn style_contract_examples_are_fmt_idempotent() {
    for rel in STYLE_CONTRACT_EXAMPLES {
        let original = read(rel);
        let once = smc_cli::format_source_text(&original);
        let twice = smc_cli::format_source_text(&once);
        assert_eq!(once, twice, "{rel}: formatting is not idempotent");
    }
}

#[test]
fn style_contract_examples_have_no_trailing_whitespace() {
    for rel in STYLE_CONTRACT_EXAMPLES {
        let content = read(rel);
        for (i, line) in content.lines().enumerate() {
            assert_eq!(
                line,
                line.trim_end_matches([' ', '\t']),
                "{rel}:{} has trailing whitespace",
                i + 1
            );
        }
    }
}

#[test]
fn style_contract_examples_have_no_tab_indentation() {
    for rel in STYLE_CONTRACT_EXAMPLES {
        let content = read(rel);
        assert!(
            !content.contains('\t'),
            "{rel} contains a tab character; Section A requires no tab characters. \
             (This does not by itself prove 4-space nesting depth, which is a Section B \
             canonical presentation rule, not machine-checked.)"
        );
    }
}

#[test]
fn style_contract_examples_have_exactly_one_final_newline() {
    for rel in STYLE_CONTRACT_EXAMPLES {
        let content = read(rel);
        assert!(content.ends_with('\n'), "{rel} is missing a final newline");
        assert!(
            !content.ends_with("\n\n"),
            "{rel} has trailing blank line(s)"
        );
    }
}

#[test]
fn style_contract_rustlike_examples_check_compile_verify_and_run() {
    for rel in RUSTLIKE_STYLE_CONTRACT_EXAMPLES {
        let input = repo_path(rel);
        cli_ok(
            vec!["check".to_string(), input.clone()],
            &format!("smc check for {input}"),
        );
        cli_ok(
            vec!["run".to_string(), input.clone()],
            &format!("smc run for {input}"),
        );

        let dir = mk_temp_dir("smc_style_contract_examples");
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
}

#[test]
fn style_contract_logos_example_qualifies_via_dump_ast_and_dump_ir() {
    let input = repo_path(LOGOS_STYLE_CONTRACT_EXAMPLE);
    cli_ok(
        vec!["dump-ast".to_string(), input.clone()],
        &format!("smc dump-ast for {input}"),
    );
    cli_ok(
        vec![
            "dump-ir".to_string(),
            input,
            "--profile".to_string(),
            "logos".to_string(),
        ],
        "smc dump-ir --profile logos",
    );
}

#[test]
fn style_contract_logos_example_is_honestly_rejected_by_rustlike_check() {
    let input = repo_path(LOGOS_STYLE_CONTRACT_EXAMPLE);
    // The Logos profile does not compile/verify/run through the Rust-like
    // SemCode/VM path. `smc check` must keep failing on it; a pass here would
    // mean unsupported/proposed syntax silently became executable.
    let _ = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
}

#[test]
fn style_contract_logos_readme_declares_its_non_executable_status() {
    let readme = read("examples/canonical/quad_cycle_logos/README.md");
    assert!(
        readme.contains("not** `check`/`compile`/`verify`/`run`-qualified"),
        "quad_cycle_logos/README.md must explicitly disclaim check/compile/verify/run qualification"
    );
}

#[test]
fn style_contract_examples_do_not_regress_to_verbose_pre_contract_shape() {
    let match_control_flow = read("examples/canonical/match_control_flow/src/main.sm");
    assert!(
        match_control_flow.contains("if slot == 0 { return N; }"),
        "match_control_flow/src/main.sm drifted away from the compact guard-return contract"
    );

    let rule_state_decision = read("examples/canonical/rule_state_decision/src/main.sm");
    assert!(
        rule_state_decision.contains("if ctx.override_state == T { return Result::Ok(T); }"),
        "rule_state_decision/src/main.sm drifted away from the compact guard-return contract"
    );

    let quad_cycle_logos = read("examples/canonical/quad_cycle_logos/src/main.sm");
    assert!(
        quad_cycle_logos.contains("):\n\nEntity") && quad_cycle_logos.contains("\n\nLaw"),
        "quad_cycle_logos/src/main.sm lost the required blank line between System/Entity/Law blocks"
    );
}

#[test]
fn source_style_document_does_not_overclaim_indentation_enforcement() {
    // Regression guard for a reviewed P2: 4-space nesting depth is a Section B
    // canonical presentation rule, not a Section A machine-checked invariant.
    // Neither `smc fmt` nor this test suite validates nesting depth -- only
    // the absence of tab characters is machine-checked.
    let doc = read("docs/spec/source_style.md");
    assert!(
        !doc.contains("Indentation step is 4 spaces per nesting level"),
        "docs/spec/source_style.md must not list 4-space nesting as a Section A required invariant"
    );
    assert!(
        doc.contains("does not structurally validate indentation depth"),
        "docs/spec/source_style.md must keep disclosing that no tool validates nesting depth"
    );
    assert!(
        doc.contains("no tool currently validates nesting depth"),
        "docs/spec/source_style.md's Section B.2 must keep disclosing that no tool validates nesting depth"
    );
}

#[test]
fn source_style_document_has_required_classification_sections() {
    let doc = read("docs/spec/source_style.md");
    for required in [
        "Semantic Canonical Source Style v0",
        "Required lexical/file invariant",
        "Canonical presentation rule",
        "Permitted alternative author style",
        "Future formatter behavior",
        "Rust-like executable surface",
        "Logos declarative surface",
        "## A. Required Lexical/File Invariants",
        "## B. Canonical Presentation Rules",
        "## C. Permitted Alternative Author Style",
        "## D. Formatter Contract",
    ] {
        assert!(
            doc.contains(required),
            "docs/spec/source_style.md is missing required section/text: {required:?}"
        );
    }
}
