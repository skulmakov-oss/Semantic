# Agent Verification Guide & CI Matrix

Status: normative verification authority
Repository: `skulmakov-oss/Semantic`

This document defines the verification commands, testing tiers, Admission Guard gates, and CI parity requirements for all contributions to the Semantic repository.

---

## 1. Verification Doctrine

- **Evidence Before Assertions**: Never assert that a fix, feature, or document is complete without running fresh verification commands and providing exact exit codes and output logs.
- **Never Weaken Tests**: It is strictly forbidden to delete, weaken, disable, or comment out tests or assertions to achieve a passing status.
- **Fail-Closed Verification**: If a test or check fails unexpectedly, stop and investigate using systematic debugging; never apply workarounds or silent shims.
- **Windows Command-Line Safety**: On Windows systems, `cargo fmt --all --check` can exceed the `CreateProcess` command-line length limit. Always use `pwsh -File scripts/workspace_fmt_check.ps1` or `admission_guard.ps1` locally.

---

## 2. Local Admission Guard (`scripts/admission_guard.ps1`)

The repository provides [`scripts/admission_guard.ps1`](../../scripts/admission_guard.ps1) as the canonical local pre-admission gate. It provides several operational modes:

### A. Quick Gate (`-Quick`)
Fast compilation and formatting validation for iterative development.
```powershell
pwsh -File scripts/admission_guard.ps1 -Quick
```
**Underlying Steps**:
1. `cargo check --workspace --all-targets`
2. `Invoke-WorkspaceFmtCheck` (`pwsh -File scripts/workspace_fmt_check.ps1`)

---

### B. PR-Ready Gate (`-PRReady`)
Standard pre-PR submission validation gate.
```powershell
pwsh -File scripts/admission_guard.ps1 -PRReady
```
**Underlying Steps**:
1. `cargo check --workspace --all-targets`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `Invoke-WorkspaceFmtCheck`
4. `cargo test --workspace --quiet`
5. `cargo test -q --test public_api_contracts`

---

### C. CI Parity Gate (`-CIParity`)
Full local mirror of the GitHub Actions CI pipeline (`.github/workflows/ci.yml`).
```powershell
pwsh -File scripts/admission_guard.ps1 -CIParity
```
**Underlying Steps**:
1. **Formatting & Lints**: `workspace_fmt_check.ps1` + `cargo clippy --workspace --all-targets -- -D warnings`
2. **Boundary Enforcement**:
   - `cargo test --test legacy_guards --quiet`
   - `cargo test --test frontend_boundaries --quiet`
   - `cargo test --test ir_opt_boundaries --quiet`
   - `cargo test --test dependency_boundaries --quiet`
3. **Public API Guard**:
   - `cargo test --test public_api_contracts --quiet`
4. **Runtime Release Gates**:
   - `cargo test --test golden_semcode --quiet`
   - `cargo test --test prometheus_runtime_matrix --quiet`
   - `cargo test --test prometheus_runtime_goldens --quiet`
   - `cargo test --test prometheus_runtime_negative_goldens --quiet`
   - `cargo test --test prometheus_runtime_compat_matrix --quiet`
5. **Release Bundle Process**:
   - `pwsh -File scripts/verify_release_bundle.ps1 -ManifestPath <temp_manifest>`
6. **Full Std Suite**:
   - `cargo test --all-targets --quiet`
7. **No-Std Compilation**:
   - `cargo check --no-default-features --quiet`

---

### D. Readiness Gate (`-Readiness`)
Validates CLI binaries, bundle packaging, project root models, and 7hell smoke tests.
```powershell
pwsh -File scripts/admission_guard.ps1 -Readiness
```
**Underlying Steps**:
1. `cargo build --bin smc --bin svm`
2. `pwsh -File scripts/verify_release_bundle.ps1`
3. Canonical project-root smoke (`examples/qualification/pcc9_project_root_minimal`)
4. Package-baseline smoke (`examples/qualification/pcc9_project_root_package_baseline`)
5. Smc 7hell human and JSON smoke (`tests/fixtures/7hell_e1/valid_minimal.sm`)

---

### E. Merge Preflight (`-MergePreflight`)
Full clean-tree merge check in an isolated temporary worktree against the base branch (`origin/main`).
```powershell
pwsh -File scripts/admission_guard.ps1 -MergePreflight
```

---

## 3. GitHub Actions CI Matrix

The repository CI workflow (`.github/workflows/ci.yml`) executes the following automated jobs:

| CI Job | Purpose | Primary Commands |
|---|---|---|
| `pr-ready` | Format & lint validation | `cargo fmt --all --check`<br>`cargo clippy --workspace --all-targets -- -D warnings` |
| `boundary-enforcement` | Module, root cleanliness & legacy perimeter checks | `cargo test --test legacy_guards --quiet`<br>`cargo test --test frontend_boundaries --quiet`<br>`cargo test --test ir_opt_boundaries --quiet`<br>`cargo test --test dependency_boundaries --quiet` |
| `public-api-guard` | Public API contract stability | `cargo test --test public_api_contracts --quiet` |
| `runtime-release-gates` | SemCode & PROMETHEUS runtime matrices | `cargo test --test golden_semcode --quiet`<br>`cargo test --test prometheus_runtime_matrix --quiet`<br>`cargo test --test prometheus_runtime_goldens --quiet`<br>`cargo test --test prometheus_runtime_negative_goldens --quiet`<br>`cargo test --test prometheus_runtime_compat_matrix --quiet` |
| `release-bundle-process` | Release packaging process verification | `pwsh -File scripts/verify_release_bundle.ps1` |
| `pcc-qualification-7hell` | Fast PCC qualification gate (Windows) | `.\tools\7hell\run_ci.ps1` |
| `test-std` | Full test suite across all targets & doctests | `cargo test --workspace --all-targets --quiet`<br>`cargo test --workspace --doc --quiet` |
| `check-no-std` | Zero-dependency core compatibility | `cargo check --no-default-features --quiet` |

---

## 4. Specialized Verification Scripts

### Harness Scope Check
Verifies that all modified or staged files adhere to the allowed and forbidden path rules defined in `.harness/current.task.yaml`:
```powershell
pwsh -File scripts/harness-check.ps1
```

### Workspace Format Check (Windows Safe)
Runs per-package `cargo fmt --check` across all workspace members, avoiding Windows command-line truncation:
```powershell
pwsh -File scripts/workspace_fmt_check.ps1
```

### 7hell PCC Qualification
- Fast CI check:
  ```powershell
  pwsh -File tools/7hell/run_ci.ps1
  ```
- Full scheduled qualification:
  ```powershell
  pwsh -File tools/7hell/run.ps1
  ```

---

## 5. Verification Checklist for Agents

Before claiming any task complete, agents must execute and record results for:

1. [ ] **Harness Verification**: `pwsh -File scripts/harness-check.ps1` (zero forbidden paths; zero unallowed paths).
2. [ ] **Formatting**: `pwsh -File scripts/workspace_fmt_check.ps1` (zero format diffs).
3. [ ] **Compilation**: `cargo check --workspace --all-targets` (zero compile errors).
4. [ ] **Clippy**: `cargo clippy --workspace --all-targets -- -D warnings` (zero warnings).
5. [ ] **Legacy & Boundary Guards**: `cargo test --test legacy_guards --quiet` (zero boundary violations).
6. [ ] **Public API Guard**: `cargo test --test public_api_contracts --quiet` (zero unauthorized API changes).
7. [ ] **Targeted Tests**: All tests relevant to the changed component pass cleanly.
8. [ ] **Git Diff Hygiene**: `git diff --check` (no whitespace anomalies or conflict markers).
