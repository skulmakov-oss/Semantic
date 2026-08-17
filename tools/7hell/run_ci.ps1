$ErrorActionPreference = "Stop"

function Assert-Success {
    param([string]$message)
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAIL: $message" -ForegroundColor Red
        exit 1
    }
}

# Shared with scripts/admission_guard.ps1: one authoritative fmt-check
# mechanism, not two copies that can drift apart. See #1796 - bare
# `cargo fmt --check` is not root-scoped (its actual default scope already
# reaches the full local-package graph) and hits the same Windows
# CreateProcess argument-length limit as `cargo fmt --all --check` once the
# package set is large enough; it only happened to survive on CI's short
# checkout path.
. (Join-Path $PSScriptRoot "..\..\scripts\workspace_fmt_check.ps1")

Write-Host "========================================="
Write-Host "  7hell PCC Qualification Fast Gate      "
Write-Host "========================================="
Write-Host ""

# -----------------------------------------------------------------------------
# Hell 1
Write-Host "[ Hell 1 ] Workspace Health..." -ForegroundColor Cyan
try {
    Invoke-WorkspaceFmtCheck
} catch {
    Write-Host "FAIL: cargo fmt failed - $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
cargo check --workspace --all-features
Assert-Success "cargo check failed"
Write-Host "PASS: Hell 1" -ForegroundColor Green

# -----------------------------------------------------------------------------
# Hell 2
Write-Host "`n[ Hell 2 ] Trust Boundary Guards..." -ForegroundColor Cyan
cargo test -p semantic_language --test trust_boundary_guards
Assert-Success "Trust boundary guards test failed"

$vm_tree = (cargo tree --edges normal -p sm-vm 2>&1) -join "`n"
if ($vm_tree -match "sm-ir" -or $vm_tree -match "sm-emit" -or $vm_tree -match "prom-ui") {
    Write-Host "FAIL: sm-vm depends on higher level crates (sm-ir, sm-emit, or prom-ui)" -ForegroundColor Red
    exit 1
}

$verify_tree = (cargo tree --edges normal -p sm-verify 2>&1) -join "`n"
if ($verify_tree -match "sm-ir" -or $verify_tree -match "sm-emit") {
    Write-Host "FAIL: sm-verify depends on higher level crates (sm-ir or sm-emit)" -ForegroundColor Red
    exit 1
}

$format_tree = (cargo tree --edges normal -p sm-format 2>&1) -join "`n"
if ($format_tree -match "sm-ir") {
    Write-Host "FAIL: sm-format depends on sm-ir" -ForegroundColor Red
    exit 1
}
Write-Host "PASS: Hell 2" -ForegroundColor Green

# -----------------------------------------------------------------------------
# Hell 3
Write-Host "`n[ Hell 3 ] SemCode Format Authority..." -ForegroundColor Cyan
cargo test -p sm-format --all-features
Assert-Success "sm-format tests failed"

$leak1 = Get-ChildItem -Path crates -Recurse -Filter *.rs | Select-String -Pattern "sm_ir::semcode_decode|sm_ir::semcode_format"
if ($leak1) {
    Write-Host "FAIL: Leakage of sm_ir decoding/formatting found in crates!" -ForegroundColor Red
    exit 1
}
$leak2 = Get-ChildItem -Path crates -Recurse -Filter *.rs | Select-String -Pattern "sm_emit::semcode_format"
if ($leak2) {
    Write-Host "FAIL: Leakage of sm_emit formatting found in crates!" -ForegroundColor Red
    exit 1
}
Write-Host "PASS: Hell 3" -ForegroundColor Green

# -----------------------------------------------------------------------------
# Hell 4
Write-Host "`n[ Hell 4 ] Verifier Negative Corpus..." -ForegroundColor Cyan
cargo test -p sm-verify --all-features --features sm-ir/profile-rust
Assert-Success "sm-verify tests failed"
Write-Host "PASS: Hell 4" -ForegroundColor Green

# -----------------------------------------------------------------------------
# Hell 5
Write-Host "`n[ Hell 5 ] VM Ownership Semantics..." -ForegroundColor Cyan
cargo test -p sm-vm --all-features
Assert-Success "sm-vm tests failed"
Write-Host "PASS: Hell 5" -ForegroundColor Green

Write-Host "`n========================================="
Write-Host "  FAST 7HELL GATE PASSED!               " -ForegroundColor Green
Write-Host "========================================="
