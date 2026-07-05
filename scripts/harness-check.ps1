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

function Get-PathsFromYaml {
    param([string]$FilePath, [string]$Key)
    $lines = Get-Content $FilePath
    $paths = @()
    $inKey = $false
    foreach ($line in $lines) {
        if ($line -match "^\s*${Key}:\s*$") {
            $inKey = $true
            continue
        }
        if ($inKey) {
            if ($line -match '^\s*-\s+(.+)$') {
                $path = $matches[1].Trim("'", '"')
                $paths += $path
            } elseif ($line -match '^\s*[a-zA-Z0-9_]+:') {
                $inKey = $false
            }
        }
    }
    return $paths
}

$yamlAllowed = Get-PathsFromYaml -FilePath $taskFile -Key "allowed_paths"
if ($yamlAllowed.Count -eq 0) {
    Write-Host "[harness:error] failed to read allowed_paths from current.task.yaml"
    exit 1
}

$yamlForbidden = Get-PathsFromYaml -FilePath $taskFile -Key "forbidden_paths"

$allowedPatterns = @()
foreach ($p in $yamlAllowed) {
    $escaped = [regex]::Escape($p)
    $escaped = $escaped -replace '\\\*\\\*', '.*'
    $escaped = $escaped -replace '\\\*', '[^/]*'
    $allowedPatterns += "^$escaped`$"
}

$forbiddenPatterns = @()
foreach ($p in $yamlForbidden) {
    $escaped = [regex]::Escape($p)
    $escaped = $escaped -replace '\\\*\\\*', '.*'
    $escaped = $escaped -replace '\\\*', '[^/]*'
    $forbiddenPatterns += "^$escaped`$"
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
