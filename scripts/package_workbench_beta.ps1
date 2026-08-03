# Packages the archived React/TypeScript/Tauri Workbench app
# (apps/workbench_ts_tauri_legacy). This is NOT the canonical Workbench --
# the canonical native Workbench is examples/workbench_semantic, which has
# no Node/npm/Vite/React/Tauri/WebView dependency. This script is kept only
# for historical packaging of the archived reference app.
param(
    [string]$OutputRoot = "artifacts/workbench/beta-smoke",
    [int]$LaunchSmokeSeconds = 8
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function New-Directory([string]$Path) {
    New-Item -ItemType Directory -Path $Path -Force | Out-Null
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

function Invoke-CapturedStep {
    param(
        [string]$Name,
        [string]$FilePath,
        [string[]]$ArgumentList,
        [string]$WorkingDirectory,
        [string]$LogsDirectory,
        [bool]$ExpectFailure = $false
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

    $succeeded = if ($ExpectFailure) {
        $process.ExitCode -ne 0
    } else {
        $process.ExitCode -eq 0
    }

    if (-not $succeeded) {
        $expectation = if ($ExpectFailure) { "non-zero" } else { "zero" }
        throw "step '$Name' exited with code $($process.ExitCode), expected $expectation"
    }

    [pscustomobject]@{
        name = $Name
        filePath = $FilePath
        arguments = @($ArgumentList)
        commandLine = (@($FilePath) + $ArgumentList) -join ' '
        workingDirectory = $WorkingDirectory
        exitCode = $process.ExitCode
        expectFailure = $ExpectFailure
        succeeded = $true
        durationMs = [int][Math]::Round(($finishedAt - $startedAt).TotalMilliseconds)
        stdoutPath = $stdoutPath
        stderrPath = $stderrPath
    }
}

function Get-FileSummary([string]$Path) {
    $item = Get-Item -LiteralPath $Path
    $hash = Get-FileHash -LiteralPath $Path -Algorithm SHA256
    [pscustomobject]@{
        absolutePath = $item.FullName
        fileName = $item.Name
        sizeBytes = [int64]$item.Length
        sha256 = $hash.Hash.ToLowerInvariant()
    }
}

function Write-SmokeWorkspace([string]$WorkspaceRoot, [bool]$Invalid) {
    New-Directory (Join-Path $WorkspaceRoot "src")
    New-Directory (Join-Path $WorkspaceRoot "build")
    $manifest = @'
[package]
name = "workbench-beta-smoke"
version = "0.1.0"
edition = "v1"
entry = "src/main.sm"
'@
    Set-Content -LiteralPath (Join-Path $WorkspaceRoot "Semantic.toml") -Value $manifest -NoNewline

    $source = if ($Invalid) {
@'
fn main() {
    let broken: i32 =
}
'@
    } else {
@'
fn allow()->quad{    
    return T;
}

fn main(){
let state: quad = allow();
if state == T {
return;
} else {
return;
}
}
'@
    }

    Set-Content -LiteralPath (Join-Path $WorkspaceRoot "src/main.sm") -Value $source -NoNewline
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = (Resolve-Path (Join-Path $scriptDir "..")).Path
$outputDirectory = Join-Path $repoRoot $OutputRoot
$logsDirectory = Join-Path $outputDirectory "logs"
$workspaceDirectory = Join-Path $outputDirectory "workspace"
$packageDirectory = Join-Path $outputDirectory "package"
$extractedDirectory = Join-Path $outputDirectory "package-extract"
$tauriTargetDirectory = Join-Path $outputDirectory "tauri-target"
$bundleManifestPath = Join-Path $outputDirectory "workbench_beta_package_manifest.json"
$releaseManifestPath = Join-Path $outputDirectory "workbench_beta_release_bundle_manifest.json"
$jsonReportPath = Join-Path $outputDirectory "workbench_beta_smoke_latest.json"
$markdownReportPath = Join-Path $outputDirectory "workbench_beta_smoke_latest.md"
$zipPath = Join-Path $packageDirectory "semantic-workbench-beta-portable.zip"

New-Directory $outputDirectory
New-Directory $logsDirectory
Remove-IfExists $workspaceDirectory
Remove-IfExists $packageDirectory
Remove-IfExists $extractedDirectory
Remove-IfExists $tauriTargetDirectory
New-Directory $workspaceDirectory
New-Directory $packageDirectory
New-Directory $extractedDirectory
New-Directory $tauriTargetDirectory

$workbenchDirectory = Join-Path $repoRoot "apps/workbench_ts_tauri_legacy"
$tauriDirectory = Join-Path $workbenchDirectory "src-tauri"
$workbenchReleaseDirectory = Join-Path $tauriTargetDirectory "release"
$workbenchExePath = Join-Path $workbenchReleaseDirectory "semantic-workbench-app.exe"
$smcReleasePath = Join-Path $repoRoot "target/release/smc.exe"
$svmReleasePath = Join-Path $repoRoot "target/release/svm.exe"
$smokeSourcePath = Join-Path $workspaceDirectory "src/main.sm"
$smokeBytecodePath = Join-Path $workspaceDirectory "build/main.smc"

$steps = [System.Collections.Generic.List[object]]::new()

$steps.Add((Invoke-CapturedStep -Name "workbench lint" -FilePath "npm.cmd" -ArgumentList @("run", "lint") -WorkingDirectory $workbenchDirectory -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "workbench build" -FilePath "npm.cmd" -ArgumentList @("run", "build") -WorkingDirectory $workbenchDirectory -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "workbench tauri tests" -FilePath "cargo" -ArgumentList @("test", "--manifest-path", "apps/workbench_ts_tauri_legacy/src-tauri/Cargo.toml") -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "semantic release binaries" -FilePath "cargo" -ArgumentList @("build", "--release", "--bin", "smc", "--bin", "svm") -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$tauriBuildCommand = "`$env:CARGO_TARGET_DIR='$tauriTargetDirectory'; npm.cmd run tauri:build -- --no-bundle"
$steps.Add((Invoke-CapturedStep -Name "workbench release build" -FilePath "pwsh" -ArgumentList @("-NoProfile", "-Command", $tauriBuildCommand) -WorkingDirectory $workbenchDirectory -LogsDirectory $logsDirectory))

if (-not (Test-Path -LiteralPath $workbenchExePath)) {
    throw "expected packaged Workbench executable at '$workbenchExePath'"
}

Copy-Item -LiteralPath $workbenchExePath -Destination (Join-Path $packageDirectory "semantic-workbench-app.exe")
Copy-Item -LiteralPath (Join-Path $workbenchDirectory "README.md") -Destination (Join-Path $packageDirectory "README.md")
Compress-Archive -Path (Join-Path $packageDirectory "*") -DestinationPath $zipPath -Force
Expand-Archive -LiteralPath $zipPath -DestinationPath $extractedDirectory -Force

$launchedExePath = Join-Path $extractedDirectory "semantic-workbench-app.exe"
if (-not (Test-Path -LiteralPath $launchedExePath)) {
    throw "portable package is missing '$launchedExePath'"
}

$launchStartedAt = Get-Date
$launchProcess = Start-Process -FilePath $launchedExePath -WorkingDirectory $extractedDirectory -PassThru
Start-Sleep -Seconds $LaunchSmokeSeconds
$launchProcess.Refresh()
$launchRunning = -not $launchProcess.HasExited
$launchExitCode = if ($launchProcess.HasExited) { $launchProcess.ExitCode } else { $null }
if ($launchRunning) {
    Stop-Process -Id $launchProcess.Id -Force
}
$launchFinishedAt = Get-Date
if (-not $launchRunning) {
    throw "portable Workbench package exited before the launch smoke window completed"
}

Write-SmokeWorkspace -WorkspaceRoot $workspaceDirectory -Invalid $true
$steps.Add((Invoke-CapturedStep -Name "smoke diagnostics check" -FilePath $smcReleasePath -ArgumentList @("check", $smokeSourcePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory -ExpectFailure $true))

Write-SmokeWorkspace -WorkspaceRoot $workspaceDirectory -Invalid $false
$steps.Add((Invoke-CapturedStep -Name "smoke format check before write" -FilePath $smcReleasePath -ArgumentList @("fmt", "--check", $smokeSourcePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory -ExpectFailure $true))
$steps.Add((Invoke-CapturedStep -Name "smoke format write" -FilePath $smcReleasePath -ArgumentList @("fmt", $smokeSourcePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "smoke format check after write" -FilePath $smcReleasePath -ArgumentList @("fmt", "--check", $smokeSourcePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "smoke compile" -FilePath $smcReleasePath -ArgumentList @("compile", $smokeSourcePath, "-o", $smokeBytecodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "smoke verify" -FilePath $smcReleasePath -ArgumentList @("verify", $smokeBytecodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "smoke disasm" -FilePath $svmReleasePath -ArgumentList @("disasm", $smokeBytecodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "smoke run" -FilePath $svmReleasePath -ArgumentList @("run", $smokeBytecodePath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))
$steps.Add((Invoke-CapturedStep -Name "release bundle verify" -FilePath "pwsh" -ArgumentList @("-File", (Join-Path $repoRoot "scripts/verify_release_bundle.ps1"), "-ManifestPath", $releaseManifestPath) -WorkingDirectory $repoRoot -LogsDirectory $logsDirectory))

$bundleManifest = [pscustomobject]@{
    generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    productName = "Semantic Workbench"
    version = "0.1.0"
    packageFormat = "portable-zip"
    releaseExecutable = Get-FileSummary -Path $workbenchExePath
    portableZip = Get-FileSummary -Path $zipPath
    extractedExecutable = Get-FileSummary -Path $launchedExePath
}
$bundleManifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $bundleManifestPath

$report = [pscustomobject]@{
    generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    gitBranch = (git branch --show-current)
    gitCommit = (git rev-parse HEAD)
    outputRoot = (Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $outputDirectory)
    workspaceRoot = (Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $workspaceDirectory)
    portablePackage = $bundleManifest
    launchSmoke = [pscustomobject]@{
        launched = $launchRunning
        launchSmokeSeconds = $LaunchSmokeSeconds
        durationMs = [int][Math]::Round(($launchFinishedAt - $launchStartedAt).TotalMilliseconds)
        executable = (Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $launchedExePath)
        exitCodeBeforeStop = $launchExitCode
    }
    steps = $steps
}
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $jsonReportPath

$markdown = @(
    "# Workbench Beta Smoke Report"
    ""
    "- Generated: $($report.generatedAtUtc)"
    "- Branch: $($report.gitBranch)"
    "- Commit: $($report.gitCommit)"
    "- Output root: $($report.outputRoot)"
    "- Workspace root: $($report.workspaceRoot)"
    "- Package format: portable-zip"
    "- Portable zip: $(Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $zipPath)"
    "- Release executable: $(Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $workbenchExePath)"
    "- Launch smoke: passed ($LaunchSmokeSeconds s window)"
    ""
    "## Acceptance Coverage"
    ""
    "- Packaged app launched from the extracted portable beta package."
    "- Smoke loop covered diagnostics, format check/write, compile, verify, disasm, run, and release bundle verification."
    "- Full command captures are preserved under 'artifacts/workbench/beta-smoke/logs/'."
    ""
    "## Step Summary"
    ""
    "| Step | Expectation | Exit | Duration (ms) | Stdout | Stderr |"
    "| --- | --- | ---: | ---: | --- | --- |"
)

foreach ($step in $steps) {
    $expectation = if ($step.expectFailure) { "expected failure" } else { "success" }
    $stdoutRelative = Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $step.stdoutPath
    $stderrRelative = Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $step.stderrPath
    $markdown += "| $($step.name) | $expectation | $($step.exitCode) | $($step.durationMs) | '$stdoutRelative' | '$stderrRelative' |"
}

$markdown += ""
$markdown += "## Bundle Inventory"
$markdown += ""
$markdown += "| Artifact | Size (bytes) | SHA256 |"
$markdown += "| --- | ---: | --- |"
$markdown += "| $(Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $bundleManifest.releaseExecutable.absolutePath) | $($bundleManifest.releaseExecutable.sizeBytes) | '$($bundleManifest.releaseExecutable.sha256)' |"
$markdown += "| $(Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $bundleManifest.portableZip.absolutePath) | $($bundleManifest.portableZip.sizeBytes) | '$($bundleManifest.portableZip.sha256)' |"

$markdown -join "`n" | Set-Content -LiteralPath $markdownReportPath

Write-Host "Workbench beta smoke evidence written to:"
Write-Host "  $(Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $jsonReportPath)"
Write-Host "  $(Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $markdownReportPath)"
Write-Host "  $(Get-RepoRelativePath -RepoRoot $repoRoot -AbsolutePath $bundleManifestPath)"
