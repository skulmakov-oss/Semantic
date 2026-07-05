Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))

function Fail {
    param([string]$Message)
    Write-Error $Message
    exit 1
}

function Assert-FileExists {
    param(
        [string]$Path,
        [string]$Label
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Fail "FAIL: Missing required ${Label}: $Path"
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

function Assert-NotContains {
    param(
        [string]$Content,
        [string]$Needle,
        [string]$File
    )

    if ($Content.IndexOf($Needle, [System.StringComparison]::OrdinalIgnoreCase) -ge 0) {
        Fail "FAIL: Forbidden phrase found in ${File}: $Needle"
    }
}

function Assert-MatchesRegex {
    param(
        [string]$Content,
        [string]$Pattern,
        [string]$File
    )

    if (-not [regex]::IsMatch($Content, $Pattern, [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)) {
        Fail "FAIL: Missing required regex anchor in ${File}: ${Pattern}"
    }
}

function Assert-Order {
    param(
        [string]$Content,
        [string[]]$Needles,
        [string]$File
    )

    $previousIndex = -1

    foreach ($needle in $Needles) {
        $index = $Content.IndexOf($needle, [System.StringComparison]::OrdinalIgnoreCase)
        if ($index -lt 0) {
            Fail "FAIL: Missing required anchor in ${File}: ${needle}"
        }

        if ($index -le $previousIndex) {
            Fail "FAIL: Anchor order mismatch in ${File}: ${needle}"
        }

        $previousIndex = $index
    }
}

$paths = @{
    basis = Join-Path $repoRoot "docs/spec/ui/projection_bundle_reader_parser_basis.md"
    matrix = Join-Path $repoRoot "docs/roadmap/post_ui/projection_bundle_level4_evidence_matrix.md"
    gate = Join-Path $repoRoot "docs/spec/ui/projection_bundle_reader_parser_entry_gate.md"
    readerCloseout = Join-Path $repoRoot "docs/roadmap/post_ui/projection_bundle_reader_evidence_closeout.md"
    closeout = Join-Path $repoRoot "docs/roadmap/post_ui/intent_driven_projection_closeout.md"
}

foreach ($path in $paths.Values) {
    Assert-FileExists -Path $path -Label "file"
}

$contents = @{}
foreach ($pair in $paths.GetEnumerator()) {
    $contents[$pair.Key] = Read-Text -Path $pair.Value
}

$basisAnchors = @(
    "ProjectionBundle Reader/Parser Basis v0",
    "Current achieved level: Level 3 baseline",
    "Reader evidence status: narrow reader-facing fixture evidence only",
    "General Level 4 status: not claimed",
    "Loader status: not claimed",
    "Runtime status: not claimed",
    "Production UI status: not claimed",
    "Authority status: non-authorizing",
    "It does not implement a reader.",
    "It does not implement a parser.",
    "This basis does not select JSON, YAML, TOML, binary, or any final serialization.",
    "Every rejected input must produce one stable primary error.",
    "Reader/parser output is representation only.",
    "General Level 4 may not be claimed until the Level-4 evidence matrix is complete."
)

foreach ($anchor in $basisAnchors) {
    Assert-Contains -Content $contents.basis -Needle $anchor -File $paths.basis
}

Assert-MatchesRegex -Content $contents.basis -Pattern 'Levels 4.?7 are not claimed' -File $paths.basis

$matrixAnchors = @(
    "ProjectionBundle Level-4 Evidence Matrix",
    "Current achieved level: Level 3 baseline",
    "General Level 4 status: not claimed",
    "Level 5+ status: not claimed",
    "Level 5+ is not claimed",
    "Reader/parser basis exists",
    "Approved input boundary",
    "Approved output boundary",
    "Positive deterministic output golden tests",
    "Negative deterministic rejection golden tests",
    "Missing-field rejection tests",
    "Malformed-field rejection tests",
    "Unknown-field policy tests",
    "Duplicate-field policy tests",
    "Field-ordering policy tests",
    "Placeholder trust rejection tests",
    "No loader path",
    "No runtime path",
    "No activation path",
    "No production UI wiring",
    "Claim-boundary guard",
    "General Level 4 is not currently achieved.",
    "The matrix is a promotion tracker, not a promotion claim."
)

foreach ($anchor in $matrixAnchors) {
    Assert-Contains -Content $contents.matrix -Needle $anchor -File $paths.matrix
}

Assert-Contains -Content $contents.gate -Needle '`docs/spec/ui/projection_bundle_reader_parser_basis.md` defines the basis for any future reader/parser claim.' -File $paths.gate
Assert-Contains -Content $contents.gate -Needle "This gate remains active." -File $paths.gate
Assert-Contains -Content $contents.gate -Needle "The existence of the basis does not satisfy general Level 4." -File $paths.gate
Assert-Contains -Content $contents.gate -Needle "The evidence matrix must be complete before any general Level 4 reader/parser behavior may be claimed." -File $paths.gate
Assert-Contains -Content $contents.gate -Needle "loader behavior is not claimed" -File $paths.gate
Assert-Contains -Content $contents.gate -Needle "runtime behavior is not claimed" -File $paths.gate
Assert-Contains -Content $contents.gate -Needle "production UI behavior is not claimed" -File $paths.gate
Assert-Contains -Content $contents.readerCloseout -Needle "The reader/parser basis and Level-4 evidence matrix define future promotion requirements. They do not change the current achieved level." -File $paths.readerCloseout
Assert-Contains -Content $contents.closeout -Needle "The reader/parser basis and Level-4 evidence matrix define future promotion requirements. They do not claim general Level 4 behavior, loader behavior, runtime behavior, or production UI behavior." -File $paths.closeout
Assert-Contains -Content $contents.readerCloseout -Needle "Level 5+ is not claimed" -File $paths.readerCloseout
Assert-Order -Content $contents.closeout -Needles @(
    "docs/spec/ui/projection_bundle_reader_parser_entry_gate.md",
    "docs/spec/ui/projection_bundle_reader_parser_basis.md",
    "docs/roadmap/post_ui/projection_bundle_level4_evidence_matrix.md",
    "docs/roadmap/post_ui/projection_bundle_reader_evidence_closeout.md"
) -File $paths.closeout

$forbiddenContents = @{
    basis = $contents.basis
    matrix = $contents.matrix
}

$forbiddenPhrases = @(
    "Level 4 achieved",
    "Level 4 implemented",
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
    foreach ($entry in $forbiddenContents.GetEnumerator()) {
        Assert-NotContains -Content $entry.Value -Needle $phrase -File $paths[$entry.Key]
    }
}

Write-Host "PASS: ProjectionBundle reader/parser basis remains non-authorizing"
