param(
    [Parameter(Mandatory = $true)]
    [string]$Tag,
    [string]$Repository = "skulmakov-oss/Semantic-Language",
    [string]$AssetsDirectory,
    [string]$OutputRoot = "artifacts/release-asset-smoke"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

. (Join-Path $PSScriptRoot "release_asset_output.ps1")

function New-Directory([string]$Path) {
    New-Item -ItemType Directory -Force -Path $Path | Out-Null
}

function Remove-IfExists([string]$Path) {
    if (Test-Path -LiteralPath $Path) {
        Remove-Item -LiteralPath $Path -Recurse -Force
    }
}

function Get-RepoRelativePath([string]$RepoRoot, [string]$AbsolutePath) {
    $repoUri = [System.Uri]((Resolve-Path $RepoRoot).Path.TrimEnd('\') + '\')
    $targetUri = [System.Uri]((Resolve-Path $AbsolutePath).Path)
    $relative = $repoUri.MakeRelativeUri($targetUri).ToString()
    return [System.Uri]::UnescapeDataString($relative).Replace('\', '/')
}

function Get-FileSummary([string]$Path) {
    $item = Get-Item -LiteralPath $Path
    $hash = Get-FileHash -LiteralPath $Path -Algorithm SHA256
    [pscustomobject]@{
        absolutePath = $item.FullName
        fileName = $item.Name
        sizeBytes = [int64]$item.Length
        sha256 = $hash.Hash.ToLowerInvariant()
        signingState = "unsigned"
    }
}

function Invoke-CapturedStep {
    param(
        [string]$Name,
        [string]$FilePath,
        [string[]]$ArgumentList,
        [string]$WorkingDirectory,
        [string]$LogsDirectory
    )

    $safeName = ($Name.ToLowerInvariant() -replace '[^a-z0-9]+', '-').Trim('-')
    $stdoutPath = Join-Path $LogsDirectory "$safeName.stdout.txt"
    $stderrPath = Join-Path $LogsDirectory "$safeName.stderr.txt"
    Remove-IfExists $stdoutPath
    Remove-IfExists $stderrPath

    $startedAt = Get-Date
    $process = Start-Process `
        -FilePath $FilePath `
        -ArgumentList $ArgumentList `
        -WorkingDirectory $WorkingDirectory `
        -NoNewWindow `
        -Wait `
        -PassThru `
        -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath
    $finishedAt = Get-Date

    if ($process.ExitCode -ne 0) {
        throw "step '$Name' exited with code $($process.ExitCode)"
    }

    [pscustomobject]@{
        name = $Name
        filePath = $FilePath
        arguments = @($ArgumentList)
        commandLine = (@($FilePath) + $ArgumentList) -join ' '
        workingDirectory = $WorkingDirectory
        exitCode = $process.ExitCode
        durationMs = [int][Math]::Round(($finishedAt - $startedAt).TotalMilliseconds)
        stdoutPath = $stdoutPath
        stderrPath = $stderrPath
    }
}

function Assert-ExistingNonEmptyFile([string]$Path, [string]$Label) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "$Label missing at '$Path'"
    }
    $item = Get-Item -LiteralPath $Path
    if ($item.Length -le 0) {
        throw "$Label at '$Path' is empty"
    }
}

function Assert-FileContains([string]$Path, [string[]]$Patterns) {
    $content = Get-Content -LiteralPath $Path -Raw
    foreach ($pattern in $Patterns) {
        if ($content -notmatch [regex]::Escape($pattern)) {
            throw "expected '$pattern' in '$Path'"
        }
    }
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = (Resolve-Path (Join-Path $scriptDir "..")).Path
$outputDirectory = Join-Path $repoRoot $OutputRoot
$tagOutputDirectory = Join-Path $outputDirectory $Tag
$logsDirectory = Join-Path $tagOutputDirectory "logs"
$workspaceDirectory = Join-Path $tagOutputDirectory "workspace"
$downloadDirectory = Join-Path $tagOutputDirectory "downloaded"
$extractDirectory = Join-Path $tagOutputDirectory "zip-extract"
$jsonReportPath = Join-Path $tagOutputDirectory "release_asset_smoke_report.json"
$markdownReportPath = Join-Path $tagOutputDirectory "release_asset_smoke_report.md"

$zipAssetName = "semantic-language-windows-x64-$Tag.zip"
$requiredAssets = @("smc.exe", "svm.exe", $zipAssetName)

New-Directory $outputDirectory
# Resolves (and, if needed, preserves) caller-provided assets before the
# per-tag output directory is reset.
$assetRoot = Initialize-ReleaseAssetOutput -TagOutputDirectory $tagOutputDirectory `
    -AssetsDirectory $AssetsDirectory -RequiredAssets $requiredAssets
New-Directory $logsDirectory
New-Directory $workspaceDirectory

$release = gh release view $Tag --repo $Repository --json assets,tagName,isPrerelease,isDraft,publishedAt,url | ConvertFrom-Json
if ($release.isDraft) {
    throw "release '$Tag' is still draft"
}

if ($assetRoot) {
    Write-Host "using caller-provided release assets from '$assetRoot'"
} else {
    New-Directory $downloadDirectory
    $assetRoot = $downloadDirectory
    foreach ($assetName in $requiredAssets) {
        gh release download $Tag --repo $Repository --dir $assetRoot --pattern $assetName --clobber | Out-Null
    }
}

$assetFiles = [ordered]@{}
foreach ($assetName in $requiredAssets) {
    $assetPath = Join-Path $assetRoot $assetName
    Assert-ExistingNonEmptyFile -Path $assetPath -Label $assetName
    $assetFiles[$assetName] = $assetPath
}

$releaseAssetsByName = @{}
foreach ($asset in $release.assets) {
    $releaseAssetsByName[$asset.name] = $asset
}

$assetSummaries = [ordered]@{}
foreach ($assetName in $requiredAssets) {
    if (-not $releaseAssetsByName.Contains($assetName)) {
        throw "release '$Tag' does not publish required asset '$assetName'"
    }
    $summary = Get-FileSummary -Path $assetFiles[$assetName]
    $expectedDigest = $releaseAssetsByName[$assetName].digest
    $expectedSha = $expectedDigest -replace '^sha256:', ''
    if ($summary.sha256 -ne $expectedSha) {
        throw "asset '$assetName' sha256 mismatch: expected '$expectedSha', got '$($summary.sha256)'"
    }
    $assetSummaries[$assetName] = $summary
}

Expand-Archive -LiteralPath $assetFiles[$zipAssetName] -DestinationPath $extractDirectory -Force
$zipContents = Get-ChildItem -LiteralPath $extractDirectory -File | Sort-Object Name
$zipNames = @($zipContents | ForEach-Object { $_.Name })
if (($zipNames -join ',') -ne 'smc.exe,svm.exe') {
    throw "zip asset '$zipAssetName' has unexpected contents: $($zipNames -join ', ')"
}

$extractedSmc = Join-Path $extractDirectory "smc.exe"
$extractedSvm = Join-Path $extractDirectory "svm.exe"
$extractedSmcSummary = Get-FileSummary -Path $extractedSmc
$extractedSvmSummary = Get-FileSummary -Path $extractedSvm
if ($extractedSmcSummary.sha256 -ne $assetSummaries["smc.exe"].sha256) {
    throw "zip-contained smc.exe does not match standalone release asset"
}
if ($extractedSvmSummary.sha256 -ne $assetSummaries["svm.exe"].sha256) {
    throw "zip-contained svm.exe does not match standalone release asset"
}

$minimalSourcePath = Join-Path $workspaceDirectory "smoke_minimal.sm"
$builtinSourcePath = Join-Path $workspaceDirectory "smoke_builtin_f64.sm"
$traceSourcePath = Join-Path $repoRoot "examples/semantic_policy_overdrive_trace.sm"
$minimalSemcodePath = Join-Path $workspaceDirectory "smoke_minimal.smc"
$builtinSemcodePath = Join-Path $workspaceDirectory "smoke_builtin_f64.smc"
$traceSemcodePath = Join-Path $workspaceDirectory "semantic_policy_overdrive_trace.smc"
$minimalDisasmPath = Join-Path $workspaceDirectory "smoke_minimal.disasm.txt"
$builtinDisasmPath = Join-Path $workspaceDirectory "smoke_builtin_f64.disasm.txt"
$traceDisasmPath = Join-Path $workspaceDirectory "semantic_policy_overdrive_trace.disasm.txt"

$minimalSource = @'
fn main() {
    return;
}
'@

$builtinSource = @'
fn main() {
    let y: f64 = sqrt(16.0) - 1.0;
    return;
}
'@

Set-Content -LiteralPath $minimalSourcePath -Value $minimalSource -NoNewline
Set-Content -LiteralPath $builtinSourcePath -Value $builtinSource -NoNewline

$steps = [System.Collections.Generic.List[object]]::new()

$steps.Add((Invoke-CapturedStep -Name "minimal compile" -FilePath $extractedSmc -ArgumentList @("compile", $minimalSourcePath, "-o", $minimalSemcodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "minimal run" -FilePath $extractedSvm -ArgumentList @("run", $minimalSemcodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "minimal disasm" -FilePath $extractedSvm -ArgumentList @("disasm", $minimalSemcodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
Copy-Item -LiteralPath $steps[$steps.Count - 1].stdoutPath -Destination $minimalDisasmPath
Assert-FileContains -Path $minimalDisasmPath -Patterns @("SEMCODE0", "RET")

$steps.Add((Invoke-CapturedStep -Name "builtin-f64 compile" -FilePath $extractedSmc -ArgumentList @("compile", $builtinSourcePath, "-o", $builtinSemcodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "builtin-f64 run" -FilePath $extractedSvm -ArgumentList @("run", $builtinSemcodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "builtin-f64 disasm" -FilePath $extractedSvm -ArgumentList @("disasm", $builtinSemcodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
Copy-Item -LiteralPath $steps[$steps.Count - 1].stdoutPath -Destination $builtinDisasmPath
Assert-FileContains -Path $builtinDisasmPath -Patterns @("SEMCODE1", "CALL", "SUB_F64")

$steps.Add((Invoke-CapturedStep -Name "semantic-trace compile" -FilePath $extractedSmc -ArgumentList @("compile", $traceSourcePath, "-o", $traceSemcodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "semantic-trace run" -FilePath $extractedSvm -ArgumentList @("run", $traceSemcodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "semantic-trace disasm" -FilePath $extractedSvm -ArgumentList @("disasm", $traceSemcodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
Copy-Item -LiteralPath $steps[$steps.Count - 1].stdoutPath -Destination $traceDisasmPath
Assert-FileContains -Path $traceDisasmPath -Patterns @(
    "SEMCODE1",
    "fusion_consensus_state",
    "policy_trace_guard",
    "policy_trace_quality",
    "policy_trace"
)

$steps.Add((Invoke-CapturedStep -Name "release toolchain version" -FilePath $extractedSmc -ArgumentList @("version", "--json") -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$releaseToolchainJson = Get-Content -LiteralPath $steps[$steps.Count - 1].stdoutPath -Raw | ConvertFrom-Json
$sourceFingerprint = if ($releaseToolchainJson.source_fingerprint) { $releaseToolchainJson.source_fingerprint } else { $releaseToolchainJson.source_hash }

$report = [ordered]@{
    generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    repository = $Repository
    tag = $release.tagName
    publishedAt = $release.publishedAt
    releaseUrl = $release.url
    prerelease = [bool]$release.isPrerelease
    assetsDirectory = $assetRoot
    outputRoot = (Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $tagOutputDirectory)
    standaloneAssets = @(
        $assetSummaries["smc.exe"],
        $assetSummaries["svm.exe"],
        $assetSummaries[$zipAssetName]
    )
    zipContents = @(
        $extractedSmcSummary,
        $extractedSvmSummary
    )
    smokeScenarios = @(
        [ordered]@{
            name = "Minimal compile-run-disasm"
            source = (Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $minimalSourcePath)
            expectedSignals = @("SEMCODE0", "RET")
            disasm = (Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $minimalDisasmPath)
            result = "pass"
        },
        [ordered]@{
            name = "Verified-path f64 builtin pipeline"
            source = (Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $builtinSourcePath)
            expectedSignals = @("SEMCODE1", "CALL", "SUB_F64")
            disasm = (Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $builtinDisasmPath)
            result = "pass"
        },
        [ordered]@{
            name = "Heavy semantic policy trace"
            source = (Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $traceSourcePath)
            expectedSignals = @("SEMCODE1", "fusion_consensus_state", "policy_trace_guard", "policy_trace_quality", "policy_trace")
            disasm = (Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $traceDisasmPath)
            result = "pass"
        }
    )
    toolchainEvidence = [ordered]@{
        compilerVersion = $releaseToolchainJson.toolchain_version
        sourceFingerprint = $sourceFingerprint
        sourceHash = $sourceFingerprint
        enabledFeatures = $releaseToolchainJson.enabled_features
        semcodeFormat = $releaseToolchainJson.semcode_format
        verifierProfile = $releaseToolchainJson.verifier_profile
        signing = $releaseToolchainJson.signing
    }
    signingState = "unsigned"
    trustPolicy = [ordered]@{
        signingState = "unsigned"
        digestAlgorithm = "SHA-256"
        digestStandard = "FIPS PUB 180-4"
        signatureVerification = "explicitly-unsupported"
        verifierBinding = "exact-artifact-hash"
        producingToolchain = [ordered]@{
            compilerVersion = $releaseToolchainJson.toolchain_version
            sourceFingerprint = $sourceFingerprint
            sourceHash = $sourceFingerprint
            enabledFeatures = $releaseToolchainJson.enabled_features
            semcodeFormat = $releaseToolchainJson.semcode_format
            verifierProfile = $releaseToolchainJson.verifier_profile
            signing = $releaseToolchainJson.signing
        }
    }
    steps = $steps
}

$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $jsonReportPath

$markdown = @(
    "# Release Asset Smoke Report"
    ""
    "- Generated: $($report.generatedAtUtc)"
    "- Repository: $Repository"
    "- Tag: $($report.tag)"
    "- Published: $($report.publishedAt)"
    "- Release URL: $($report.releaseUrl)"
    "- Output root: $($report.outputRoot)"
    "- Release Signing: unsigned (explicitly unsigned per SSF-10)"
    "- Digest Algorithm: SHA-256 (algorithm defined by FIPS PUB 180-4)"
    "- Release Toolchain Version: $($report.toolchainEvidence.compilerVersion)"
    "- Release Compiler Source Fingerprint: $($report.toolchainEvidence.sourceFingerprint)"
    ""
    "## Asset Hashes"
    ""
    "| Asset | Size (bytes) | SHA256 | Signing State |"
    "| --- | ---: | --- | --- |"
)

foreach ($asset in $report.standaloneAssets) {
    $markdown += "| $($asset.fileName) | $($asset.sizeBytes) | $($asset.sha256) | $($asset.signingState) |"
}

$markdown += ""
$markdown += "## Zip Contents"
$markdown += ""
$markdown += "| File | Size (bytes) | SHA256 | Signing State |"
$markdown += "| --- | ---: | --- | --- |"
foreach ($asset in $report.zipContents) {
    $markdown += "| $($asset.fileName) | $($asset.sizeBytes) | $($asset.sha256) | $($asset.signingState) |"
}

$markdown += ""
$markdown += "## Smoke Scenarios"
$markdown += ""
$markdown += "| Scenario | Source | Expected signals | Result |"
$markdown += "| --- | --- | --- | --- |"
foreach ($scenario in $report.smokeScenarios) {
    $markdown += "| $($scenario.name) | $($scenario.source) | $($scenario.expectedSignals -join ', ') | $($scenario.result) |"
}

$markdown += ""
$markdown += "## Step Logs"
$markdown += ""
$markdown += "| Step | Exit | Duration (ms) | Stdout | Stderr |"
$markdown += "| --- | ---: | ---: | --- | --- |"
foreach ($step in $steps) {
    $stdoutRelative = Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $step.stdoutPath
    $stderrRelative = Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $step.stderrPath
    $markdown += "| $($step.name) | $($step.exitCode) | $($step.durationMs) | $stdoutRelative | $stderrRelative |"
}

$markdown -join "`n" | Set-Content -LiteralPath $markdownReportPath

Write-Host "release asset smoke verification passed"
Write-Host "report: $(Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $jsonReportPath)"
Write-Host "report: $(Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $markdownReportPath)"
