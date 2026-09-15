// #1919 Stage 2B regressions: `smc check`/`smc lint`/`smc watch` must
// distinguish "the multi-module project mechanism does not apply to this
// root" from "the mechanism applied and returned an authoritative
// failure." Only the first may fall back to the narrower single-file
// check; the second must fail closed, exactly as
// `docs/roadmap/stable_foundation/ssf09_diagnostic_authority_decision.md`
// (Decision A) requires. See `crates/smc-cli/src/app.rs`'s
// `check_root_with_project_authority`.

use std::path::PathBuf;
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

fn path_str(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn cli_ok(command: &str, path: &std::path::Path) {
    let p = path_str(path);
    smc_cli::run(vec![command.to_string(), p.clone()])
        .unwrap_or_else(|err| panic!("smc {command} failed for {p}: {err}"));
}

fn cli_err(command: &str, path: &std::path::Path) -> String {
    let p = path_str(path);
    smc_cli::run(vec![command.to_string(), p.clone()])
        .expect_err(&format!("smc {command} unexpectedly passed for {p}"))
}

// T1 - genuinely single-file input: an ordinary RustLike program with no
// project structure at all must continue to work through the
// (legitimate, non-fallback) single-file mechanism. Proves the fix does
// not simply delete the ability to check standalone files.
#[test]
fn t1_genuinely_single_file_input_still_works() {
    let dir = mk_temp_dir("p1919_t1_single_file");
    let root = dir.join("main.sm");
    std::fs::write(&root, "fn main() {\n    return;\n}\n").expect("write root");

    cli_ok("check", &root);

    let _ = std::fs::remove_dir_all(&dir);
}

// T7 - mechanism genuinely not applicable: same underlying fact as T1,
// verified through `smc lint` too (a second, independent caller of the
// same seam) - proves the legitimate fallback case identified during
// discovery is preserved at every call site, not just `check`.
#[test]
fn t7_mechanism_not_applicable_fallback_still_occurs_via_lint() {
    let dir = mk_temp_dir("p1919_t7_lint_single_file");
    let root = dir.join("main.sm");
    std::fs::write(&root, "fn main() {\n    return;\n}\n").expect("write root");

    cli_ok("lint", &root);

    let _ = std::fs::remove_dir_all(&dir);
}

// T2 - project success: a valid multi-module project must still succeed
// through the real project mechanism.
#[test]
fn t2_valid_project_succeeds_through_project_mechanism() {
    let dir = mk_temp_dir("p1919_t2_project_ok");
    let root = dir.join("root.sm");
    let dep = dir.join("dep.sm");
    std::fs::write(
        &root,
        "\nImport \"dep.sm\"\nLaw \"R\" [priority 1]:\n    When true -> System.recovery()\n",
    )
    .expect("write root");
    std::fs::write(&dep, "\nEntity A:\n    state x: quad\n").expect("write dep");

    cli_ok("check", &root);

    let _ = std::fs::remove_dir_all(&dir);
}

// T3 - cyclic import / dependency graph failure: the project-level
// cycle error must surface, and the single-file fallback must not run
// (proven by the message containing the project mechanism's own cyclic-
// import diagnosis, not a RustLike-flavored "expected top-level" error).
#[test]
fn t3_cyclic_import_surfaces_project_error_not_fallback() {
    let dir = mk_temp_dir("p1919_t3_cycle");
    let root = dir.join("root.sm");
    let a = dir.join("a.sm");
    std::fs::write(&root, "\nImport \"a.sm\"\nEntity A:\n    state x: quad\n").expect("write root");
    std::fs::write(&a, "\nImport \"root.sm\"\nEntity B:\n    state y: quad\n")
        .expect("write a (completes the cycle)");

    let err = cli_err("check", &root);
    assert!(
        err.contains("cyclic import detected") || err.contains("E0238"),
        "expected the project mechanism's own cyclic-import error, got: {err}"
    );
    assert!(
        !err.contains("expected top-level"),
        "must not be reinterpreted as a single-file RustLike error: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// T4 - duplicate symbol / namespace collision: an alias collision across
// two imported modules is a project-level namespace error. It must
// survive, with no narrower retry.
#[test]
fn t4_namespace_collision_survives_no_narrower_retry() {
    let dir = mk_temp_dir("p1919_t4_alias_collision");
    let root = dir.join("root.sm");
    let a = dir.join("a.sm");
    let b = dir.join("b.sm");
    std::fs::write(
        &root,
        "\nImport \"a.sm\" as Core\nImport \"b.sm\" as Core\nLaw \"R\" [priority 1]:\n    When true -> System.recovery()\n",
    )
    .expect("write root");
    std::fs::write(
        &a,
        "\nLaw \"A\" [priority 1]:\n    When true -> System.recovery()\n",
    )
    .expect("write a");
    std::fs::write(
        &b,
        "\nLaw \"B\" [priority 1]:\n    When true -> System.recovery()\n",
    )
    .expect("write b");

    let err = cli_err("check", &root);
    assert!(
        err.contains("E0241"),
        "expected the project mechanism's own alias-collision error, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// T5 - imported-module failure: a failure originating from an imported
// module (not the root) must not be replaced by a root-file single-file
// result. The root alone, checked in isolation, would succeed trivially
// (it has no content of its own beyond the import) - proving this is
// not accidentally passing for an unrelated reason.
#[test]
fn t5_imported_module_failure_not_replaced_by_root_only_result() {
    let dir = mk_temp_dir("p1919_t5_imported_failure");
    let root = dir.join("root.sm");
    let a = dir.join("a.sm");
    std::fs::write(
        &root,
        "\nImport \"a.sm\"\nLaw \"R\" [priority 1]:\n    When true -> System.recovery()\n",
    )
    .expect("write root");
    // Duplicate Entity inside the imported module - a genuine semantic
    // error (E0220) that only exists once the module is actually loaded.
    std::fs::write(
        &a,
        "\nEntity A:\n    state x: quad\nEntity A:\n    prop y: bool\n",
    )
    .expect("write a");

    let err = cli_err("check", &root);
    assert!(
        err.contains("E0220") || err.contains("duplicate Entity"),
        "expected the imported module's own duplicate-Entity error, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// T6 - project recognized, missing dependency/module: must remain a
// project-level failure.
#[test]
fn t6_missing_dependency_remains_project_level_failure() {
    let dir = mk_temp_dir("p1919_t6_missing_dep");
    let root = dir.join("root.sm");
    std::fs::write(
        &root,
        "\nImport \"missing.sm\"\nEntity A:\n    state x: quad\n",
    )
    .expect("write root");

    let err = cli_err("check", &root);
    assert!(
        err.contains("failed to resolve import") || err.contains("missing.sm"),
        "expected the project mechanism's own missing-dependency error, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// T8 - the most important regression: an adversarial fixture where the
// project path is an authoritative failure AND the single-file path
// (checking the root's raw text alone, ignoring the missing import
// target) would silently succeed. The project failure must win.
#[test]
fn t8_project_failure_wins_over_fallback_success() {
    let dir = mk_temp_dir("p1919_t8_adversarial");
    let root = dir.join("root.sm");
    // The root's own text is syntactically complete and would parse and
    // analyze successfully in total isolation - `Import` lines carry no
    // requirement that their target exist as far as single-file syntax
    // checking is concerned.
    std::fs::write(
        &root,
        "\nImport \"missing.sm\"\nEntity A:\n    state x: quad\n",
    )
    .expect("write root");

    // Control: prove the single-file path really would succeed on this
    // exact text, so the test is not vacuous.
    let single_file_src = std::fs::read_to_string(&root).expect("read root");
    let profile = sm_profile::ParserProfile::foundation_default();
    assert!(
        sm_sema::check_source_with_profile(&single_file_src, &profile).is_ok(),
        "control check failed: the single-file path must succeed on this text for the \
         adversarial fixture to test anything"
    );

    // The real CLI path must still fail - the project-level missing-
    // dependency error must win over the single-file path's success.
    let err = cli_err("check", &root);
    assert!(
        err.contains("failed to resolve import") || err.contains("missing.sm"),
        "the project failure must win over the single-file fallback's success, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// T9 - regression fixture for the cross-grammar correction: a genuine
// RustLike program using the pre-existing "executable helper import"
// convention (`Import "helper.sm"` followed by real RustLike content)
// must still be admitted. `Import` alone is Shared evidence for Logos
// too, so a Logos-admission-only applicability probe misclassifies this
// as a Logos project entry and routes it into the strict Logos-only
// project loader, which then fails on ordinary RustLike syntax. This is
// exactly the shape of `examples/canonical/wave2_local_helper_import/
// src/main.sm`, which surfaced the regression via the workspace's own
// `canonical_examples` test before this dedicated regression existed.
#[test]
fn t9_executable_helper_import_convention_still_admitted() {
    let dir = mk_temp_dir("p1919_t9_helper_import");
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

    cli_ok("check", &root);

    let _ = std::fs::remove_dir_all(&dir);
}

// P10 - Decision F final consumer migration (#1931): a real
// `Exclusive`/`Exclusive` cross-grammar conflict must remain terminal
// ambiguity, never an implicit project-mechanism win. Before this
// checkpoint the routing table happened to get this right by
// construction (it mirrored `resolve_surface_authority` by hand); this
// is the forward-looking regression that would catch a future
// projection defect now that routing consumes the canonical resolver
// directly instead.
#[test]
fn p10_real_exclusive_vs_exclusive_ambiguity_is_not_routed_to_project_loader() {
    let dir = mk_temp_dir("p1931_p10_ambiguous");
    let root = dir.join("root.sm");
    std::fs::write(
        &root,
        "fn main() {\n    return;\n}\n\nEntity A:\n    state x: quad\n",
    )
    .expect("write root");

    let err = cli_err("check", &root);
    assert!(
        err.contains("AMBIGUOUS"),
        "expected the canonical, sm-sema-owned ambiguity diagnostic, got: {err}"
    );
    assert!(
        !err.contains("cyclic import") && !err.contains("failed to resolve import"),
        "must not be misrouted through the project loader's own diagnostics: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// P11 - Decision F final consumer migration (#1931): genuinely blank/
// comment-only input (`NoClaim`/`NoClaim`) must remain a terminal
// no-surface-claim result, never routed into the project loader (which
// would fail closed for the wrong reason - "no Logos entry to recurse
// imports from" - rather than surfacing the actual no-evidence outcome).
#[test]
fn p11_blank_input_no_surface_claim_is_not_routed_to_project_loader() {
    let dir = mk_temp_dir("p1931_p11_no_claim");
    let root = dir.join("root.sm");
    std::fs::write(&root, "// just a comment\n").expect("write root");

    let err = cli_err("check", &root);
    assert!(
        err.contains("NO SURFACE CLAIM"),
        "expected the canonical, sm-sema-owned no-surface-claim diagnostic, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
