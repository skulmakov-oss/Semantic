param([string]$CheckerPath = (Join-Path $PSScriptRoot '../scripts/harness-check.ps1'), [string[]]$CaseNames = @())
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$checker = [IO.File]::ReadAllText($CheckerPath).Replace("`r`n", "`n")
$workflowText = [IO.File]::ReadAllText((Join-Path $PSScriptRoot '../.github/workflows/harness-trusted.yml')).Replace("`r`n", "`n")
$gitExecutable = (Get-Command git -CommandType Application | Select-Object -First 1).Source
$sandbox = Join-Path ([IO.Path]::GetTempPath()) ('semantic-harness-' + [guid]::NewGuid().ToString('N'))
$results = [Collections.Generic.List[object]]::new()

function Write-FixtureFile([string]$Root, [string]$Path, [string]$Content) {
    $file = Join-Path $Root $Path
    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($file)) | Out-Null
    [IO.File]::WriteAllText($file, $Content, [Text.UTF8Encoding]::new($false))
}

function Git([string]$Root, [string[]]$Arguments) {
    $output = & $gitExecutable -C $Root -c core.fsmonitor=false -c core.hooksPath=/dev/null -c commit.gpgSign=false @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) { throw "fixture git failed: $Arguments $output" }
    return ($output -join "`n").Trim()
}

function Envelope([bool]$Migration = $false, [string[]]$Allowed = @('crates/sm-vm/**'),
                  [string[]]$Forbidden = @('crates/prom-*/**', 'Cargo.toml', '.github/**')) {
    $type = if ($Migration) { 'governance_migration' } else { 'foundation_delivery' }
    return @"
task:
  id: FIXTURE
  type: $type
  mode: active
  authorized_by: owner fixture
scope:
  allowed_paths:
$($Allowed | ForEach-Object { "    - $_" } | Join-String -Separator "`n")
  forbidden_paths:
$($Forbidden | ForEach-Object { "    - $_" } | Join-String -Separator "`n")
authorization:
  merge_after_review_and_checks: false
  stable_promotion: false
governance_constraints:
  release_tag_changes: false
  merge_without_owner_go: false
constraints:
  no_merge_without_owner_go: true
  no_release_or_tag: true
  release_assets_immutable: true
  evidence_before_claims: true
  one_logical_change_per_pr: true
"@
}

function Case([string]$Name, [string]$BaseEnvelope, [hashtable]$Changes, [bool]$Pass,
              [string]$Reason = '', [switch]$WrongBase, [switch]$Repeat,
              [switch]$Replacement, [string]$UnsafePath, [switch]$Workflow,
              [string]$BaseBranch = 'main', [string]$BaseRepository = 'skulmakov-oss/Semantic') {
    if ($CaseNames.Count -and $Name -cnotin $CaseNames) { return }
    $root = Join-Path $sandbox $Name
    [IO.Directory]::CreateDirectory($root) | Out-Null
    Git $root @('init', '-q') | Out-Null
    Git $root @('config', 'user.email', 'harness-fixture@example.invalid') | Out-Null
    Git $root @('config', 'user.name', 'Harness fixture') | Out-Null
    Git $root @('config', 'core.autocrlf', 'false') | Out-Null
    Git $root @('config', 'core.ignorecase', 'false') | Out-Null
    Write-FixtureFile $root '.harness/current.task.yaml' $BaseEnvelope
    Write-FixtureFile $root 'scripts/harness-check.ps1' $checker
    Write-FixtureFile $root '.github/workflows/harness-trusted.yml' $workflowText
    Write-FixtureFile $root 'crates/sm-vm/src/lib.rs' 'base'
    Write-FixtureFile $root 'Cargo.toml' 'base'
    Git $root @('add', '.') | Out-Null
    Git $root @('commit', '-qm', 'base') | Out-Null
    $base = Git $root @('rev-parse', 'HEAD')
    # This executable is the exact BASE blob; HEAD may replace its own checker.
    $trusted = Join-Path $sandbox ('_trusted/' + $Name + '.ps1')
    Write-FixtureFile $sandbox ('_trusted/' + $Name + '.ps1') ((Git $root @('show', ($base + ':scripts/harness-check.ps1'))) + "`n")
    foreach ($path in $Changes.Keys) {
        if ($null -eq $Changes[$path]) { Remove-Item -LiteralPath (Join-Path $root $path) }
        else { Write-FixtureFile $root $path $Changes[$path] }
    }
    Git $root @('add', '--', '.harness', 'scripts', 'crates', 'Cargo.toml') | Out-Null
    foreach ($path in $Changes.Keys) {
        if ($null -ne $Changes[$path]) { Git $root @('add', '--', $path) | Out-Null }
    }
    if ($UnsafePath) {
        # Write a hostile tree object directly: Windows index/worktree APIs reject
        # these names before the checker could exercise its own fail-closed rule.
        $blob = Git $root @('rev-parse', ($base + ':crates/sm-vm/src/lib.rs'))
        $bytes = [Text.Encoding]::UTF8.GetBytes("100644 $UnsafePath" + [char]0) + [Convert]::FromHexString($blob)
        $start = [Diagnostics.ProcessStartInfo]::new($gitExecutable)
        $start.UseShellExecute = $false; $start.RedirectStandardInput = $true; $start.RedirectStandardOutput = $true
        foreach ($arg in @('-C', $root, 'hash-object', '--literally', '-w', '-t', 'tree', '--stdin')) { $start.ArgumentList.Add($arg) }
        $process = [Diagnostics.Process]::Start($start)
        try {
            $process.StandardInput.BaseStream.Write([byte[]]$bytes, 0, $bytes.Length); $process.StandardInput.Close()
            $tree = $process.StandardOutput.ReadToEnd().Trim(); $process.WaitForExit()
            if ($process.ExitCode) { throw 'hostile tree creation failed' }
        } finally {
            if (-not $process.HasExited) { $process.Kill(); $process.WaitForExit() }
            $process.Dispose()
        }
        $head = Git $root @('commit-tree', $tree, '-p', $base, '-m', 'hostile candidate')
    } else {
        Git $root @('commit', '-qm', 'candidate') | Out-Null
        $head = Git $root @('rev-parse', 'HEAD')
    }
    if ($Replacement) { Git $root @('replace', $head, $base) | Out-Null }
    if ($WrongBase) { $base = $head }
    if ($Workflow) {
        Git $root @('update-ref', 'refs/pull/1/head', $head) | Out-Null
        $checkout = Join-Path $sandbox ('_hosted/' + $Name)
        Git $root @('clone', '-q', '--no-checkout', $root, $checkout) | Out-Null
        Git $checkout @('config', 'core.autocrlf', 'false') | Out-Null
        Git $checkout @('checkout', '-q', '--detach', $base) | Out-Null
        $lines = (Git $root @('show', ($base + ':.github/workflows/harness-trusted.yml'))) -split "`n"
        $run = [Array]::IndexOf($lines, '        run: |')
        if ($run -lt 0) { throw 'trusted workflow run block missing' }
        $program = ($lines[($run + 1)..($lines.Length - 1)] | ForEach-Object { $_.Substring(10) }) -join "`n"
        Write-FixtureFile $sandbox ('_trusted/' + $Name + '-workflow.ps1') $program
        $environment = @{ BASE_REF = $BaseBranch; BASE_REPOSITORY = $BaseRepository; REPOSITORY = 'skulmakov-oss/Semantic'; BASE_SHA = $base; HEAD_SHA = $head; PR_NUMBER = '1' }
        $saved = @{}
        foreach ($variable in $environment.Keys) {
            $saved[$variable] = [Environment]::GetEnvironmentVariable($variable)
            [Environment]::SetEnvironmentVariable($variable, $environment[$variable])
        }
        Push-Location $checkout
        try {
            $output = (& pwsh -NoProfile -File (Join-Path $sandbox ('_trusted/' + $Name + '-workflow.ps1')) 2>&1 | Out-String).Trim()
            $exitCode = $LASTEXITCODE
            if ((Git $checkout @('rev-parse', 'HEAD')) -cne $base) { throw 'workflow checked out candidate' }
        } finally {
            Pop-Location
            foreach ($variable in $saved.Keys) { [Environment]::SetEnvironmentVariable($variable, $saved[$variable]) }
        }
    } else {
        $output = (& pwsh -NoProfile -File $trusted -RepositoryPath $root -BaseSha $base -HeadSha $head 2>&1 | Out-String).Trim()
        $exitCode = $LASTEXITCODE
    }
    $actual = $exitCode -eq 0
    if ($actual -ne $Pass -or ($Reason -and $output -notlike "*$Reason*")) {
        throw "$Name expected=$Pass actual=$actual exit=$exitCode reason=$Reason`n$output"
    }
    if ($Repeat) {
        $again = (& pwsh -NoProfile -File $trusted -RepositoryPath $root -BaseSha $base -HeadSha $head 2>&1 | Out-String).Trim()
        if ($LASTEXITCODE -ne $exitCode -or $again -cne $output) { throw "$Name nondeterministic result" }
    }
    $results.Add([pscustomobject]@{ Case = $Name; Expected = $(if ($Pass) { 'PASS' } else { 'FAIL' }); Actual = $(if ($actual) { 'PASS' } else { 'FAIL' }) })
}

try {
    $stable = Envelope
    $migration = Envelope $true @('.harness/current.task.yaml', 'scripts/harness-check.ps1', '.github/**', 'tests/harness_scope_enforcement.ps1', 'docs/agents/WORKFLOW.md') @('crates/**', 'Cargo.toml')
    Case 'P1' $stable @{'crates/sm-vm/src/lib.rs' = 'allowed'} $true -Repeat
    Case 'N1' $stable @{'.harness/current.task.yaml' = $stable + "`n# edit"} $false 'envelope-immutable'
    Case 'N2' $stable @{'.harness/current.task.yaml' = $stable.Replace('crates/sm-vm/**', 'crates/prom-*/**'); 'crates/prom-foo/src/lib.rs' = 'forbidden'} $false 'envelope-immutable'
    Case 'N3' $stable @{'.harness/current.task.yaml' = $stable.Replace('crates/sm-vm/**', 'Cargo.toml'); 'Cargo.toml' = 'forbidden'} $false 'envelope-immutable'
    Case 'N4' $stable @{'.harness/current.task.yaml' = $stable.Replace('crates/sm-vm/**', '.github/**'); '.github/workflows/x.yml' = 'forbidden'} $false 'envelope-immutable'
    Case 'N5' $stable @{'scripts/harness-check.ps1' = 'exit 0'; 'crates/sm-vm/src/lib.rs' = 'allowed'} $false 'checker-immutable'
    Case 'N6' $migration @{'.harness/current.task.yaml' = $stable; 'crates/sm-vm/src/lib.rs' = 'payload'} $false 'transition-envelope-only'
    Case 'P2' $migration @{'.harness/current.task.yaml' = $stable} $true -Repeat
    $stableTests = Envelope $false @('tests/**') @('tests/harness_scope_enforcement.ps1')
    Case 'P3' $migration @{'.harness/current.task.yaml' = $stableTests} $true -Repeat
    Case 'N10' $migration @{'.harness/current.task.yaml' = (Envelope $false @('tests/**'))} $false 'control-plane-authority'
    Case 'N11' $stableTests @{'tests/harness_scope_enforcement.ps1' = 'candidate data'} $false 'forbidden-path'
    Case 'P4' $stableTests @{'tests/some_foundation_test.rs' = 'ordinary test'} $true
    Case 'docs-carve-out' $migration @{'.harness/current.task.yaml' = (Envelope $false @('docs/**') @('docs/agents/WORKFLOW.md'))} $true
    Case 'docs-unprotected' $migration @{'.harness/current.task.yaml' = (Envelope $false @('docs/**'))} $false 'control-plane-authority'
    Case 'workflow-carve-out' $migration @{'.harness/current.task.yaml' = (Envelope $false @('.github/**', 'tests/**') @('.github/**', 'tests/harness_scope_enforcement.ps1'))} $true
    Case 'workflow-root-only-denial' $migration @{'.harness/current.task.yaml' = (Envelope $false @('.github/**') @('.github'))} $false 'control-plane-authority'
    Case 'workflow-child-only-denial' $migration @{'.harness/current.task.yaml' = (Envelope $false @('.github/**') @('.github/workflows/**'))} $false 'control-plane-authority'
    Case 'tests-carve-out-case' $migration @{'.harness/current.task.yaml' = (Envelope $false @('tests/**') @('tests/HARNESS_scope_enforcement.ps1'))} $false 'control-plane-authority'
    Case 'tests-carve-out-adjacent' $migration @{'.harness/current.task.yaml' = (Envelope $false @('tests/**') @('tests/harness_scope_enforcement.ps1.bak'))} $false 'control-plane-authority'
    Case 'N7' $migration @{'.harness/current.task.yaml' = $stable.Replace('merge_after_review_and_checks: false', 'merge_after_review_and_checks: true')} $false 'merge_after_review_and_checks'
    Case 'N8' $migration @{'.harness/current.task.yaml' = $stable.Replace('no_release_or_tag: true', 'no_release_or_tag: false')} $false 'no_release_or_tag'
    foreach ($path in @('.github-malicious/x', 'scripts/harness-check.ps1.bak', 'Cargo.toml.old')) {
        Case ('N9-' + $results.Count) $stable @{$path = 'adjacent'} $false 'outside-scope'
    }
    foreach ($path in @('.github/x', '.github/workflows/x.yml')) {
        Case ('workflow-' + $results.Count) $migration @{$path = 'data only'} $true
        Case ('workflow-denied-' + $results.Count) $stable @{$path = 'forbidden'} $false 'workflow-immutable'
    }
    Case 'case-sensitive' $migration @{'.GITHUB/x' = 'wrong case'} $false 'outside-scope'
    Case 'new-file' $stable @{'crates/sm-vm/src/new.rs' = 'new'} $true
    Case 'deletion' $stable @{'crates/sm-vm/src/lib.rs' = $null} $true
    Case 'rename-source' $stable @{'Cargo.toml' = $null; 'crates/sm-vm/src/new.rs' = 'base'} $false 'forbidden-path'
    Case 'deny-precedence' (Envelope $false @('crates/**') @('crates/prom-*/**')) @{'crates/prom-foo/src/lib.rs' = 'deny'} $false 'forbidden-path'
    Case 'duplicates' (Envelope $false @('crates/sm-vm/**', 'crates/sm-vm/**')) @{'crates/sm-vm/src/lib.rs' = 'allow'} $true
    Case 'transition-control-plane' $migration @{'.harness/current.task.yaml' = (Envelope $false @('scripts/**'))} $false 'control-plane-authority'
    Case 'transition-deny-precedence' $migration @{'.harness/current.task.yaml' = (Envelope $false @('crates/**') @('crates/prom-*/**'))} $true
    Case 'transition-duplicate-key' $migration @{'.harness/current.task.yaml' = $stable.Replace('  no_release_or_tag: true', "  no_release_or_tag: true`n  no_release_or_tag: false")} $false 'duplicate-key'
    Case 'wrong-base-provenance' $stable @{'scripts/harness-check.ps1' = 'exit 0'} $false 'checker-provenance' -WrongBase
    Case 'literal-shell-path' $stable @{'crates/sm-vm/src/$(throw evil).rs' = 'data'} $true
    Case 'replacement-object' $stable @{'Cargo.toml' = 'forbidden'} $false 'forbidden-path' -Replacement -Repeat
    Case 'newline-path' $stable @{} $false 'unsafe-path' -UnsafePath "bad`nname.rs" -Repeat
    Case 'backslash-path' $stable @{} $false 'unsafe-path' -UnsafePath 'bad\name.rs' -Repeat
    $landedMigration = [IO.File]::ReadAllText((Join-Path $PSScriptRoot '../.harness/current.task.yaml'))
    Case 'landed-envelope' $landedMigration @{'.github/workflows/harness-trusted.yml' = 'governance only'} $true
    Case 'landed-transition' $landedMigration @{'.harness/current.task.yaml' = $stable} $true
    Case 'removed-merge-gate' $migration @{'.harness/current.task.yaml' = $stable.Replace('  no_merge_without_owner_go: true', '')} $false 'no_merge_without_owner_go'
    Case 'removed-evidence' $migration @{'.harness/current.task.yaml' = $stable.Replace('  evidence_before_claims: true', '')} $false 'evidence_before_claims'
    Case 'unknown-release-authority' $migration @{'.harness/current.task.yaml' = $stable.Replace('authorization:', "authorization:`n  release: true")} $false 'unexpected-authorization'
    Case 'invalid-task-scalar' $migration @{'.harness/current.task.yaml' = $stable.Replace('id: FIXTURE', 'id: true')} $false 'missing-task-field'
    Case 'hosted-base-fetch' $stable @{'crates/sm-vm/src/lib.rs' = 'allowed'} $true -Workflow
    Case 'hosted-noncanonical-branch' $stable @{'crates/sm-vm/src/lib.rs' = 'allowed'} $false 'Noncanonical' -Workflow -BaseBranch 'contributor-controlled'
    Case 'hosted-noncanonical-repository' $stable @{'crates/sm-vm/src/lib.rs' = 'allowed'} $false 'Noncanonical' -Workflow -BaseRepository 'untrusted/fork'
    if ($results.Count -eq 0) { throw 'no regression cases selected' }
    $results | Format-Table -AutoSize
    Write-Output "[harness-tests] $($results.Count) cases matched expectations"
} finally {
    $resolved = [IO.Path]::GetFullPath($sandbox)
    $tempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    if (-not $resolved.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($resolved) -notlike 'semantic-harness-*') { throw 'unsafe fixture cleanup target' }
    if (Test-Path -LiteralPath $resolved) { Remove-Item -LiteralPath $resolved -Recurse -Force }
}
