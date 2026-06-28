# Admission Guard for Semantic.
#
# This script provides a repeatable local pre-admission signal. It does not
# replace GitHub Actions and must not be treated as a release gate by itself.
#
# Formatting gate:
# cargo fmt --all --check
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

function Invoke-LocalCiStep {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Name,

        [Parameter(Mandatory = $true)]
        [scriptblock] $Command
    )

    Write-Host ""
    Write-Host "== $Name =="
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "local_ci step failed: $Name"
    }
}

function Assert-CleanWorkingTreeForMergePreflight {
    $statusLines = git status --porcelain --untracked-files=all
    if ($LASTEXITCODE -ne 0) {
        throw "failed to inspect working tree status before merge preflight"
    }

    $relevantLines = @(
        $statusLines | Where-Object {
            $_ -and ($_ -notmatch '\.claude([\\/]|$)')
        }
    )

    if ($relevantLines.Count -gt 0) {
        $details = $relevantLines -join [Environment]::NewLine
        throw "merge preflight requires a clean working tree; commit/stash changes first`n$details"
    }
}

function Invoke-LocalCiMergePreflight {
    param(
        [Parameter(Mandatory = $true)]
        [string] $BaseRef
    )

    Assert-CleanWorkingTreeForMergePreflight

    Invoke-LocalCiStep "merge preflight against $BaseRef" {
        git fetch origin main
        if ($LASTEXITCODE -ne 0) {
            throw "failed to fetch origin main for merge preflight"
        }

        $HeadSha = git rev-parse HEAD
        if ($LASTEXITCODE -ne 0) {
            throw "failed to capture current HEAD SHA for merge preflight"
        }

        $MergePreflightRoot = Join-Path $TempRoot "semantic_merge_preflight"
        New-Item -ItemType Directory -Force -Path $MergePreflightRoot | Out-Null

        $MergePreflightWorktree = Join-Path $MergePreflightRoot ([System.Guid]::NewGuid().ToString("N"))
        $WorktreeCreated = $false

        try {
            git worktree add --detach $MergePreflightWorktree $BaseRef
            if ($LASTEXITCODE -ne 0) {
                throw "failed to create merge preflight worktree at '$MergePreflightWorktree'"
            }
            $WorktreeCreated = $true

            Push-Location $MergePreflightWorktree
            try {
                git merge --no-commit --no-ff $HeadSha
                if ($LASTEXITCODE -ne 0) {
                    throw "merge preflight failed: merging current HEAD into '$BaseRef' produced conflicts"
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
    Invoke-LocalCiStep "cargo fmt --all --check" {
        cargo fmt --all --check
    }
}

function Invoke-PRReadyGate {
    Invoke-LocalCiStep "cargo check --workspace --all-targets" {
        cargo check --workspace --all-targets
    }
    Invoke-LocalCiStep "cargo clippy --workspace --all-targets" {
        cargo clippy --workspace --all-targets -- -D warnings
    }
    Invoke-LocalCiStep "cargo fmt --all --check" {
        cargo fmt --all --check
    }
    Invoke-LocalCiStep "cargo test --workspace --quiet" {
        cargo test --workspace --quiet
    }
    Invoke-LocalCiStep "cargo test -q --test public_api_contracts" {
        cargo test -q --test public_api_contracts
    }
}

function Invoke-CIParityGate {
    Invoke-LocalCiStep "ci/pr-ready: cargo fmt --all --check" {
        cargo fmt --all --check
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
        if (Test-Path $ProjectRootSmokeTempDir) {
            Remove-Item -Recurse -Force $ProjectRootSmokeTempDir
        }
        New-Item -ItemType Directory -Force -Path $ProjectRootSmokeTempDir | Out-Null

        $ProjectRootSmokeOut = Join-Path $ProjectRootSmokeTempDir "out.smc"

        & $SmcBinary check $ProjectRootSmokeFixture
        & $SmcBinary run $ProjectRootSmokeFixture
        & $SmcBinary compile $ProjectRootSmokeFixture -o $ProjectRootSmokeOut
        & $SmcBinary verify $ProjectRootSmokeOut
        & $SmcBinary run-smc $ProjectRootSmokeOut

        if (Test-Path $ProjectRootSmokeTempDir) {
            Remove-Item -Recurse -Force $ProjectRootSmokeTempDir
        }
    }
    Invoke-LocalCiStep "package-baseline project-root fixture smoke" {
        if (Test-Path $PackageBaselineSmokeTempDir) {
            Remove-Item -Recurse -Force $PackageBaselineSmokeTempDir
        }
        New-Item -ItemType Directory -Force -Path $PackageBaselineSmokeTempDir | Out-Null

        $PackageBaselineSmokeOut = Join-Path $PackageBaselineSmokeTempDir "out-package-baseline.smc"

        & $SmcBinary check $PackageBaselineSmokeFixture
        & $SmcBinary run $PackageBaselineSmokeFixture
        & $SmcBinary compile $PackageBaselineSmokeFixture -o $PackageBaselineSmokeOut
        & $SmcBinary verify $PackageBaselineSmokeOut
        & $SmcBinary run-smc $PackageBaselineSmokeOut

        if (Test-Path $PackageBaselineSmokeTempDir) {
            Remove-Item -Recurse -Force $PackageBaselineSmokeTempDir
        }
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

if ($Quick) {
    Write-Host "`n=== GATE MODE: Quick ==="
    Invoke-QuickGate
    Write-Host "`nADMISSION GUARD QUICK PASS"
    exit 0
}

if ($PRReady) {
    Write-Host "`n=== GATE MODE: PRReady ==="
    Invoke-PRReadyGate
    Write-Host "`nADMISSION GUARD PR-READY PASS"
    exit 0
}

if ($Readiness) {
    Write-Host "`n=== GATE MODE: Readiness ==="
    Invoke-ReadinessGate
    Write-Host "`nADMISSION GUARD READINESS PASS"
    exit 0
}

if ($CIParity) {
    Write-Host "`n=== GATE MODE: CIParity ==="
    Invoke-CIParityGate
    Write-Host "`nADMISSION GUARD CI PARITY PASS"
    exit 0
}

if ($MergePreflight) {
    Write-Host "`n=== GATE MODE: MergePreflight ==="
    Invoke-PRReadyGate
    Invoke-ReadinessGate
    Invoke-LegacyAdditionalChecks
    Invoke-LocalCiMergePreflight -BaseRef $BaseRef
    Invoke-LocalCiStep "git diff --check" {
        git diff --check
    }
    Write-Host "`nADMISSION GUARD MERGE PREFLIGHT PASS"
    exit 0
}

if ($FullPreflight) {
    Write-Host "`n=== GATE MODE: FullPreflight ==="
    Invoke-PRReadyGate
    Invoke-ReadinessGate
    Invoke-LegacyAdditionalChecks
    Invoke-LocalCiMergePreflight -BaseRef $BaseRef
    Invoke-LocalCiStep "git diff --check" {
        git diff --check
    }
    Write-Host "`nADMISSION GUARD FULL PREFLIGHT PASS"
    exit 0
}

Write-Host "`n=== GATE MODE: Legacy Default ==="
Invoke-PRReadyGate
Invoke-ReadinessGate
Invoke-LegacyAdditionalChecks
Invoke-LocalCiStep "git diff --check" {
    git diff --check
}
Write-Host "`nlocal_ci passed"
