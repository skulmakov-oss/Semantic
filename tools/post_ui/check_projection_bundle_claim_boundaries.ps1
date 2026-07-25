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

# Keep every ProjectionBundle claim-bearing doc in this canonical scanner.
# The aggregate POST-UI guard must not drift across separate forbidden lists.
$paths = @{
    basis = Join-Path $repoRoot "docs/spec/ui/projection_bundle_basis.md"
    gate = Join-Path $repoRoot "docs/spec/ui/projection_bundle_reader_parser_entry_gate.md"
    readerParserBasis = Join-Path $repoRoot "docs/spec/ui/projection_bundle_reader_parser_basis.md"
    level4Matrix = Join-Path $repoRoot "docs/roadmap/post_ui/projection_bundle_level4_evidence_matrix.md"
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

# Boundary-safe negative statements such as "not claimed" or
# "does not claim loader behavior" must remain allowed.
# This guard rejects affirmative/readiness claims, not vocabulary itself.
$forbiddenPhrases = @(
    "Level 4 achieved",
    "Level 4 implemented",
    "Level-4 achieved",
    "Level-4 implemented",
    "general Level 4 achieved",
    "general Level 4 reader/parser behavior is achieved",
    "reader/parser implemented",
    "parser implemented",
    "general parser implemented",
    "ProjectionBundle parser implemented",
    "ProjectionBundle reader/parser implemented",
    "loader-ready",
    "runtime-ready",
    "production-ready",
    "loader ready",
    "runtime ready",
    "production ready",
    "activation path ready",
    "public API ready",
    "activation-ready",
    "verification-ready",
    "security proven",
    "Level 5 achieved",
    "Level 5 implemented",
    "Level-5 achieved",
    "Level-5 implemented",
    "Level 5+ achieved",
    "Level 5+ implemented",
    "Level 5 ready",
    "Level 5 is ready",
    "Level 5 readiness achieved",
    "Level 5 readiness established",
    "Level 5 readiness is established",
    "Level 5 readiness proven",
    "Level-5 ready",
    "Level-5 is ready",
    "Level-5 readiness achieved",
    "Level-5 readiness established",
    "Level-5 readiness is established",
    "Level-5 readiness proven",
    "Level 5+ ready",
    "Level 5+ is ready",
    "Level 5+ readiness achieved",
    "Level 5+ readiness established",
    "Level 5+ readiness is established",
    "Level 5+ readiness proven",
    "Level-5+ achieved",
    "Level-5+ implemented",
    "Level-5+ ready",
    "Level-5+ is ready",
    "Level-5+ readiness achieved",
    "Level-5+ readiness established",
    "Level-5+ readiness is established",
    "Level-5+ readiness proven"
)

foreach ($phrase in $forbiddenPhrases) {
    Assert-NotContains -Contents $contents -Needle $phrase
}

# ---------------------------------------------------------------------------
# Current-state consistency checks for the post-Issue-#1543 UI-DNA2 docs.
#
# These guard a different failure mode than the forbidden-phrase list above:
# not overclaiming readiness, but *contradicting yourself* -- declaring a
# bounded contour implemented/open/promoted in one place while another part
# of the same document still says it is unconditionally unauthorized/closed,
# left over from before Issue #1543 supplied the bounded owner authorization.
#
# Historical statements are fine and expected (UI-DNA2-8A's original freeze
# posture, WP4B's original baseline) as long as they are explicitly marked
# as historical. This section is deliberately separate from the doc family
# checked above, which predates and is unrelated to Issue #1543/#1544/#1545.
# ---------------------------------------------------------------------------

$currentStatePaths = @{
    projectionBundleV0 = Join-Path $repoRoot "docs/spec/ui/projection_bundle_v0.md"
    roadmap = Join-Path $repoRoot "docs/roadmap/post_ui/ui_dna2_implementation_roadmap.md"
}

foreach ($path in $currentStatePaths.Values) {
    Assert-FileExists -Path $path
}

$currentStateContents = @{}
foreach ($pair in $currentStatePaths.GetEnumerator()) {
    $currentStateContents[$pair.Key] = Read-Text -Path $pair.Value
}

# A phrase is acceptable only when a historical-scoping marker appears
# shortly before it. Without one, it reads as an unconditional, current
# claim -- which is exactly the contradiction this guards against.
$historicalMarkers = @(
    "Historical",
    "historically",
    "at the time",
    "before Issue #1543",
    "prior to Issue #1543",
    "WP4B baseline",
    "originally"
)

function Assert-NoUnscopedPhrase {
    param(
        [string]$Content,
        [string]$File,
        [string]$Phrase,
        [int]$LookbackChars = 400
    )

    $searchFrom = 0
    while ($true) {
        $idx = $Content.IndexOf($Phrase, $searchFrom, [System.StringComparison]::OrdinalIgnoreCase)
        if ($idx -lt 0) { break }

        $windowStart = [Math]::Max(0, $idx - $LookbackChars)
        $window = $Content.Substring($windowStart, $idx - $windowStart)

        $scoped = $false
        foreach ($marker in $historicalMarkers) {
            if ($window.IndexOf($marker, [System.StringComparison]::OrdinalIgnoreCase) -ge 0) {
                $scoped = $true
                break
            }
        }

        if (-not $scoped) {
            Fail "FAIL: Unscoped current-state contradiction in ${File}: '$Phrase' appears without a nearby historical marker (checked $LookbackChars chars back). If this is current-state truth, remove it or scope the surrounding claim; if it is historical, add an explicit historical marker (e.g. 'Historical (...):') immediately before it."
        }

        $searchFrom = $idx + $Phrase.Length
    }
}

# PROMOTE WITH LIMITS and an unqualified "production promotion ... NOT
# AUTHORIZED" for the same contour cannot both be true. A qualifier
# (general/unrestricted/critical) makes the NOT AUTHORIZED statement a
# legitimate, narrower claim instead of a direct contradiction.
function Assert-PromotionClaimsScoped {
    param([string]$Content, [string]$File)

    if ($Content.IndexOf("PROMOTE WITH LIMITS", [System.StringComparison]::OrdinalIgnoreCase) -lt 0) {
        return
    }

    $pattern = [regex]'(?i)production promotion[^.\n]{0,80}NOT AUTHORIZED'
    foreach ($m in $pattern.Matches($Content)) {
        $contextStart = [Math]::Max(0, $m.Index - 80)
        $contextLength = [Math]::Min($Content.Length, $m.Index + $m.Length + 80) - $contextStart
        $context = $Content.Substring($contextStart, $contextLength)
        if ($context -notmatch '(?i)general|unrestricted|critical') {
            Fail "FAIL: In ${File}, PROMOTE WITH LIMITS is recorded but this unqualified statement contradicts it: '$($m.Value)'. Qualify it (general/unrestricted/critical) or remove it."
        }
    }
}

# If a document defines formal umbrella-closure criteria, the closure
# mechanism it actually uses to justify closing (carrying incomplete phases
# forward in a checkpoint matrix, rather than requiring every phase complete
# or spun into a child issue) must be one of the stated criteria -- not just
# asserted in prose elsewhere in the same document.
function Assert-ClosureCriterionIncludesCarriedForward {
    param([string]$Content, [string]$File)

    $hasClosureRule = ($Content.IndexOf("may close only when", [System.StringComparison]::OrdinalIgnoreCase) -ge 0)
    if (-not $hasClosureRule) {
        return
    }

    if ($Content.IndexOf("carried forward", [System.StringComparison]::OrdinalIgnoreCase) -lt 0) {
        Fail "FAIL: In ${File}, a closure rule ('may close only when ...') is defined, but no criterion covers phases that are carried forward (incomplete, but explicitly tracked) rather than completed or handed to a child issue -- yet the closure explanation elsewhere in this repository relies on exactly that mechanism. Add a carried-forward option to the rule itself."
    }
}

Assert-NoUnscopedPhrase -Content $currentStateContents.projectionBundleV0 -File $currentStatePaths.projectionBundleV0 -Phrase "UI-DNA2-8B is not authorized"
Assert-NoUnscopedPhrase -Content $currentStateContents.projectionBundleV0 -File $currentStatePaths.projectionBundleV0 -Phrase "UI-DNA2-8C is not authorized"
Assert-NoUnscopedPhrase -Content $currentStateContents.roadmap -File $currentStatePaths.roadmap -Phrase "Gate D activation and integration remain closed"

Assert-PromotionClaimsScoped -Content $currentStateContents.projectionBundleV0 -File $currentStatePaths.projectionBundleV0
Assert-PromotionClaimsScoped -Content $currentStateContents.roadmap -File $currentStatePaths.roadmap

Assert-ClosureCriterionIncludesCarriedForward -Content $currentStateContents.roadmap -File $currentStatePaths.roadmap

Write-Host "PASS: ProjectionBundle claim boundaries remain aligned"
Write-Host "PASS: UI-DNA2 current-state documents contain no unscoped contradictions"
