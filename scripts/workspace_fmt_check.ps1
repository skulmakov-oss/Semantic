# Shared Windows-safe replacement for `cargo fmt --all --check`.
#
# Dot-source this file, then call Invoke-WorkspaceFmtCheck. Used by both
# scripts/admission_guard.ps1 and tools/7hell/run_ci.ps1 so there is exactly
# one authoritative package-discovery/check mechanism, not two copies that
# can silently drift apart.

function Invoke-WorkspaceFmtCheck {
    # `cargo fmt --all --check` (and, on this workspace, even bare
    # `cargo fmt --check` - its actual default scope is not root-only; see
    # #1796) spawns rustfmt with every source file's absolute path as a
    # single argument list. On this workspace (554+ .rs files) that list is
    # ~60K characters against Windows' ~32K CreateProcess command-line
    # limit, and the invocation fails with OS error 206 before rustfmt
    # itself ever runs - deterministically once the package set crosses
    # that size, regardless of which command triggered it. Loop per package
    # instead: same coverage (every package still gets checked, none
    # skipped), just split across multiple invocations that each stay under
    # the limit, and unaffected by checkout path length.
    #
    # The package set comes from the full `cargo metadata` resolution
    # (not --no-deps, and not a hand-maintained list), filtered to local
    # (source = null) packages. `cargo fmt --all` covers "all packages, and
    # also their local path-based dependencies" (per `cargo fmt --help`) -
    # a broader set than just [workspace.members], since a local path
    # dependency does not have to be a declared member to be part of that
    # set (e.g. this repo's own ui-shell-kit is pulled in only as a path
    # dependency of a member and isn't listed in [workspace.members]).
    # --no-deps would miss a local path dependency that lives outside the
    # workspace root's directory tree; filtering the full graph by
    # source = null does not, matching --all's actual scope exactly.
    $metadataJson = cargo metadata --format-version 1
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed while deriving the workspace package set for fmt check"
    }
    $packages = ($metadataJson | ConvertFrom-Json).packages |
        Where-Object { $null -eq $_.source } |
        Select-Object -ExpandProperty name |
        Sort-Object

    $failed = @()
    foreach ($pkg in $packages) {
        cargo fmt -p $pkg -- --check
        if ($LASTEXITCODE -ne 0) {
            $failed += $pkg
        }
    }

    if ($failed.Count -gt 0) {
        throw "cargo fmt --check failed for package(s): $($failed -join ', ')"
    }

    Write-Host "cargo fmt --check passed for all $($packages.Count) workspace packages"
}
