Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Narrow drift guard for docs/spec/source_style.md (Semantic Canonical Source
# Style v0, issue #1538). This does not parse Semantic source; syntax-level
# claims are checked by `tests/canonical_source_style.rs` against the real
# frontend/toolchain instead.

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$specPath = Join-Path $repoRoot "docs/spec/source_style.md"
$examplesReadmePath = Join-Path $repoRoot "examples/canonical/README.md"

function Fail {
    param([string]$Message)
    Write-Host "FAIL: $Message" -ForegroundColor Red
    exit 1
}

function Assert-FileExists {
    param([string]$Path, [string]$Label)
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Fail "Missing $($Label): $Path"
    }
}

function Assert-Contains {
    param([string]$Content, [string]$Text, [string]$Label, [string]$Where)
    if (-not $Content.Contains($Text)) {
        Fail "$Label missing from $($Where): '$Text'"
    }
}

# --- 1. The contract document itself must exist. ---------------------------
Assert-FileExists -Path $specPath -Label "source style contract"
$spec = Get-Content -Raw -LiteralPath $specPath

# --- 2. Required classification vocabulary/sections must be present. -------
$requiredSections = @(
    "Semantic Canonical Source Style v0",
    "Required lexical/file invariant",
    "Canonical presentation rule",
    "Permitted alternative author style",
    "Future formatter behavior",
    "Rust-like executable surface",
    "Logos declarative surface",
    "## A. Required Lexical/File Invariants",
    "## B. Canonical Presentation Rules",
    "## C. Permitted Alternative Author Style",
    "## D. Formatter Contract"
)
foreach ($section in $requiredSections) {
    Assert-Contains -Content $spec -Text $section -Label "Required section" -Where $specPath
}

# --- 3. The formatter contract must not overclaim automatic rewriting. -----
# `smc fmt` is lexical-only today; the contract must keep saying so.
$formatterDisclaimers = @(
    "does **not** currently:",
    "re-indent code;",
    "wrap or unwrap lines to a width target;",
    "convert between block-bodied and expression-bodied functions;",
    "distinguish Rust-like from Logos indentation"
)
foreach ($text in $formatterDisclaimers) {
    Assert-Contains -Content $spec -Text $text -Label "Formatter non-claim" -Where $specPath
}

# --- 4. The required canonical examples must exist on disk. ----------------
$requiredExamples = @(
    "examples/canonical/match_control_flow/src/main.sm",
    "examples/canonical/match_control_flow/README.md",
    "examples/canonical/rule_state_decision/src/main.sm",
    "examples/canonical/rule_state_decision/README.md",
    "examples/canonical/quad_cycle_logos/src/main.sm",
    "examples/canonical/quad_cycle_logos/README.md"
)
foreach ($rel in $requiredExamples) {
    Assert-FileExists -Path (Join-Path $repoRoot $rel) -Label "required canonical example file"
}

# --- 5. Public sample references must point at fixtures that still exist. --
Assert-FileExists -Path $examplesReadmePath -Label "canonical examples index"
$examplesReadme = Get-Content -Raw -LiteralPath $examplesReadmePath
$referencedDirs = [System.Collections.Generic.HashSet[string]]::new()
foreach ($m in [regex]::Matches($examplesReadme, "examples/canonical/([A-Za-z0-9_]+)/")) {
    [void]$referencedDirs.Add($m.Groups[1].Value)
}
foreach ($name in $referencedDirs) {
    $dirPath = Join-Path $repoRoot "examples/canonical/$name"
    if (-not (Test-Path -LiteralPath $dirPath -PathType Container)) {
        Fail "$examplesReadmePath references missing example directory: examples/canonical/$name"
    }
}

# --- 6. Current-facing docs must never show the two surfaces merged. -------
# The one forbidden illustrative shape is #1538's own non-executable sketch:
# a `System` block feeding straight into a Rust-like `fn main()`.
$forbiddenCombination = "System QuadCycle {"
$currentFacingDocs = @(
    "README.md",
    "docs/wiki/current_status.md",
    "docs/examples_index.md",
    "docs/language/semantic_syntax_signature.md",
    "docs/language/semantic_linguist_entry_draft.md",
    "docs/spec/source_style.md"
)
foreach ($rel in $currentFacingDocs) {
    $path = Join-Path $repoRoot $rel
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        continue
    }
    $content = Get-Content -Raw -LiteralPath $path
    if ($content.Contains($forbiddenCombination)) {
        Fail "$path presents the unsupported merged Rust-like+Logos combination '$forbiddenCombination' as current"
    }
}

# --- 7. Canonical examples must currently pass `smc fmt --check`. ----------
$exampleFiles = @(
    "examples/canonical/match_control_flow/src/main.sm",
    "examples/canonical/rule_state_decision/src/main.sm",
    "examples/canonical/quad_cycle_logos/src/main.sm"
)
Push-Location $repoRoot
try {
    foreach ($rel in $exampleFiles) {
        cargo run --quiet --bin smc -- fmt --check $rel | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Fail "smc fmt --check reports unformatted canonical example: $rel"
        }
    }
}
finally {
    Pop-Location
}

Write-Host "PASS: Semantic Canonical Source Style v0 contract is intact" -ForegroundColor Green
