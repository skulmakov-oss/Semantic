# scripts/context_checkpoint_check.ps1
# Deterministic, dependency-free validator for Semantic context checkpoints

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

function Test-ShaFormat {
    param ([string]$Sha)
    if ([string]::IsNullOrWhiteSpace($Sha)) { return $false }
    return ($Sha -match '^[0-9a-fA-F]{7,40}$')
}

function Validate-CheckpointObject {
    param (
        [psobject]$Json,
        [string]$SourceLabel = "Checkpoint"
    )

    $errors = [System.Collections.Generic.List[string]]::new()

    # 1. Schema version
    if ($null -eq $Json.schema_version -or $Json.schema_version -ne 1) {
        $errors.Add("[$SourceLabel] Unsupported or missing schema_version: $($Json.schema_version) (expected 1)")
    }

    # 2. Checkpoint metadata
    if ($null -eq $Json.checkpoint) {
        $errors.Add("[$SourceLabel] Missing 'checkpoint' metadata section")
    } else {
        if (-not ($Json.checkpoint.id -match '^chk-[a-zA-Z0-9_-]+$')) {
            $errors.Add("[$SourceLabel] Invalid or missing checkpoint.id: '$($Json.checkpoint.id)' (expected pattern '^chk-[a-zA-Z0-9_-]+$')")
        }
        if ([string]::IsNullOrWhiteSpace($Json.checkpoint.created_at)) {
            $errors.Add("[$SourceLabel] Missing checkpoint.created_at timestamp")
        }
    }

    # 3. Task metadata
    if ($null -eq $Json.task) {
        $errors.Add("[$SourceLabel] Missing 'task' section")
    } else {
        if ($null -eq $Json.task.issue -or $Json.task.issue -lt 1) {
            $errors.Add("[$SourceLabel] Invalid or missing task.issue: $($Json.task.issue)")
        }
        if ([string]::IsNullOrWhiteSpace($Json.task.branch)) {
            $errors.Add("[$SourceLabel] Missing task.branch")
        }
        $validPhases = @("UNDERSTAND", "AUTHORIZE", "IMPLEMENT", "VERIFY", "REVIEW_CONVERGENCE", "MERGE_HANDOFF", "COMPLETED")
        if (-not $validPhases.Contains($Json.task.phase)) {
            $errors.Add("[$SourceLabel] Invalid task.phase: '$($Json.task.phase)' (expected one of $($validPhases -join ', '))")
        }
    }

    # 4. Repository metadata
    if ($null -eq $Json.repository) {
        $errors.Add("[$SourceLabel] Missing 'repository' section")
    } else {
        if (-not (Test-ShaFormat $Json.repository.base_sha)) {
            $errors.Add("[$SourceLabel] Malformed or missing repository.base_sha: '$($Json.repository.base_sha)'")
        }
        if (-not (Test-ShaFormat $Json.repository.head_sha)) {
            $errors.Add("[$SourceLabel] Malformed or missing repository.head_sha: '$($Json.repository.head_sha)'")
        }
    }

    # 5. Authority metadata
    if ($null -eq $Json.authority) {
        $errors.Add("[$SourceLabel] Missing 'authority' section")
    } else {
        if ([string]::IsNullOrWhiteSpace($Json.authority.harness_task_id)) {
            $errors.Add("[$SourceLabel] Missing authority.harness_task_id")
        }
        if ($null -eq $Json.authority.references -or $Json.authority.references.Count -eq 0) {
            $errors.Add("[$SourceLabel] authority.references must contain at least one authority file reference")
        } else {
            foreach ($ref in $Json.authority.references) {
                if ([string]::IsNullOrWhiteSpace($ref.path)) {
                    $errors.Add("[$SourceLabel] authority reference contains empty path")
                }
                if (-not (Test-ShaFormat $ref.blob_sha)) {
                    $errors.Add("[$SourceLabel] authority reference '$($ref.path)' contains malformed blob_sha: '$($ref.blob_sha)'")
                }
            }
        }
    }

    # 6. ID Uniqueness across typed entities
    $seenIds = [System.Collections.Generic.HashSet[string]]::new()
    $checkId = {
        param ($item, $section)
        if ($null -ne $item.id) {
            if ([string]::IsNullOrWhiteSpace($item.id)) {
                $errors.Add("[$SourceLabel] $section contains an empty id")
            } elseif (-not $seenIds.Add($item.id)) {
                $errors.Add("[$SourceLabel] Duplicate entry id: '$($item.id)' in $section")
            }
        }
    }

    # 7. Facts
    if ($null -eq $Json.facts) {
        $errors.Add("[$SourceLabel] Missing 'facts' array")
    } else {
        foreach ($f in $Json.facts) {
            & $checkId $f "facts"
            if ($f.category -ne "PROVEN_FACT") {
                $errors.Add("[$SourceLabel] Invalid category for fact '$($f.id)': '$($f.category)' (expected 'PROVEN_FACT')")
            }
            if ([string]::IsNullOrWhiteSpace($f.statement)) {
                $errors.Add("[$SourceLabel] Fact '$($f.id)' has empty statement")
            }
            if ([string]::IsNullOrWhiteSpace($f.provenance)) {
                $errors.Add("[$SourceLabel] Fact '$($f.id)' has missing provenance")
            }
        }
    }

    # 8. Owner Decisions
    if ($null -eq $Json.owner_decisions) {
        $errors.Add("[$SourceLabel] Missing 'owner_decisions' array")
    } else {
        foreach ($od in $Json.owner_decisions) {
            & $checkId $od "owner_decisions"
            if ($od.category -ne "OWNER_DECISION") {
                $errors.Add("[$SourceLabel] Invalid category for owner_decision '$($od.id)': '$($od.category)' (expected 'OWNER_DECISION')")
            }
            if ([string]::IsNullOrWhiteSpace($od.decision)) {
                $errors.Add("[$SourceLabel] Owner decision '$($od.id)' has empty decision text")
            }
            if ([string]::IsNullOrWhiteSpace($od.source)) {
                $errors.Add("[$SourceLabel] Owner decision '$($od.id)' has missing source")
            }
        }
    }

    # 9. Review Findings
    if ($null -eq $Json.review_findings) {
        $errors.Add("[$SourceLabel] Missing 'review_findings' array")
    } else {
        $validReviewStatus = @("ACTIVE", "ADDRESSED", "ACCEPTED", "REJECTED")
        foreach ($rf in $Json.review_findings) {
            & $checkId $rf "review_findings"
            if ($rf.category -ne "REVIEWER_CLAIM") {
                $errors.Add("[$SourceLabel] Invalid category for review finding '$($rf.id)': '$($rf.category)' (expected 'REVIEWER_CLAIM')")
            }
            if ([string]::IsNullOrWhiteSpace($rf.thread_id)) {
                $errors.Add("[$SourceLabel] Review finding '$($rf.id)' missing thread_id")
            }
            if ([string]::IsNullOrWhiteSpace($rf.claim)) {
                $errors.Add("[$SourceLabel] Review finding '$($rf.id)' missing claim description")
            }
            if (-not $validReviewStatus.Contains($rf.status)) {
                $errors.Add("[$SourceLabel] Review finding '$($rf.id)' has invalid status: '$($rf.status)'")
            }
        }
    }

    # 10. Hypotheses, Unresolved Questions, Blockers
    if ($null -eq $Json.hypotheses) { $errors.Add("[$SourceLabel] Missing 'hypotheses' array") }
    if ($null -eq $Json.unresolved_questions) { $errors.Add("[$SourceLabel] Missing 'unresolved_questions' array") }
    if ($null -eq $Json.blockers) { $errors.Add("[$SourceLabel] Missing 'blockers' array") }

    # 11. Verification state
    if ($null -eq $Json.verification) {
        $errors.Add("[$SourceLabel] Missing 'verification' array")
    } else {
        $validVerifStatus = @("SUCCESS", "FAILED", "PENDING")
        foreach ($v in $Json.verification) {
            if ([string]::IsNullOrWhiteSpace($v.command)) {
                $errors.Add("[$SourceLabel] Verification entry missing command")
            }
            if (-not (Test-ShaFormat $v.head_sha)) {
                $errors.Add("[$SourceLabel] Verification entry '$($v.command)' has malformed head_sha: '$($v.head_sha)'")
            }
            if ($null -eq $v.exit_code) {
                $errors.Add("[$SourceLabel] Verification entry '$($v.command)' missing exit_code")
            }
            if (-not $validVerifStatus.Contains($v.status)) {
                $errors.Add("[$SourceLabel] Verification entry '$($v.command)' has invalid status: '$($v.status)'")
            }
            if ($v.status -eq "SUCCESS" -and $v.exit_code -ne 0) {
                $errors.Add("[$SourceLabel] Verification entry '$($v.command)' marked SUCCESS with non-zero exit_code: $($v.exit_code)")
            }
        }
    }

    # 12. Fallback authorization integrity
    if ($null -eq $Json.fallback) {
        $errors.Add("[$SourceLabel] Missing 'fallback' section")
    } else {
        if ($Json.fallback.used -eq $true -and [string]::IsNullOrWhiteSpace($Json.fallback.authorization)) {
            $errors.Add("[$SourceLabel] fallback.used is true but fallback.authorization is missing or empty")
        }
    }

    # 13. Budget threshold integrity
    if ($null -eq $Json.budget) {
        $errors.Add("[$SourceLabel] Missing 'budget' section")
    } else {
        if ($null -ne $Json.budget.soft_threshold -and $null -ne $Json.budget.hard_threshold) {
            if ($Json.budget.soft_threshold -ge $Json.budget.hard_threshold) {
                $errors.Add("[$SourceLabel] budget.soft_threshold ($($Json.budget.soft_threshold)) must be strictly less than budget.hard_threshold ($($Json.budget.hard_threshold))")
            }
            if ($Json.budget.soft_threshold -le 0.0 -or $Json.budget.soft_threshold -ge 1.0) {
                $errors.Add("[$SourceLabel] budget.soft_threshold ($($Json.budget.soft_threshold)) must be between 0.0 and 1.0 exclusive")
            }
            if ($Json.budget.hard_threshold -le 0.0 -or $Json.budget.hard_threshold -ge 1.0) {
                $errors.Add("[$SourceLabel] budget.hard_threshold ($($Json.budget.hard_threshold)) must be between 0.0 and 1.0 exclusive")
            }
        }
    }

    return $errors
}

function Test-AgainstCurrentRepository {
    param (
        [psobject]$Json,
        [string]$SourceLabel = "Checkpoint"
    )

    $staleness = [System.Collections.Generic.List[string]]::new()

    # 1. Live HEAD check
    $currentHead = (git rev-parse HEAD).Trim()
    $chkHead = $Json.repository.head_sha
    if ($currentHead -notlike "$chkHead*" -and $chkHead -notlike "$currentHead*") {
        $staleness.Add("[$SourceLabel] HEAD mismatch: Checkpoint head_sha '$chkHead' != Live git HEAD '$currentHead'")
    }

    # 2. Active Harness task check
    if (Test-Path ".harness/current.task.yaml") {
        $harnessContent = Get-Content ".harness/current.task.yaml" -Raw
        if ($harnessContent -match 'id:\s*([^\r\n]+)') {
            $liveTaskId = $Matches[1].Trim()
            if ($Json.authority.harness_task_id -ne $liveTaskId) {
                $staleness.Add("[$SourceLabel] Harness Task ID mismatch: Checkpoint '$($Json.authority.harness_task_id)' != Live '$liveTaskId'")
            }
        }
    }

    # 3. Authority file blob hash checks
    if ($null -ne $Json.authority.references) {
        foreach ($ref in $Json.authority.references) {
            $path = $ref.path
            if (Test-Path $path) {
                $liveBlobSha = (git hash-object $path).Trim()
                if ($ref.blob_sha -notlike "$liveBlobSha*" -and $liveBlobSha -notlike "$($ref.blob_sha)*") {
                    $staleness.Add("[$SourceLabel] Authority hash mismatch for '$path': Checkpoint blob_sha '$($ref.blob_sha)' != Live '$liveBlobSha'")
                }
            } else {
                $staleness.Add("[$SourceLabel] Authority file '$path' does not exist in current working tree")
            }
        }
    }

    return $staleness
}

# --- Built-in Self-Test Suite ---
if ($SelfTest) {
    Write-Host "[SelfTest] Running built-in checkpoint validator qualification suite..." -ForegroundColor Cyan

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
    "pr": null,
    "branch": "feat/test",
    "phase": "IMPLEMENT"
  },
  "repository": {
    "base_sha": "15bd096df652764231e4203b8e964b49b0b5ad38",
    "head_sha": "15bd096df652764231e4203b8e964b49b0b5ad38"
  },
  "authority": {
    "harness_task_id": "SEMANTIC-STABLE-FOUNDATION-SSF-07",
    "references": [
      {
        "path": "AGENTS.md",
        "blob_sha": "abcdef1234567890abcdef1234567890abcdef12"
      }
    ]
  },
  "facts": [
    {
      "id": "fact-1",
      "category": "PROVEN_FACT",
      "statement": "Test fact",
      "provenance": "AGENTS.md:L10"
    }
  ],
  "owner_decisions": [
    {
      "id": "dec-1",
      "category": "OWNER_DECISION",
      "decision": "Proceed with test",
      "source": "issue #1849"
    }
  ],
  "review_findings": [],
  "hypotheses": [],
  "unresolved_questions": [],
  "blockers": [],
  "verification": [
    {
      "command": "cargo test",
      "head_sha": "15bd096df652764231e4203b8e964b49b0b5ad38",
      "exit_code": 0,
      "result": "pass",
      "status": "SUCCESS"
    }
  ],
  "completed": [],
  "next_actions": [],
  "fallback": {
    "used": false,
    "authorization": null
  },
  "budget": {
    "telemetry_available": false,
    "soft_threshold": 0.65,
    "hard_threshold": 0.80
  }
}
"@

    # Test 1: Valid checkpoint -> PASS
    $validObj = $validBaseJson | ConvertFrom-Json
    $errors = Validate-CheckpointObject $validObj "ValidCase"
    if ($errors.Count -ne 0) {
        throw "SelfTest Failed: Valid case reported unexpected errors: $($errors -join '; ')"
    }
    Write-Host "  [PASS] Positive Valid Checkpoint" -ForegroundColor Green

    # Test 2: Missing head_sha -> FAIL
    $badHead = $validBaseJson | ConvertFrom-Json
    $badHead.repository.head_sha = ""
    $errors = Validate-CheckpointObject $badHead "MissingHead"
    if ($errors.Count -eq 0) { throw "SelfTest Failed: Missing head_sha did not produce validation error" }
    Write-Host "  [PASS] Negative Missing head_sha detected" -ForegroundColor Green

    # Test 3: Malformed SHA -> FAIL
    $badSha = $validBaseJson | ConvertFrom-Json
    $badSha.repository.base_sha = "not-a-valid-hex-sha-!!!"
    $errors = Validate-CheckpointObject $badSha "MalformedSha"
    if ($errors.Count -eq 0) { throw "SelfTest Failed: Malformed SHA did not produce validation error" }
    Write-Host "  [PASS] Negative Malformed SHA detected" -ForegroundColor Green

    # Test 4: Unknown classification category -> FAIL
    $badCat = $validBaseJson | ConvertFrom-Json
    $badCat.facts[0].category = "UNKNOWN_CATEGORY"
    $errors = Validate-CheckpointObject $badCat "BadCategory"
    if ($errors.Count -eq 0) { throw "SelfTest Failed: Unknown category did not produce validation error" }
    Write-Host "  [PASS] Negative Unknown Category detected" -ForegroundColor Green

    # Test 5: Fallback used without authorization -> FAIL
    $badFallback = $validBaseJson | ConvertFrom-Json
    $badFallback.fallback.used = $true
    $badFallback.fallback.authorization = $null
    $errors = Validate-CheckpointObject $badFallback "MissingFallbackAuth"
    if ($errors.Count -eq 0) { throw "SelfTest Failed: Missing fallback authorization did not produce error" }
    Write-Host "  [PASS] Negative Fallback without authorization detected" -ForegroundColor Green

    # Test 6: Verification SUCCESS with exit_code 1 -> FAIL
    $badVerif = $validBaseJson | ConvertFrom-Json
    $badVerif.verification[0].exit_code = 1
    $badVerif.verification[0].status = "SUCCESS"
    $errors = Validate-CheckpointObject $badVerif "InvalidVerifExit"
    if ($errors.Count -eq 0) { throw "SelfTest Failed: SUCCESS status with non-zero exit code did not produce error" }
    Write-Host "  [PASS] Negative Inconsistent verification exit_code detected" -ForegroundColor Green

    # Test 7: Soft threshold >= Hard threshold -> FAIL
    $badBudget = $validBaseJson | ConvertFrom-Json
    $badBudget.budget.soft_threshold = 0.85
    $badBudget.budget.hard_threshold = 0.80
    $errors = Validate-CheckpointObject $badBudget "InvertedThresholds"
    if ($errors.Count -eq 0) { throw "SelfTest Failed: Soft >= Hard budget threshold did not produce error" }
    Write-Host "  [PASS] Negative Inverted budget thresholds detected" -ForegroundColor Green

    # Test 8: Live repository authority mismatch detection
    $staleObj = $validBaseJson | ConvertFrom-Json
    $staleObj.authority.harness_task_id = "STALE-NONEXISTENT-TASK-ID"
    $staleErrors = Test-AgainstCurrentRepository $staleObj "StaleTaskTest"
    if ($staleErrors.Count -eq 0) { throw "SelfTest Failed: Mismatched harness task ID did not produce staleness error" }
    Write-Host "  [PASS] Staleness detection against live repository verified" -ForegroundColor Green

    Write-Host "`n[SelfTest] All 8 qualification checks passed successfully.`n" -ForegroundColor Green
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

    try {
        $rawContent = Get-Content $Checkpoint -Raw
        $json = $rawContent | ConvertFrom-Json
    } catch {
        Write-Error "Invalid JSON in checkpoint file '$Checkpoint': $_"
        exit 1
    }

    $errors = Validate-CheckpointObject $json (Split-Path $Checkpoint -Leaf)

    if ($AgainstCurrentRepo) {
        $repoErrors = Test-AgainstCurrentRepository $json (Split-Path $Checkpoint -Leaf)
        $errors.AddRange($repoErrors)
    }

    if ($errors.Count -gt 0) {
        Write-Host "`n[FAIL] Checkpoint validation failed with $($errors.Count) error(s):" -ForegroundColor Red
        foreach ($err in $errors) {
            Write-Host "  - $err" -ForegroundColor Red
        }
        exit 1
    }

    Write-Host "`n[OK] Checkpoint '$Checkpoint' is structurally valid." -ForegroundColor Green
    if ($AgainstCurrentRepo) {
        Write-Host "[OK] Checkpoint aligns with current repository HEAD and authority snapshot." -ForegroundColor Green
    }
    exit 0
}

if (-not $SelfTest -and [string]::IsNullOrWhiteSpace($Checkpoint)) {
    Write-Host "Usage: pwsh -File scripts/context_checkpoint_check.ps1 -Checkpoint <path> [-AgainstCurrentRepo] [-SelfTest]"
    exit 1
}
