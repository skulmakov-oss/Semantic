# Admission Guard for Semantic.
#
# This script provides a repeatable local pre-admission signal. It does not
# replace GitHub Actions and must not be treated as a release gate by itself.
#
# Formatting gate:
# equivalent to `cargo fmt --all --check`, run per-package (see
# Invoke-WorkspaceFmtCheck) so it doesn't exceed the Windows CreateProcess
# command-line length limit on this workspace.
#
# Every native command in a multi-command step is checked individually
# (Invoke-CheckedNative), the merge-preflight base ref is fetched as requested
# (Update-MergePreflightBaseRef), and the default/merge/full modes finish by
# failing on any tracked change (Assert-TrackedWorktreeUnchanged). Mode
# composition lives in Get-AdmissionGuardPlan so FullPreflight stays a
# superset of the default guard. Helpers: scripts/admission_guard_lib.ps1.
#
# This script includes formatting because the baseline has been normalized.
# Do not use formatting as a substitute for behavior checks. After any manual
# formatting attempt, inspect `git diff --name-only` and revert unrelated churn.

param(
    [switch] $Quick,
    [switch] $PRReady,
    [switch] $MergePreflight,
    [switch] $Readiness,
    [switch] $CIParity,
    [switch] $FullPreflight,

    [string] $BaseRef = "origin/main"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $RepoRoot

. (Join-Path $PSScriptRoot "workspace_fmt_check.ps1")
. (Join-Path $PSScriptRoot "admission_guard_lib.ps1")

function Invoke-LocalCiMergePreflight {
    param(
        [Parameter(Mandatory = $true)]
        [string] $BaseRef
    )

    Assert-CleanWorkingTreeForMergePreflight

    Invoke-LocalCiStep "merge preflight against $BaseRef" {
        $BaseSha = Update-MergePreflightBaseRef -BaseRef $BaseRef

        $HeadSha = git rev-parse HEAD
        if ($LASTEXITCODE -ne 0) {
            throw "failed to capture current HEAD SHA for merge preflight"
        }

        $MergePreflightRoot = Join-Path $TempRoot "semantic_merge_preflight"
        New-Item -ItemType Directory -Force -Path $MergePreflightRoot | Out-Null

        $MergePreflightWorktree = Join-Path $MergePreflightRoot ([System.Guid]::NewGuid().ToString("N"))
        $WorktreeCreated = $false

        try {
            git worktree add --detach $MergePreflightWorktree $BaseSha
            if ($LASTEXITCODE -ne 0) {
                throw "failed to create merge preflight worktree at '$MergePreflightWorktree'"
            }
            $WorktreeCreated = $true

            Push-Location $MergePreflightWorktree
            try {
                git merge --no-commit --no-ff $HeadSha
                if ($LASTEXITCODE -ne 0) {
                    throw "merge preflight failed: merging current HEAD into '$BaseRef' ($BaseSha) produced conflicts"
                }

                cargo test --all-targets --quiet
                if ($LASTEXITCODE -ne 0) {
                    throw "merge preflight failed: cargo test --all-targets --quiet"
                }

                cargo check --no-default-features --quiet
                if ($LASTEXITCODE -ne 0) {
                    throw "merge preflight failed: cargo check --no-default-features --quiet"
                }
            } finally {
                Pop-Location
            }
        } finally {
            Set-Location $RepoRoot
            if ($WorktreeCreated) {
                git worktree remove --force $MergePreflightWorktree
                if ($LASTEXITCODE -ne 0) {
                    Write-Warning "failed to remove merge preflight worktree '$MergePreflightWorktree'; remove it manually"
                    if (Test-Path $MergePreflightWorktree) {
                        Remove-Item -Recurse -Force $MergePreflightWorktree -ErrorAction SilentlyContinue
                    }
                }
            } elseif (Test-Path $MergePreflightWorktree) {
                Remove-Item -Recurse -Force $MergePreflightWorktree -ErrorAction SilentlyContinue
            }
        }
    }
}

$TempRoot = if ($env:TEMP) {
    $env:TEMP
} elseif ($env:TMP) {
    $env:TMP
} else {
    [System.IO.Path]::GetTempPath()
}

function Invoke-QuickGate {
    Invoke-LocalCiStep "cargo check --workspace --all-targets" {
        cargo check --workspace --all-targets
    }
    Invoke-LocalCiStep "cargo fmt --all --check (per-package, Windows-safe)" {
        Invoke-WorkspaceFmtCheck
    }
}

function Invoke-PRReadyGate {
    Invoke-LocalCiStep "cargo check --workspace --all-targets" {
        cargo check --workspace --all-targets
    }
    Invoke-LocalCiStep "cargo clippy --workspace --all-targets" {
        cargo clippy --workspace --all-targets -- -D warnings
    }
    Invoke-LocalCiStep "cargo fmt --all --check (per-package, Windows-safe)" {
        Invoke-WorkspaceFmtCheck
    }
    Invoke-LocalCiStep "cargo test --workspace --quiet" {
        cargo test --workspace --quiet
    }
    Invoke-LocalCiStep "cargo test -q --test public_api_contracts" {
        cargo test -q --test public_api_contracts
    }
}

function Invoke-CIParityGate {
    Invoke-LocalCiStep "ci/pr-ready: cargo fmt --all --check (per-package, Windows-safe)" {
        Invoke-WorkspaceFmtCheck
    }
    Invoke-LocalCiStep "ci/pr-ready: cargo clippy --workspace --all-targets -- -D warnings" {
        cargo clippy --workspace --all-targets -- -D warnings
    }

    Invoke-LocalCiStep "ci/boundary-enforcement: cargo test --test legacy_guards --quiet" {
        cargo test --test legacy_guards --quiet
    }
    Invoke-LocalCiStep "ci/boundary-enforcement: cargo test --test frontend_boundaries --quiet" {
        cargo test --test frontend_boundaries --quiet
    }
    Invoke-LocalCiStep "ci/boundary-enforcement: cargo test --test ir_opt_boundaries --quiet" {
        cargo test --test ir_opt_boundaries --quiet
    }
    Invoke-LocalCiStep "ci/boundary-enforcement: cargo test --test dependency_boundaries --quiet" {
        cargo test --test dependency_boundaries --quiet
    }

    Invoke-LocalCiStep "ci/public-api-guard: cargo test --test public_api_contracts --quiet" {
        cargo test --test public_api_contracts --quiet
    }

    Invoke-LocalCiStep "ci/runtime-release-gates: cargo test --test golden_semcode --quiet" {
        cargo test --test golden_semcode --quiet
    }
    Invoke-LocalCiStep "ci/runtime-release-gates: cargo test --test prometheus_runtime_matrix --quiet" {
        cargo test --test prometheus_runtime_matrix --quiet
    }
    Invoke-LocalCiStep "ci/runtime-release-gates: cargo test --test prometheus_runtime_goldens --quiet" {
        cargo test --test prometheus_runtime_goldens --quiet
    }
    Invoke-LocalCiStep "ci/runtime-release-gates: cargo test --test prometheus_runtime_negative_goldens --quiet" {
        cargo test --test prometheus_runtime_negative_goldens --quiet
    }
    Invoke-LocalCiStep "ci/runtime-release-gates: cargo test --test prometheus_runtime_compat_matrix --quiet" {
        cargo test --test prometheus_runtime_compat_matrix --quiet
    }

    $ManifestPath = Join-Path $TempRoot "semantic_v1_release_bundle_manifest.json"
    Invoke-LocalCiStep "ci/release-bundle-process: verify release bundle process" {
        pwsh -File scripts/verify_release_bundle.ps1 -ManifestPath $ManifestPath
    }

    Invoke-LocalCiStep "ci/test-std: cargo test --all-targets --quiet" {
        cargo test --all-targets --quiet
    }

    Invoke-LocalCiStep "ci/check-no-std: cargo check --no-default-features --quiet" {
        cargo check --no-default-features --quiet
    }
}

function Invoke-ReadinessGate {
    $ManifestPath = Join-Path $TempRoot "semantic_v1_release_bundle_manifest.json"
    $ProjectRootSmokeFixture = Join-Path $RepoRoot "examples/qualification/pcc9_project_root_minimal"
    $ProjectRootSmokeTempDir = Join-Path $TempRoot "semantic_project_root_local_ci_smoke"
    $PackageBaselineSmokeFixture = Join-Path $RepoRoot "examples/qualification/pcc9_project_root_package_baseline"
    $PackageBaselineSmokeTempDir = Join-Path $TempRoot "semantic_project_root_package_baseline_local_ci_smoke"
    $ExeSuffix = if ($IsWindows) { ".exe" } else { "" }
    $SmcBinary = Join-Path $RepoRoot "target/debug/smc$ExeSuffix"

    Invoke-LocalCiStep "cargo build --bin smc --bin svm" {
        cargo build --bin smc --bin svm
    }
    Invoke-LocalCiStep "verify release bundle process" {
        pwsh -File scripts/verify_release_bundle.ps1 -ManifestPath $ManifestPath
    }
    Invoke-LocalCiStep "canonical project-root fixture smoke" {
        Invoke-SmcProjectRootSmoke -SmcBinary $SmcBinary -FixtureRoot $ProjectRootSmokeFixture `
            -TempDirectory $ProjectRootSmokeTempDir -ArtifactName "out.smc"
    }
    Invoke-LocalCiStep "package-baseline project-root fixture smoke" {
        Invoke-SmcProjectRootSmoke -SmcBinary $SmcBinary -FixtureRoot $PackageBaselineSmokeFixture `
            -TempDirectory $PackageBaselineSmokeTempDir -ArtifactName "out-package-baseline.smc"
    }
    Invoke-LocalCiStep "smc 7hell human smoke" {
        & $SmcBinary 7hell tests/fixtures/7hell_e1/valid_minimal.sm
    }
    Invoke-LocalCiStep "smc 7hell json smoke" {
        & $SmcBinary 7hell tests/fixtures/7hell_e1/valid_minimal.sm --json
    }
}

function Invoke-LegacyAdditionalChecks {
    Invoke-LocalCiStep "cargo check --no-default-features --quiet" {
        cargo check --no-default-features --quiet
    }
    Invoke-LocalCiStep "cargo test --test legacy_guards --quiet" {
        cargo test --test legacy_guards --quiet
    }
    Invoke-LocalCiStep "cargo test --test frontend_boundaries --quiet" {
        cargo test --test frontend_boundaries --quiet
    }
    Invoke-LocalCiStep "cargo test --test ir_opt_boundaries --quiet" {
        cargo test --test ir_opt_boundaries --quiet
    }
    Invoke-LocalCiStep "cargo test --test dependency_boundaries --quiet" {
        cargo test --test dependency_boundaries --quiet
    }
}

function Invoke-AdmissionGuardGate {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Gate
    )

    switch ($Gate) {
        "Quick" { Invoke-QuickGate }
        "PRReady" { Invoke-PRReadyGate }
        "Readiness" { Invoke-ReadinessGate }
        "CIParity" { Invoke-CIParityGate }
        "LegacyAdditional" { Invoke-LegacyAdditionalChecks }
        "MergePreflight" { Invoke-LocalCiMergePreflight -BaseRef $BaseRef }
        "DiffCheck" {
            Invoke-LocalCiStep "git diff --check" {
                git diff --check
            }
        }
        "TrackedClean" {
            Invoke-LocalCiStep "tracked worktree unchanged (git diff --exit-code)" {
                Assert-TrackedWorktreeUnchanged
            }
        }
        default { throw "unknown admission guard gate '$Gate'" }
    }
}

$Modes = @(
    @{ Enabled = $Quick.IsPresent; Mode = "Quick"; Banner = "Quick"; Pass = "ADMISSION GUARD QUICK PASS" },
    @{ Enabled = $PRReady.IsPresent; Mode = "PRReady"; Banner = "PRReady"; Pass = "ADMISSION GUARD PR-READY PASS" },
    @{ Enabled = $Readiness.IsPresent; Mode = "Readiness"; Banner = "Readiness"; Pass = "ADMISSION GUARD READINESS PASS" },
    @{ Enabled = $CIParity.IsPresent; Mode = "CIParity"; Banner = "CIParity"; Pass = "ADMISSION GUARD CI PARITY PASS" },
    @{ Enabled = $MergePreflight.IsPresent; Mode = "MergePreflight"; Banner = "MergePreflight"; Pass = "ADMISSION GUARD MERGE PREFLIGHT PASS" },
    @{ Enabled = $FullPreflight.IsPresent; Mode = "FullPreflight"; Banner = "FullPreflight"; Pass = "ADMISSION GUARD FULL PREFLIGHT PASS" }
)

$Selected = $Modes | Where-Object { $_.Enabled } | Select-Object -First 1
if (-not $Selected) {
    $Selected = @{ Mode = "Default"; Banner = "Legacy Default"; Pass = "local_ci passed" }
}

Write-Host "`n=== GATE MODE: $($Selected.Banner) ==="
foreach ($Gate in (Get-AdmissionGuardPlan -Mode $Selected.Mode)) {
    Invoke-AdmissionGuardGate -Gate $Gate
}
Write-Host "`n$($Selected.Pass)"
exit 0
