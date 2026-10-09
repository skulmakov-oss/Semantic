[CmdletBinding()]
param([string]$BaseSha, [string]$HeadSha, [string]$RepositoryPath = (Join-Path $PSScriptRoot '..'))

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$mode = if ($BaseSha -or $HeadSha) { 'trusted-pr' } else { 'local' }
$envelopePath = '.harness/current.task.yaml'
$checkerPath = 'scripts/harness-check.ps1'
$controlPlane = @($envelopePath, $checkerPath, '.github/**', 'AGENTS.md', 'CONSTRAINTS.md',
    'tests/harness_scope_enforcement.ps1', 'docs/agents/WORKFLOW.md')

function Git([string[]]$Arguments) {
    $start = [Diagnostics.ProcessStartInfo]::new('git')
    $start.UseShellExecute = $false
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.StandardOutputEncoding = [Text.UTF8Encoding]::new($false, $true)
    foreach ($arg in @('--no-replace-objects', '-C', $RepositoryPath) + $Arguments) { $start.ArgumentList.Add($arg) }
    $process = [Diagnostics.Process]::Start($start)
    try {
        $errorText = $process.StandardError.ReadToEndAsync()
        $output = $process.StandardOutput.ReadToEnd()
        $process.WaitForExit()
        if ($process.ExitCode -ne 0) { throw "git-failed command=$($Arguments[0]) $($errorText.GetAwaiter().GetResult().Trim())" }
        return $output
    } finally { $process.Dispose() }
}

function Sorted-Paths([string]$Output) {
    $paths = [Collections.Generic.SortedSet[string]]::new([StringComparer]::Ordinal)
    foreach ($path in $Output.Split([char]0, [StringSplitOptions]::RemoveEmptyEntries)) {
        if ($path -match '[\x00-\x1f\x7f\\]' -or $path.StartsWith('/') -or $path.Split('/') -contains '..' -or $path.Split('/') -contains '.') {
            throw ('unsafe-path path=' + (ConvertTo-Json -InputObject $path -Compress))
        }
        [void]$paths.Add($path)
    }
    return ,@($paths)
}

function Scalar([string]$Value) {
    if ($Value -ceq 'true') { return $true }
    if ($Value -ceq 'false') { return $false }
    if ($Value -match '^(["'']).*\1$') { return $Value.Substring(1, $Value.Length - 2) }
    if ($Value -match '^[!&*{]' -or $Value -match '\s#' -or $Value -match ':\s') { throw 'unsupported-yaml-scalar' }
    return $Value
}

# Deliberately bounded YAML: mappings at 0/2 spaces, scalar lists at 4 spaces,
# and literal/folded prose. No YAML tags, anchors, expressions, or duplicate keys.
function Envelope([string]$Text) {
    $document = [Collections.Generic.Dictionary[string,object]]::new([StringComparer]::Ordinal)
    $section = $null; $list = $null; $block = $false
    foreach ($line in ($Text -split '\r?\n')) {
        if ($line -match '^\s*(#.*)?$') { continue }
        if ($line.Contains("`t")) { throw 'unsupported-yaml-indentation' }
        if ($line -cmatch '^([a-z][a-z0-9_]*):$') {
            $name = $Matches[1]
            if ($document.ContainsKey($name)) { throw "duplicate-key key=$name" }
            $section = [Collections.Generic.Dictionary[string,object]]::new([StringComparer]::Ordinal)
            $document.Add($name, $section); $list = $null; $block = $false
        } elseif ($null -ne $section -and $line -cmatch '^  ([a-z][a-z0-9_]*):(?: (.*))?$') {
            $name = $Matches[1]; $value = if ($Matches.ContainsKey(2)) { $Matches[2] } else { '' }
            if ($section.ContainsKey($name)) { throw "duplicate-key key=$name" }
            $block = $value -cmatch '^[>|]-?$'
            $list = $null
            if ($value -ceq '') { $list = [Collections.Generic.List[object]]::new(); $section.Add($name, $list) }
            elseif ($block) { $section.Add($name, '(block)') }
            else { $section.Add($name, (Scalar $value)) }
        } elseif ($null -ne $section -and $block -and $line.StartsWith('    ')) {
            continue
        } elseif ($null -ne $list -and $line -cmatch '^    - (.+)$') {
            $list.Add((Scalar $Matches[1]))
        } else { throw 'unsupported-yaml-structure' }
    }
    foreach ($name in @('task', 'scope', 'authorization', 'constraints')) {
        if (-not $document.ContainsKey($name)) { throw "missing-section section=$name" }
    }
    foreach ($name in @('id', 'type', 'mode', 'authorized_by')) {
        if (-not $document['task'].ContainsKey($name) -or $document['task'][$name] -isnot [string] -or
            -not $document['task'][$name]) { throw "missing-task-field key=$name" }
    }
    if ($document['task']['mode'] -cne 'active') { throw 'inactive-envelope' }
    foreach ($name in @('allowed_paths', 'forbidden_paths')) {
        if (-not $document['scope'].ContainsKey($name) -or $document['scope'][$name] -isnot [Collections.Generic.List[object]]) { throw "invalid-path-list key=$name" }
        foreach ($pattern in $document['scope'][$name]) { Pattern ([string]$pattern) | Out-Null }
    }
    if ($document['scope']['allowed_paths'].Count -eq 0) { throw 'empty-allowed-scope' }
    return ,$document
}

function Pattern([string]$Path) {
    if (-not $Path -or $Path -cnotmatch '^[A-Za-z0-9_.* /-]+$' -or $Path.StartsWith('/') -or
        $Path.Split('/') -contains '..' -or $Path.Split('/') -contains '.' -or $Path.Contains('//')) { throw "invalid-pattern pattern=$Path" }
    $subtree = $Path.EndsWith('/**', [StringComparison]::Ordinal)
    $prefix = if ($subtree) { $Path.Substring(0, $Path.Length - 3) } else { $Path }
    if ($prefix.Contains('**')) { throw "unsupported-glob pattern=$Path" }
    $regex = [regex]::Escape($prefix).Replace('\*', '[^/]*')
    if ($subtree) { $regex += '(?:/.*)?' }
    return '^' + $regex + '$'
}

function Matches-Path([string]$Path, $Patterns) {
    foreach ($pattern in $Patterns) { if ($Path -cmatch (Pattern ([string]$pattern))) { return $true } }
    return $false
}

# Exact patterns are decidable; wildcard prefix overlap is conservative.
function May-Overlap([string]$Left, [string]$Right) {
    if (-not $Left.Contains('*')) { return Matches-Path $Left @($Right) }
    if (-not $Right.Contains('*')) { return Matches-Path $Right @($Left) }
    $a = $Left.Split('*')[0].TrimEnd('/'); $b = $Right.Split('*')[0].TrimEnd('/')
    return $a.StartsWith($b, [StringComparison]::Ordinal) -or $b.StartsWith($a, [StringComparison]::Ordinal)
}

function Flag($Document, [string]$Section, [string]$Key, [bool]$Expected) {
    if (-not $Document.ContainsKey($Section) -or -not $Document[$Section].ContainsKey($Key) -or
        $Document[$Section][$Key] -isnot [bool] -or $Document[$Section][$Key] -ne $Expected) { throw "protected-invariant key=$Section.$Key expected=$Expected" }
}

function Validate-Transition($Base, $Candidate) {
    Flag $Candidate 'authorization' 'merge_after_review_and_checks' $false
    Flag $Candidate 'authorization' 'stable_promotion' $false
    Flag $Candidate 'constraints' 'no_merge_without_owner_go' $true
    Flag $Candidate 'constraints' 'no_release_or_tag' $true
    Flag $Candidate 'constraints' 'release_assets_immutable' $true
    Flag $Candidate 'constraints' 'evidence_before_claims' $true
    Flag $Candidate 'constraints' 'one_logical_change_per_pr' $true
    Flag $Candidate 'governance_constraints' 'release_tag_changes' $false
    Flag $Candidate 'governance_constraints' 'merge_without_owner_go' $false
    foreach ($key in $Candidate['authorization'].Keys) {
        if (-not $Base['authorization'].ContainsKey($key) -or $Candidate['authorization'][$key] -isnot [bool]) { throw "unexpected-authorization key=$key" }
    }
    foreach ($allow in $Candidate['scope']['allowed_paths']) {
        if ($Candidate['task']['type'] -ceq 'governance_migration') {
            if ($allow -cnotin $Base['scope']['allowed_paths'] -or -not (Matches-Path $allow $controlPlane)) { throw "governance-class-expansion pattern=$allow" }
        } else {
            foreach ($protected in $controlPlane) {
                if (-not (May-Overlap $allow $protected)) { continue }
                $denied = $false
                if ($protected.EndsWith('/**', [StringComparison]::Ordinal)) {
                    # A root-only or child-only denial cannot protect a whole subtree.
                    $root = $protected.Substring(0, $protected.Length - 3)
                    foreach ($deny in $Candidate['scope']['forbidden_paths']) {
                        if ($deny.EndsWith('/**', [StringComparison]::Ordinal) -and (Matches-Path $root @($deny))) { $denied = $true; break }
                    }
                } else { $denied = Matches-Path $protected $Candidate['scope']['forbidden_paths'] }
                if (-not $denied) { throw "control-plane-authority pattern=$allow protected=$protected" }
            }
        }
    }
}

function Git-Envelope([string]$Sha) {
    $entry = (Git @('ls-tree', $Sha, '--', $envelopePath)).Trim()
    if ($entry -cnotmatch '^100644 blob [0-9a-f]+\t') { throw 'envelope-not-regular-blob' }
    $object = "${Sha}:$envelopePath"
    if ([long](Git @('cat-file', '-s', $object)).Trim() -gt 65536) { throw 'envelope-size-limit' }
    return Envelope (Git @('show', $object))
}

try {
    $RepositoryPath = [IO.Path]::GetFullPath($RepositoryPath)
    if ($mode -ceq 'trusted-pr') {
        foreach ($sha in @($BaseSha, $HeadSha)) {
            if ($sha -cnotmatch '^[0-9a-f]{40}$' -or (Git @('cat-file', '-t', $sha)).Trim() -cne 'commit') { throw 'immutable-commit-required' }
        }
        $expectedChecker = (Git @('rev-parse', "${BaseSha}:$checkerPath")).Trim()
        $actualChecker = (Git @('hash-object', '--no-filters', '--', $PSCommandPath)).Trim()
        if ($actualChecker -cne $expectedChecker) { throw 'checker-provenance executable-must-equal-base-blob' }
        $base = Git-Envelope $BaseSha
        $paths = Sorted-Paths (Git @('diff', '--no-ext-diff', '--no-textconv', '--no-renames', '--name-only', '-z', $BaseSha, $HeadSha, '--'))
    } else {
        $base = Envelope ([IO.File]::ReadAllText((Join-Path $RepositoryPath $envelopePath)))
        $paths = Sorted-Paths ((Git @('diff', '--no-ext-diff', '--no-textconv', '--no-renames', '--name-only', '-z', '--cached', '--')) +
            (Git @('diff', '--no-ext-diff', '--no-textconv', '--no-renames', '--name-only', '-z', '--')))
    }
    Write-Output "[harness] mode=$mode base=$BaseSha head=$HeadSha"
    foreach ($path in $paths) { Write-Output ('[harness:path] ' + (ConvertTo-Json -InputObject $path -Compress)) }
    $migration = $base['task']['type'] -ceq 'governance_migration'
    if ($mode -ceq 'trusted-pr') {
        if ($envelopePath -cin $paths) {
            if (-not $migration) { throw "envelope-immutable path=$envelopePath" }
            if ($paths.Count -ne 1) { throw 'transition-envelope-only' }
            Validate-Transition $base (Git-Envelope $HeadSha)
        } elseif (-not $migration) {
            if ($checkerPath -cin $paths) { throw "checker-immutable path=$checkerPath" }
            foreach ($path in $paths) {
                if (Matches-Path $path @('.github/**')) { throw "workflow-immutable path=$path" }
            }
        }
    }
    foreach ($path in $paths) {
        if (Matches-Path $path $base['scope']['forbidden_paths']) { throw "forbidden-path path=$path" }
        if (-not (Matches-Path $path $base['scope']['allowed_paths'])) { throw "outside-scope path=$path" }
        if ($mode -ceq 'trusted-pr' -and $migration -and -not (Matches-Path $path $controlPlane)) { throw "governance-control-plane-only path=$path" }
    }
    if ($mode -ceq 'local') {
        Git @('diff', '--check', '--cached') | Out-Null
        Git @('diff', '--check') | Out-Null
    }
    Write-Output "[harness] PASS mode=$mode"
} catch {
    Write-Output "[harness:error] mode=$mode base=$BaseSha head=$HeadSha rule=$($_.Exception.Message)"
    exit 1
}
