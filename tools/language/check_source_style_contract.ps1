Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Narrow drift guard for docs/spec/source_style.md (Semantic Canonical Source
# Style v0, issues #1538/#1549). This does not parse Semantic source;
# syntax-level claims are checked by `tests/canonical_source_style.rs`
# against the real frontend/toolchain instead. This script discovers the
# canonical example surface from disk rather than relying on a hardcoded
# file list, so a newly added example cannot silently skip these checks.

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$specPath = Join-Path $repoRoot "docs/spec/source_style.md"
$examplesReadmePath = Join-Path $repoRoot "examples/canonical/README.md"
$canonicalRoot = Join-Path $repoRoot "examples/canonical"

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

# --- 3b. Indentation depth must stay a Section B claim, not Section A. -----
# Regression guard for a reviewed P2: no tool validates nesting depth today;
# only the absence of tab characters is machine-checked (Section A).
if ($spec.Contains("Indentation step is 4 spaces per nesting level")) {
    Fail "$specPath lists 4-space nesting as a Section A required invariant; it must live in Section B (no tool validates nesting depth)"
}
Assert-Contains -Content $spec -Text "does not structurally validate indentation depth" -Label "Indentation-depth non-claim" -Where $specPath
Assert-Contains -Content $spec -Text "no tool currently validates nesting depth" -Label "Indentation-depth non-claim" -Where $specPath

# --- 4. Discover the canonical example surface from disk. ------------------
if (-not (Test-Path -LiteralPath $canonicalRoot -PathType Container)) {
    Fail "Missing canonical examples directory: $canonicalRoot"
}
$diskExampleNames = @(
    Get-ChildItem -LiteralPath $canonicalRoot -Directory | ForEach-Object { $_.Name }
) | Sort-Object -Unique
if ($diskExampleNames.Count -eq 0) {
    Fail "No example directories found under $canonicalRoot"
}
foreach ($name in $diskExampleNames) {
    $mainPath = Join-Path $canonicalRoot "$name/src/main.sm"
    $readmePath = Join-Path $canonicalRoot "$name/README.md"
    Assert-FileExists -Path $mainPath -Label "canonical example entry file for '$name'"
    Assert-FileExists -Path $readmePath -Label "canonical example README for '$name'"
}

# --- 5. The authoritative inventory table must cover every discovered ------
#        example, and every table row must point at a real directory.
Assert-FileExists -Path $examplesReadmePath -Label "canonical examples authoritative inventory"
$examplesReadme = Get-Content -Raw -LiteralPath $examplesReadmePath
$marker = "## Canonical Examples"
$markerIndex = $examplesReadme.IndexOf($marker)
if ($markerIndex -lt 0) {
    Fail "$examplesReadmePath is missing the '$marker' authoritative inventory heading"
}
$tableSection = $examplesReadme.Substring($markerIndex)
$inventoryNames = [System.Collections.Generic.HashSet[string]]::new()
foreach ($line in ($tableSection -split "`n")) {
    $trimmed = $line.Trim()
    if (-not $trimmed.StartsWith("|")) { continue }
    if ($trimmed.Replace(" ", "").StartsWith("|---")) { continue }
    $cells = $trimmed.Trim("|").Split("|") | ForEach-Object { $_.Trim() }
    if ($cells.Count -ne 7) { continue }
    if ($cells[0] -eq "#") { continue }
    $inventoryNames.Add($cells[1].Trim("``")) | Out-Null
}
if ($inventoryNames.Count -eq 0) {
    Fail "$examplesReadmePath authoritative inventory table parsed to zero rows"
}

foreach ($name in $diskExampleNames) {
    if (-not $inventoryNames.Contains($name)) {
        Fail "examples/canonical/$name has no row in the authoritative inventory table in $examplesReadmePath"
    }
}
foreach ($name in $inventoryNames) {
    if (-not (Test-Path -LiteralPath (Join-Path $canonicalRoot $name) -PathType Container)) {
        Fail "$examplesReadmePath inventory row '$name' points at a missing examples/canonical/ directory"
    }
}

# --- 5b. docs/examples_index.md must not reference a missing example. ------
$examplesIndexPath = Join-Path $repoRoot "docs/examples_index.md"
Assert-FileExists -Path $examplesIndexPath -Label "examples index"
$examplesIndexContent = Get-Content -Raw -LiteralPath $examplesIndexPath
foreach ($m in [regex]::Matches($examplesIndexContent, "examples/canonical/([A-Za-z0-9_]+)/")) {
    $name = $m.Groups[1].Value
    if (-not (Test-Path -LiteralPath (Join-Path $canonicalRoot $name) -PathType Container)) {
        Fail "$examplesIndexPath references missing example directory: examples/canonical/$name"
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
    "docs/spec/source_style.md",
    "examples/canonical/README.md"
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

# --- 6b. A current doc must not point at the historical draft pack as if ---
#         it were the current canonical pack.
$historicalLabels = @("historical", "planning-only", "older")
foreach ($rel in $currentFacingDocs) {
    $path = Join-Path $repoRoot $rel
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { continue }
    $content = Get-Content -Raw -LiteralPath $path
    if (-not $content.Contains("readiness_draft_canonical")) { continue }
    $labelled = $false
    foreach ($label in $historicalLabels) {
        if ($content.Contains($label)) { $labelled = $true; break }
    }
    if (-not $labelled) {
        Fail "$path mentions readiness_draft_canonical without labelling it historical/planning-only/older"
    }
}

# --- 6c. A non-executable inventory row must never be shown with a run/ ----
#         compile/verify command in current-facing docs (that would claim a
#         stronger qualification level than the row actually registers).
$nonExecutableNames = @()
foreach ($line in ($tableSection -split "`n")) {
    $trimmed = $line.Trim()
    if (-not $trimmed.StartsWith("|")) { continue }
    if ($trimmed.Replace(" ", "").StartsWith("|---")) { continue }
    $cells = $trimmed.Trim("|").Split("|") | ForEach-Object { $_.Trim() }
    if ($cells.Count -ne 7) { continue }
    if ($cells[0] -eq "#") { continue }
    $qualification = $cells[4]
    if ($qualification -notlike "*executable*") {
        $nonExecutableNames += $cells[1].Trim("``")
    }
}
foreach ($rel in @("README.md", "docs/wiki/current_status.md", "docs/examples_index.md")) {
    $path = Join-Path $repoRoot $rel
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { continue }
    $content = Get-Content -Raw -LiteralPath $path
    foreach ($name in $nonExecutableNames) {
        foreach ($verb in @("run", "compile", "verify")) {
            if ($content -match "$verb\s+examples/canonical/$name(/|\b)") {
                Fail "$path shows '$verb' against non-executable example '$name'; only its registered qualification command is allowed"
            }
        }
    }
}

# --- 7. The complete canonical pack must currently pass `smc fmt --check`. -
Push-Location $repoRoot
try {
    cargo run --quiet --bin smc -- fmt --check examples/canonical | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Fail "smc fmt --check reports unformatted file(s) under examples/canonical"
    }
}
finally {
    Pop-Location
}

Write-Host "PASS: Semantic Canonical Source Style v0 contract is intact ($($diskExampleNames.Count) canonical examples discovered)" -ForegroundColor Green
