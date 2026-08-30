# scripts/context_checkpoint_check.ps1
# Deterministic, dependency-free validator for Semantic context checkpoints
#
# Responsibility split (see docs/agents/CONTEXT.md section 7 for the full writeup):
#   - .harness/context-checkpoint.schema.json is the structural source of truth,
#     enforced here via PowerShell 7's built-in Test-Json (proven against this
#     schema's Draft 2020-12 keyword subset with negative tests; no external
#     dependency). Test-Json does NOT enforce the "format" keyword, so
#     checkpoint.created_at gets one supplemental deterministic check below.
#   - This script additionally enforces cross-field / referential / repository
#     invariants that JSON Schema cannot express on its own: the mandatory
#     authority anchor set, checkpoint-global ID uniqueness, fallback
#     owner-decision referential integrity, budget telemetry-mode coherence,
#     and live-repository/Harness staleness detection (fail-closed).

[CmdletBinding()]
param (
    [Parameter(Mandatory = $false)]
    [string]$Checkpoint,

    [Parameter(Mandatory = $false)]
    [switch]$AgainstCurrentRepo,

    [Parameter(Mandatory = $false)]
    [switch]$SelfTest
)

$ErrorActionPreference = "Stop"

$MandatoryAuthorityAnchors = @("AGENTS.md", "CONSTRAINTS.md", ".harness/current.task.yaml")

function Test-DateTimeFormat {
    # Test-Json does not validate the JSON Schema "format" keyword (proven
    # empirically: a garbage string passes Test-Json against format:date-time).
    # This is the deterministic, dependency-free supplement for RFC 3339 date-time.
    param ([string]$Value)
    if ([string]::IsNullOrWhiteSpace($Value)) { return $false }
    if ($Value -notmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$') { return $false }
    $parsed = [DateTimeOffset]::MinValue
    return [DateTimeOffset]::TryParse(
        $Value,
        [System.Globalization.CultureInfo]::InvariantCulture,
        [System.Globalization.DateTimeStyles]::RoundtripKind,
        [ref]$parsed
    )
}

function Get-HarnessTaskId {
    # Pure helper: extract task.id from raw .harness/current.task.yaml content.
    # Deliberately synthetic-content-testable so self-tests never need to
    # touch the real Harness file to prove malformed/missing-id handling.
    param ([string]$Content)
    if ([string]::IsNullOrWhiteSpace($Content)) { return $null }
    if ($Content -match '(?m)^\s*id:\s*(\S.*?)\s*$') {
        return $Matches[1].Trim()
    }
    return $null
}

function Test-SchemaCompliance {
    param (
        [Parameter(Mandatory = $true)][string]$RawJson,
        [Parameter(Mandatory = $true)][string]$SchemaText,
        [string]$SourceLabel = "Checkpoint"
    )
    $errors = [System.Collections.Generic.List[string]]::new()
    $isValid = $RawJson | Test-Json -Schema $SchemaText -ErrorAction SilentlyContinue -ErrorVariable schemaErrs
    if (-not $isValid) {
        if ($schemaErrs -and $schemaErrs.Count -gt 0) {
            foreach ($e in $schemaErrs) {
                $errors.Add("[$SourceLabel][Schema] $($e.Exception.Message)")
            }
        } else {
            $errors.Add("[$SourceLabel][Schema] Checkpoint does not conform to context-checkpoint.schema.json")
        }
    }
    return $errors
}

function Test-SemanticInvariants {
    param (
        [psobject]$Json,
        [string]$RawJson,
        [string]$SourceLabel = "Checkpoint"
    )
    $errors = [System.Collections.Generic.List[string]]::new()

    # created_at format (schema's "format" keyword is annotation-only under Test-Json).
    # Checked against the RAW JSON TEXT, not $Json.checkpoint.created_at: ConvertFrom-Json
    # silently auto-coerces ISO-8601-looking strings into [datetime] objects, which then
    # lose their original text and re-stringify in the current culture's default format
    # (e.g. "08/30/2026 09:00:00") -- checking the parsed object would produce false failures
    # on genuinely valid dates and could mask a non-ISO string .NET's loose parser still accepts.
    if ($null -ne $Json.checkpoint) {
        $createdAtMatch = [regex]::Match($RawJson, '"created_at"\s*:\s*"([^"]*)"')
        $createdAtRaw = if ($createdAtMatch.Success) { $createdAtMatch.Groups[1].Value } else { $null }
        if (-not (Test-DateTimeFormat $createdAtRaw)) {
            $errors.Add("[$SourceLabel] checkpoint.created_at is not a valid RFC 3339 date-time: '$createdAtRaw'")
        }
    }

    # Mandatory authority anchor set + duplicate-path rejection (cross-item; out of schema's reach)
    if ($null -ne $Json.authority -and $null -ne $Json.authority.references) {
        $seenPaths = [System.Collections.Generic.HashSet[string]]::new()
        $refPaths = @()
        foreach ($ref in $Json.authority.references) {
            if ([string]::IsNullOrWhiteSpace($ref.path)) { continue } # schema already rejects this
            $refPaths += $ref.path
            if (-not $seenPaths.Add($ref.path)) {
                $errors.Add("[$SourceLabel] Duplicate authority reference path: '$($ref.path)'")
            }
        }
        foreach ($anchor in $MandatoryAuthorityAnchors) {
            if ($refPaths -notcontains $anchor) {
                $errors.Add("[$SourceLabel] Missing mandatory authority anchor '$anchor' (required set: $($MandatoryAuthorityAnchors -join ', '))")
            }
        }
    }

    # Checkpoint-global ID uniqueness across every typed category (cross-array; out of schema's reach)
    $idSections = @("facts", "owner_decisions", "review_findings", "hypotheses", "unresolved_questions", "blockers")
    $seenIds = [System.Collections.Generic.HashSet[string]]::new()
    foreach ($section in $idSections) {
        $items = $Json.$section
        if ($null -eq $items) { continue }
        foreach ($item in $items) {
            if ($null -eq $item.id -or [string]::IsNullOrWhiteSpace([string]$item.id)) { continue } # schema-level failure
            if (-not $seenIds.Add($item.id)) {
                $errors.Add("[$SourceLabel] Duplicate checkpoint-global id '$($item.id)' in '$section' (ids must be unique across $($idSections -join ', '))")
            }
        }
    }

    # Verification status/exit_code coherence (cross-field within one entry)
    if ($null -ne $Json.verification) {
        foreach ($v in $Json.verification) {
            if ($v.status -eq "SUCCESS" -and $v.exit_code -ne 0) {
                $errors.Add("[$SourceLabel] Verification entry '$($v.command)' marked SUCCESS with non-zero exit_code: $($v.exit_code)")
            }
        }
    }

    # Fallback authorization referential integrity (must resolve to a real owner_decisions entry)
    if ($null -ne $Json.fallback) {
        $used = $Json.fallback.used
        $decisionId = $Json.fallback.authorization_decision_id
        if ($used -eq $true) {
            if ([string]::IsNullOrWhiteSpace([string]$decisionId)) {
                $errors.Add("[$SourceLabel] fallback.used is true but fallback.authorization_decision_id is missing")
            } else {
                $match = $null
                if ($null -ne $Json.owner_decisions) {
                    $match = $Json.owner_decisions | Where-Object { $_.id -eq $decisionId } | Select-Object -First 1
                }
                if ($null -eq $match) {
                    $errors.Add("[$SourceLabel] fallback.authorization_decision_id '$decisionId' does not resolve to any owner_decisions entry")
                } elseif ($match.category -ne "OWNER_DECISION") {
                    $errors.Add("[$SourceLabel] fallback.authorization_decision_id '$decisionId' does not resolve to an OWNER_DECISION entry")
                }
            }
        } elseif ($used -eq $false -and $null -ne $decisionId) {
            $errors.Add("[$SourceLabel] fallback.used is false but fallback.authorization_decision_id is not null: '$decisionId'")
        }
    }

    # Budget telemetry-mode coherence (cross-field: false must pair with null/null, true requires both + soft < hard)
    if ($null -ne $Json.budget) {
        $telemetryAvailable = $Json.budget.telemetry_available
        $soft = $Json.budget.soft_threshold
        $hard = $Json.budget.hard_threshold
        if ($telemetryAvailable -eq $false) {
            if ($null -ne $soft -or $null -ne $hard) {
                $errors.Add("[$SourceLabel] budget.telemetry_available is false (Event-Trigger Mode) but soft_threshold/hard_threshold are not both null: soft=$soft hard=$hard")
            }
        } elseif ($telemetryAvailable -eq $true) {
            if ($null -eq $soft -or $null -eq $hard) {
                $errors.Add("[$SourceLabel] budget.telemetry_available is true (Telemetry Mode) but soft_threshold/hard_threshold are not both present: soft=$soft hard=$hard")
            } elseif ($soft -ge $hard) {
                $errors.Add("[$SourceLabel] budget.soft_threshold ($soft) must be strictly less than budget.hard_threshold ($hard)")
            }
        }
    }

    return $errors
}

function Validate-Checkpoint {
    param (
        [Parameter(Mandatory = $true)][string]$RawJson,
        [Parameter(Mandatory = $true)][string]$SchemaText,
        [string]$SourceLabel = "Checkpoint"
    )
    $errors = [System.Collections.Generic.List[string]]::new()
    foreach ($e in @(Test-SchemaCompliance -RawJson $RawJson -SchemaText $SchemaText -SourceLabel $SourceLabel)) { $errors.Add($e) }

    $json = $null
    try {
        $json = $RawJson | ConvertFrom-Json -ErrorAction Stop
    } catch {
        $errors.Add("[$SourceLabel] Invalid JSON: $($_.Exception.Message)")
        return $errors
    }

    foreach ($e in @(Test-SemanticInvariants -Json $json -RawJson $RawJson -SourceLabel $SourceLabel)) { $errors.Add($e) }
    return $errors
}

function Test-AgainstCurrentRepository {
    param (
        [psobject]$Json,
        [string]$SourceLabel = "Checkpoint",
        [string]$HarnessPath = ".harness/current.task.yaml"
    )

    $staleness = [System.Collections.Generic.List[string]]::new()

    # 1. Live HEAD check -- exact identity (schema now requires full 40-hex, so prefix matching is neither
    #    needed nor safe: "A starts with B OR B starts with A" was the finding this replaces).
    $currentHead = (git rev-parse HEAD).Trim()
    $chkHead = $Json.repository.head_sha
    if ($currentHead -ne $chkHead) {
        $staleness.Add("[$SourceLabel] HEAD mismatch: Checkpoint head_sha '$chkHead' != Live git HEAD '$currentHead'")
    }

    # 2. Harness task check -- FAIL CLOSED. Missing, unreadable, malformed (id unparseable), and
    #    mismatched task id all produce an error; there is no silent skip path.
    if (-not (Test-Path $HarnessPath)) {
        $staleness.Add("[$SourceLabel] Harness authority file '$HarnessPath' does not exist -- cannot verify active task authority (fail-closed)")
    } else {
        $harnessContent = $null
        try {
            $harnessContent = Get-Content $HarnessPath -Raw -ErrorAction Stop
        } catch {
            $staleness.Add("[$SourceLabel] Harness authority file '$HarnessPath' could not be read: $($_.Exception.Message) (fail-closed)")
        }
        if ($null -ne $harnessContent) {
            $liveTaskId = Get-HarnessTaskId $harnessContent
            if ($null -eq $liveTaskId) {
                $staleness.Add("[$SourceLabel] Harness authority file '$HarnessPath' is malformed -- task id could not be deterministically extracted (fail-closed)")
            } elseif ($Json.authority.harness_task_id -ne $liveTaskId) {
                $staleness.Add("[$SourceLabel] Harness Task ID mismatch (STALE): Checkpoint '$($Json.authority.harness_task_id)' != Live '$liveTaskId'")
            }
        }
    }

    # 3. Authority file blob hash checks -- exact identity, not prefix matching.
    if ($null -ne $Json.authority -and $null -ne $Json.authority.references) {
        foreach ($ref in $Json.authority.references) {
            $path = $ref.path
            if (Test-Path $path) {
                $liveBlobSha = (git hash-object $path).Trim()
                if ($ref.blob_sha -ne $liveBlobSha) {
                    $staleness.Add("[$SourceLabel] Authority hash mismatch for '$path': Checkpoint blob_sha '$($ref.blob_sha)' != Live '$liveBlobSha'")
                }
            } else {
                $staleness.Add("[$SourceLabel] Authority file '$path' does not exist in current working tree")
            }
        }
    }

    return $staleness
}

# --- Structural schema (loaded once; required by both -SelfTest and file validation) ---
$SchemaPath = Join-Path (Split-Path $PSScriptRoot -Parent) ".harness/context-checkpoint.schema.json"
if (-not (Test-Path $SchemaPath)) {
    Write-Error "Structural schema not found at '$SchemaPath'"
    exit 1
}
$SchemaText = Get-Content $SchemaPath -Raw

# --- Built-in Self-Test Suite ---
if ($SelfTest) {
    Write-Host "[SelfTest] Running built-in checkpoint validator qualification suite..." -ForegroundColor Cyan

    $HEX40 = ("a1b2c3d4" * 5)   # 40 hex chars -- syntactically valid full SHA-1 for structural fixtures
    $HEX40_ALT = ("9f8e7d6c" * 5)

    $validBaseJson = @"
{
  "schema_version": 1,
  "checkpoint": {
    "id": "chk-test-01",
    "created_at": "2026-08-30T09:00:00Z",
    "supersedes": null
  },
  "task": {
    "issue": 1849,
    "pr": 1851,
    "branch": "feat/test",
    "phase": "IMPLEMENT"
  },
  "repository": {
    "base_sha": "$HEX40",
    "head_sha": "$HEX40"
  },
  "authority": {
    "harness_task_id": "SEMANTIC-STABLE-FOUNDATION-SSF-07",
    "references": [
      { "path": "AGENTS.md", "blob_sha": "$HEX40" },
      { "path": "CONSTRAINTS.md", "blob_sha": "$HEX40_ALT" },
      { "path": ".harness/current.task.yaml", "blob_sha": "$HEX40" }
    ]
  },
  "facts": [
    { "id": "fact-1", "category": "PROVEN_FACT", "statement": "Test fact", "provenance": "AGENTS.md:L10" }
  ],
  "owner_decisions": [
    { "id": "dec-1", "category": "OWNER_DECISION", "decision": "Proceed with test", "source": "issue #1849" }
  ],
  "review_findings": [],
  "hypotheses": [],
  "unresolved_questions": [],
  "blockers": [],
  "verification": [
    { "command": "cargo test", "head_sha": "$HEX40", "exit_code": 0, "result": "pass", "status": "SUCCESS" }
  ],
  "completed": [],
  "next_actions": [],
  "fallback": {
    "used": false,
    "authorization_decision_id": null
  },
  "budget": {
    "telemetry_available": false,
    "soft_threshold": null,
    "hard_threshold": null
  }
}
"@

    $passed = 0
    $failed = 0
    $failedNames = [System.Collections.Generic.List[string]]::new()

    function Invoke-Case {
        param (
            [string]$Name,
            [scriptblock]$Errors,
            [bool]$ExpectValid
        )
        $script:__caseErrors = @(& $Errors)
        $count = $script:__caseErrors.Count
        $ok = if ($ExpectValid) { $count -eq 0 } else { $count -gt 0 }
        if ($ok) {
            Write-Host "  [PASS] $Name" -ForegroundColor Green
            $script:passed++
        } else {
            $detail = if ($count -gt 0) { ($script:__caseErrors -join ' | ') } else { "(no errors, but FAIL was expected)" }
            Write-Host "  [FAIL] $Name -- $detail" -ForegroundColor Red
            $script:failed++
            $script:failedNames.Add($Name)
        }
    }

    function New-ValidObj { return $validBaseJson | ConvertFrom-Json }
    function Get-Raw($obj) { return $obj | ConvertTo-Json -Depth 12 }
    function Validate($obj, [string]$label = "Case") {
        return Validate-Checkpoint -RawJson (Get-Raw $obj) -SchemaText $SchemaText -SourceLabel $label
    }

    # --- Structural (schema-backed) ---
    Invoke-Case "Positive: valid complete checkpoint" { Validate (New-ValidObj) } $true

    Invoke-Case "Negative: invalid JSON syntax" {
        Validate-Checkpoint -RawJson "{ not valid json " -SchemaText $SchemaText -SourceLabel "BadJson"
    } $false

    Invoke-Case "Negative: unknown top-level property" {
        $o = New-ValidObj
        $o | Add-Member -MemberType NoteProperty -Name "unexpected_extra" -Value "x"
        Validate $o "UnknownTopLevel"
    } $false

    Invoke-Case "Negative: unknown nested property (task)" {
        $o = New-ValidObj
        $o.task | Add-Member -MemberType NoteProperty -Name "unexpected_nested" -Value "x"
        Validate $o "UnknownNested"
    } $false

    Invoke-Case "Negative: missing required nested field (repository.head_sha)" {
        $raw = ($validBaseJson | ConvertFrom-Json | ConvertTo-Json -Depth 12) | ConvertFrom-Json
        $raw.repository.PSObject.Properties.Remove('head_sha')
        Validate $raw "MissingHeadSha"
    } $false

    Invoke-Case "Negative: invalid created_at (fails supplemental format check, not caught by Test-Json)" {
        $o = New-ValidObj
        $o.checkpoint.created_at = "not-a-date"
        Validate $o "BadCreatedAt"
    } $false

    Invoke-Case "Negative: short repository SHA (7-40 char prefix no longer accepted)" {
        $o = New-ValidObj
        $o.repository.base_sha = "abc1234"
        Validate $o "ShortRepoSha"
    } $false

    Invoke-Case "Negative: short authority blob SHA" {
        $o = New-ValidObj
        $o.authority.references[0].blob_sha = "abc1234"
        Validate $o "ShortBlobSha"
    } $false

    Invoke-Case "Negative: malformed hypotheses entry (missing evidence_needed)" {
        $o = New-ValidObj
        $o.hypotheses = @([pscustomobject]@{ id = "hyp-1"; category = "HYPOTHESIS"; statement = "x" })
        Validate $o "MalformedHypothesis"
    } $false

    Invoke-Case "Negative: malformed blocker entry (missing required_decision)" {
        $o = New-ValidObj
        $o.blockers = @([pscustomobject]@{ id = "blk-1"; category = "BLOCKER"; description = "x" })
        Validate $o "MalformedBlocker"
    } $false

    Invoke-Case "Negative: malformed completed entry (missing evidence)" {
        $o = New-ValidObj
        $o.completed = @([pscustomobject]@{ step = "did a thing" })
        Validate $o "MalformedCompleted"
    } $false

    Invoke-Case "Negative: malformed next_actions entry (missing action)" {
        $o = New-ValidObj
        $o.next_actions = @([pscustomobject]@{ target = "somewhere" })
        Validate $o "MalformedNextAction"
    } $false

    Invoke-Case "Negative: verification entry missing result" {
        $o = New-ValidObj
        $o.verification = @([pscustomobject]@{ command = "cargo test"; head_sha = $HEX40; exit_code = 0; status = "SUCCESS" })
        Validate $o "MissingVerificationResult"
    } $false

    Invoke-Case "Negative: threshold exactly 0 (exclusiveMinimum)" {
        $o = New-ValidObj
        $o.budget.telemetry_available = $true
        $o.budget.soft_threshold = 0.0
        $o.budget.hard_threshold = 0.8
        Validate $o "ThresholdZero"
    } $false

    Invoke-Case "Negative: threshold exactly 1 (exclusiveMaximum)" {
        $o = New-ValidObj
        $o.budget.telemetry_available = $true
        $o.budget.soft_threshold = 0.5
        $o.budget.hard_threshold = 1.0
        Validate $o "ThresholdOne"
    } $false

    # --- Semantic layer: mandatory authority anchors ---
    Invoke-Case "Negative: missing AGENTS.md authority anchor" {
        $o = New-ValidObj
        $o.authority.references = @($o.authority.references | Where-Object { $_.path -ne "AGENTS.md" })
        Validate $o "MissingAgentsAnchor"
    } $false

    Invoke-Case "Negative: missing CONSTRAINTS.md authority anchor" {
        $o = New-ValidObj
        $o.authority.references = @($o.authority.references | Where-Object { $_.path -ne "CONSTRAINTS.md" })
        Validate $o "MissingConstraintsAnchor"
    } $false

    Invoke-Case "Negative: missing Harness authority anchor" {
        $o = New-ValidObj
        $o.authority.references = @($o.authority.references | Where-Object { $_.path -ne ".harness/current.task.yaml" })
        Validate $o "MissingHarnessAnchor"
    } $false

    Invoke-Case "Negative: duplicate authority path" {
        $o = New-ValidObj
        $dup = [pscustomobject]@{ path = "AGENTS.md"; blob_sha = $HEX40_ALT }
        $o.authority.references = @($o.authority.references) + $dup
        Validate $o "DuplicateAuthorityPath"
    } $false

    # --- Semantic layer: global ID uniqueness ---
    Invoke-Case "Negative: duplicate id across typed categories (facts vs hypotheses)" {
        $o = New-ValidObj
        $o.hypotheses = @([pscustomobject]@{ id = "fact-1"; category = "HYPOTHESIS"; statement = "x"; evidence_needed = "y" })
        Validate $o "DuplicateGlobalId"
    } $false

    # --- Semantic layer: fallback referential integrity ---
    Invoke-Case "Negative: fallback used=true with null authorization_decision_id" {
        $o = New-ValidObj
        $o.fallback.used = $true
        $o.fallback.authorization_decision_id = $null
        Validate $o "FallbackNullAuth"
    } $false

    Invoke-Case "Negative: fallback used=true with unknown owner decision id" {
        $o = New-ValidObj
        $o.fallback.used = $true
        $o.fallback.authorization_decision_id = "decision-does-not-exist"
        Validate $o "FallbackUnknownDecision"
    } $false

    Invoke-Case "Negative: fallback used=true referencing a non-owner entity (a fact id)" {
        $o = New-ValidObj
        $o.fallback.used = $true
        $o.fallback.authorization_decision_id = "fact-1"
        Validate $o "FallbackNonOwnerEntity"
    } $false

    Invoke-Case "Negative: fallback used=false with non-null authorization_decision_id" {
        $o = New-ValidObj
        $o.fallback.used = $false
        $o.fallback.authorization_decision_id = "dec-1"
        Validate $o "FallbackUsedFalseWithAuth"
    } $false

    Invoke-Case "Positive: valid owner-authorized fallback reference" {
        $o = New-ValidObj
        $o.fallback.used = $true
        $o.fallback.authorization_decision_id = "dec-1"
        Validate $o "FallbackValidReference"
    } $true

    # --- Semantic layer: verification coherence ---
    Invoke-Case "Negative: verification SUCCESS with non-zero exit_code" {
        $o = New-ValidObj
        $o.verification[0].exit_code = 1
        $o.verification[0].status = "SUCCESS"
        Validate $o "InvalidVerifExit"
    } $false

    # --- Semantic layer: budget telemetry-mode coherence ---
    Invoke-Case "Negative: telemetry_available=false with fabricated thresholds" {
        $o = New-ValidObj
        $o.budget.telemetry_available = $false
        $o.budget.soft_threshold = 0.65
        $o.budget.hard_threshold = 0.80
        Validate $o "TelemetryFalseFabricatedThresholds"
    } $false

    Invoke-Case "Negative: telemetry_available=true with a missing threshold" {
        $o = New-ValidObj
        $o.budget.telemetry_available = $true
        $o.budget.soft_threshold = 0.65
        $o.budget.hard_threshold = $null
        Validate $o "TelemetryTrueMissingThreshold"
    } $false

    Invoke-Case "Negative: soft_threshold >= hard_threshold" {
        $o = New-ValidObj
        $o.budget.telemetry_available = $true
        $o.budget.soft_threshold = 0.85
        $o.budget.hard_threshold = 0.80
        Validate $o "InvertedThresholds"
    } $false

    Invoke-Case "Positive: valid telemetry thresholds" {
        $o = New-ValidObj
        $o.budget.telemetry_available = $true
        $o.budget.soft_threshold = 0.65
        $o.budget.hard_threshold = 0.80
        Validate $o "ValidTelemetryThresholds"
    } $true

    # --- Repository / Harness staleness (fail-closed) ---
    Invoke-Case "Negative: HEAD mismatch against live repository" {
        $o = New-ValidObj
        $o.repository.head_sha = $HEX40_ALT
        Test-AgainstCurrentRepository -Json $o -SourceLabel "HeadMismatch"
    } $false

    Invoke-Case "Negative: Harness task id mismatch against live repository" {
        $o = New-ValidObj
        $o.repository.head_sha = (git rev-parse HEAD).Trim()
        $o.authority.harness_task_id = "STALE-NONEXISTENT-TASK-ID"
        Test-AgainstCurrentRepository -Json $o -SourceLabel "StaleTaskId"
    } $false

    Invoke-Case "Negative: Harness file missing (fail-closed, synthetic path only)" {
        $o = New-ValidObj
        Test-AgainstCurrentRepository -Json $o -SourceLabel "HarnessMissing" -HarnessPath ".harness/__selftest_missing__.yaml"
    } $false

    $tmpHarness = [System.IO.Path]::GetTempFileName()
    try {
        Set-Content -Path $tmpHarness -Value "this is not valid harness yaml content at all" -NoNewline
        Invoke-Case "Negative: Harness file malformed / task id unparseable (fail-closed, temp file only)" {
            $o = New-ValidObj
            Test-AgainstCurrentRepository -Json $o -SourceLabel "HarnessMalformed" -HarnessPath $tmpHarness
        } $false
    } finally {
        Remove-Item -Path $tmpHarness -Force -ErrorAction SilentlyContinue
    }

    # --- Get-HarnessTaskId pure-helper unit tests (synthetic content; real Harness file never touched) ---
    Invoke-Case "Get-HarnessTaskId: valid synthetic content extracts id" {
        $id = Get-HarnessTaskId "task:`n  id: SYNTHETIC-TASK-42`n  title: `"x`"`n"
        if ($id -ne "SYNTHETIC-TASK-42") { @("expected SYNTHETIC-TASK-42, got '$id'") } else { @() }
    } $true

    Invoke-Case "Get-HarnessTaskId: malformed synthetic content returns null" {
        $id = Get-HarnessTaskId "this file has no id field at all"
        if ($null -ne $id) { @("expected null, got '$id'") } else { @() }
    } $true

    Write-Host ""
    if ($failed -eq 0) {
        Write-Host "[SelfTest] All $passed qualification checks passed successfully." -ForegroundColor Green
    } else {
        Write-Host "[SelfTest] $passed passed, $failed FAILED: $($failedNames -join '; ')" -ForegroundColor Red
    }
    Write-Host ""

    if ($failed -gt 0) { exit 1 }
    if ([string]::IsNullOrWhiteSpace($Checkpoint)) {
        exit 0
    }
}

# --- File Validation ---
if (-not [string]::IsNullOrWhiteSpace($Checkpoint)) {
    if (-not (Test-Path $Checkpoint)) {
        Write-Error "Checkpoint file not found: '$Checkpoint'"
        exit 1
    }

    Write-Host "[Validator] Inspecting checkpoint: '$Checkpoint'..." -ForegroundColor Cyan

    $rawContent = Get-Content $Checkpoint -Raw
    $errors = Validate-Checkpoint -RawJson $rawContent -SchemaText $SchemaText -SourceLabel (Split-Path $Checkpoint -Leaf)

    if ($AgainstCurrentRepo) {
        try {
            $json = $rawContent | ConvertFrom-Json -ErrorAction Stop
            $repoErrors = Test-AgainstCurrentRepository -Json $json -SourceLabel (Split-Path $Checkpoint -Leaf)
            foreach ($e in @($repoErrors)) { $errors.Add($e) }
        } catch {
            $errors.Add("Cannot run -AgainstCurrentRepo checks: checkpoint JSON is invalid ($($_.Exception.Message))")
        }
    }

    if ($errors.Count -gt 0) {
        Write-Host "`n[FAIL] Checkpoint validation failed with $($errors.Count) error(s):" -ForegroundColor Red
        foreach ($err in $errors) {
            Write-Host "  - $err" -ForegroundColor Red
        }
        exit 1
    }

    Write-Host "`n[OK] Checkpoint '$Checkpoint' is structurally valid (schema-compliant) and passes semantic invariants." -ForegroundColor Green
    if ($AgainstCurrentRepo) {
        Write-Host "[OK] Checkpoint aligns with current repository HEAD and authority snapshot." -ForegroundColor Green
    }
    exit 0
}

if (-not $SelfTest -and [string]::IsNullOrWhiteSpace($Checkpoint)) {
    Write-Host "Usage: pwsh -File scripts/context_checkpoint_check.ps1 -Checkpoint <path> [-AgainstCurrentRepo] [-SelfTest]"
    exit 1
}
