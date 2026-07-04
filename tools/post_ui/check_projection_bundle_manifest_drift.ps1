Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$sketchPath = Join-Path $repoRoot "tests/fixtures/post_ui/projection_bundle/manifest_minimal.sketch.md"
$draftPath = Join-Path $repoRoot "tools/post_ui/projection_bundle_manifest_draft.rs"

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
        Fail "FAIL: Missing $($Label): $Path"
    }
}

function Assert-AnchorInBoth {
    param(
        [string]$Name,
        [string]$Text,
        [string]$SketchContent,
        [string]$DraftContent
    )

    if (-not $SketchContent.Contains($Text)) {
        Fail "FAIL: Missing anchor '$Name' in $($sketchPath): $Text"
    }

    if (-not $DraftContent.Contains($Text)) {
        Fail "FAIL: Missing anchor '$Name' in $($draftPath): $Text"
    }
}

Assert-FileExists -Path $sketchPath -Label "manifest sketch"
Assert-FileExists -Path $draftPath -Label "manifest draft"

$sketch = Get-Content -Raw -LiteralPath $sketchPath
$draft = Get-Content -Raw -LiteralPath $draftPath

$draftConstantMarker = "pub const MINIMAL_SKETCH_MANIFEST_DRAFT"
$draftConstantStart = $draft.IndexOf($draftConstantMarker)
if ($draftConstantStart -lt 0) {
    Fail "FAIL: Missing Rust manifest draft constant marker: $draftConstantMarker"
}

$draftEvidence = $draft.Substring($draftConstantStart)

$sharedAnchors = @(
    @{ Name = "bundle id"; Text = "bundle.example.minimal" },
    @{ Name = "bundle version"; Text = "0-sketch" },
    @{ Name = "projection id"; Text = "ExampleMinimalProjection" },
    @{ Name = "semantic source ref"; Text = "semantic.source.example" },
    @{ Name = "projection source ref"; Text = "projection.source.example" },
    @{ Name = "ui ir ref"; Text = "ui_ir.example.minimal" },
    @{ Name = "binding graph ref"; Text = "binding_graph.example.minimal" },
    @{ Name = "action ir ref"; Text = "action_ir.example.minimal" },
    @{ Name = "role dictionary version"; Text = "ui-roles.0-sketch" },
    @{ Name = "renderer profile"; Text = "semantic-shell.reference-sketch" },
    @{ Name = "safety class"; Text = "VerifiedDynamic" },
    @{ Name = "criticality"; Text = "NonCritical" },
    @{ Name = "freshness policy"; Text = "FreshForControl" },
    @{ Name = "placeholder hash"; Text = "sha256:SKETCH-NOT-A-REAL-HASH" },
    @{ Name = "placeholder signature"; Text = "signature:SKETCH-NOT-A-REAL-SIGNATURE" },
    @{ Name = "placeholder creator"; Text = "semantic-projection-compiler.SKETCH" },
    @{ Name = "placeholder timestamp"; Text = "not-a-real-timestamp" },
    @{ Name = "compiler identity"; Text = "semantic-projection-compiler.0-sketch" },
    @{ Name = "runtime tree streaming policy key"; Text = "allow_runtime_tree_streaming" },
    @{ Name = "production activation policy key"; Text = "allow_production_activation" },
    @{ Name = "pending unknown policy key"; Text = "allow_critical_update_during_pending_unknown" },
    @{ Name = "quarantine policy key"; Text = "allow_critical_update_during_quarantine" }
)

foreach ($anchor in $sharedAnchors) {
    Assert-AnchorInBoth -Name $anchor.Name -Text $anchor.Text -SketchContent $sketch -DraftContent $draftEvidence
}

$policyAnchors = @(
    @{ Name = "runtime tree streaming disabled"; Text = "allow_runtime_tree_streaming: false" },
    @{ Name = "production activation disabled"; Text = "allow_production_activation: false" },
    @{ Name = "pending unknown updates disabled"; Text = "allow_critical_update_during_pending_unknown: false" },
    @{ Name = "quarantine updates disabled"; Text = "allow_critical_update_during_quarantine: false" },
    @{ Name = "verification required"; Text = "require_verification: true" },
    @{ Name = "safe update boundary required"; Text = "require_safe_update_boundary: true" }
)

foreach ($anchor in $policyAnchors) {
    Assert-AnchorInBoth -Name $anchor.Name -Text $anchor.Text -SketchContent $sketch -DraftContent $draftEvidence
}

Write-Host "PASS: ProjectionBundle manifest sketch/draft anchors match"
