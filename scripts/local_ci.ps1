$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# Local workflow-equivalent checker for Semantic.
#
# This script provides a repeatable local pre-admission signal. It does not
# replace GitHub Actions and must not be treated as a release gate by itself.
#
# Intentionally excluded for now:
# cargo fmt --check
#
# Reason:
# Existing formatting baseline drift causes unrelated failures. Formatting
# baseline normalization must be handled by a separate dedicated PR.
#
# This script does not run cargo fmt. Do not use formatting as part of behavior
# PRs until the formatting baseline is normalized. After any manual formatting
# attempt, inspect `git diff --name-only` and revert unrelated churn.

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

$TempRoot = if ($env:TEMP) {
    $env:TEMP
} elseif ($env:TMP) {
    $env:TMP
} else {
    [System.IO.Path]::GetTempPath()
}

$ManifestPath = Join-Path $TempRoot "semantic_v1_release_bundle_manifest.json"
$ExeSuffix = if ($IsWindows) { ".exe" } else { "" }
$SmcBinary = Join-Path $RepoRoot "target/debug/smc$ExeSuffix"

Invoke-LocalCiStep "cargo test --bin smc --quiet" {
    cargo test --bin smc --quiet
}

Invoke-LocalCiStep "cargo test --test 7hell_e1_report_snapshots --quiet" {
    cargo test --test 7hell_e1_report_snapshots --quiet
}

Invoke-LocalCiStep "cargo test --all-targets --quiet" {
    cargo test --all-targets --quiet
}

Invoke-LocalCiStep "cargo check --no-default-features --quiet" {
    cargo check --no-default-features --quiet
}

Invoke-LocalCiStep "verify release bundle process" {
    pwsh -File scripts/verify_release_bundle.ps1 -ManifestPath $ManifestPath
}

Invoke-LocalCiStep "cargo build --bin smc --bin svm" {
    cargo build --bin smc --bin svm
}

Invoke-LocalCiStep "smc 7hell human smoke" {
    & $SmcBinary 7hell tests/fixtures/7hell_e1/valid_minimal.sm
}

Invoke-LocalCiStep "smc 7hell json smoke" {
    & $SmcBinary 7hell tests/fixtures/7hell_e1/valid_minimal.sm --json
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

Invoke-LocalCiStep "cargo test --test public_api_contracts --quiet" {
    cargo test --test public_api_contracts --quiet
}

Invoke-LocalCiStep "git diff --check" {
    git diff --check
}

Write-Host ""
Write-Host "local_ci passed"
