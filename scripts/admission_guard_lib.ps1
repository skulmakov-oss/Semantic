# Shared helpers for scripts/admission_guard.ps1.
#
# Dot-source this file. It defines functions only and performs no work on
# load, so tests/admission_guard_scripts.rs can exercise each helper against
# throwaway fixtures (fake binaries, temporary git repositories) without
# running the full guard.

function Invoke-LocalCiStep {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Name,

        [Parameter(Mandatory = $true)]
        [scriptblock] $Command
    )

    Write-Host ""
    Write-Host "== $Name =="
    $global:LASTEXITCODE = 0
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "local_ci step failed: $Name"
    }
}

# Runs one native command and fails immediately on a non-zero exit code.
#
# PowerShell does not stop a scriptblock when a native command fails, and
# Invoke-LocalCiStep only sees the final $LASTEXITCODE, so every native
# invocation inside a multi-command step must be checked individually.
function Invoke-CheckedNative {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Description,

        [Parameter(Mandatory = $true)]
        [string] $FilePath,

        [string[]] $ArgumentList = @()
    )

    $global:LASTEXITCODE = 0
    & $FilePath @ArgumentList
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) {
        throw "command failed with exit code ${exitCode}: $Description"
    }
}

# Project-root smoke: check -> run -> compile -> verify -> run-smc, where each
# command can fail the sequence on its own.
function Invoke-SmcProjectRootSmoke {
    param(
        [Parameter(Mandatory = $true)]
        [string] $SmcBinary,

        [Parameter(Mandatory = $true)]
        [string] $FixtureRoot,

        [Parameter(Mandatory = $true)]
        [string] $TempDirectory,

        [Parameter(Mandatory = $true)]
        [string] $ArtifactName
    )

    if (Test-Path -LiteralPath $TempDirectory) {
        Remove-Item -Recurse -Force -LiteralPath $TempDirectory
    }
    New-Item -ItemType Directory -Force -Path $TempDirectory | Out-Null

    $artifact = Join-Path $TempDirectory $ArtifactName
    try {
        Invoke-CheckedNative "smc check $FixtureRoot" $SmcBinary @("check", $FixtureRoot)
        Invoke-CheckedNative "smc run $FixtureRoot" $SmcBinary @("run", $FixtureRoot)
        Invoke-CheckedNative "smc compile $FixtureRoot -o $artifact" $SmcBinary @("compile", $FixtureRoot, "-o", $artifact)
        Invoke-CheckedNative "smc verify $artifact" $SmcBinary @("verify", $artifact)
        Invoke-CheckedNative "smc run-smc $artifact" $SmcBinary @("run-smc", $artifact)
    } finally {
        if (Test-Path -LiteralPath $TempDirectory) {
            Remove-Item -Recurse -Force -LiteralPath $TempDirectory
        }
    }
}

# Refreshes the merge-preflight base ref and returns the commit it resolves to.
#
# A `<remote>/<branch>` base ref naming a configured remote is fetched with an
# explicit refspec, so `-BaseRef origin/release` refreshes `origin/release`
# rather than `origin/main`. Any other ref is used as it exists locally (with
# a warning, since it cannot be refreshed). A base ref that does not resolve
# to a commit fails deterministically instead of producing a preflight against
# a missing or stale base.
function Update-MergePreflightBaseRef {
    param(
        [Parameter(Mandatory = $true)]
        [string] $BaseRef
    )

    if ([string]::IsNullOrWhiteSpace($BaseRef)) {
        throw "merge preflight base ref must not be empty"
    }

    $remotes = @(git remote)
    if ($LASTEXITCODE -ne 0) {
        throw "failed to list git remotes for merge preflight"
    }

    $parts = $BaseRef -split '/', 2
    if ($parts.Count -eq 2 -and $parts[1] -ne "" -and ($remotes -contains $parts[0])) {
        $remote = $parts[0]
        $branch = $parts[1]
        git fetch --quiet $remote "+refs/heads/${branch}:refs/remotes/${remote}/${branch}"
        if ($LASTEXITCODE -ne 0) {
            throw "failed to fetch merge preflight base ref '$BaseRef' from remote '$remote'"
        }
    } else {
        Write-Warning "merge preflight base ref '$BaseRef' does not name a configured remote branch; using the local ref without fetching"
    }

    $sha = git rev-parse --verify --quiet "$BaseRef^{commit}"
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($sha)) {
        throw "merge preflight base ref '$BaseRef' does not resolve to a commit"
    }
    return $sha.Trim()
}

# Fails when tracked files differ from HEAD, in the worktree or the index.
#
# `git diff --check` only reports whitespace errors and conflict markers; it
# exits 0 for ordinary tracked modifications. `git diff --exit-code` exits 1
# for any tracked change and >1 on error.
function Assert-TrackedWorktreeUnchanged {
    $unstaged = git diff --name-only
    $unstagedExit = $LASTEXITCODE
    git diff --quiet --exit-code
    $worktreeExit = $LASTEXITCODE
    $staged = git diff --cached --name-only
    git diff --cached --quiet --exit-code
    $indexExit = $LASTEXITCODE

    if ($unstagedExit -gt 1 -or $worktreeExit -gt 1 -or $indexExit -gt 1) {
        throw "failed to inspect tracked worktree changes"
    }
    if ($worktreeExit -ne 0 -or $indexExit -ne 0) {
        $changed = @($unstaged) + @($staged) | Where-Object { $_ } | Sort-Object -Unique
        throw "tracked files differ from HEAD; commit, stash, or revert them first`n$($changed -join [Environment]::NewLine)"
    }
}

function Assert-CleanWorkingTreeForMergePreflight {
    $statusLines = git status --porcelain --untracked-files=all
    if ($LASTEXITCODE -ne 0) {
        throw "failed to inspect working tree status before merge preflight"
    }

    $relevantLines = @(
        $statusLines | Where-Object {
            $_ -and ($_ -notmatch '\.claude([\\/]|$)')
        }
    )

    if ($relevantLines.Count -gt 0) {
        $details = $relevantLines -join [Environment]::NewLine
        throw "merge preflight requires a clean working tree; commit/stash changes first`n$details"
    }
}

# Ordered gate plan for each admission-guard mode. The guard dispatches from
# this plan, so tests can prove that the full preflight is a superset of the
# default guard instead of relying on the two call sequences staying in sync.
function Get-AdmissionGuardPlan {
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet("Quick", "PRReady", "Readiness", "CIParity", "MergePreflight", "FullPreflight", "Default")]
        [string] $Mode
    )

    switch ($Mode) {
        "Quick" { return @("Quick") }
        "PRReady" { return @("PRReady") }
        "Readiness" { return @("Readiness") }
        "CIParity" { return @("CIParity") }
        "MergePreflight" {
            return @("PRReady", "Readiness", "LegacyAdditional", "MergePreflight", "DiffCheck", "TrackedClean")
        }
        "FullPreflight" {
            return @("PRReady", "Readiness", "LegacyAdditional", "MergePreflight", "DiffCheck", "TrackedClean")
        }
        "Default" {
            return @("PRReady", "Readiness", "LegacyAdditional", "DiffCheck", "TrackedClean")
        }
    }
}
