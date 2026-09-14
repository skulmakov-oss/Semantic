// Decision F (2026-09-14) anti-duplication guard: production code outside
// `sm-front` must not independently resolve cross-grammar source-surface
// authority by destructuring/comparing multiple `GrammarAdmission` values.
// `sm-front::resolve_surface_authority` is the sole canonical
// implementation - see `docs/roadmap/stable_foundation/
// ssf09_diagnostic_authority_decision.md` and
// `docs/architecture/module_ownership_map.md`.
//
// The sequence `#1670` (sm-sema) -> `#1919` (smc-cli) -> `#1920` (sm-ir
// attempted, PR #1929) demonstrated that leaving this to reviewer
// convention drifts: each independent local copy diverged from the
// frozen law in a different way before converging (or, for #1920,
// before being caught and stopped as an architectural blocker). This
// guard makes the rule mechanically enforced, not merely documented.
//
// KNOWN LIMITATION (accepted for now, per owner decision): this is a
// text-based heuristic, not an AST-aware one - it cannot perfectly
// distinguish "a production match block independently re-deriving
// ownership" from "a test constructing `GrammarAdmission` fixtures and
// passing them into the canonical resolver," beyond requiring the
// tell-tale two-tuple `match (` shape all three historical violations
// shared. If this proves unable to reliably tell production code from
// `#[cfg(test)]` fixtures in practice, escalate to an AST-aware guard
// (e.g. a `syn`-based dev-dependency) as a separate owner decision -
// do not add `syn` speculatively.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

fn tracked_rs_files() -> Vec<String> {
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .output()
        .expect("run git ls-files");
    assert!(output.status.success(), "git ls-files must succeed");
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| String::from_utf8_lossy(entry).replace('\\', "/"))
        .filter(|p| p.ends_with(".rs"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// `sm-front` is the frozen canonical owner (Decision F) - its own
/// implementation and tests legitimately contain the full pattern this
/// guard looks for.
const CANONICAL_OWNER_PREFIX: &str = "crates/sm-front/";

/// Tracked, temporary exceptions: consumers whose own local cross-grammar
/// resolver predates Decision F and are explicitly scheduled for a
/// separate future migration PR, not folded into the Decision F
/// foundation PR itself. Remove an entry only in the same PR that
/// deletes its local resolver and switches it to consume
/// `sm_front::resolve_surface_authority` directly.
const PENDING_MIGRATION_EXCEPTIONS: &[&str] = &[
    // `smc-cli`'s `resolve_project_route` (`#1919`) - scheduled for
    // migration in a follow-up PR per the Decision F consumer-migration
    // plan; not touched by the Decision F foundation PR itself.
    "crates/smc-cli/src/app.rs",
    // This guard's own file: its synthetic test snippets are string
    // literals shaped like the forbidden pattern on purpose (F-M10), not
    // production code - scanning them as if they were real would defeat
    // the guard's own self-tests.
    "tests/surface_authority_guard.rs",
];

/// The two-tuple `match (a, b) { ... }` shape every one of the three
/// historical local resolvers shared (`sm-sema`'s former
/// `resolve_surface_authority`, `smc-cli`'s `resolve_project_route`, and
/// `sm-ir`'s attempted `logos_owns_outright`). A window of lines after
/// this literal is scanned for cross-grammar variant co-occurrence -
/// generous enough to cover all three historical match bodies (14-28
/// lines) without needing full brace-matching.
const MATCH_WINDOW_LINES: usize = 20;

/// Cross-grammar re-derivation requires naming at least two of the three
/// `GrammarAdmission` variants together in the same match. A file merely
/// discussing the concept in a comment, or matching on a single grammar's
/// admission alone, does not trip this.
const VARIANT_MARKERS: &[&str] = &["NoClaim", "Shared(", "Exclusive("];

fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| match line.find("//") {
            Some(idx) => &line[..idx],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn find_cross_grammar_match_violation(code: &str) -> Option<usize> {
    let lines: Vec<&str> = code.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if !line.contains("match (") {
            continue;
        }
        let end = (i + MATCH_WINDOW_LINES).min(lines.len());
        let window = lines[i..end].join("\n");
        let hits = VARIANT_MARKERS
            .iter()
            .filter(|marker| window.contains(*marker))
            .count();
        if hits >= 2 {
            return Some(i + 1); // 1-indexed line number
        }
    }
    None
}

#[test]
fn no_consumer_independently_resolves_cross_grammar_surface_authority() {
    let mut violations = Vec::new();
    for path in tracked_rs_files() {
        if path.starts_with(CANONICAL_OWNER_PREFIX) {
            continue;
        }
        if PENDING_MIGRATION_EXCEPTIONS.contains(&path.as_str()) {
            continue;
        }
        let Ok(src) = fs::read_to_string(Path::new(&path)) else {
            continue;
        };
        if !src.contains("GrammarAdmission") {
            continue;
        }
        let stripped = strip_line_comments(&src);
        if let Some(line) = find_cross_grammar_match_violation(&stripped) {
            violations.push(format!("{path}:{line}"));
        }
    }
    assert!(
        violations.is_empty(),
        "found production code outside sm-front independently resolving cross-grammar \
         GrammarAdmission authority (Decision F forbids this - consume \
         sm_front::resolve_surface_authority instead): {violations:?}"
    );
}

/// F-M10 (Decision F mutation model): proves this guard actually catches
/// a new local reimplementation, not just documents an intention. This
/// test does not itself construct the violation in a tracked file (that
/// would defeat its own purpose); it instead unit-tests the detection
/// logic directly against a synthetic snippet shaped like a genuine
/// consumer-local cross-grammar resolver.
#[test]
fn detection_logic_catches_a_synthetic_second_resolver() {
    let synthetic_violation = r#"
fn some_new_consumer_resolver(logos: &GrammarAdmission<i32>, rustlike: &GrammarAdmission<i32>) -> bool {
    match (logos, rustlike) {
        (GrammarAdmission::NoClaim, _) => false,
        (GrammarAdmission::Shared(_), GrammarAdmission::Exclusive(_)) => false,
        _ => true,
    }
}
"#;
    assert!(
        find_cross_grammar_match_violation(&strip_line_comments(synthetic_violation)).is_some(),
        "detection logic must flag a synthetic two-grammar match on GrammarAdmission variants"
    );
}

/// The guard must not flag legitimate single-grammar admission checks or
/// canonical-resolver call sites that merely construct fixtures.
#[test]
fn detection_logic_does_not_flag_single_grammar_or_canonical_call_sites() {
    let single_grammar_match = r#"
fn describe(admission: &GrammarAdmission<i32>) -> &'static str {
    match admission {
        GrammarAdmission::NoClaim => "no claim",
        GrammarAdmission::Shared(_) => "shared",
        GrammarAdmission::Exclusive(_) => "exclusive",
    }
}
"#;
    assert!(
        find_cross_grammar_match_violation(&strip_line_comments(single_grammar_match)).is_none(),
        "a match over a single GrammarAdmission value (no two-tuple `match (`) must not be flagged"
    );

    let canonical_call_site = r#"
fn consume() {
    let verdict = resolve_surface_authority(logos, rustlike);
    match verdict {
        SurfaceAuthority::LogosOwns(_) => {}
        SurfaceAuthority::RustLikeOwns(_) => {}
        SurfaceAuthority::Ambiguous { .. } => {}
        SurfaceAuthority::NoSurfaceClaim => {}
    }
}
"#;
    assert!(
        find_cross_grammar_match_violation(&strip_line_comments(canonical_call_site)).is_none(),
        "matching on the canonical SurfaceAuthority verdict (not raw GrammarAdmission variants) \
         must not be flagged"
    );
}
