Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$taskFile = Join-Path $PSScriptRoot '..\.harness\current.task.yaml'
$taskFile = [System.IO.Path]::GetFullPath($taskFile)

Write-Host "[harness] checking local harness state"

if (-not (Test-Path -LiteralPath $taskFile)) {
    throw "[harness:error] missing $taskFile"
}

function Get-GitChangedPaths {
    $paths = @()
    $paths += git diff --name-only --cached
    $paths += git diff --name-only
    $paths | Where-Object { $_ -and $_.Trim() -ne '' } | ForEach-Object { $_.Replace('\', '/') } | Sort-Object -Unique
}

$forbiddenPatterns = @(
    '^crates/sm-vm/.*',
    '^crates/sm-verify/.*',
    '^crates/prom-.*',
    '^crates/sm-ir/.*',
    '^crates/sm-emit/.*',
    '^Cargo\.toml$',
    '^Cargo\.lock$'
)

function Get-AllowedPathsFromYaml {
    param([string]$FilePath)
    $lines = Get-Content $FilePath
    $paths = @()
    $inAllowedPaths = $false
    foreach ($line in $lines) {
        if ($line -match '^\s*allowed_paths:\s*$') {
            $inAllowedPaths = $true
            continue
        }
        if ($inAllowedPaths) {
            if ($line -match '^\s*-\s+(.+)$') {
                $path = $matches[1].Trim("'", '"')
                $paths += $path
            } elseif ($line -match '^\s*[a-zA-Z0-9_]+:') {
                $inAllowedPaths = $false
            }
        }
    }
    return $paths
}

$yamlPaths = Get-AllowedPathsFromYaml -FilePath $taskFile
if ($yamlPaths.Count -eq 0) {
    Write-Host "[harness:error] failed to read allowed_paths from current.task.yaml"
    exit 1
}

$allowedPatterns = @()
foreach ($p in $yamlPaths) {
    $escaped = [regex]::Escape($p)
    $escaped = $escaped -replace '\\\*\\\*', '.*'
    $escaped = $escaped -replace '\\\*', '[^/]*'
    $allowedPatterns += "^$escaped`$"
}

$changedPaths = Get-GitChangedPaths

foreach ($path in $changedPaths) {
    foreach ($pattern in $forbiddenPatterns) {
        if ($path -match $pattern) {
            Write-Host "[harness:error] forbidden path changed: $path"
            exit 1
        }
    }

    $matchedAllowed = $false
    foreach ($pattern in $allowedPatterns) {
        if ($path -match $pattern) {
            $matchedAllowed = $true
            break
        }
    }

    if (-not $matchedAllowed) {
        Write-Host "[harness:error] path outside allowed scope: $path"
        exit 1
    }
}

git diff --check

Write-Host "[harness] ok"
