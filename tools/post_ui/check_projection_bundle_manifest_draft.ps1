Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$draftPath = Join-Path $repoRoot "tools/post_ui/projection_bundle_manifest_draft.rs"

function Fail {
    param([string]$Message)
    Write-Error $Message
    exit 1
}

function Assert-FileExists {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Fail "FAIL: Missing draft file: $Path"
    }
}

function Assert-NotContainsText {
    param(
        [string]$Path,
        [string]$Content,
        [string]$Forbidden
    )

    if ($Content.Contains($Forbidden)) {
        Fail "FAIL: Forbidden text found in $($Path): $Forbidden"
    }
}

Assert-FileExists -Path $draftPath

$rustc = Get-Command rustc -ErrorAction SilentlyContinue
if ($null -eq $rustc) {
    Fail "FAIL: rustc not found"
}

$draft = Get-Content -Raw -LiteralPath $draftPath
$scanDraft = $draft -replace [regex]::Escape('#![forbid(unsafe_code)]'), ''

$forbiddenStrings = @(
    "serde",
    "Serialize",
    "Deserialize",
    "FromStr",
    "TryFrom",
    "TryInto",
    "std::fs",
    "std::io",
    "std::path",
    "File",
    "read_to_string",
    "include_str!",
    "include_bytes!",
    "Command",
    "process",
    "unsafe",
    "fn main",
    "#[test]",
    "mod tests"
)

foreach ($forbidden in $forbiddenStrings) {
    Assert-NotContainsText -Path $draftPath -Content $scanDraft -Forbidden $forbidden
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("post_ui_projection_bundle_manifest_draft_" + [guid]::NewGuid().ToString("N"))
$null = New-Item -ItemType Directory -Path $tempRoot -Force

try {
    $outputPath = Join-Path $tempRoot "projection_bundle_manifest_draft.rmeta"
    $rustcExe = $rustc.Path
    if ([string]::IsNullOrWhiteSpace($rustcExe)) {
        $rustcExe = $rustc.Source
    }
    & $rustcExe --edition=2021 --crate-type lib --emit=metadata -o $outputPath $draftPath
    $exitCode = 0
    if (Get-Variable -Name LASTEXITCODE -Scope 0 -ErrorAction SilentlyContinue) {
        $exitCode = [int]$global:LASTEXITCODE
    }
    if ($exitCode -ne 0) {
        Fail "FAIL: rustc metadata compilation failed for $draftPath"
    }
}
finally {
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}

Write-Host "PASS: ProjectionBundle manifest draft types compile"
