Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$readerPath = Join-Path $repoRoot "tools/post_ui/projection_bundle_sketch_reader_draft.rs"
$positiveExpectedPath = Join-Path $repoRoot "tests/fixtures/post_ui/projection_bundle/expected/manifest_minimal.reader.out.txt"
$negativeExpectedPath = Join-Path $repoRoot "tests/fixtures/post_ui/projection_bundle/expected/negative_pack.reader.out.txt"

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
Assert-FileExists -Path $positiveExpectedPath -Label "positive golden output"
Assert-FileExists -Path $negativeExpectedPath -Label "negative golden report"

function Normalize-Lf {
    param([string]$Text)

    return $Text -replace "`r`n", "`n" -replace "`r", "`n"
}

function Compare-NormalizedText {
    param(
        [string]$Actual,
        [string]$Expected,
        [string]$FailureMessage
    )

    $actualNormalized = (Normalize-Lf $Actual).TrimEnd("`r", "`n")
    $expectedNormalized = (Normalize-Lf $Expected).TrimEnd("`r", "`n")

    if ($actualNormalized -ne $expectedNormalized) {
        Fail $FailureMessage
    }
}

function Invoke-ReaderMode {
    param(
        [string]$BinaryPath,
        [string[]]$Arguments,
        [string]$TempRoot,
        [string]$Label
    )

    $stdoutPath = Join-Path $TempRoot "$Label.stdout.txt"
    $stderrPath = Join-Path $TempRoot "$Label.stderr.txt"

    $process = Start-Process `
        -FilePath $BinaryPath `
        -ArgumentList $Arguments `
        -NoNewWindow `
        -Wait `
        -PassThru `
        -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath

    $stdout = if (Test-Path -LiteralPath $stdoutPath) {
        Get-Content -LiteralPath $stdoutPath -Raw
    } else {
        ""
    }

    $stderr = if (Test-Path -LiteralPath $stderrPath) {
        Get-Content -LiteralPath $stderrPath -Raw
    } else {
        ""
    }

    if ($process.ExitCode -ne 0) {
        $stderrText = (Normalize-Lf $stderr).TrimEnd("`r", "`n")
        if (-not [string]::IsNullOrWhiteSpace($stderrText)) {
            Fail "FAIL: projection bundle sketch reader draft failed: $stderrText"
        }

        Fail "FAIL: projection bundle sketch reader draft failed"
    }

    if (-not [string]::IsNullOrWhiteSpace($stderr)) {
        $stderrText = (Normalize-Lf $stderr).TrimEnd("`r", "`n")
        if (-not [string]::IsNullOrWhiteSpace($stderrText)) {
            Fail "FAIL: unexpected stderr from reader draft: $stderrText"
        }
    }

    return $stdout
}

$rustcInfo = Get-Command rustc -ErrorAction SilentlyContinue
if ($null -eq $rustcInfo) {
    Fail "FAIL: rustc not found"
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("post_ui_projection_bundle_sketch_reader_" + [guid]::NewGuid().ToString("N"))
$null = New-Item -ItemType Directory -Path $tempRoot -Force

try {
    $testBinaryPath = Join-Path $tempRoot "projection_bundle_sketch_reader_draft_tests.exe"
    $binaryPath = Join-Path $tempRoot "projection_bundle_sketch_reader_draft.exe"
    $rustcExe = $rustcInfo.Path
    if ([string]::IsNullOrWhiteSpace($rustcExe)) {
        $rustcExe = $rustcInfo.Source
    }

    & $rustcExe --edition=2021 --test -o $testBinaryPath $readerPath
    if ($LASTEXITCODE -ne 0) {
        Fail "FAIL: rust unit tests failed for $readerPath"
    }

    & $testBinaryPath
    if ($LASTEXITCODE -ne 0) {
        Fail "FAIL: rust unit tests failed for $readerPath"
    }

    & $rustcExe --edition=2021 -o $binaryPath $readerPath
    if ($LASTEXITCODE -ne 0) {
        Fail "FAIL: rustc compilation failed for $readerPath"
    }

    $expected = "PASS: ProjectionBundle sketch reader draft accepted positive and rejected negative manifest anchors"
    $defaultOutput = Invoke-ReaderMode -BinaryPath $binaryPath -Arguments @($repoRoot) -TempRoot $tempRoot -Label "default"
    Compare-NormalizedText -Actual $defaultOutput -Expected $expected -FailureMessage "FAIL: missing success output: $expected"

    $positiveOutput = Invoke-ReaderMode -BinaryPath $binaryPath -Arguments @("--emit-positive-output", $repoRoot) -TempRoot $tempRoot -Label "positive"
    $positiveExpected = Get-Content -LiteralPath $positiveExpectedPath -Raw
    Compare-NormalizedText -Actual $positiveOutput -Expected $positiveExpected -FailureMessage "FAIL: positive reader output mismatch"

    $negativeOutput = Invoke-ReaderMode -BinaryPath $binaryPath -Arguments @("--emit-negative-report", $repoRoot) -TempRoot $tempRoot -Label "negative"
    $negativeExpected = Get-Content -LiteralPath $negativeExpectedPath -Raw
    # The exact golden negative report remains the source of truth for policy evidence cases.
    Compare-NormalizedText -Actual $negativeOutput -Expected $negativeExpected -FailureMessage "FAIL: negative reader report mismatch"

    Write-Output $expected
}
finally {
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}
