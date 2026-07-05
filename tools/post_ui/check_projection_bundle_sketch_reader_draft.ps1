Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$readerPath = Join-Path $repoRoot "tools/post_ui/projection_bundle_sketch_reader_draft.rs"

function Fail {
    param([string]$Message)
    Write-Error $Message
    exit 1
}

function Assert-FileExists {
    param([string]$Path, [string]$Label)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Fail "FAIL: Missing $($Label): $Path"
    }
}

Assert-FileExists -Path $readerPath -Label "sketch reader draft"

$rustcInfo = Get-Command rustc -ErrorAction SilentlyContinue
if ($null -eq $rustcInfo) {
    Fail "FAIL: rustc not found"
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("post_ui_projection_bundle_sketch_reader_" + [guid]::NewGuid().ToString("N"))
$null = New-Item -ItemType Directory -Path $tempRoot -Force

try {
    $binaryPath = Join-Path $tempRoot "projection_bundle_sketch_reader_draft.exe"
    $rustcExe = $rustcInfo.Path
    if ([string]::IsNullOrWhiteSpace($rustcExe)) {
        $rustcExe = $rustcInfo.Source
    }

    & $rustcExe --edition=2021 -o $binaryPath $readerPath
    if ($LASTEXITCODE -ne 0) {
        Fail "FAIL: rustc compilation failed for $readerPath"
    }

    $output = & $binaryPath $repoRoot 2>&1
    if ($LASTEXITCODE -ne 0) {
        Fail "FAIL: ProjectionBundle sketch reader draft failed"
    }

    $outputText = $output | Out-String
    $expected = "PASS: ProjectionBundle sketch reader draft accepted positive and rejected negative manifest anchors"
    if (-not $outputText.Contains($expected)) {
        Fail "FAIL: missing success output: $expected"
    }

    Write-Output $expected
}
finally {
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}
