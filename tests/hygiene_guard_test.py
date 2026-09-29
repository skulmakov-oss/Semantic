#!/usr/bin/env python3
"""
Unit tests for scripts/check_repository_hygiene.py
=================================================
Verifies detection accuracy, false-positive resistance, secret scanning,
index-blob inspection, fail-closed handling, and allowlist behavior.
"""

import unittest
from unittest.mock import patch
import os
import sys

SCRIPTS_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "scripts"))
sys.path.insert(0, SCRIPTS_DIR)

import check_repository_hygiene as guard

class TestRepositoryHygieneGuard(unittest.TestCase):
    def test_prohibited_extensions(self):
        for ext in guard.PROHIBITED_EXTENSIONS:
            violations = guard.check_file(f"some/fake/file{ext}")
            self.assertTrue(len(violations) > 0, f"Expected violation for extension {ext}")

    def test_secret_environment_files(self):
        # Forbidden force-added .env variations
        for bad_env in [".env", ".env.local", ".env.production", ".env.staging", "config/.env"]:
            violations = guard.check_file(bad_env)
            self.assertTrue(
                any("[Secret Leak]" in msg for _, msg in violations),
                f"Expected [Secret Leak] violation for {bad_env}"
            )

        # Approved template exceptions
        for good_env in [".env.example", ".env.template", ".env.sample", "config/.env.example"]:
            with patch.object(guard, "get_file_content_bytes", return_value=b"KEY=dummy_value\n"):
                violations = guard.check_file(good_env)
                self.assertFalse(
                    any("[Secret Leak]" in msg for _, msg in violations),
                    f"Unexpected violation for approved template {good_env}"
                )

    def test_prohibited_keys_and_certs(self):
        for secret_file in ["id_rsa.key", "server.pem", "cert.p12", "wallet.pfx", "keystore.jks"]:
            violations = guard.check_file(f"secrets/{secret_file}")
            self.assertTrue(
                any("[Secret Leak]" in msg for _, msg in violations),
                f"Expected [Secret Leak] violation for {secret_file}"
            )

    def test_local_path_detection(self):
        sample_win_line = r'let path = "C:\Users\developer\Desktop\repo";'
        match = any(pattern.search(sample_win_line) for pattern, _ in guard.LOCAL_PATH_PATTERNS)
        self.assertTrue(match, "Failed to detect Windows local user path")

        sample_unix_line = 'export WORKSPACE="/home/developer/code"'
        match = any(pattern.search(sample_unix_line) for pattern, _ in guard.LOCAL_PATH_PATTERNS)
        self.assertTrue(match, "Failed to detect Unix user path")

    def test_agent_artifact_detection(self):
        sample_transcript = "log_path = 'path/to/.system_generated/logs/transcript_full.jsonl'"
        match = any(pattern.search(sample_transcript) for pattern, _ in guard.LOCAL_PATH_PATTERNS)
        self.assertTrue(match, "Failed to detect agent transcript artifact")

        sample_brain = "artifact in brain/6104fcc3-36a3-4a39-a219-082677efd3ba/scratch"
        match = any(pattern.search(sample_brain) for pattern, _ in guard.LOCAL_PATH_PATTERNS)
        self.assertTrue(match, "Failed to detect agent brain UUID path")

    def test_architectural_transcript_permitted(self):
        line = "/// Returns whether the NativeBackend winit app facade transcript contract is available."
        for pattern, desc in guard.LOCAL_PATH_PATTERNS:
            m = pattern.search(line)
            if m and "transcript" in m.group(0).lower():
                self.fail("Architectural transcript keyword incorrectly flagged by pattern")

    def test_staged_inspects_git_index_blob(self):
        # Staged index blob has a violation, while disk copy is clean
        bad_staged_content = b'let p = "C:\\\\Users\\\\developer\\\\secret";\n'
        with patch.object(guard, "get_file_content_bytes") as mock_get:
            mock_get.return_value = bad_staged_content
            violations = guard.check_file("crates/test/src/lib.rs", from_index=True)
            mock_get.assert_called_once_with("crates/test/src/lib.rs", from_index=True)
            self.assertTrue(
                any("Windows User Path Leak" in msg for _, msg in violations),
                "Failed to detect violation in staged index blob"
            )

    def test_fail_closed_on_read_or_decode_error(self):
        # 1. Read error triggers SCAN_ERROR
        with patch.object(guard, "get_file_content_bytes", side_effect=RuntimeError("Index blob missing")):
            violations = guard.check_file("crates/test/src/lib.rs", from_index=True)
            self.assertTrue(
                any("[SCAN_ERROR]" in msg for _, msg in violations),
                "Scanner failed to fail closed on read error"
            )

        # 2. Corrupted non-binary decode error triggers SCAN_ERROR
        # Odd number of invalid UTF-8 bytes without null bytes fails UTF-8 & UTF-16
        corrupt_bytes = b"\xff\xff\xff"
        with patch.object(guard, "get_file_content_bytes", return_value=corrupt_bytes):
            violations = guard.check_file("crates/test/src/bad.txt", from_index=False)
            self.assertTrue(
                any("[SCAN_ERROR]" in msg for _, msg in violations),
                "Scanner failed to fail closed on decode error"
            )

    def test_allowlist_coverage(self):
        self.assertTrue(guard.is_allowlisted("reports/security/historical_pr_review_audit.csv"))
        self.assertTrue(guard.is_allowlisted("reports/security/Semantic_Project_Health_Ledger.xlsx"))
        self.assertTrue(guard.is_allowlisted("reports/security/dashboard/index.html"))
        self.assertTrue(guard.is_allowlisted("tests/hygiene_guard_test.py"))
        self.assertFalse(guard.is_allowlisted("crates/sm-front/src/parser.rs"))

    def test_known_tracked_debt_registry(self):
        self.assertIn("extract.py", guard.KNOWN_TRACKED_DEBT)
        self.assertIn("fix.py", guard.KNOWN_TRACKED_DEBT)
        self.assertIn("fix2.py", guard.KNOWN_TRACKED_DEBT)
        self.assertIn("test_wgpu.rs", guard.KNOWN_TRACKED_DEBT)
        self.assertIn("run_log.txt", guard.KNOWN_TRACKED_DEBT)
        self.assertIn("output.txt", guard.KNOWN_TRACKED_DEBT)
        self.assertIn("output2.txt", guard.KNOWN_TRACKED_DEBT)

    def test_secrets_under_reports_security_are_rejected(self):
        # Broad content allowlists must NEVER bypass secret or temporary file detection
        for bad_file in [
            "reports/security/.env",
            "reports/security/.env.production",
            "reports/security/private.key",
            "reports/security/cert.pem",
            "reports/security/leak.tmp",
        ]:
            violations = guard.check_file(bad_file)
            self.assertTrue(
                len(violations) > 0,
                f"Expected violation for {bad_file} despite reports/security content allowlist"
            )

    def test_utf16_content_scan_detects_leaks(self):
        # UTF-16LE without BOM (as written by PowerShell terminal redirections)
        leaked_raw = "let secret_path = 'C:\\Users\\developer\\secret';".encode("utf-16le")
        with patch.object(guard, "get_file_content_bytes", return_value=leaked_raw):
            violations = guard.check_file("captured_output.txt")
            self.assertTrue(
                any("Windows User Path Leak" in msg for _, msg in violations),
                "Failed to detect Windows User Path Leak in UTF-16LE text"
            )

        # UTF-16 with BOM
        leaked_bom = b"\xff\xfe" + leaked_raw
        with patch.object(guard, "get_file_content_bytes", return_value=leaked_bom):
            violations = guard.check_file("captured_output_bom.txt")
            self.assertTrue(
                any("Windows User Path Leak" in msg for _, msg in violations),
                "Failed to detect Windows User Path Leak in UTF-16 text with BOM"
            )

    def test_reports_security_new_report_not_exempt_from_content_scan(self):
        # Any NEW file in reports/security/ that is not in the explicit ALLOWLIST_PATHS must be scanned
        leaked_content = b"Notes: Reproducible on C:\\Users\\developer\\Desktop\\repo\n"
        with patch.object(guard, "get_file_content_bytes", return_value=leaked_content):
            violations = guard.check_file("reports/security/new_unallowlisted_report.md")
            self.assertTrue(
                any("Windows User Path Leak" in msg for _, msg in violations),
                "New unallowlisted report in reports/security/ incorrectly bypassed content scan"
            )

if __name__ == "__main__":
    unittest.main()
