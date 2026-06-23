$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# Legacy compatibility wrapper.
# New workflows should call scripts/admission_guard.ps1.

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$GuardScript = Join-Path $ScriptDir "admission_guard.ps1"

if ($args.Count -eq 0) {
    & pwsh -File $GuardScript -CIParity
} else {
    & pwsh -File $GuardScript @args
}
exit $LASTEXITCODE
