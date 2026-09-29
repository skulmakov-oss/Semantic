#!/usr/bin/env python3
"""
Semantic Repository Hygiene Guard
=================================
Automated regression guard that verifies tracked Git content is free from:
- Accidental local environment paths (C:\\Users\\<user>, /Users/<user>, /home/<user>)
- AI agent session transcripts and local brain artifacts (.gemini/brain, transcript_full.jsonl)
- Temporary build outputs, debug logs, and ephemeral scratch scripts
- Secrets, credentials, or private configuration files (.env, *.pem, *.key, *.p12, *.pfx)

Usage:
  python scripts/check_repository_hygiene.py               # Checks all tracked files
  python scripts/check_repository_hygiene.py --staged      # Checks only staged index blobs
  python scripts/check_repository_hygiene.py --strict      # Fails if known tracked debt remains
"""

import os
import sys
import re
import argparse
import subprocess

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))

# Signatures for accidental local environment & agent artifact leaks
LOCAL_PATH_PATTERNS = [
    (re.compile(r"[C|c]:[/\\]+Users[/\\]+[a-zA-Z0-9_.-]+", re.IGNORECASE), "Windows User Path Leak"),
    (re.compile(r"(?:^|[\s\"'(=])/(?:home|Users)/[a-zA-Z0-9_.-]+(?:/|[/\s\"'])"), "Unix User Path Leak"),
    (re.compile(r"\.gemini[/\\]antigravity[/\\]brain"), "Antigravity Agent Brain Leak"),
    (re.compile(r"\.system_generated[/\\]logs[/\\]transcript"), "Agent Transcript Leak"),
    (re.compile(r"brain/[0-9a-fA-F-]{36}/"), "UUID Brain Path Leak"),
    (re.compile(r"transcript_full\.jsonl"), "Agent Full Transcript Reference"),
]

PROHIBITED_EXTENSIONS = {
    ".tmp", ".temp", ".bak", ".swp", ".swo", ".pyc"
}

PROHIBITED_SECRET_EXTENSIONS = {
    ".key", ".pem", ".p12", ".pfx", ".pkcs12", ".keystore", ".jks"
}

ALLOWED_ENV_TEMPLATES = {
    ".env.example", ".env.template", ".env.sample"
}

KNOWN_BINARY_EXTENSIONS = {
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".pdf", ".zip", ".tar",
    ".gz", ".wasm", ".smc", ".bin", ".exe", ".dll", ".so", ".dylib",
    ".xlsx", ".parquet", ".db"
}

# Known Tracked Hygiene Debt (Legacy files purged via git rm)
KNOWN_TRACKED_DEBT = {
    "extract.py": "Ephemeral transcript extractor script with hardcoded local path",
    "fix.py": "One-off regex modification script",
    "fix2.py": "One-off proof injection script",
    "test_wgpu.rs": "Root-level ad-hoc GPU test binary",
    "run_log.txt": "Terminal compilation log dump (15k lines)",
    "output.txt": "PowerShell cargo run redirection dump",
    "output2.txt": "PowerShell cargo run redirection dump",
}

# Explicit allowlist for legitimate audit evidence, historical reports, and documentation
ALLOWLIST_PATHS = {
    # Audit evidence and ledger artifacts
    "reports/security/historical_pr_review_audit.csv",
    "reports/security/historical_pr_review_audit.json",
    "reports/security/HISTORICAL_PR_REVIEW_AUDIT.md",
    "reports/security/Semantic_Project_Health_Ledger.xlsx",
    "reports/security/Semantic_Project_Health_Dashboard.html",
    "reports/security/dashboard/ledger_data.json",
    "reports/security/dashboard/ledger_data.js",
    "reports/security/dashboard/index.html",
    "reports/security/dashboard/app.js",
    "reports/security/dashboard/README.md",
    "reports/security/REPOSITORY_HYGIENE_AUDIT.md",
    # Historical admission logs & roadmap audits documenting past local CI runs
    "docs/roadmap/pcc/cli_public_sample_qualification_audit.md",
    "docs/roadmap/pcc/collections_core_audit.md",
    "docs/roadmap/pcc/control_flow_core_audit.md",
    "docs/roadmap/pcc/pcc_stack_bridge_audit.md",
    "docs/roadmap/pcc/pcc_stack_external_diff_sampling_captured.md",
    "docs/roadmap/pcc/pcc_stack_external_inventory.md",
    "docs/roadmap/pcc/pcc_stack_linguist_wording_audit.md",
    "docs/roadmap/pcc/post_merge_sequence_ownership_closeout.md",
    "docs/roadmap/pcc/record_field_ownership_audit.md",
    "docs/roadmap/pcc/text_core_audit.md",
    "docs/roadmap/post_ui/r12_ui_project_board_reconciliation.md",
    "docs/roadmap/post_ui/r12_ui_project_board_status_metadata_reconciliation_followup.md",
    "docs/roadmap/post_ui/ui_dna2_ownership_and_compatibility_freeze.md",
    "docs/roadmap/post_ui/ui_dna2_prom_ui_reconciliation.md",
    "docs/roadmap/repository_truth_audit_2026-04-22.md",
    "reports/cold_start_rehearsal_2026-04-24.md",
    # Historical workbench package manifests and capture logs
    "artifacts/workbench/beta-smoke/workbench_beta_package_manifest.json",
    "artifacts/workbench/beta-smoke/workbench_beta_smoke_latest.json",
    "artifacts/workbench/screenshots/capture_log.txt",
    "artifacts/workbench/drag-verification/drag_verify_log.txt",
    "artifacts/workbench/native-launch-smoke/report.md",
    "artifacts/workbench/native-launch-smoke/stdout.log",
    "artifacts/workbench/native-launch-smoke/stderr.log",
    # Ignore, guard and unit test scripts themselves
    ".gitignore",
    "scripts/check_repository_hygiene.py",
    "tests/hygiene_guard_test.py",
}

def get_tracked_files(staged_only=False):
    """Retrieve list of files to check from Git."""
    if staged_only:
        # Check files added, copied, or modified in index (exclude deleted)
        cmd = ["git", "diff", "--name-only", "--cached", "--diff-filter=d"]
    else:
        cmd = ["git", "ls-files"]
    try:
        res = subprocess.run(cmd, cwd=REPO_ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, check=True)
        files = [f.strip() for f in res.stdout.splitlines() if f.strip()]
        return files
    except subprocess.CalledProcessError as e:
        sys.exit(f"Git execution error: {e.stderr}")

KNOWN_TEXT_EXTENSIONS = {
    ".rs", ".py", ".md", ".txt", ".json", ".csv", ".toml", ".yaml",
    ".yml", ".sh", ".ps1", ".js", ".ts", ".html", ".css", ".sm",
    ".xml", ".ini", ".cfg", ".conf", ".sql", ".graphql", ".mdx"
}

# Explicit allowlists for secret/temporary fixtures (exact paths only, empty by default)
ALLOWED_SECRET_FIXTURES = set()
ALLOWED_TEMPORARY_FIXTURES = set()

def is_content_allowlisted(rel_path):
    """Check if file content is allowlisted from historical user path / transcript checks."""
    norm = rel_path.replace("\\", "/")
    return norm in ALLOWLIST_PATHS

# Backward-compatible alias
is_allowlisted = is_content_allowlisted

def get_file_content_bytes(rel_path, from_index=False):
    """Retrieve raw bytes of a file from Git index or disk (fail-closed)."""
    norm = rel_path.replace("\\", "/")
    if from_index:
        cmd = ["git", "show", f":{norm}"]
        res = subprocess.run(cmd, cwd=REPO_ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if res.returncode != 0:
            err = res.stderr.decode("utf-8", errors="replace").strip()
            raise RuntimeError(f"Failed to read index blob for '{norm}': {err}")
        return res.stdout
    else:
        full_path = os.path.join(REPO_ROOT, rel_path)
        if os.path.exists(full_path):
            with open(full_path, "rb") as f:
                return f.read()
        else:
            # Fall back to git index if file is tracked in git ls-files but absent from disk
            cmd = ["git", "show", f":{norm}"]
            res = subprocess.run(cmd, cwd=REPO_ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            if res.returncode != 0:
                raise RuntimeError(f"File missing on disk and cannot be read from index: '{norm}'")
            return res.stdout

def check_file(rel_path, from_index=False):
    """Check a single file for hygiene violations. Returns list of (line_no, message)."""
    violations = []
    norm_path = rel_path.replace("\\", "/")
    basename = os.path.basename(norm_path).lower()
    _, ext = os.path.splitext(norm_path)
    ext_lower = ext.lower()

    # 1. Prohibited Environment Files (reject force-added .env, .env.* except approved templates)
    # NEVER bypassed by broad directory allowlists like reports/security/
    if basename == ".env" or basename.startswith(".env."):
        if basename not in ALLOWED_ENV_TEMPLATES and norm_path not in ALLOWED_SECRET_FIXTURES:
            violations.append((0, f"[Secret Leak] Prohibited environment file tracked: '{norm_path}'"))

    # 2. Prohibited Secret Keys and Certificates (*.key, *.pem, *.p12, *.pfx, *.jks)
    # NEVER bypassed by broad directory allowlists like reports/security/
    if ext_lower in PROHIBITED_SECRET_EXTENSIONS:
        if norm_path not in ALLOWED_SECRET_FIXTURES:
            violations.append((0, f"[Secret Leak] Prohibited private key / certificate file tracked: '{norm_path}'"))

    # 3. Prohibited Temporary Extensions (*.tmp, *.bak, *.swp)
    if ext_lower in PROHIBITED_EXTENSIONS:
        if norm_path not in ALLOWED_TEMPORARY_FIXTURES:
            violations.append((0, f"Prohibited temporary extension '{ext}' in tracked file: '{norm_path}'"))

    # 4. Known Tracked Debt Check (content skipped; file identity already recorded as debt)
    if norm_path in KNOWN_TRACKED_DEBT:
        return violations

    # 5. Skip textual pattern inspection for content-allowlisted paths
    if is_content_allowlisted(norm_path):
        return violations

    # 6. Read content bytes (fail closed on read error)
    try:
        raw_bytes = get_file_content_bytes(norm_path, from_index=from_index)
    except Exception as e:
        violations.append((0, f"[SCAN_ERROR] Failed to read content of '{norm_path}': {e}"))
        return violations

    if not raw_bytes:
        return violations

    # 7. Explicit binary extension check (skip content scan for known binaries)
    if ext_lower in KNOWN_BINARY_EXTENSIONS:
        return violations

    # 8. Text decoding (supports UTF-8 with/without BOM, UTF-16LE/BE with/without BOM)
    text = None

    # Check explicit BOMs first
    if raw_bytes.startswith(b"\xef\xbb\xbf"):
        try:
            text = raw_bytes.decode("utf-8-sig")
        except UnicodeDecodeError:
            pass
    elif raw_bytes.startswith(b"\xff\xfe") or raw_bytes.startswith(b"\xfe\xff"):
        try:
            text = raw_bytes.decode("utf-16")
        except UnicodeDecodeError:
            pass

    # If raw_bytes contains NUL bytes, it is likely UTF-16LE/BE without BOM (e.g. PowerShell terminal dumps)
    if text is None and b"\x00" in raw_bytes[:8192]:
        sample = raw_bytes[:min(len(raw_bytes), 1024)]
        if len(sample) >= 2:
            nulls_odd = sample[1::2].count(0)
            nulls_even = sample[0::2].count(0)
            if nulls_odd > len(sample) // 4 or nulls_even > len(sample) // 4:
                try:
                    text = raw_bytes.decode("utf-16")
                except UnicodeDecodeError:
                    try:
                        text = raw_bytes.decode("utf-16le")
                    except UnicodeDecodeError:
                        try:
                            text = raw_bytes.decode("utf-16be")
                        except UnicodeDecodeError:
                            pass

    # Try standard UTF-8 (only if raw_bytes has no embedded NUL bytes)
    if text is None:
        try:
            decoded = raw_bytes.decode("utf-8")
            if "\x00" not in decoded:
                text = decoded
        except UnicodeDecodeError:
            pass

    # Fallback to UTF-16 / UTF-16LE if still not decoded
    if text is None:
        try:
            text = raw_bytes.decode("utf-16")
        except UnicodeDecodeError:
            try:
                text = raw_bytes.decode("utf-16le")
            except UnicodeDecodeError:
                pass

    # If still not decoded:
    if text is None:
        # If it contains NUL bytes and is NOT a known text extension, treat as unknown binary
        if b"\x00" in raw_bytes[:8192] and ext_lower not in KNOWN_TEXT_EXTENSIONS:
            return violations
        # Otherwise, fail closed
        violations.append((0, f"[SCAN_ERROR] Failed to decode text in '{norm_path}'"))
        return violations

    # 9. Search for forbidden local patterns
    for line_no, line in enumerate(text.splitlines(), 1):
        for pattern, desc in LOCAL_PATH_PATTERNS:
            match = pattern.search(line)
            if match:
                matched_str = match.group(0)
                # Filter out false positive architectural transcript keywords
                if "transcript" in matched_str.lower() and ("renderer transcript" in line.lower() or "app facade transcript" in line.lower()):
                    continue
                violations.append((
                    line_no,
                    f"[{desc}] match '{matched_str}' in {norm_path}:{line_no} -> {line.strip()[:100]}"
                ))

    return violations

def main():
    parser = argparse.ArgumentParser(description="Audit and verify Semantic repository hygiene.")
    parser.add_argument("--staged", action="store_true", help="Scan only staged files from git index")
    parser.add_argument("--strict", action="store_true", help="Fail if known tracked debt is present")
    args = parser.parse_args()

    files = get_tracked_files(staged_only=args.staged)
    mode_str = "staged changes (git index blobs)" if args.staged else f"all {len(files)} tracked files"
    print(f"Scanning {mode_str} for repository hygiene violations...\n")

    total_violations = 0
    debt_found = []

    for f in files:
        norm = f.replace("\\", "/")

        if norm in KNOWN_TRACKED_DEBT:
            debt_found.append((norm, KNOWN_TRACKED_DEBT[norm]))
            # In strict mode, known debt is also treated as violation
            if args.strict:
                total_violations += 1
                print(f"STRICT VIOLATION: Tracked debt file '{norm}': {KNOWN_TRACKED_DEBT[norm]}")

        violations = check_file(f, from_index=args.staged)
        if violations:
            for line_no, msg in violations:
                print(f"VIOLATION: {msg}")
                total_violations += 1

    print("\n" + "=" * 60)
    print("HYGIENE SCAN SUMMARY:")
    print(f"  Scanned Files: {len(files)}")
    print(f"  New Violations: {total_violations}")
    print(f"  Known Tracked Debt Files: {len(debt_found)}")
    for d, desc in debt_found:
        print(f"    - {d} ({desc})")
    print("=" * 60)

    if total_violations > 0:
        print("\nFAILURE: Repository hygiene check FAILED with active violations.")
        sys.exit(1)
    else:
        print("\nSUCCESS: Repository hygiene check PASSED! (Zero unauthorized leaks).")
        sys.exit(0)

if __name__ == "__main__":
    main()
