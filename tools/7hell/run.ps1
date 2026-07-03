$ErrorActionPreference = "Stop"

function Assert-Success {
    param([string]$message)
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAIL: $message" -ForegroundColor Red
        exit 1
    }
}

Write-Host "========================================="
Write-Host "  7hell PCC Qualification Mini-Runner    "
Write-Host "========================================="
Write-Host ""

# -----------------------------------------------------------------------------
# Hell 1
Write-Host "[ Hell 1 ] Workspace Health..." -ForegroundColor Cyan
cargo fmt --check
Assert-Success "cargo fmt failed"
cargo check --workspace --all-features
Assert-Success "cargo check failed"
cargo test --workspace --all-features
Assert-Success "cargo test failed"
Write-Host "PASS: Hell 1" -ForegroundColor Green

# -----------------------------------------------------------------------------
# Hell 2
Write-Host "`n[ Hell 2 ] Trust Boundary Guards..." -ForegroundColor Cyan
# Run the trust boundary guards test in semantic_language if it exists
cargo test -p semantic_language --test trust_boundary_guards
Assert-Success "Trust boundary guards test failed"

# Check cargo tree for strict architectural rules
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

# check no leakage
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

# -----------------------------------------------------------------------------
# Hell 6
Write-Host "`n[ Hell 6 ] Source to SemCode Smoke..." -ForegroundColor Cyan
if (-Not (Test-Path "target/7hell")) {
    New-Item -ItemType Directory -Path "target/7hell" | Out-Null
}
cargo run --bin smc -- compile crates/sm-front/tests/adt_match_local.sm -o target/7hell/adt_match_local.smc
Assert-Success "Source compilation smoke test failed"
cargo run --bin smc -- compile tests/fixtures/pcc6_option_result_diagnostics/positive_option_result_ownership.sm -o target/7hell/positive_option_result_ownership.smc
Assert-Success "Option/Result compilation smoke test failed"
cargo run --bin smc -- compile tests/fixtures/pcc4_records/positive_record_field_ownership.sm -o target/7hell/positive_record_field_ownership.smc
Assert-Success "Record field compilation smoke test failed"
cargo run --bin smc -- compile tests/fixtures/pcc_tuple_ownership/positive_tuple_ownership.sm -o target/7hell/positive_tuple_ownership.smc
Assert-Success "Tuple ownership compilation smoke test failed"
cargo test --test cli_public_smoke_matrix
Assert-Success "Public CLI smoke matrix test failed"
Write-Host "Running PCC control-flow negative diagnostics..." -ForegroundColor Cyan
cargo test --test pcc_control_flow_negative
Assert-Success "PCC control-flow negative diagnostics test failed"
Write-Host "Running PCC text negative diagnostics..." -ForegroundColor Cyan
cargo test --test pcc_text_negative
Assert-Success "PCC text negative diagnostics test failed"
Write-Host "Running PCC collections negative diagnostics..." -ForegroundColor Cyan
cargo test --test pcc_collections_negative
Assert-Success "PCC collections negative diagnostics test failed"
Write-Host "Running PCC stdlib negative diagnostics..." -ForegroundColor Cyan
cargo test --test pcc_stdlib_negative
Assert-Success "PCC stdlib negative diagnostics test failed"
Remove-Item -Path "target/7hell/adt_match_local.smc" -Force -ErrorAction Ignore
Remove-Item -Path "target/7hell/positive_option_result_ownership.smc" -Force -ErrorAction Ignore
Remove-Item -Path "target/7hell/positive_record_field_ownership.smc" -Force -ErrorAction Ignore
Remove-Item -Path "target/7hell/positive_tuple_ownership.smc" -Force -ErrorAction Ignore
Write-Host "PASS: Hell 6" -ForegroundColor Green

# -----------------------------------------------------------------------------
# Hell 7
Write-Host "`n[ Hell 7 ] PCC Documentation Integrity..." -ForegroundColor Cyan
$docs = @(
    "docs/roadmap/pcc/adt_payload_ownership_matrix.md",
    "docs/roadmap/pcc/adt_payload_ownership_slice_closeout.md",
    "docs/architecture/adt_payload_ownership_paths.md"
)

foreach ($doc in $docs) {
    if (-Not (Test-Path $doc)) {
        Write-Host "FAIL: Missing documentation file: $doc" -ForegroundColor Red
        exit 1
    }
}
Write-Host "PASS: Hell 7" -ForegroundColor Green

Write-Host "`n========================================="
Write-Host "  ALL 7 GATES PASSED!                    " -ForegroundColor Green
Write-Host "========================================="
