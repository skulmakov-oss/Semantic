# Semantic Repository Hygiene & `.gitignore` Hardening Audit

**Repository:** `skulmakov-oss/Semantic`  
**Audit Target:** `origin/main` (Audit Baseline Commit: `e41ff311`)  
**Date:** 2026-09-29  
**Status:** Completed & Validated  

---

## 1. Executive Summary

A comprehensive repository hygiene audit was conducted to prevent accidental leaks of local developer environments, IDE configurations, temporary build outputs, AI-agent session transcripts, and secrets into the Semantic codebase.

The audit established and implemented:
1. **`.gitignore` was hardened** from an unstructured 40-line list into an organized, 10-section policy covering Rust, Python tooling, Node/frontend, IDE/editors, OS junk, local AI agents, secrets/environment, temporary logs, and Semantic Workbench development artifacts.
2. **Untracked local artifacts were neutralized**: 8 unversioned local directories and tool caches (`.codex/`, `.cursor/`, `.zed/`, `docs/.obsidian/`, `.playwright-mcp/`, `.mcp.json`, `crates/smc-cli/.semantic-cache/`, `.vscode/*`) are now cleanly ignored without hiding canonical project content.
3. **No canonical repository files or test vectors were compromised**: Crucial project artifacts (`Cargo.lock`, `Cargo.toml`, `docs/**`, `tests/**`, `reports/security/**`, `tools/*.py`, `scripts/*.ps1`) remain safely tracked. Specifically, `Cargo.lock` is **not** ignored (reproducible workspace builds preserved), and `.smc` rules target only generated outputs while explicitly unignoring test vectors (`!tests/**/*.smc`, `!examples/canonical/**/*.smc`).
4. **Existing tracked debt was cataloged and purged**: 7 legacy files in Git history (`extract.py`, `fix.py`, `fix2.py`, `output.txt`, `output2.txt`, `run_log.txt`, `test_wgpu.rs`) were audited and removed via `git rm`. `test_wgpu.rs` was verified to be a byte-for-byte exact duplicate of `examples/quad_logic_calculator/src/bin.rs`, meaning zero functionality or test coverage was lost.
5. **An automated regression guard was implemented**: [`scripts/check_repository_hygiene.py`](../../scripts/check_repository_hygiene.py) and verified with automated unit tests [`tests/hygiene_guard_test.py`](../../tests/hygiene_guard_test.py).
6. **Continuous Integration enforcement**: A dedicated, fast pre-flight gate job `repository-hygiene` was integrated into [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) to run on every push and pull request.

---

## 2. Existing Ignore Policy & Gaps Found

Prior to this hardening, `.gitignore` contained only 40 lines with several critical coverage gaps:

| Gap Identified | Pre-Existing Risk | Hardened Resolution |
| :--- | :--- | :--- |
| **Missing AI Agent Directories** | `.codex/`, `.cursor/`, `.continue/`, `.aider*` were untracked and exposed to accidental staging. | Added explicit root patterns `/.codex/`, `/.cursor/`, `/.gemini/`, `/.continue/`, `/.aider*`. |
| **Local MCP Artifacts** | `.playwright-mcp/` (100+ MB of temporary logs/YML) and `.mcp.json` were untracked. | Added `/.playwright-mcp/` and `.mcp.json`. |
| **Editor Settings** | `.zed/` and `docs/.obsidian/` were untracked. | Added `.zed/`, `.obsidian/`, `docs/.obsidian/`. |
| **Selective VS Code Policy** | No `.vscode/` rule existed; local configs (`.vscode/mcp.json`) were exposed. | Added selective `.vscode/*` with negation allowlist for shared workspace configs (`settings.json`, `tasks.json`, etc.). |
| **Sub-crate Caches** | Only root `/.semantic-cache/` was ignored; `crates/smc-cli/.semantic-cache/` remained untracked. | Added recursive `**/.semantic-cache/`. |
| **Python Tooling Caches** | Python tools in `tools/` and `scripts/` risked leaking `__pycache__/`, `.pytest_cache/`, `.venv/`. | Added comprehensive Python virtual environment and cache patterns. |
| **Secrets & Env Files** | No `.env` rule existed. | Added `.env`, `.env.*` with exceptions for templates (`!.env.example`). |
| **Temporary Root Dumps** | Terminal logs like `output*.txt` and `run_log.txt` had no ignore rules. | Added `/output*.txt`, `/run_log.txt`, `*.tmp`, `*.temp`. |
| **Blanket vs Targeted `.smc`** | A blanket `*.smc` rule would silently ignore future test fixtures outside `tests/golden/`. | Scoped to compiler output paths (`/out.smc`, `/*.smc`, `/tmp/**/*.smc`, `/target/**/*.smc`, `/artifacts/local/**/*.smc`, `examples/**/calculator.smc`) with negation rules for test suites (`!tests/**/*.smc`, `!examples/canonical/**/*.smc`). |
| **`Cargo.lock` Preservation** | Adding `Cargo.lock` to `.gitignore` harms build reproducibility for Rust CLI/runtime applications. | `Cargo.lock` is explicitly **excluded** from `.gitignore` and documented as intentionally tracked. |

---

## 3. Hardened `.gitignore` Structure

The hardened `.gitignore` is organized into clean, maintainable sections:

```gitignore
# ============================================================
# Rust
# ============================================================
/target/
**/target/
/target_parser/
**/*.rs.bk

# Generated SemCode binary outputs (preserve test vectors and canonical fixtures)
/out.smc
/*.smc
/tmp/**/*.smc
/target/**/*.smc
/artifacts/local/**/*.smc
examples/**/calculator.smc
!tests/**/*.smc
!examples/canonical/**/*.smc

# Cargo.lock intentionally tracked for reproducible workspace builds

# ============================================================
# Python Tooling & Virtual Environments
# ============================================================
__pycache__/
*.py[cod]
*$py.class
.pytest_cache/
.mypy_cache/
.ruff_cache/
.coverage
htmlcov/
.venv/
venv/
ENV/
env/

# ============================================================
# Node / Frontend Tooling
# ============================================================
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*
.pnpm-debug.log*
.cache/

# ============================================================
# IDE / Editor Environments
# ============================================================
.idea/
*.swp
*.swo
*~
*.bak
.vscode/*
!.vscode/settings.json
!.vscode/tasks.json
!.vscode/launch.json
!.vscode/extensions.json
.zed/
.obsidian/
docs/.obsidian/

# ============================================================
# Operating System Junk
# ============================================================
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db
Desktop.ini

# ============================================================
# Local AI Agents & Tooling Sessions
# ============================================================
/.gemini/
/.claude/
/.codex/
/.cursor/
/.continue/
/.aider*
/.playwright-mcp/
/.codebase-memory/
/.ccl/
.mcp.json

# ============================================================
# Secrets & Environment Configurations
# ============================================================
.env
.env.*
!.env.example
!.env.template
!.env.sample

# ============================================================
# Logs, Dumps & Ephemeral Outputs
# ============================================================
/output*.txt
/run_log.txt
*.tmp
*.temp
/scratch/
/patch.txt

# ============================================================
# Semantic Local Development & Workbench Artifacts
# ============================================================
.semantic-cache/
**/.semantic-cache/
/examples/baseline/
/lancedb/
/models/
.workbench_evidence/
/artifacts/workbench/screenshots/*.png
/artifacts/workbench/drag-verification/*.png
/.harness/context/
/.harness/reports/context/
```

---

## 4. Tracked Hygiene Debt Inventory (`TRACKED_HYGIENE_DEBT`)

A thorough scan of all 2,309 tracked repository files identified **7 legacy files at the root level** that constituted tracked hygiene debt.

Each file was audited, verified against canonical assets, and removed via `git rm`:

| File | Size / Lines | Nature & Evidence Content | Audit Finding | Status |
| :--- | :--- | :--- | :--- | :--- |
| **`extract.py`** | 20 lines | Ephemeral script used to recover code from an agent transcript; hardcodes personal path `C:/Users/said3/...`. | One-off extraction utility with no persistent repository role. | **PURGED** (`git rm`) |
| **`fix.py`** | 11 lines | One-off scratch script that applied a regex derive modification to `Action` enum. | Modifications already landed in canonical crates. | **PURGED** (`git rm`) |
| **`fix2.py`** | 21 lines | One-off proof injection script for calculator examples. | Already integrated into repository history. | **PURGED** (`git rm`) |
| **`output.txt`** | 22 lines (UTF-16LE) | PowerShell redirection output (`cargo run -p prom-ui-demo > output.txt 2>&1`) containing Windows error strings. | Stale terminal output dump. | **PURGED** (`git rm`) |
| **`output2.txt`** | 33 lines (UTF-16LE) | PowerShell redirection output containing local developer build paths (`C:\Users\said3\...`). | Stale terminal output dump. | **PURGED** (`git rm`) |
| **`run_log.txt`** | 15,020 lines (395 KB) | Terminal dump of compiler output containing local Windows paths (`C:\Users\said3\...`). | Stale terminal log dump. | **PURGED** (`git rm`) |
| **`test_wgpu.rs`** | 10 lines | Root-level GPU test binary (`wgpu::Instance::default()`). | **Identical**: Byte-for-byte duplicate of `examples/quad_logic_calculator/src/bin.rs`. Root copy was redundant. | **PURGED** (`git rm`) |

---

## 5. Local Topology Findings & Classification

A full regex search across all tracked repository files for local environment signatures (`C:\Users\`, `C:/Users/`, `/Users/`, `/home/`, `.gemini/`, `.claude/`, `.codex/`, `brain/`, `transcript`) yielded 4 categories:

### A. Accidental Local Path Leaks in Code / Logs (Tracked Debt)
- `extract.py:3` — hardcoded path to agent brain transcript directory.
- `output.txt` & `output2.txt` — PowerShell command redirection traces.
- `run_log.txt:11` — compiler stderr output dump with local Windows paths.
*Action:* All 7 tracked debt files were completely removed from the Git index (`git rm`).

### B. Legitimate Historical Documentation & Audit Records
- `docs/roadmap/pcc/...` & `docs/roadmap/post_ui/...` — historical roadmap audit documents recording previous local CI reproduction steps and test results.
- `reports/cold_start_rehearsal_2026-04-24.md` — historical rehearsal report.
- `reports/security/historical_pr_review_audit.csv` & `reports/security/Semantic_Project_Health_Ledger.xlsx` — authoritative audit evidence where historical review comments and sanitized PR bodies are cataloged.
*Action:* Explicitly allowlisted in the hygiene regression guard.

### C. Legitimate Architectural Domain Concepts
- `crates/prom-ui-backend-native/src/lib.rs` (`native_backend_winit_app_facade_transcript_available`)
- `docs/architecture/ui_workbench_consumption_boundary.md` (`renderer transcript semantics`)
- `docs/roadmap/post_ui/r12_ui_winit_run_loop_integration_boundary.md` (`renderer transcript != audit authority`)
*Action:* The word `transcript` in these contexts is a core PROMETHEUS UI architectural concept (the deterministic record of UI draw commands and window events). The hygiene guard explicitly distinguishes this from agent session transcripts.

### D. Historical Workbench Smoke Test Artifacts
- `artifacts/workbench/beta-smoke/workbench_beta_package_manifest.json`
- `artifacts/workbench/screenshots/capture_log.txt`
- `artifacts/workbench/drag-verification/drag_verify_log.txt`
- `artifacts/workbench/native-launch-smoke/report.md`
*Action:* Allowlisted as preserved smoke test evidence.

---

## 6. Hygiene Regression Guard & CI Integration

To guarantee that no future local paths, session artifacts, or temporary files enter the repository, an automated regression guard was created:

### Tool: [`scripts/check_repository_hygiene.py`](../../scripts/check_repository_hygiene.py)
Features:
- **Zero False Positives**: Uses strict regex signatures and an explicit allowlist for historical documentation.
- **Fail-Closed Security**: Detects unexpected read/decode errors as `[SCAN_ERROR]` violations, preventing corrupted or uninspected files from slipping through.
- **Secret & Key Prevention**: Directly rejects force-added environment files (`.env`, `.env.*` except approved templates `.env.example`, `.env.template`, `.env.sample`) and private keys/certificates (`*.key`, `*.pem`, `*.p12`, `*.pfx`, `*.keystore`).
- **Exact Allowlist Enforcement**: Content allowlisting strictly requires exact relative paths in `ALLOWLIST_PATHS`, eliminating broad directory bypasses for new files in `reports/security/`.
- **UTF-16 & PowerShell Dump Support**: Scans UTF-8 (with/without BOM) and UTF-16LE/BE (with/without BOM), ensuring terminal redirection dumps containing NUL bytes cannot evade path leak detection.
- **Index-Blob Inspection**: In `--staged` mode, directly queries Git index blobs via `git show :<path>` rather than the disk working tree, preventing staged leaks from hiding behind cleaned working trees.
- **Fast Execution**: Scans all tracked files in ~2 seconds.
- **Flexible Modes**:
  - `python scripts/check_repository_hygiene.py`: Scans all tracked files, reports new violations and summarizes known tracked debt.
  - `python scripts/check_repository_hygiene.py --staged`: Scans only staged changes from the Git index (suitable for pre-commit hooks).
  - `python scripts/check_repository_hygiene.py --strict`: Fails with non-zero exit code if any violations or known tracked debt remain. (Currently exits with code 0 as all 7 debt files have been purged).

### Unit Tests: [`tests/hygiene_guard_test.py`](../../tests/hygiene_guard_test.py)
A test suite verifying:
- Detection of prohibited temporary extensions (`.tmp`, `.bak`, `.swp`).
- Rejection of force-added secret environment files (`.env`, `.env.production`) and approval of templates (`.env.example`).
- Rejection of private keys and certificates (`*.key`, `*.pem`, `*.p12`, `*.pfx`).
- Detection of Windows (`C:\Users\...`) and Unix (`/home/...`) paths with single or escaped backslashes.
- Detection of agent session transcripts and brain UUIDs.
- Permissibility of legitimate architectural transcript terms (`renderer transcript`).
- Staged mode inspection of Git index blobs even if working tree is modified.
- Fail-closed error reporting on read or decode failures.
- Accurate allowlisting of security reports and ledger files.
- Accurate tracking of the 7 known debt files.
- Rejection of secret and temporary files under `reports/security/` (preventing allowlist bypass).
- Detection of Windows user path leaks in UTF-16LE text with and without BOM.
- Enforcement that new, unallowlisted reports in `reports/security/` undergo full content scanning.

```text
Ran 13 tests in 1.08s — OK
```

### CI Workflow: [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml)
A dedicated, lightweight fast-gate job was added:

```yaml
  repository-hygiene:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: "3.11"
      - name: Run hygiene guard tests
        run: python tests/hygiene_guard_test.py
      - name: Verify repository hygiene
        run: python scripts/check_repository_hygiene.py --strict
```

---

## 7. Verification Evidence

### A. Git Check-Ignore Verification
```bash
# Untracked junk properly ignored:
$ git check-ignore -v .codex/config.toml .cursor/mcp.json .zed/settings.json .mcp.json .playwright-mcp/test.log crates/smc-cli/.semantic-cache/file docs/.obsidian/app.json .env .env.local .venv/bin/python __pycache__/a.pyc
.gitignore:81:/.codex/	.codex/config.toml
.gitignore:82:/.cursor/	.cursor/mcp.json
.gitignore:60:.zed/	.zed/settings.json
.gitignore:88:.mcp.json	.mcp.json
.gitignore:85:/.playwright-mcp/	.playwright-mcp/test.log
.gitignore:112:**/.semantic-cache/	crates/smc-cli/.semantic-cache/file
.gitignore:62:docs/.obsidian/	docs/.obsidian/app.json
.gitignore:93:.env	.env
.gitignore:94:.env.*	.env.local
.gitignore:32:.venv/	.venv/bin/python
.gitignore:24:__pycache__/	__pycache__/a.pyc

# Canonical project files and test vectors are NOT ignored (exit code 1, empty stdout):
$ git check-ignore -v Cargo.lock Cargo.toml tests/golden_v1/calculator.smc examples/canonical/hello.sm crates/sm-front/Cargo.toml tools/generate_health_dashboard.py reports/security/historical_pr_review_audit.csv reports/security/Semantic_Project_Health_Ledger.xlsx reports/security/Semantic_Project_Health_Dashboard.html tests/golden_snapshots/runtime/capability_denial.txt docs/DNA.md
(clean — no matches)
```

### B. Hygiene Guard Scan (Strict Mode)
```bash
$ python scripts/check_repository_hygiene.py --strict
Scanning all 2351 tracked files for repository hygiene violations...

============================================================
HYGIENE SCAN SUMMARY:
  Scanned Files: 2351
  New Violations: 0
  Known Tracked Debt Files: 0
============================================================

SUCCESS: Repository hygiene check PASSED! (Zero unauthorized leaks).
```

---

## 8. Summary of Changes

1. **`.gitignore`**:
   - Organized into 10 structured sections.
   - Preserves `Cargo.lock` for reproducible builds.
   - Refined `.smc` ignore policy: ignores generated binaries (`/out.smc`, `/*.smc`, `/tmp/**/*.smc`, `/target/**/*.smc`, `examples/**/calculator.smc`), preserves test fixtures (`!tests/**/*.smc`, `!examples/canonical/**/*.smc`).
   - Cleanly neutralizes all 8 untracked development/AI caches.
2. **Tracked Debt Purge**:
   - `git rm` applied to 7 legacy files (`extract.py`, `fix.py`, `fix2.py`, `output.txt`, `output2.txt`, `run_log.txt`, `test_wgpu.rs`).
   - `test_wgpu.rs` verified 100% duplicate of `examples/quad_logic_calculator/src/bin.rs`.
3. **Hygiene Regression Guard**:
   - [`scripts/check_repository_hygiene.py`](../../scripts/check_repository_hygiene.py) implements automated repository scanning with `--staged` and `--strict` flags.
   - [`tests/hygiene_guard_test.py`](../../tests/hygiene_guard_test.py) covers 13 test scenarios (100% pass rate).
4. **CI Workflow**:
   - Added `repository-hygiene` job to [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml).
