# Agent Verification Guide & CI Matrix

Status: normative verification authority
Repository: `skulmakov-oss/Semantic`

This document defines the verification commands, testing tiers, Admission Guard gates, and CI parity requirements for all contributions to the Semantic repository.

---

## 1. Verification Doctrine

- **Evidence Before Assertions**: Never assert that a fix, feature, or document is complete without running fresh verification commands and providing exact exit codes and output logs.
- **Risk-Based Verification**: Verification effort must scale with the change's risk level (R0 through R3). Informational (R0) documentation edits do not require running heavy full-workspace regression runs; fast checks, formatting, and git diff/harness checks suffice.
- **Never Weaken Tests**: It is strictly forbidden to delete, weaken, disable, or comment out tests or assertions to achieve a passing status.
- **Fail-Closed Verification**: If a test or check fails unexpectedly, stop and investigate using systematic debugging; never apply workarounds or silent shims.
- **Windows Command-Line Safety**: On Windows systems, `cargo fmt --all --check` can exceed the `CreateProcess` command-line length limit. Always use `pwsh -File scripts/workspace_fmt_check.ps1` or `admission_guard.ps1` locally.

---

## 2. Local Admission Guard (`scripts/admission_guard.ps1`)

The repository provides [`scripts/admission_guard.ps1`](../../scripts/admission_guard.ps1) as the canonical local pre-admission gate with multiple operational modes:

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
Standard pre-PR submission validation gate for internal and feature changes.
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
Replicates the primary checks of the GitHub Actions CI pipeline (`.github/workflows/ci.yml`).
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
6. **Std Suite (Local Target Scope)**:
   - `cargo test --all-targets --quiet` (note: executes targets for current default packages rather than full `--workspace` scope)
7. **No-Std Compilation**:
   - `cargo check --no-default-features --quiet`

#### Known Coverage Gaps vs GitHub Actions CI
While `-CIParity` mirrors the primary test jobs, it has the following specific differences from `.github/workflows/ci.yml`:
1. **Workspace-Wide Target Selection**: In GitHub CI's `test-std` job, CI executes `cargo test --workspace --all-targets --quiet`. In local `scripts/admission_guard.ps1 -CIParity`, Step 6 executes `cargo test --all-targets --quiet` (without `--workspace`), which selects targets for default root/member packages rather than explicitly enumerating every workspace package. Full workspace validation locally requires running `cargo test --workspace --all-targets`. Aligning the local helper script is tracked as separate tooling maintenance.
2. **Doctests**: `-CIParity` does not run `cargo test --workspace --doc --quiet` (run in CI job `test-std`).
3. **7hell Fast Gate**: `-CIParity` does not run `.\tools\7hell\run_ci.ps1` (run in CI job `pcc-qualification-7hell`).
4. **SARIF / Upload Steps**: `-CIParity` does not run GitHub-specific report upload actions or multi-OS matrix runners.

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

### Harness Scope Checks
- **Pre-Commit Working Tree Check**: Evaluates uncommitted/staged working-tree changes against `.harness/current.task.yaml`:
  ```powershell
  pwsh -File scripts/harness-check.ps1
  ```
- **Post-Commit PR Scope Check**: Evaluates all committed files against the base branch:
  ```powershell
  git diff --name-only origin/main...HEAD
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

### Context Economy & Checkpoint Validation
Validates machine-readable context checkpoints against `.harness/context-checkpoint.schema.json` and checks repository staleness. See [`docs/agents/CONTEXT.md`](CONTEXT.md#7-checkpoint-schema--validation) section 7 for the full schema-vs-semantic-vs-repository responsibility split.
- **Structural Validation & Schema Compliance** (enforced via PowerShell's built-in `Test-Json` against the schema, plus a supplemental RFC 3339 check for the one schema keyword `Test-Json` does not evaluate):
  ```powershell
  pwsh -File scripts/context_checkpoint_check.ps1 -Checkpoint <path>
  ```
- **Staleness & Drift Detection** (validates against live HEAD, active Harness task, and authority file hashes):
  ```powershell
  pwsh -File scripts/context_checkpoint_check.ps1 -Checkpoint <path> -AgainstCurrentRepo
  ```
- **Validator Built-in Self-Test Suite**:
  ```powershell
  pwsh -File scripts/context_checkpoint_check.ps1 -SelfTest
  ```

---

## 5. Verification Checklist by Risk Tier

### Tier R0 (Informational / Docs)
1. [ ] Formatting: `pwsh -File scripts/workspace_fmt_check.ps1`
2. [ ] Harness (Pre-Commit): `pwsh -File scripts/harness-check.ps1`
3. [ ] Git Diff Hygiene: `git diff --check origin/main`
4. [ ] Committed Scope: `git diff --name-only origin/main...HEAD`

### Tier R1 (Private / Isolated)
1. [ ] All R0 checks
2. [ ] PR-Ready Gate: `pwsh -File scripts/admission_guard.ps1 -PRReady`
3. [ ] Component Unit Tests: `cargo test -p <crate> --quiet`

### Tier R2 (Boundary / Contract)
1. [ ] All R1 checks
2. [ ] CI Parity Gate: `pwsh -File scripts/admission_guard.ps1 -CIParity`
3. [ ] Boundary Guards: `cargo test --test legacy_guards --test public_api_contracts --quiet`
4. [ ] Golden / Compatibility Fixtures: `cargo test --test golden_semcode --quiet`

### Tier R3 (Critical / Systemic)
1. [ ] All R2 checks
2. [ ] Full Preflight: `pwsh -File scripts/admission_guard.ps1 -FullPreflight`
3. [ ] 7hell Qualification: `pwsh -File tools/7hell/run_ci.ps1`
4. [ ] Fresh-context adversarial doubt review
