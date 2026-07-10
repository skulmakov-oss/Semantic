# PCC Stack External Diff Sampling Retry Plan

## Status

Result: `PASS-WITH-WARNINGS`

This is `PCC-PORT-1B`.

This document is audit-only.

No files were ported.
No cherry-pick was performed.
No merge was performed.
No patch was applied.
No implementation code was changed.
No tests/examples/7hell files were changed.

## Purpose

Explain that `PCC-PORT-1` was blocked because external PR diff evidence was
unavailable through `gh` / GitHub API connectivity in the previous retry.

This document defines safe alternative evidence sources for retrying external
diff sampling.

## Source-of-truth local repo

- path: `C:\Users\said3\Desktop\EXOcode\Semantic_phase1_prom_ui`
- branch: `main`
- HEAD: `cbb54af2518943950d3be5d0ed66520a762d1a34`
- main == origin/main: `yes`
- baseline 7hell: `PASS`
- bridge audit: `FAIL`
- selective port plan: `PASS-WITH-WARNINGS`
- external inventory: `PASS-WITH-WARNINGS`
- external diff sampling: `BLOCKED`
- dirty tree: untracked local files are present
- unexpected untracked files:
  - `docs/language/semantic_sugar_track_rfc.md`
  - `docs/roadmap/pcc/local_practical_core_readiness_audit.md`
  - `docs/roadmap/pcc/local_repo_mismatch_audit.md`
  - `docs/roadmap/pcc/pcc_stack_bridge_audit.md`
  - `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md`
  - `docs/roadmap/pcc/pcc_stack_external_inventory.md`
  - `docs/roadmap/pcc/pcc_stack_selective_port_plan.md`

## External PR reference

- PR: `#1301`
- repo: `skulmakov-oss/Semantic`
- title: `docs/test(pcc): close practical core phase and sync CTF follow-ups`
- state: `MERGED`
- merge SHA: `736b8bb066ea68e7e6d2e79ff300f77117c51561`
- head SHA: `987857f961a2e17141490925f135849a0f6f7ef8`
- changed files: `119`
- commits: `7`
- local availability of merge SHA: `absent`

## Previous sampling failure

Record:

- `gh pr view` result: `PASS` in this retry window
- `gh pr diff --name-only` result: `PASS` in this retry window
- `gh pr diff --patch` result: `PASS` in this retry window
- failure reason: prior retry window hit GitHub API / connectivity failure
- conclusion: `PCC-PORT-1` was blocked until a usable file list / patch source
  became available

Expected conclusion:

```text
PCC-PORT-1 is BLOCKED.
No safe slice can be selected without external diff evidence.
```

## Retry evidence sources

| Source | Availability | Expected evidence | Risk | Decision |
|---|---:|---|---|---|
| GitHub UI file list | unknown | exact changed paths | manual copy risk | allowed |
| GitHub API / connector | currently available in this retry window | file list + patches | access limits | preferred if available |
| GitHub CLI retry | available in this retry window | file list + patch | network failure | retry later if it fails again |
| Saved patch / diff file | not present locally | complete diff | stale export risk | allowed if explicitly provided |
| Old workspace read-only | available as a local repository path | path list / old files | source-of-truth confusion | read-only only |

## Required evidence before PCC-PORT-2

Before moving to `PCC-PORT-2`, require:

- exact PR `#1301` file list;
- representative patch / content samples from each major layer;
- confirmation that the sampled files are from PR `#1301`;
- classification of each sampled file;
- statement that no files were applied locally.

## Sampling minimum for retry success

A successful retry must sample at least:

### Linguist templates

```text
docs/roadmap/issues/issue_linguist_semantic_readiness.md
docs/roadmap/issues/issue_linguist_semantic_usage_evidence.md
```

### Candidate probes

```text
examples/pcc_candidates/loop_control_flow_probe/AUDIT.md
examples/pcc_candidates/option_result_control_flow/AUDIT.md
```

### Canonical examples

```text
examples/canonical/match_control_flow/src/main.sm
examples/canonical/text_core/src/main.sm
examples/canonical/collections_core/src/main.sm
```

### Negative fixtures

```text
tests/fixtures/pcc/control_flow/fail/if_quad_condition.sm
tests/fixtures/pcc/text/fail/text_plus_i32.sm
tests/fixtures/pcc/stdlib/fail/print_i32.sm
```

### Negative harnesses

```text
tests/pcc_control_flow_negative.rs
tests/pcc_text_negative.rs
```

### 7hell wiring

```text
tools/7hell/run.ps1
tools/7hell/run.sh
```

### PCC / CTF sync docs

```text
docs/roadmap/pcc/pcc_ctf_sync_checkpoint.md
docs/roadmap/pcc/pcc_ctf_sync_closeout.md
```

### Post-UI docs

```text
docs/roadmap/post_ui/r12_ui_renderer_boundary.md
docs/roadmap/post_ui/r12_ui_windowing_boundary.md
```

## Decision rules after retry

If evidence is complete:

```text
Proceed to PCC-PORT-2: first small safe slice proposal.
```

If evidence is partial:

```text
Repeat sampling or restrict PCC-PORT-2 to layers with sufficient evidence.
```

If evidence is unavailable:

```text
Remain BLOCKED.
Do not select a safe slice.
```

## Do-not-port-as-monolith rule

PR `#1301` must not be cherry-picked, merged, or patch-applied as a single
unit.

## First likely safe slice hypothesis

This is only a hypothesis, not approval:

```text
Linguist readiness templates may be the first safe slice,
because they are mostly independent from the missing PCC Practical Core stack.
```

But this cannot proceed until diff / content evidence confirms:

- no submit-readiness claim;
- usage-evidence blocker remains explicit;
- no dependency on missing PCC closeouts.

## Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Selecting a safe slice without diff evidence | high | Keep PCC-PORT-1 BLOCKED until evidence exists. |
| Treating old workspace as source-of-truth | high | Read-only evidence only. |
| Applying saved patch accidentally | high | Patch is evidence only, never applied. |
| Manual GitHub UI copy errors | medium | Record source and timestamp. |
| Linguist templates overclaim readiness | medium | Verify content before any future port. |

## Recommended next step

Recommended next step:

```text
Obtain PR #1301 changed-file list through GitHub UI / API / gh retry.
```

Then rerun `PCC-PORT-1` as actual sampling, or create:

```text
PCC-PORT-1C: external diff sampling with captured file list
```

## Final verdict

Use one of:

- `PASS-WITH-WARNINGS`
- `BLOCKED`

Expected:

```text
PASS-WITH-WARNINGS
```

if the retry plan is complete and clearly prevents unsafe transfer.

Use:

```text
BLOCKED
```

only if even a retry plan cannot be responsibly written.
