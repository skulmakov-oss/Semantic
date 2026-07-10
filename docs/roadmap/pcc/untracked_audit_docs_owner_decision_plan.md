# Untracked Audit Docs Owner Decision Plan

## Status

Result: PASS-WITH-WARNINGS

This is an owner-decision plan.

No files were deleted.
No files were moved.
No files were staged.
No files were committed.
No git clean was run.
No code/tests/examples/7hell files were changed.

## Purpose

Prepare an owner decision for remaining untracked audit docs after the
`#1302`, `#1303`, and `#1304` merge cycle.

## Source repo state

- branch: `codex/pcc-bridge-port-audit-trail`
- HEAD: `7b838f9e8c7035ffc317c0aec3296104033888a6`
- origin/main: `a708a4ab36d86adadfa226dc07bd5dc79287326e`
- main == origin/main: `yes`
- dirty tree summary: 11 untracked audit docs remain
- untracked file count: `11`

## Already preserved evidence trail

The `#1304` evidence trail is already tracked:

- `docs/roadmap/pcc/external/pr_1301_changed_files.txt`
- `docs/roadmap/pcc/housekeeping_audit_docs_classification.md`
- `docs/roadmap/pcc/pcc_stack_bridge_audit.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling_captured.md`
- `docs/roadmap/pcc/pcc_stack_external_inventory.md`
- `docs/roadmap/pcc/pcc_stack_linguist_wording_audit.md`

This owner-decision plan is not part of that tracked evidence trail.

## Remaining untracked audit docs

| File | Prior disposition | Owner decision label | Reason | Proposed future action |
|---|---|---|---|---|
| `docs/roadmap/pcc/housekeeping_track_candidate_commit_plan.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | OWNER-DELETE-CANDIDATE | Obsolete planning artifact superseded by tracked evidence and later disposition review. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/local_holdbacks_cleanup_plan.md` | KEEP-LOCAL | OWNER-KEEP-LOCAL | Local sanitation note showing the original holdback paths are absent in this checkout. | Keep local. |
| `docs/roadmap/pcc/local_holdbacks_sanitation_audit.md` | KEEP-LOCAL | OWNER-KEEP-LOCAL | Local sanitation snapshot for the absent holdback set and temporary investigation context. | Keep local. |
| `docs/roadmap/pcc/local_practical_core_readiness_audit.md` | KEEP-LOCAL | OWNER-KEEP-LOCAL | Early mismatch/readiness audit that preserves investigation history. | Keep local. |
| `docs/roadmap/pcc/local_repo_mismatch_audit.md` | KEEP-LOCAL | OWNER-KEEP-LOCAL | Repo mismatch audit that preserves investigation history. | Keep local. |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | OWNER-DELETE-CANDIDATE | Blocked sampling attempt superseded by retry and captured sampling evidence. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | OWNER-DELETE-CANDIDATE | Retry plan superseded by the captured sampling evidence. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | OWNER-DELETE-CANDIDATE | Safe-slice proposal superseded by the actual Linguist slice port and merge. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | OWNER-DELETE-CANDIDATE | Pre-port plan superseded by the actual 5-file Linguist port and merge. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/pcc_stack_selective_port_plan.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | OWNER-DELETE-CANDIDATE | Selective-port plan superseded by the selected safe slice and later port evidence. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/untracked_audit_docs_disposition_plan.md` | NEEDS-REVIEW | NEEDS-OWNER-REVIEW | This is the current working disposition artifact and should be reviewed before any further classification action. | No immediate action; owner reviews this plan first. |

## Recommended owner decision

Choose one:

### Option A - Keep all local

No deletion, no commit. Leave docs as local investigation residue.

### Option B - Delete only owner-approved obsolete planning docs

Prepare a future manual deletion list, but do not delete now.

### Option C - Track one more narrow docs slice

Only if a file has unique repo-level evidence not already preserved by `#1304`.

### Option D - No action

Accept that untracked docs remain temporarily.

## Proposed deletion candidates

The following files may be deleted later only after explicit owner approval:

- `docs/roadmap/pcc/housekeeping_track_candidate_commit_plan.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md`
- `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md`
- `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md`
- `docs/roadmap/pcc/pcc_stack_selective_port_plan.md`

Do not delete them in this task.

## Files to keep local

The following files should remain untracked for now:

- `docs/roadmap/pcc/local_holdbacks_cleanup_plan.md`
- `docs/roadmap/pcc/local_holdbacks_sanitation_audit.md`
- `docs/roadmap/pcc/local_practical_core_readiness_audit.md`
- `docs/roadmap/pcc/local_repo_mismatch_audit.md`

## Files not recommended for tracking

The following files are planning residue and should not enter repo history
unless an owner explicitly approves pruning or reclassification:

- `docs/roadmap/pcc/housekeeping_track_candidate_commit_plan.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md`
- `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md`
- `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md`
- `docs/roadmap/pcc/pcc_stack_selective_port_plan.md`

## Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| deleting useful investigation history | medium/high | owner approval required |
| tracking obsolete planning docs | medium | `#1304` evidence trail already preserved |
| losing context too early | medium | keep local until owner decides |
| accidental `git clean` | high | explicitly forbidden |
| accidental `git add .` | high | explicitly forbidden |

## Recommended next step

Choose one:

- owner approves no action;
- owner approves deletion list later;
- owner keeps all docs local;
- owner requests one more review.

## Final verdict

PASS-WITH-WARNINGS.

The important repo-level evidence trail is already preserved by `#1304`.
The remaining untracked audit docs are local planning/investigation residue.
Most are obsolete planning docs that should only be deleted after owner
approval, while the local investigation docs can remain untracked. The owner
should review the current plan artifact before any further disposition change.
