// #1933 regressions: `read_source_with_package_admission` used to
// eagerly treat any successfully-RustLike-parseable root with imports as
// a required executable bundle, before Auto authority classification
// ever ran - collapsing a genuinely `Ambiguous` root (a bare
// `Import "a.sm"`, Shared/Shared evidence for both grammars) into either
// an unrelated bundling error (if the import target wasn't valid
// RustLike) or, worse, a silent, wrong RustLike success (if it was).
// `prepare_source` now classifies the raw root exactly once via the
// canonical `sm_front::resolve_surface_authority`, before any bundling
// can run: only `RustLikeOwned(Ok(_))` ever authorizes composing an
// executable bundle, and that classification is never re-derived once
// made - a composed/bundled effective source is an internal RustLike
// composition artifact, never fed back into Auto/Logos classification.
//
// See docs/roadmap/stable_foundation/ssf09_diagnostic_authority_decision.md's
// #1933 addendum for the full frozen contract, and
// tests/dump_ir_hash_ir_surface_authority.rs /
// tests/dump_ast_hash_ast_surface_authority.rs / tests/project_check_authority.rs
// for the sibling per-command Auto-outcome matrices these tests
// complement rather than duplicate - this file focuses specifically on
// the executable-bundling-vs-authority-ordering invariant across
// multiple commands and profiles at once.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn path_str(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn cli_err(args: Vec<&str>) -> String {
    let owned: Vec<String> = args.into_iter().map(String::from).collect();
    let display = owned.join(" ");
    smc_cli::run(owned).expect_err(&format!("smc {display} unexpectedly passed"))
}

fn cli_ok(args: Vec<&str>) {
    let owned: Vec<String> = args.into_iter().map(String::from).collect();
    let display = owned.join(" ");
    smc_cli::run(owned).unwrap_or_else(|err| panic!("smc {display} failed: {err}"));
}

// A - the original #1933 repro: a bare `Import "a.sm"` root, `a.sm`
// genuinely Logos-only. Raw root is genuinely Ambiguous - every command
// must surface the canonical ambiguity diagnostic, never a bundling
// error, and must never need `a.sm` to be readable to do so (proven by
// B below, where `a.sm` deliberately doesn't exist at all).
#[test]
fn a_ambiguous_root_with_logos_only_helper_surfaces_canonical_ambiguity_everywhere() {
    let dir = mk_temp_dir("p1933_a_ambiguous_logos_helper");
    let root = dir.join("root.sm");
    let helper = dir.join("a.sm");
    std::fs::write(&root, "Import \"a.sm\"\n").expect("write root");
    std::fs::write(&helper, "Entity A:\n    state x: quad\n").expect("write helper");
    let root_p = path_str(&root);

    for cmd in ["check", "dump-ir", "hash-ir", "dump-ast", "hash-ast"] {
        let err = cli_err(vec![cmd, &root_p]);
        assert!(
            err.contains("AMBIGUOUS"),
            "{cmd}: expected the canonical ambiguity diagnostic, got: {err}"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

// B - the "worse counterexample" found during this checkpoint's DISCOVER
// phase: a genuinely Ambiguous root whose import target happens to be
// *valid RustLike*. Before #1933, `read_source_with_package_admission`
// would eagerly bundle this and silently succeed as if the root were
// unambiguously RustLike-owned (e.g. `check` reporting
// "program must define fn main()" instead of surfacing the ambiguity at
// all) - the single most dangerous shape this checkpoint's DISCOVER
// phase found, since helper content was retroactively deciding root
// ownership. MANDATORY per the implementation brief.
#[test]
fn b_ambiguous_root_stays_ambiguous_even_when_helper_would_bundle_successfully() {
    let dir = mk_temp_dir("p1933_b_worse_counterexample");
    let root = dir.join("root.sm");
    let helper = dir.join("a.sm");
    std::fs::write(&root, "Import \"a.sm\"\n").expect("write root");
    std::fs::write(&helper, "fn helper_fn() -> i32 {\n    return 1;\n}\n").expect("write helper");
    let root_p = path_str(&root);

    for cmd in ["check", "dump-ir", "hash-ir", "dump-ast", "hash-ast"] {
        let err = cli_err(vec![cmd, &root_p]);
        assert!(
            err.contains("AMBIGUOUS"),
            "{cmd}: helper content must never retroactively decide root ownership - expected \
             the canonical ambiguity diagnostic, got: {err}"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

// C - helper-independent ambiguity: the import target doesn't exist at
// all. Proves executable bundling never even attempts to resolve/read
// the target before authority terminates - if it did, this would fail
// with a missing-file error instead of the canonical ambiguity.
#[test]
fn c_ambiguous_root_never_resolves_a_nonexistent_helper() {
    let dir = mk_temp_dir("p1933_c_missing_helper_ambiguous");
    let root = dir.join("root.sm");
    std::fs::write(&root, "Import \"does_not_exist.sm\"\n").expect("write root");
    // Deliberately never create does_not_exist.sm.
    let root_p = path_str(&root);

    let err = cli_err(vec!["check", &root_p]);
    assert!(
        err.contains("AMBIGUOUS"),
        "expected the canonical ambiguity diagnostic, not a missing-helper error, got: {err}"
    );
    // Not `!err.contains("does_not_exist")`: the diagnostic legitimately
    // echoes the root's own source line (which mentions the import spec
    // as plain text) to show *where* the ambiguity is - that is
    // rendering context, not evidence the target was resolved. The
    // absence of any I/O error is the real proof the target was never
    // touched.
    assert!(
        !err.to_lowercase().contains("os error") && !err.contains("failed to resolve"),
        "the nonexistent import target must never be inspected (no I/O attempt) before \
         authority terminates, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// D - a RustLike-authoritative root whose own parse fails, with imports
// present. Must surface the RustLike surface error directly, never a
// bundler-domain "must parse on the Rust-like source path" wrapper
// (which would mean bundling ran on a root that never should have
// authorized it).
#[test]
fn d_rustlike_authoritative_root_failure_survives_with_imports_present() {
    let dir = mk_temp_dir("p1933_d_rustlike_root_failure");
    let root = dir.join("root.sm");
    let helper = dir.join("helper.sm");
    // `fn` gives unambiguous RustLike-exclusive basis; the malformed
    // parameter list makes the root's own parse fail.
    std::fs::write(&root, "Import \"helper.sm\"\n\nfn main(\n").expect("write root");
    std::fs::write(&helper, "fn score() -> i32 {\n    return 1;\n}\n").expect("write helper");
    let root_p = path_str(&root);

    let err = cli_err(vec!["check", &root_p]);
    // Positive check first (M4 mutation testing found the negative check
    // alone insufficient: a mutation that silently substitutes the
    // *helper's* own diagnostic - e.g. "program must define fn main()",
    // since helper.sm has no main - passed the negative check below
    // without ever producing the bundler wording it forbids).
    assert!(
        err.contains("expected identifier"),
        "expected the root's own RustLike parse diagnostic (a malformed `fn main(` should \
         report a parser-level 'expected identifier' failure), got: {err}"
    );
    assert!(
        !err.contains("must parse on the Rust-like source path"),
        "a root-level RustLike parse failure must never be reported as a helper/bundler \
         failure - bundling must never have run at all, got: {err}"
    );
    assert!(
        !err.contains("program must define fn main"),
        "a root-level RustLike parse failure must never be silently replaced by the \
         *helper's* own semantic diagnostic, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// E - a Logos-authoritative root whose own parse fails, with `Import`
// syntax present too (`Entity` promotes basis to Exclusive, dominating
// the earlier Shared evidence from `Import` - Decision E/F's monotonic
// basis promotion). `check_root_with_project_authority` routes any
// `LogosOwns(_)` - Ok or Err - to the project mechanism, which wraps the
// failure in its own established `E0239`/"failed to parse module"
// diagnostic (pre-#1933, pre-#1919 behavior, unrelated to bundling).
// What must NOT happen is executable bundling ever running here - its
// distinct "must parse on the Rust-like source path" wording, or any
// I/O attempt against the (nonexistent) import target, would prove
// RustLike composition was attempted on a Logos-owned root.
#[test]
fn e_logos_authoritative_root_failure_never_attempts_executable_bundling() {
    let dir = mk_temp_dir("p1933_e_logos_root_failure");
    let root = dir.join("root.sm");
    std::fs::write(&root, "Import \"nonexistent.sm\"\nEntity A\n").expect("write root");
    let root_p = path_str(&root);

    let err = cli_err(vec!["check", &root_p]);
    // Positive check first (Level 3 review found the negative-only checks
    // below insufficient on their own, same class of gap M4 mutation
    // testing found in test D): the root's own Logos parse genuinely
    // fails (`Entity A` with no trailing `:`), and that failure must
    // reach the pre-existing, unrelated E0239 project-mechanism wrapper -
    // never silently become an empty or wrong-domain error.
    assert!(
        err.contains("E0239") && err.contains("expected ':'"),
        "expected the pre-existing E0239-wrapped Logos parse failure (missing ':' after \
         'Entity A'), got: {err}"
    );
    assert!(
        !err.contains("must parse on the Rust-like source path"),
        "a Logos-owned root must never trigger executable bundling's own error wording, \
         got: {err}"
    );
    assert!(
        !err.to_lowercase().contains("os error") && !err.contains("failed to resolve"),
        "the unrelated import target must never be I/O-resolved for a Logos-owned root, \
         got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// I - explicit `--profile logos` must never enter executable bundling,
// even against a root that would otherwise bundle successfully under
// Auto/explicit-RustLike. Covers all five profile-capable commands.
//
// `dump-ir`/`hash-ir` render real Logos IR text directly and succeed.
// `compile`/`dump-bytecode`/`hash-smc` route explicit Logos through
// `compile_program_to_ir_with_options_and_profile`, which cannot lower
// Logos law IR into SemCode function IR at all ("SemCode function IR
// requires RustLike frontend") - a genuine, pre-existing, orthogonal
// domain limitation, not something #1933 changes. What matters here is
// that this failure is *that* limitation, never a bundler-domain error
// or an attempted read of the (deliberately bundle-eligible) helper.
#[test]
fn i_explicit_logos_never_bundles_across_all_five_profile_capable_commands() {
    let dir = mk_temp_dir("p1933_i_explicit_logos_no_bundle");
    let root = dir.join("root.sm");
    let helper = dir.join("a.sm");
    // Ambiguous under Auto (Shared/Shared), but importantly: under
    // explicit Logos, this text is valid (trivially empty) Logos syntax
    // - if bundling were ever attempted, it would either fail chasing
    // `a.sm` or produce RustLike-flavored output, neither of which is
    // "succeeds/fails as empty Logos".
    std::fs::write(&root, "Import \"a.sm\"\n").expect("write root");
    std::fs::write(&helper, "fn helper_fn() -> i32 {\n    return 1;\n}\n").expect("write helper");
    let root_p = path_str(&root);

    cli_ok(vec!["dump-ir", &root_p, "--profile", "logos"]);
    cli_ok(vec!["hash-ir", &root_p, "--profile", "logos"]);

    for (cmd, extra_args) in [
        ("dump-bytecode", vec![]),
        ("hash-smc", vec![]),
        ("compile", vec!["-o", "unused.smc"]),
    ] {
        let mut args = vec![cmd, &root_p];
        args.extend(extra_args);
        args.extend(["--profile", "logos"]);
        let err = cli_err(args);
        assert!(
            err.contains("requires RustLike frontend"),
            "{cmd}: expected the pre-existing Logos-to-SemCode domain limitation, not a \
             bundler-related failure, got: {err}"
        );
        assert!(
            !err.contains("must parse on the Rust-like source path") && !err.contains("a.sm"),
            "{cmd}: must never attempt executable bundling or reference the helper under \
             explicit Logos, got: {err}"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

// J - the hard-explicit-RustLike `run` consumer (single-file-argument
// form, `cmd_run_controlled_observation` - never had an Auto or Logos
// path at all) still correctly composes an executable bundle for a
// legitimate T9-shaped program, proving the bundling gate change didn't
// regress its one and only code path.
#[test]
fn j_hard_rustlike_consumer_still_composes_legitimate_bundle() {
    let dir = mk_temp_dir("p1933_j_hard_rustlike_consumer");
    let root = dir.join("main.sm");
    let helper = dir.join("helper.sm");
    std::fs::write(
        &root,
        "Import \"helper.sm\"\n\nfn main() {\n    let value: i32 = score(1);\n    assert(value == 1);\n    return;\n}\n",
    )
    .expect("write root");
    std::fs::write(
        &helper,
        "fn score(value: i32) -> i32 {\n    return value;\n}\n",
    )
    .expect("write helper");
    let root_p = path_str(&root);

    cli_ok(vec!["run", &root_p]);

    let _ = std::fs::remove_dir_all(&dir);
}
