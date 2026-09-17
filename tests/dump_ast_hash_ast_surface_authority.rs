// #1934 regressions: `smc dump-ast`/`smc hash-ast` implemented
//
//   if let Ok(logos) = parse_logos_program_with_profile(...) { Logos AST }
//   else { parse_program_with_profile(...) }
//
// which collapses an authoritative Logos parse *failure*, a genuine
// cross-grammar `Ambiguous` tie, and a genuine `NoSurfaceClaim` input into
// "try RustLike anyway" - the same shape of defect #1932 already repaired
// for `dump-ir`/`hash-ir`. Both commands now route through one shared
// private helper (`render_ast_for_source` in `crates/smc-cli/src/app.rs`)
// that consumes `sm_front::resolve_surface_authority` directly. Every test
// below runs BOTH `dump-ast` and `hash-ast` against the same fixture and
// asserts the same outcome shape for each - proving the two commands do
// not (and structurally cannot easily) diverge, since they share one seam.
//
// F01 lesson (#1932): `dump-ast`/`hash-ast` share one on-disk AST cache
// keyed by (canonicalized path, content fingerprint) - see `ast_pack_key`
// in `crates/smc-cli/src/app.rs`. Running both commands against the same
// path would give them the same cache key, letting the second command be
// served entirely from the first's cache write instead of actually
// executing. Every success-case test below gives each command its own
// fresh path with identical content, from the start - not discovered
// after the fact this time.

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

// Asserts both `dump-ast` and `hash-ast` succeed on the same source content
// - each against its own fresh path, so neither command's own execution can
// be silently skipped by reading back the other's on-disk AST cache entry
// (F01).
fn both_ok(dir: &std::path::Path, src: &str) {
    let dump_path = dir.join("dump_target.sm");
    let hash_path = dir.join("hash_target.sm");
    std::fs::write(&dump_path, src).expect("write dump-ast fixture");
    std::fs::write(&hash_path, src).expect("write hash-ast fixture");
    cli_ok("dump-ast", &dump_path);
    cli_ok("hash-ast", &hash_path);
}

// Asserts both `dump-ast` and `hash-ast` fail on the same input and returns
// (dump_ast_err, hash_ast_err) for further assertions. Safe to share one
// path: `render_ast_for_source`'s `?` short-circuits before the AST cache
// is ever written on an `Err`, so a failed cold attempt never populates the
// cache the second command's lookup could hit.
fn both_err(path: &std::path::Path) -> (String, String) {
    (cli_err("dump-ast", path), cli_err("hash-ast", path))
}

// AST-A1 - genuine RustLike success under Auto: both commands must still
// succeed on an ordinary RustLike program.
#[test]
fn ast_a1_rustlike_success_under_auto() {
    let dir = mk_temp_dir("ast1934_a1_rustlike_ok");

    both_ok(&dir, "fn main() {\n    return;\n}\n");

    let _ = std::fs::remove_dir_all(&dir);
}

// AST-A2 - genuine Logos success under Auto: both commands must render
// real Logos AST (proving dump-ast/hash-ast's whole reason to exist over
// a generic redirect).
#[test]
fn ast_a2_logos_success_under_auto() {
    let dir = mk_temp_dir("ast1934_a2_logos_ok");

    both_ok(
        &dir,
        "\nEntity A:\n    state x: quad\nLaw \"L\" [priority 1]:\n    When true -> System.recovery()\n",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// AST-A3 - authoritative Logos error: a genuine positive Logos claim whose
// own parse fails must survive exactly, on both commands, with zero
// RustLike-shaped replacement.
#[test]
fn ast_a3_authoritative_logos_error_survives_on_both_commands() {
    let dir = mk_temp_dir("ast1934_a3_logos_err");
    let root = dir.join("main.sm");
    // Missing ':' after the Entity name - genuine Exclusive evidence,
    // genuine parse failure.
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
    for (label, err) in [("dump-ast", &dump_err), ("hash-ast", &hash_err)] {
        assert!(
            !err.contains("expected top-level") && !err.contains("expected primary expression"),
            "{label}: must not be replaced by a RustLike-shaped parse error, got: {err}"
        );
    }
    assert_eq!(
        dump_err, hash_err,
        "dump-ast and hash-ast must surface the identical authoritative Logos error"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// AST-A4 - real Exclusive/Exclusive ambiguity: the exact mandated fixture.
// Both commands must produce a terminal ambiguity, never a RustLike
// fallback error.
#[test]
fn ast_a4_real_exclusive_vs_exclusive_ambiguity_on_both_commands() {
    let dir = mk_temp_dir("ast1934_a4_ambiguous_exclusive");
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
    for (label, err) in [("dump-ast", &dump_err), ("hash-ast", &hash_err)] {
        assert!(
            err.contains("AMBIGUOUS"),
            "{label}: expected terminal ambiguity, got: {err}"
        );
    }
    assert_eq!(
        dump_err, hash_err,
        "dump-ast and hash-ast must surface the identical ambiguity diagnostic"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// AST-A5 - NoSurfaceClaim: blank/comment-only source. Both commands must
// produce the terminal no-surface-claim diagnostic, never a forced
// RustLike "missing main"/parser error.
#[test]
fn ast_a5_no_surface_claim_on_both_commands() {
    let dir = mk_temp_dir("ast1934_a5_no_claim");
    let root = dir.join("main.sm");
    let src = "// just a comment\n";
    std::fs::write(&root, src).expect("write root");

    // Control: prove the fixture really is NoClaim/NoClaim.
    let profile = foundation_profile();
    let tokens = lex(src).expect("lex");
    let logos = admit_logos_program_with_profile(src, &tokens, &profile);
    let rustlike = admit_program_with_profile(src, &tokens, &profile);
    assert!(
        matches!(logos, GrammarAdmission::NoClaim),
        "control check: fixture must be Logos NoClaim, got: {logos:?}"
    );
    assert!(
        matches!(rustlike, GrammarAdmission::NoClaim),
        "control check: fixture must be RustLike NoClaim, got: {rustlike:?}"
    );

    let (dump_err, hash_err) = both_err(&root);
    for (label, err) in [("dump-ast", &dump_err), ("hash-ast", &hash_err)] {
        assert!(
            err.contains("NO SURFACE CLAIM"),
            "{label}: expected the terminal no-surface-claim diagnostic, got: {err}"
        );
    }
    assert_eq!(
        dump_err, hash_err,
        "dump-ast and hash-ast must surface the identical no-surface-claim diagnostic"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// AST-A6 - authoritative RustLike error: genuine RustLike-exclusive
// evidence whose own parse fails (no competing Logos evidence at all)
// must survive exactly, on both commands, with zero Logos retry.
#[test]
fn ast_a6_authoritative_rustlike_error_survives_on_both_commands() {
    let dir = mk_temp_dir("ast1934_a6_rustlike_err");
    let root = dir.join("main.sm");
    // Missing closing brace - genuine RustLike-exclusive evidence,
    // genuine parse failure, zero Logos-exclusive evidence anywhere in
    // the text.
    let src = "fn main() {\n    return;\n";
    std::fs::write(&root, src).expect("write root");

    // Control: prove the fixture really is Logos NoClaim / RustLike
    // Exclusive(Err) - or the assertions below would not prove what they
    // claim to.
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
        "dump-ast and hash-ast must surface the identical authoritative RustLike error"
    );
    assert!(
        !dump_err.contains("AMBIGUOUS") && !dump_err.contains("NO SURFACE CLAIM"),
        "must be the authoritative RustLike parse error itself, not a misclassified \
         ambiguity/no-claim diagnostic, got: {dump_err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
