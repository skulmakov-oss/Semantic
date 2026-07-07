$ErrorActionPreference = "Stop"

$RepoRoot = (Get-Item .).FullName
$ToolsDir = Join-Path $RepoRoot "tools/post_ui"
$ProbesDir = Join-Path $RepoRoot "tests/fixtures/post_ui/projection_bundle/probes"
$ExpectedOutput = Join-Path $RepoRoot "tests/fixtures/post_ui/projection_bundle/expected/probes.reader.out.txt"
$ExePath = Join-Path $ToolsDir "projection_bundle_sketch_reader_draft.exe"

# Compile the reader
Write-Host "Compiling projection_bundle_sketch_reader_draft.rs..."
rustc (Join-Path $ToolsDir "projection_bundle_sketch_reader_draft.rs") -o $ExePath
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

# Verify or Generate
if (Test-Path $ExpectedOutput) {
    $expectedContent = Get-Content -Raw $ExpectedOutput
    $expectedContent = $expectedContent -replace "`r`n", "`n"

    if ($actualOutput -ne $expectedContent) {
        Write-Error "FAIL: Probe output diff found!
If this is expected, remove the expected file and run again to capture the new output."
        # Dump diff to console
        $actualFile = [System.IO.Path]::GetTempFileName()
        Set-Content -Path $actualFile -Value $actualOutput
        git diff --no-index $ExpectedOutput $actualFile
        Remove-Item $actualFile
        exit 1
    } else {
        Write-Host "PASS: Probe output matches expected snapshot"
    }
} else {
    Write-Host "No expected output found. Generating..."
    Set-Content -Path $ExpectedOutput -Value $actualOutput
    Write-Host "Snapshot captured to $ExpectedOutput"
}
