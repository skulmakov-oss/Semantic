param (
    [switch]$UpdateSnapshot
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Get-Item .).FullName
$ToolsDir = Join-Path $RepoRoot "tools/post_ui"
$ProbesDir = Join-Path $RepoRoot "tests/fixtures/post_ui/projection_bundle/probes"
$ExpectedOutput = Join-Path $RepoRoot "tests/fixtures/post_ui/projection_bundle/expected/probes.reader.out.txt"

$TempDir = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), [System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TempDir | Out-Null
$ExePath = Join-Path $TempDir "projection_bundle_sketch_reader_draft.exe"

try {
    # Compile the reader
    Write-Host "Compiling projection_bundle_sketch_reader_draft.rs..."
    rustc --edition=2021 (Join-Path $ToolsDir "projection_bundle_sketch_reader_draft.rs") -o $ExePath
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Compilation failed"
        exit 1
    }

    # Run probes and capture
    $actualOutput = ""
    $probes = Get-ChildItem -Path $ProbesDir -Filter "*.sketch.md" | Sort-Object Name
    foreach ($probe in $probes) {
        Write-Host "Probing $($probe.Name)..."
        $out = & $ExePath "--probe-fixture" $probe.FullName
        $actualOutput += $out -join "`n"
        $actualOutput += "`n"
    }

    # Normalize line endings
    $actualOutput = $actualOutput -replace "`r`n", "`n"

    if ($UpdateSnapshot) {
        Write-Host "Updating snapshot..."
        Set-Content -Path $ExpectedOutput -Value $actualOutput -NoNewline
        Write-Host "Snapshot captured to $ExpectedOutput"
    } else {
        if (-not (Test-Path $ExpectedOutput)) {
            Write-Error "FAIL: Expected output file missing. Run with -UpdateSnapshot to generate it."
            exit 1
        }
        $expectedContent = Get-Content -Raw $ExpectedOutput
        $expectedContent = $expectedContent -replace "`r`n", "`n"

        if ($actualOutput -ne $expectedContent) {
            Write-Error "FAIL: Probe output diff found! If this is expected, run with -UpdateSnapshot to update."
            # Dump diff to console
            $actualFile = [System.IO.Path]::GetTempFileName()
            Set-Content -Path $actualFile -Value $actualOutput -NoNewline
            git diff --no-index $ExpectedOutput $actualFile
            Remove-Item $actualFile
            exit 1
        } else {
            Write-Host "PASS: Probe output matches expected snapshot"
        }
    }
} finally {
    if (Test-Path $TempDir) {
        Remove-Item -Recurse -Force $TempDir
    }
}
