# Live smoke test for the canonical native Workbench binary
# (examples/workbench_semantic). Builds the real workbench_semantic.exe,
# launches it as a real OS process against a throwaway project directory,
# lets it run its real native winit/wgpu window + event loop for a short
# window, then terminates it and checks the captured stdout for the real
# startup sequence and the absence of a panic. This is a process-level
# check (did the real binary start, open a real window, and stay alive) --
# it does not replace the InMemoryBackend adapter-boundary tests, which
# check UI *behavior* through the identical DesktopSession plumbing.
param(
    [string]$OutputRoot = "artifacts/workbench/native-launch-smoke",
    [int]$LaunchSmokeSeconds = 6
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$outputDirectory = Join-Path $repoRoot $OutputRoot
New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null

$projectDir = Join-Path $outputDirectory "smoke-project"
if (Test-Path -LiteralPath $projectDir) {
    Remove-Item -LiteralPath $projectDir -Recurse -Force
}
New-Item -ItemType Directory -Path $projectDir -Force | Out-Null
Set-Content -LiteralPath (Join-Path $projectDir "main.sm") -Value "fn main() {`n    assert(1 == 1);`n    return;`n}`n" -NoNewline

Write-Host "Building workbench_semantic..."
& cargo build -p workbench_semantic
if ($LASTEXITCODE -ne 0) {
    throw "cargo build -p workbench_semantic failed with exit code $LASTEXITCODE"
}

$exePath = Join-Path $repoRoot "target/debug/workbench_semantic.exe"
if (-not (Test-Path -LiteralPath $exePath)) {
    throw "expected built executable at '$exePath'"
}

$stdoutPath = Join-Path $outputDirectory "stdout.log"
$stderrPath = Join-Path $outputDirectory "stderr.log"

Write-Host "Launching $exePath against $projectDir for ${LaunchSmokeSeconds}s..."
$proc = Start-Process -FilePath $exePath -ArgumentList @($projectDir) `
    -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath `
    -PassThru -WindowStyle Normal

Start-Sleep -Seconds $LaunchSmokeSeconds

$stillRunning = -not $proc.HasExited
if ($stillRunning) {
    Write-Host "Process is still alive after ${LaunchSmokeSeconds}s (expected -- the real event loop blocks until the window closes). Terminating."
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
} else {
    Write-Host "Process exited on its own with code $($proc.ExitCode) before the smoke window elapsed."
}

Start-Sleep -Milliseconds 300
$stdout = if (Test-Path -LiteralPath $stdoutPath) { Get-Content -LiteralPath $stdoutPath -Raw } else { "" }
$stderr = if (Test-Path -LiteralPath $stderrPath) { Get-Content -LiteralPath $stderrPath -Raw } else { "" }

$sawBanner = $stdout -match [regex]::Escape("=== Semantic Workbench (Native Semantic + Prom UI Application) ===")
$sawProjectRoot = $stdout -match [regex]::Escape("project root:")
$sawFileDiscovery = $stdout -match [regex]::Escape("discovered 1 .sm file(s) in opened project")
$sawPanic = ($stdout -match "panicked at") -or ($stderr -match "panicked at")

$reportPath = Join-Path $outputDirectory "report.md"
$lines = @(
    "# Workbench native launch smoke -- $(Get-Date -Format o)",
    "",
    "- executable: $exePath",
    "- project dir: $projectDir",
    "- ran for at least ${LaunchSmokeSeconds}s without exiting: $stillRunning",
    "- saw startup banner: $sawBanner",
    "- saw project root line: $sawProjectRoot",
    "- saw real file discovery line ('discovered 1 .sm file(s)'): $sawFileDiscovery",
    "- saw a panic in stdout/stderr: $sawPanic",
    "",
    "## stdout",
    '```',
    $stdout,
    '```',
    "",
    "## stderr",
    '```',
    $stderr,
    '```'
)
Set-Content -LiteralPath $reportPath -Value ($lines -join "`n")

$pass = $stillRunning -and $sawBanner -and $sawProjectRoot -and $sawFileDiscovery -and (-not $sawPanic)
Write-Host "Report written to $reportPath"
if ($pass) {
    Write-Host "PASS: native Workbench launched a real process, opened its real window/event loop, and stayed alive with no panic."
    exit 0
} else {
    Write-Host "FAIL: see $reportPath for captured stdout/stderr."
    exit 1
}
