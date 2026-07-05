Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))

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
        Fail "FAIL: Missing guard: $($Path)"
    }
}

function Invoke-Guard {
    param(
        [string]$Name,
        [string]$Path
    )

    Write-Host "RUN: $Name"
    & $Path
    $exitCode = 0
    $lastExitCode = Get-Variable -Name LASTEXITCODE -Scope 0 -ErrorAction SilentlyContinue
    if ($null -ne $lastExitCode) {
        $exitCode = [int]$lastExitCode.Value
    }
    if ($exitCode -ne 0) {
        Fail "FAIL: $Name failed"
    }
}

$guards = @(
    @{
        Name = "ProjectionBundle fixture anchor guard"
        Path = Join-Path $repoRoot "tools/post_ui/check_projection_bundle_fixtures.ps1"
    },
    @{
        Name = "ProjectionBundle manifest draft type guard"
        Path = Join-Path $repoRoot "tools/post_ui/check_projection_bundle_manifest_draft.ps1"
    },
    @{
        Name = "ProjectionBundle manifest sketch/draft drift guard"
        Path = Join-Path $repoRoot "tools/post_ui/check_projection_bundle_manifest_drift.ps1"
    },
    @{
        Name = "ProjectionBundle sketch reader draft guard"
        Path = Join-Path $repoRoot "tools/post_ui/check_projection_bundle_sketch_reader_draft.ps1"
    },
    @{
        Name = "ProjectionBundle claim boundary guard"
        Path = Join-Path $repoRoot "tools/post_ui/check_projection_bundle_claim_boundaries.ps1"
    }
)

foreach ($guard in $guards) {
    Assert-FileExists -Path $guard.Path -Label "guard"
    Invoke-Guard -Name $guard.Name -Path $guard.Path
}

Write-Host "PASS: POST-UI fixture guards passed"
