use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
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

// --- Repository-wide, inventory-driven guards -------------------------
//
// The tests above pin the three flagship examples named in
// `docs/spec/source_style.md`. The tests below discover the *complete*
// canonical pack from disk and cross-check it against the authoritative
// inventory table in `examples/canonical/README.md`, so a newly added
// canonical example cannot bypass style/qualification review by simply not
// being added to a hardcoded list here.

#[derive(Debug, Clone)]
struct InventoryRow {
    name: String,
    profile: String,
    qualification: String,
    expected: String,
    style_status: String,
}

/// Parses the markdown table under
/// "## Canonical Examples — Authoritative Inventory" in
/// `examples/canonical/README.md`. This is a lexical parse of a fixed table
/// shape, not a general markdown parser.
fn parse_authoritative_inventory() -> Vec<InventoryRow> {
    let doc = read("examples/canonical/README.md");
    let marker = "## Canonical Examples — Authoritative Inventory";
    let start = doc
        .find(marker)
        .unwrap_or_else(|| panic!("examples/canonical/README.md is missing {marker:?}"));

    let mut rows = Vec::new();
    let mut past_separator = false;
    for line in doc[start..].lines().skip(1) {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            if past_separator {
                break;
            }
            continue;
        }
        if trimmed.replace(' ', "").starts_with("|---") {
            past_separator = true;
            continue;
        }
        if !past_separator {
            // the header row itself
            continue;
        }
        let cells: Vec<String> = trimmed
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().to_string())
            .collect();
        if cells.len() != 7 {
            continue;
        }
        rows.push(InventoryRow {
            name: cells[1].trim_matches('`').to_string(),
            profile: cells[2].clone(),
            qualification: cells[4].clone(),
            expected: cells[5].clone(),
            style_status: cells[6].clone(),
        });
    }
    rows
}

fn discover_canonical_example_dirs() -> BTreeSet<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/canonical");
    std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("read_dir examples/canonical: {e}"))
        .map(|entry| entry.expect("dir entry"))
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect()
}

fn discover_canonical_sm_files() -> Vec<String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {dir:?}: {e}")) {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("sm") {
                out.push(path);
            }
        }
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/canonical");
    let mut out = Vec::new();
    walk(&root, &mut out);
    out.sort();
    out.into_iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect()
}

#[test]
fn canonical_inventory_matches_discovered_example_directories() {
    let rows = parse_authoritative_inventory();
    assert!(
        !rows.is_empty(),
        "authoritative inventory table in examples/canonical/README.md parsed to zero rows"
    );
    let inventory_names: BTreeSet<String> = rows.iter().map(|r| r.name.clone()).collect();
    let disk_names = discover_canonical_example_dirs();

    let undocumented: Vec<&String> = disk_names.difference(&inventory_names).collect();
    assert!(
        undocumented.is_empty(),
        "examples/canonical/ contains directories with no authoritative inventory row: {undocumented:?}"
    );

    let dangling: Vec<&String> = inventory_names.difference(&disk_names).collect();
    assert!(
        dangling.is_empty(),
        "authoritative inventory has rows pointing at missing examples/canonical/ directories: {dangling:?}"
    );
}

#[test]
fn canonical_inventory_rows_have_complete_classification() {
    for row in parse_authoritative_inventory() {
        assert!(
            !row.profile.is_empty(),
            "{}: authoritative inventory row is missing a profile classification",
            row.name
        );
        assert!(
            !row.qualification.is_empty(),
            "{}: authoritative inventory row is missing a qualification level",
            row.name
        );
        assert!(
            !row.expected.is_empty(),
            "{}: authoritative inventory row is missing an expected result",
            row.name
        );
        assert!(
            !row.style_status.is_empty(),
            "{}: authoritative inventory row is missing a Source Style v0 status",
            row.name
        );
    }
}

#[test]
fn canonical_pack_all_sm_files_are_fmt_clean_and_lexically_sound() {
    let files = discover_canonical_sm_files();
    assert!(
        !files.is_empty(),
        "discovery found zero .sm files under examples/canonical/"
    );
    for path in files {
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        assert!(
            !content.contains('\r'),
            "{path} contains CR (must be LF-only)"
        );
        assert!(!content.contains('\t'), "{path} contains a tab character");
        for (i, line) in content.lines().enumerate() {
            assert_eq!(
                line,
                line.trim_end_matches([' ', '\t']),
                "{path}:{} has trailing whitespace",
                i + 1
            );
        }
        assert!(
            content.ends_with('\n') && !content.ends_with("\n\n"),
            "{path} must have exactly one final newline"
        );
        let formatted = smc_cli::format_source_text(&content);
        assert_eq!(formatted, content, "{path} is not `smc fmt`-clean");
        let twice = smc_cli::format_source_text(&formatted);
        assert_eq!(formatted, twice, "{path}: `smc fmt` is not idempotent");
    }
}

/// Closed vocabulary of qualification paths this suite knows how to exercise
/// against the real toolchain. `classify_qualification` is the single place
/// that decides which path a row belongs to; every other test below filters
/// through it instead of re-testing `row.qualification`/`row.profile`
/// directly, so a row can never be silently skipped by every toolchain test
/// at once (the P2 this closes: a row with an unrecognized qualification
/// string used to satisfy the aggregate `checked >= N` counts on other rows
/// while never itself being run through anything).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QualificationKind {
    Executable,
    IntentionalRejection,
    LogosProfileOnly,
}

fn classify_qualification(row: &InventoryRow) -> Vec<QualificationKind> {
    let mut kinds = Vec::new();
    if row.qualification.contains("executable") {
        kinds.push(QualificationKind::Executable);
    }
    if row.qualification.contains("intentional rejection") {
        kinds.push(QualificationKind::IntentionalRejection);
    }
    if row.profile == "Logos" {
        kinds.push(QualificationKind::LogosProfileOnly);
    }
    kinds
}

#[test]
fn canonical_inventory_every_row_matches_exactly_one_recognized_qualification_path() {
    for row in parse_authoritative_inventory() {
        let kinds = classify_qualification(&row);
        assert_eq!(
            kinds.len(),
            1,
            "{}: qualification {:?} / profile {:?} matched {} recognized paths (want exactly 1: \
             Executable, IntentionalRejection, or LogosProfileOnly). A new qualification wording \
             or profile must extend `classify_qualification` with real toolchain coverage, not just \
             prose in the inventory table.",
            row.name,
            row.qualification,
            row.profile,
            kinds.len()
        );
    }
}

#[test]
fn canonical_inventory_executable_rows_check_compile_verify_and_run() {
    let mut checked = 0;
    for row in parse_authoritative_inventory() {
        if !classify_qualification(&row).contains(&QualificationKind::Executable) {
            continue;
        }
        checked += 1;
        let input = repo_path(&format!("examples/canonical/{}/src/main.sm", row.name));
        cli_ok(
            vec!["check".to_string(), input.clone()],
            &format!("smc check for {}", row.name),
        );
        cli_ok(
            vec!["run".to_string(), input.clone()],
            &format!("smc run for {}", row.name),
        );

        let dir = mk_temp_dir("smc_inventory_examples");
        let out = dir.join("out.smc");
        let out_arg = out.to_string_lossy().replace('\\', "/");
        cli_ok(
            vec![
                "compile".to_string(),
                input.clone(),
                "-o".to_string(),
                out_arg.clone(),
            ],
            &format!("smc compile for {}", row.name),
        );
        cli_ok(
            vec!["verify".to_string(), out_arg],
            &format!("smc verify for {}", row.name),
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
    assert!(
        checked >= 2,
        "expected at least two inventory rows marked executable; found {checked}"
    );
}

#[test]
fn canonical_inventory_intentional_rejection_rows_still_reject() {
    let mut checked = 0;
    for row in parse_authoritative_inventory() {
        if !classify_qualification(&row).contains(&QualificationKind::IntentionalRejection) {
            continue;
        }
        checked += 1;
        let input = repo_path(&format!("examples/canonical/{}/src/main.sm", row.name));
        let _ = cli_err(
            vec!["check".to_string(), input],
            &format!("smc check for {} (expected to keep rejecting)", row.name),
        );
    }
    assert!(
        checked >= 1,
        "expected at least one inventory row marked intentional rejection; found {checked}"
    );
}

#[test]
fn canonical_inventory_logos_rows_qualify_via_profile_path_and_stay_honestly_non_executable() {
    let mut checked = 0;
    for row in parse_authoritative_inventory() {
        if !classify_qualification(&row).contains(&QualificationKind::LogosProfileOnly) {
            continue;
        }
        checked += 1;
        let input = repo_path(&format!("examples/canonical/{}/src/main.sm", row.name));
        cli_ok(
            vec!["dump-ast".to_string(), input.clone()],
            &format!("smc dump-ast for {}", row.name),
        );
        cli_ok(
            vec![
                "dump-ir".to_string(),
                input.clone(),
                "--profile".to_string(),
                "logos".to_string(),
            ],
            &format!("smc dump-ir --profile logos for {}", row.name),
        );
        let _ = cli_err(
            vec!["check".to_string(), input],
            &format!("smc check for {} (expected honest rejection)", row.name),
        );
    }
    assert!(
        checked >= 1,
        "expected at least one inventory row with profile Logos; found {checked}"
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
