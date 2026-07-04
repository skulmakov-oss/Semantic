Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$fixtureDir = Join-Path $PSScriptRoot "..\..\tests\fixtures\post_ui\projection_bundle"
$fixtureDir = [System.IO.Path]::GetFullPath($fixtureDir)
$readmePath = Join-Path $fixtureDir "README.md"
$sketchPath = Join-Path $fixtureDir "manifest_minimal.sketch.md"

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
        Fail "Missing $($Label): $Path"
    }
}

function Assert-Contains {
    param(
        [string]$Path,
        [string]$Content,
        [string]$Needle
    )

    if (-not $Content.Contains($Needle)) {
        Fail "Missing required text in $($Path): $Needle"
    }
}

function Assert-NotContainsRegex {
    param(
        [string]$Path,
        [string]$Content,
        [string]$Pattern,
        [string]$Description
    )

    if ([regex]::IsMatch($Content, $Pattern)) {
        Fail "Forbidden $Description in $Path matched pattern: $Pattern"
    }
}

function Assert-NoForbiddenFiles {
    param([string]$Root)

    $forbiddenPatterns = @(
        "*.json",
        "*.yaml",
        "*.yml",
        "*.toml",
        "*.rs",
        "*.bin",
        "*.wasm",
        "*.exe",
        "*.dll",
        "*.so",
        "*.dylib"
    )

    $violations = foreach ($pattern in $forbiddenPatterns) {
        Get-ChildItem -LiteralPath $Root -Recurse -File -Filter $pattern -ErrorAction SilentlyContinue
    }

    $violations = $violations | Sort-Object FullName -Unique
    if (@($violations).Count -gt 0) {
        $paths = @($violations) | ForEach-Object { $_.FullName }
        Fail ("Forbidden file artifact(s) found under {0}:{1} - {2}" -f $Root, [Environment]::NewLine, ($paths -join [Environment]::NewLine))
    }
}

Assert-FileExists -Path $readmePath -Label "README"
Assert-FileExists -Path $sketchPath -Label "manifest sketch"

Assert-NoForbiddenFiles -Root $fixtureDir

$readme = Get-Content -Raw -LiteralPath $readmePath
$sketch = Get-Content -Raw -LiteralPath $sketchPath

$readmeNeedles = @(
    "Status: inert fixture anchor",
    "Execution status: non-executing",
    "Implementation status: blocked",
    "These files are not executable.",
    "They are not final serialization.",
    "They are not parser input.",
    "They are not loader input.",
    "They are not runtime input.",
    "They are not shell-player input.",
    "They do not authorize production UI wiring.",
    "Fixtures before runtime.",
    "Evidence before authority.",
    "No loader first.",
    "No parser.",
    "No loader.",
    "No runtime.",
    "No shell player.",
    "No Rust types.",
    "No final serialization commitment.",
    "No production UI wiring.",
    "No Workbench dependency.",
    "No ui-shell-kit promotion.",
    "No verifier / VM / SemCode changes.",
    "No capability / audit authority widening.",
    "This directory is not a fixture runner.",
    "This directory is not a schema authority.",
    "This directory is not a loader contract.",
    "This directory is not a production UI entrypoint."
)

foreach ($needle in $readmeNeedles) {
    Assert-Contains -Path $readmePath -Content $readme -Needle $needle
}

$sketchNeedles = @(
    "Status: non-executing sketch",
    "Serialization status: not final",
    "Execution status: non-executing",
    "Implementation status: blocked",
    "This sketch is not final serialization.",
    "It must not be parsed by production code.",
    "It must not be loaded by a ProjectionBundle loader.",
    "It must not be treated as a runtime contract.",
    "It exists only as planning evidence.",
    "projection_bundle {",
    "bundle_id: ""bundle.example.minimal""",
    "bundle_version: ""0-sketch""",
    "projection_id: ""ExampleMinimalProjection""",
    "hash: ""sha256:SKETCH-NOT-A-REAL-HASH""",
    "signature: ""signature:SKETCH-NOT-A-REAL-SIGNATURE""",
    "allow_runtime_tree_streaming: false",
    "allow_production_activation: false",
    "allow_critical_update_during_pending_unknown: false",
    "allow_critical_update_during_quarantine: false",
    "These placeholders must not pass future verification.",
    "A future loader must not treat this sketch as loadable.",
    "A future verifier must reject the placeholder hash and signature if verification is implemented.",
    "A future runtime must not activate this sketch."
)

foreach ($needle in $sketchNeedles) {
    Assert-Contains -Path $sketchPath -Content $sketch -Needle $needle
}

Assert-Contains -Path $sketchPath -Content $sketch -Needle '```text'
Assert-NotContainsRegex -Path $sketchPath -Content $sketch -Pattern '(?m)^```(?:json|yaml|yml|toml)\s*$' -Description "final-serialization fenced code block"

Write-Host "PASS: POST-UI ProjectionBundle fixture anchor remains inert"
