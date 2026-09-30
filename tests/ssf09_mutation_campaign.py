#!/usr/bin/env python3
"""SSF-09 #1580 mutation campaign (M1-M12).

Each mutant violates exactly one frozen diagnostic/editor law. A mutant is
KILLED only when its named regression test *fails at runtime* (a test
failure line is observed); a mutant that fails to compile is reported as
INVALID, never as killed. Every source file is restored after each mutant,
also on interruption.

Usage: python3 tests/ssf09_mutation_campaign.py [--dry-run] [M1 M2 ...]

Selection fails closed: every selector must name a known mutant, and an
empty effective selection is an error (exit 2) - a typo can never report
a vacuous "0/0 KILLED" success. With no selectors, all mutants run.
`--dry-run` validates the selection and that every selected mutant's
anchor occurs exactly once, without mutating or running anything.
"""

import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent

MUTANTS = [
    (
        "M1",
        "fabricated zero-width range when no source token anchors the offset",
        "crates/sm-front/src/diagnostic_authority.rs",
        ".map(|len| offset..offset + len)",
        ".map(|len| offset..offset + len)\n        .or(Some(offset..offset))",
        ["cargo", "test", "-p", "sm-front", "--lib",
         "diagnostic_authority::tests::point_without_source_token_never_becomes_a_range"],
    ),
    (
        "M2",
        "policy violation projected under the syntax code",
        "crates/sm-front/src/diagnostic_authority.rs",
        "FrontendErrorKind::PolicyViolation => FRONTEND_POLICY_VIOLATION_CODE,",
        "FrontendErrorKind::PolicyViolation => FRONTEND_SYNTAX_CODE,",
        ["cargo", "test", "-p", "sm-sema", "--lib",
         "rustlike_policy_error_preserves_frontend_mark_and_kind"],
    ),
    (
        "M3",
        "adapter rewrites producer severity (warning -> error)",
        "crates/sm-sema/src/std_adapters.rs",
        "DiagLevel::Warning => DiagnosticSeverity::Warning,",
        "DiagLevel::Warning => DiagnosticSeverity::Error,",
        ["cargo", "test", "--test", "ssf09_editor_baseline", "schema_goldens_rootless"],
    ),
    (
        "M4",
        "Debug-derived public verifier code",
        "crates/sm-verify/src/diagnostic_authority.rs",
        'VerificationCode::UnknownOpcode => "V0009",',
        'VerificationCode::UnknownOpcode => "UnknownOpcode",',
        ["cargo", "test", "-p", "sm-verify", "--lib",
         "diagnostic_authority::tests::code_tokens_are_unique_and_never_debug_derived"],
    ),
    (
        "M5",
        "runtime admission admits a VM internal-integrity fault",
        "crates/sm-vm/src/diagnostic_admission.rs",
        'RuntimeError::Trap(RuntimeTrap::AssertionFailed) => Some("R0001"),',
        'RuntimeError::Trap(RuntimeTrap::AssertionFailed) => Some("R0001"),\n'
        '            RuntimeError::StackUnderflow => Some("R0005"),',
        ["cargo", "test", "-p", "sm-vm", "--lib",
         "diagnostic_admission::tests::internal_vm_faults_are_not_admitted"],
    ),
    (
        "M6",
        "verifier rejection cause truncated instead of preserved structurally",
        "crates/sm-vm/src/diagnostic_admission.rs",
        "DiagnosticCause::Report(report.to_canonical()?)",
        "DiagnosticCause::Report(report.to_canonical()?.into_iter().take(1).collect())",
        ["cargo", "test", "-p", "sm-vm", "--lib",
         "diagnostic_admission::tests::verifier_rejection_keeps_structured_report_cause"],
    ),
    (
        "M7",
        "machine schema reorders producer diagnostics",
        "crates/smc-cli/src/diagnostic_schema.rs",
        "    for diagnostic in list {",
        "    for diagnostic in list.iter().rev() {",
        ["cargo", "test", "--test", "ssf09_editor_baseline", "schema_goldens_rootless"],
    ),
    (
        "M8",
        "LSP positions counted in UTF-8 bytes instead of UTF-16 code units",
        "crates/smc-cli/src/lsp.rs",
        "c => character += c.len_utf16(),",
        "c => character += c.len_utf8(),",
        ["cargo", "test", "--test", "ssf09_editor_baseline",
         "lsp_utf16_positions_count_code_units"],
    ),
    (
        "M9",
        "LSP accepts a stale (non-newer) didChange",
        "crates/smc-cli/src/lsp.rs",
        "if version <= current.version {",
        "if version < current.version {",
        ["cargo", "test", "--test", "ssf09_editor_baseline",
         "lsp_stale_did_change_is_rejected_and_changes_nothing"],
    ),
    (
        "M10",
        "LSP formatting bypasses the canonical formatter's safety proof",
        "crates/smc-cli/src/lsp.rs",
        "match format_source_checked(&doc.text) {",
        "match Ok::<String, crate::formatter::FormatRefusal>("
        "doc.text.lines().map(str::trim_end).collect::<Vec<_>>().join(\"\\n\") + \"\\n\") {",
        ["cargo", "test", "--test", "ssf09_editor_baseline",
         "lsp_formatting_bridge_equals_cli_formatter_including_refusals"],
    ),
    (
        "M11",
        "canonical module identity reduced to the basename",
        "crates/smc-cli/src/canonical_check.rs",
        "module: admission.module_path,",
        "module: admission.module_path.rsplit('/').next().unwrap_or_default().to_string(),",
        ["cargo", "test", "--test", "ssf09_editor_baseline",
         "nested_module_identity_keeps_module_root_relative_path"],
    ),
    (
        "M12a",
        "canonical check ignores the editor overlay and reads disk",
        "crates/smc-cli/src/canonical_check.rs",
        "let raw_source = match request.overlay.get(&root_canon) {",
        "let raw_source = match None::<&str> {",
        ["cargo", "test", "--test", "ssf09_editor_baseline",
         "lsp_uses_open_document_overlay_not_disk"],
    ),
    (
        "M12b",
        "LSP silently drops diagnostics bound to another source of the check",
        "crates/smc-cli/src/lsp.rs",
        "for (target, diagnostic) in self.check_document(uri, &overlay) {",
        "for (target, diagnostic) in self\n"
        "                .check_document(uri, &overlay)\n"
        "                .into_iter()\n"
        "                .filter(|(target, _)| target == uri)\n"
        "            {",
        ["cargo", "test", "--test", "ssf09_editor_baseline",
         "lsp_routes_imported_module_diagnostics_to_that_module_uri"],
    ),
]


def run(mutant):
    mid, law, rel, find, replace, cmd = mutant
    path = ROOT / rel
    original = path.read_text(encoding="utf-8")
    count = original.count(find)
    if count != 1:
        return mid, law, f"SETUP-ERROR (anchor found {count}x)", ""
    try:
        path.write_text(original.replace(find, replace), encoding="utf-8")
        proc = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
        out = proc.stdout + proc.stderr
        if proc.returncode == 0:
            verdict = "SURVIVED"
        elif "test result: FAILED" in out or "panicked at" in out:
            verdict = "KILLED"
        else:
            verdict = "INVALID (did not reach a test failure)"
        return mid, law, verdict, " ".join(cmd[2:])
    finally:
        path.write_text(original, encoding="utf-8")


def select(selectors):
    """The mutants named by `selectors` (all when empty), or an error."""
    known = [m[0] for m in MUTANTS]
    unknown = [sel for sel in selectors if sel not in known]
    if unknown:
        return None, f"unknown mutant id(s): {' '.join(unknown)} (known: {' '.join(known)})"
    chosen = [m for m in MUTANTS if not selectors or m[0] in selectors]
    if not chosen:
        return None, "empty mutant selection"
    return chosen, None


def main(argv):
    args = argv[1:]
    dry_run = "--dry-run" in args
    selectors = [a for a in args if a != "--dry-run"]
    chosen, error = select(selectors)
    if error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2
    if dry_run:
        bad = []
        for mid, _law, rel, find, _replace, _cmd in chosen:
            count = (ROOT / rel).read_text(encoding="utf-8").count(find)
            if count != 1:
                bad.append(f"{mid} anchor found {count}x")
        for line in bad:
            print(f"ERROR: {line}", file=sys.stderr)
        print(f"dry run: {len(chosen)} mutant(s) selected: {' '.join(m[0] for m in chosen)}")
        return 2 if bad else 0
    results = [run(m) for m in chosen]
    for mid, law, verdict, test in results:
        print(f"{mid:5} {verdict:10} {law}  [{test}]")
    killed = sum(1 for r in results if r[2] == "KILLED")
    print(f"\n{killed}/{len(results)} KILLED")
    return 0 if results and killed == len(results) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
