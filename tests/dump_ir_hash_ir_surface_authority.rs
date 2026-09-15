// #1931 regressions: `smc dump-ir`/`smc hash-ir` under `CompileProfile::Auto`
// used to independently implement
//
//   if let Ok(logos) = parse_logos_program_with_profile(...) { Logos IR }
//   else { compile_program_to_ir_with_options_and_profile(..., RustLike, ...) }
//
// which collapses an authoritative Logos parse *failure*, a genuine
// cross-grammar `Ambiguous` tie, and a genuine `NoSurfaceClaim` input into
// "try RustLike anyway" - the same shape of defect #1920 already repaired
// in `sm-ir`'s own `Auto` path. Both commands now route through one shared
// private helper (`render_ir_for_profile` in `crates/smc-cli/src/app.rs`)
// that consumes `sm_front::resolve_surface_authority` directly. Every test
// below runs BOTH `dump-ir` and `hash-ir` against the same fixture and
// asserts the same outcome shape for each - proving the two commands do
// not (and structurally cannot easily) diverge, since they share one seam.

use sm_front::{
    admit_logos_program_with_profile, admit_program_with_profile, lex, GrammarAdmission,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn foundation_profile() -> sm_profile::ParserProfile {
    sm_profile::ParserProfile::foundation_default()
}

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

// Asserts both `dump-ir` and `hash-ir` succeed on the same source content -
// each against its own fresh path, so neither command's own execution can
// be silently skipped by reading back the other's on-disk IR cache entry.
//
// `dump-ir`/`hash-ir` share one on-disk IR cache keyed by
// (canonicalized path, content fingerprint, profile, opt) - see
// `ir_pack_key` in `crates/smc-cli/src/app.rs`. Calling both commands
// against the *same* path would give them the *same* cache key: the
// first command populates the cache, and the second would be served
// entirely from that cache entry, never reaching `render_ir_for_profile`
// itself. Two distinct paths with identical content guarantee two
// distinct cache keys, so each command's own cold path genuinely runs.
fn both_ok(dir: &std::path::Path, src: &str) {
    let dump_path = dir.join("dump_target.sm");
    let hash_path = dir.join("hash_target.sm");
    std::fs::write(&dump_path, src).expect("write dump-ir fixture");
    std::fs::write(&hash_path, src).expect("write hash-ir fixture");
    cli_ok("dump-ir", &dump_path);
    cli_ok("hash-ir", &hash_path);
}

// Asserts both `dump-ir` and `hash-ir` fail on the same input and returns
// (dump_ir_err, hash_ir_err) for further assertions.
fn both_err(path: &std::path::Path) -> (String, String) {
    (cli_err("dump-ir", path), cli_err("hash-ir", path))
}

// IR-A1 - genuine RustLike success under Auto: both commands must still
// succeed on an ordinary RustLike program.
#[test]
fn ir_a1_rustlike_success_under_auto() {
    let dir = mk_temp_dir("ir1931_a1_rustlike_ok");

    both_ok(&dir, "fn main() {\n    return;\n}\n");

    let _ = std::fs::remove_dir_all(&dir);
}

// IR-A2 - genuine Logos success under Auto: both commands must render
// real Logos IR (proving dump-ir/hash-ir's whole reason to exist over
// sm-ir's own RustLike-only Auto path - the ability to show/hash actual
// Logos IR, not a generic redirect).
#[test]
fn ir_a2_logos_success_under_auto() {
    let dir = mk_temp_dir("ir1931_a2_logos_ok");

    both_ok(
        &dir,
        "\nEntity A:\n    state x: quad\nLaw \"L\" [priority 1]:\n    When true -> System.recovery()\n",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// IR-A3 - authoritative Logos error: a genuine positive Logos claim whose
// own parse fails must survive exactly, on both commands, with zero
// RustLike-shaped replacement.
#[test]
fn ir_a3_authoritative_logos_error_survives_on_both_commands() {
    let dir = mk_temp_dir("ir1931_a3_logos_err");
    let root = dir.join("main.sm");
    // Missing ':' after the Entity name - genuine Exclusive evidence,
    // genuine parse failure (same control-checked shape as
    // `project_authority_exclusive_evidence_with_parse_failure_stays_applied`
    // in `crates/smc-cli/src/app.rs`).
    let src = "Entity A\n";
    std::fs::write(&root, src).expect("write root");

    // Control: prove the fixture really is Logos Exclusive(Err), not
    // NoClaim or something else - or this test would not prove what it
    // claims to.
    let profile = foundation_profile();
    let tokens = lex(src).expect("lex");
    let logos = admit_logos_program_with_profile(src, &tokens, &profile);
    assert!(
        matches!(logos, GrammarAdmission::Exclusive(Err(_))),
        "control check: fixture must produce Logos Exclusive(Err), got: {logos:?}"
    );

    let (dump_err, hash_err) = both_err(&root);
    for (label, err) in [("dump-ir", &dump_err), ("hash-ir", &hash_err)] {
        assert!(
            !err.contains("expected top-level") && !err.contains("expected primary expression"),
            "{label}: must not be replaced by a RustLike-shaped parse error, got: {err}"
        );
    }
    assert_eq!(
        dump_err, hash_err,
        "dump-ir and hash-ir must surface the identical authoritative Logos error"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// IR-A4 - real Exclusive/Exclusive ambiguity: the exact mandated fixture.
// Both commands must produce a terminal ambiguity, never a RustLike
// fallback error.
#[test]
fn ir_a4_real_exclusive_vs_exclusive_ambiguity_on_both_commands() {
    let dir = mk_temp_dir("ir1931_a4_ambiguous_exclusive");
    let root = dir.join("main.sm");
    let src = "fn main() {\n    return;\n}\n\nEntity A:\n    state x: quad\n";
    std::fs::write(&root, src).expect("write root");

    // Control: prove both grammars really do produce Exclusive(Err) for
    // this exact fixture - or this fixture would not distinguish
    // "ambiguity detected" from "RustLike fallback happened to also fail".
    let profile = foundation_profile();
    let tokens = lex(src).expect("lex");
    let logos = admit_logos_program_with_profile(src, &tokens, &profile);
    let rustlike = admit_program_with_profile(src, &tokens, &profile);
    assert!(
        matches!(logos, GrammarAdmission::Exclusive(Err(_))),
        "control check: fixture must produce Logos Exclusive(Err), got: {logos:?}"
    );
    assert!(
        matches!(rustlike, GrammarAdmission::Exclusive(Err(_))),
        "control check: fixture must produce RustLike Exclusive(Err), got: {rustlike:?}"
    );

    let (dump_err, hash_err) = both_err(&root);
    for (label, err) in [("dump-ir", &dump_err), ("hash-ir", &hash_err)] {
        assert!(
            err.contains("AMBIGUOUS"),
            "{label}: expected terminal ambiguity, got: {err}"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

// IR-A5 (Shared/Shared ambiguity) is NOT exercised here through the
// real CLI. Discovery during this checkpoint: `read_source_with_
// package_admission` (`crates/smc-cli/src/executable_bundle.rs`) - the
// source-read step every command including `dump-ir`/`hash-ir` and
// `check` calls before any Auto/authority classification ever runs -
// eagerly parses the root as RustLike and, if it parses with any
// `Import`, treats every import target as a RustLike "executable
// helper" module to bundle inline. A bare `Import "a.sm"` with no other
// content parses as valid RustLike (that is exactly why it is Shared
// evidence for RustLike at all), so bundling always triggers; if `a.sm`
// is not itself valid RustLike (as a genuine Logos-only Shared/Shared
// fixture requires), bundling fails with its own unrelated
// "executable helper module ... must parse on the Rust-like source
// path" error - before `render_ir_for_profile`'s `resolve_surface_
// authority` call ever sees the text. This is a pre-existing,
// orthogonal defect in the executable-bundle read step (it affects
// `smc check` identically, confirmed during this checkpoint's own
// discovery), not the Auto-routing defect #1931 targets, and out of
// this PR's scope to fix. `render_ir_for_profile`'s own Shared/Shared
// handling is proven correct directly (bypassing the bundling step) by
// `ir_a5_shared_vs_shared_ambiguity_via_render_ir_for_profile` in
// `crates/smc-cli/src/app.rs`'s own test module.

// IR-A6 - NoSurfaceClaim: blank/comment-only source. Both commands must
// produce the terminal no-surface-claim diagnostic, never a forced
// RustLike "missing main"/parser error.
#[test]
fn ir_a6_no_surface_claim_on_both_commands() {
    let dir = mk_temp_dir("ir1931_a6_no_claim");
    let root = dir.join("main.sm");
    std::fs::write(&root, "// just a comment\n").expect("write root");

    let (dump_err, hash_err) = both_err(&root);
    for (label, err) in [("dump-ir", &dump_err), ("hash-ir", &hash_err)] {
        assert!(
            err.contains("NO SURFACE CLAIM"),
            "{label}: expected the terminal no-surface-claim diagnostic, got: {err}"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

// IR-A7 - authoritative RustLike error: genuine RustLike-exclusive
// evidence whose own parse fails (no competing Logos evidence at all)
// must survive exactly, on both commands, with zero Logos retry.
#[test]
fn ir_a7_authoritative_rustlike_error_survives_on_both_commands() {
    let dir = mk_temp_dir("ir1931_a7_rustlike_err");
    let root = dir.join("main.sm");
    // Missing closing brace - genuine RustLike-exclusive evidence,
    // genuine parse failure, zero Logos-exclusive evidence anywhere in
    // the text.
    let src = "fn main() {\n    return;\n";
    std::fs::write(&root, src).expect("write root");

    // Control: prove the fixture really is Logos NoClaim / RustLike
    // Exclusive(Err) - not, say, an accidental ambiguity or no-claim
    // cell - or the assertions below would not prove what they claim to.
    let profile = foundation_profile();
    let tokens = lex(src).expect("lex");
    let logos = admit_logos_program_with_profile(src, &tokens, &profile);
    let rustlike = admit_program_with_profile(src, &tokens, &profile);
    assert!(
        matches!(logos, GrammarAdmission::NoClaim),
        "control check: fixture must be Logos NoClaim (no competing evidence), got: {logos:?}"
    );
    assert!(
        matches!(rustlike, GrammarAdmission::Exclusive(Err(_))),
        "control check: fixture must produce RustLike Exclusive(Err), got: {rustlike:?}"
    );

    let (dump_err, hash_err) = both_err(&root);
    assert_eq!(
        dump_err, hash_err,
        "dump-ir and hash-ir must surface the identical authoritative RustLike error"
    );
    assert!(
        !dump_err.contains("AMBIGUOUS") && !dump_err.contains("NO SURFACE CLAIM"),
        "must be the authoritative RustLike parse error itself, not a misclassified \
         ambiguity/no-claim diagnostic, got: {dump_err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
