Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))

function Fail {
    param([string]$Message)
    Write-Error $Message
    exit 1
}

function Assert-FileExists {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Fail "FAIL: Missing required file: $Path"
    }
}

function Read-Text {
    param([string]$Path)

    return Get-Content -LiteralPath $Path -Raw
}

function Assert-Contains {
    param(
        [string]$Content,
        [string]$Needle,
        [string]$File
    )

    if ($Content.IndexOf($Needle, [System.StringComparison]::OrdinalIgnoreCase) -lt 0) {
        Fail "FAIL: Missing required anchor in ${File}: $Needle"
    }
}

function Assert-ContainsAny {
    param(
        [hashtable]$Contents,
        [string]$Needle
    )

    foreach ($entry in $Contents.GetEnumerator()) {
        if ($entry.Value.IndexOf($Needle, [System.StringComparison]::OrdinalIgnoreCase) -ge 0) {
            return
        }
    }

    Fail "FAIL: Missing required anchor: $Needle"
}

function Assert-NotContains {
    param(
        [hashtable]$Contents,
        [string]$Needle
    )

    foreach ($entry in $Contents.GetEnumerator()) {
        $sanitized = [regex]::Replace($entry.Value, '(?s)```.*?```', ' ')
        $sanitized = [regex]::Replace($sanitized, '(?<!`)`[^`]+`(?!`)', ' ')
        if ($sanitized.IndexOf($Needle, [System.StringComparison]::OrdinalIgnoreCase) -ge 0) {
            Fail "FAIL: Forbidden claim phrase found in $($entry.Key): $Needle"
        }
    }
}

$paths = @{
    basis = Join-Path $repoRoot "docs/spec/ui/projection_bundle_basis.md"
    gate = Join-Path $repoRoot "docs/spec/ui/projection_bundle_reader_parser_entry_gate.md"
    closeout = Join-Path $repoRoot "docs/roadmap/post_ui/projection_bundle_reader_evidence_closeout.md"
    index = Join-Path $repoRoot "docs/roadmap/post_ui/intent_driven_projection_closeout.md"
}

foreach ($path in $paths.Values) {
    Assert-FileExists -Path $path
}

$contents = @{}
foreach ($pair in $paths.GetEnumerator()) {
    $contents[$pair.Key] = Read-Text -Path $pair.Value
}

Assert-Contains -Content $contents.basis -Needle "Current achieved level: Level 3 baseline" -File $paths.basis
Assert-Contains -Content $contents.basis -Needle "narrow reader-facing fixture evidence" -File $paths.basis
Assert-Contains -Content $contents.basis -Needle "Levels 4–7 are not claimed" -File $paths.basis

Assert-Contains -Content $contents.gate -Needle "General Level 4 remains not claimed" -File $paths.gate
Assert-Contains -Content $contents.gate -Needle "A separate reader/parser basis is still required before any general Level 4 reader/parser claim." -File $paths.gate

Assert-Contains -Content $contents.closeout -Needle "Current achieved level: Level 3 baseline" -File $paths.closeout
Assert-Contains -Content $contents.closeout -Needle "Reader evidence status: narrow reader-facing fixture evidence only" -File $paths.closeout
Assert-Contains -Content $contents.closeout -Needle "General Level 4 status: not claimed" -File $paths.closeout
Assert-Contains -Content $contents.closeout -Needle "Level 5+ is not claimed" -File $paths.closeout

Assert-Contains -Content $contents.index -Needle "records the current fixture-facing reader evidence contour after the golden output pack." -File $paths.index

$requiredAnchors = @(
    "loader behavior is not claimed",
    "runtime behavior is not claimed",
    "production UI behavior is not claimed",
    "General Level 4 status: not claimed",
    "Level 5+ is not claimed",
    "narrow reader-facing fixture evidence"
)

foreach ($anchor in $requiredAnchors) {
    Assert-ContainsAny -Contents $contents -Needle $anchor
}

$forbiddenPhrases = @(
    "Level 4 achieved",
    "Level 4 implemented",
    "general Level 4 achieved",
    "general Level 4 reader/parser behavior is achieved",
    "reader/parser implemented",
    "parser implemented",
    "loader-ready",
    "runtime-ready",
    "production-ready",
    "activation-ready",
    "verification-ready",
    "security proven"
)

foreach ($phrase in $forbiddenPhrases) {
    Assert-NotContains -Contents $contents -Needle $phrase
}

Write-Host "PASS: ProjectionBundle claim boundaries remain aligned"
