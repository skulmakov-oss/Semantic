# Output-directory preparation for scripts/verify_release_assets.ps1.
#
# Dot-source this file. It defines functions only.
#
# The verifier resets its per-tag output directory on every run. A caller may
# pass -AssetsDirectory pointing at assets downloaded by an earlier run, which
# typically live inside that same per-tag output directory. The caller's
# assets must therefore be resolved and preserved before the reset rather
# than deleted by it.

function Test-PathWithinDirectory {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Path,

        [Parameter(Mandatory = $true)]
        [string] $Directory
    )

    $separators = [char[]]@([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    $fullPath = [System.IO.Path]::GetFullPath($Path).TrimEnd($separators)
    $fullDirectory = [System.IO.Path]::GetFullPath($Directory).TrimEnd($separators)
    $comparison = if ($IsWindows -or $env:OS -eq "Windows_NT") {
        [System.StringComparison]::OrdinalIgnoreCase
    } else {
        [System.StringComparison]::Ordinal
    }

    if ([string]::Equals($fullPath, $fullDirectory, $comparison)) {
        return $true
    }
    foreach ($separator in $separators) {
        if ($fullPath.StartsWith($fullDirectory + $separator, $comparison)) {
            return $true
        }
    }
    return $false
}

# Resets TagOutputDirectory and returns the directory the verifier should read
# release assets from, or $null when the assets still have to be downloaded.
#
# Caller-provided assets are resolved first (a missing -AssetsDirectory fails
# before anything is deleted). When they live inside TagOutputDirectory, the
# required asset files are staged outside it, the directory is reset, and the
# staged files are restored under "caller-assets/" inside the fresh output.
function Initialize-ReleaseAssetOutput {
    param(
        [Parameter(Mandatory = $true)]
        [string] $TagOutputDirectory,

        [string] $AssetsDirectory,

        [Parameter(Mandatory = $true)]
        [string[]] $RequiredAssets
    )

    $assetRoot = $null
    if ($AssetsDirectory) {
        $assetRoot = (Resolve-Path -LiteralPath $AssetsDirectory -ErrorAction Stop).Path
    }

    $stagingDirectory = $null
    if ($assetRoot -and (Test-PathWithinDirectory -Path $assetRoot -Directory $TagOutputDirectory)) {
        $stagingDirectory = Join-Path ([System.IO.Path]::GetTempPath()) ("semantic-release-assets-" + [System.Guid]::NewGuid().ToString("N"))
        New-Item -ItemType Directory -Force -Path $stagingDirectory | Out-Null
        foreach ($assetName in $RequiredAssets) {
            $source = Join-Path $assetRoot $assetName
            if (Test-Path -LiteralPath $source -PathType Leaf) {
                Copy-Item -LiteralPath $source -Destination (Join-Path $stagingDirectory $assetName) -Force
            }
        }
    }

    try {
        if (Test-Path -LiteralPath $TagOutputDirectory) {
            Remove-Item -LiteralPath $TagOutputDirectory -Recurse -Force
        }
        New-Item -ItemType Directory -Force -Path $TagOutputDirectory | Out-Null

        if ($stagingDirectory) {
            $restored = Join-Path $TagOutputDirectory "caller-assets"
            New-Item -ItemType Directory -Force -Path $restored | Out-Null
            foreach ($staged in @(Get-ChildItem -LiteralPath $stagingDirectory -File)) {
                Copy-Item -LiteralPath $staged.FullName -Destination (Join-Path $restored $staged.Name) -Force
            }
            $assetRoot = (Resolve-Path -LiteralPath $restored).Path
        }
    } finally {
        if ($stagingDirectory -and (Test-Path -LiteralPath $stagingDirectory)) {
            Remove-Item -LiteralPath $stagingDirectory -Recurse -Force
        }
    }

    return $assetRoot
}
