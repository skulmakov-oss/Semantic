# Untracked Audit Docs Disposition Plan

## Status

Result: PASS-WITH-WARNINGS

This is an audit-only disposition plan.

No files were deleted.
No files were moved.
No files were staged.
No files were committed.
No git clean was run.
No code/tests/examples/7hell files were changed.

## Purpose

Classify remaining untracked audit docs after the `#1302`, `#1303`, and
`#1304` merge cycle.

## Source repo state

- branch: `codex/pcc-bridge-port-audit-trail`
- HEAD: `7b838f9e8c7035ffc317c0aec3296104033888a6`
- origin/main: `a708a4ab36d86adadfa226dc07bd5dc79287326e`
- main == origin/main: `yes`
- dirty tree summary: 11 untracked audit docs remain
- untracked file count: `11`

## Context

- `#1302` Linguist readiness templates: merged
- `#1303` Semantic sugar track RFC: merged
- `#1304` PCC bridge / port audit trail: merged
- original local holdbacks: absent
- remaining issue: untracked audit docs only

## Tracked evidence already preserved

The `#1304` evidence trail is already tracked:

- `docs/roadmap/pcc/external/pr_1301_changed_files.txt`
- `docs/roadmap/pcc/housekeeping_audit_docs_classification.md`
- `docs/roadmap/pcc/pcc_stack_bridge_audit.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling_captured.md`
- `docs/roadmap/pcc/pcc_stack_external_inventory.md`
- `docs/roadmap/pcc/pcc_stack_linguist_wording_audit.md`

These files are not part of the untracked disposition set.

## Untracked audit docs inventory

| File | Exists | Short summary | Disposition |
|---|---:|---|---|
| `docs/roadmap/pcc/housekeeping_track_candidate_commit_plan.md` | yes | Future housekeeping commit plan for the core evidence trail. | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL |
| `docs/roadmap/pcc/local_holdbacks_cleanup_plan.md` | yes | Sanitation plan showing the original holdback paths are absent in this checkout. | KEEP-LOCAL |
| `docs/roadmap/pcc/local_holdbacks_sanitation_audit.md` | yes | Sanitation snapshot for the absent holdback set and the later local docs note. | KEEP-LOCAL |
| `docs/roadmap/pcc/local_practical_core_readiness_audit.md` | yes | Early mismatch/readiness audit showing the claimed PCC stack was absent locally. | KEEP-LOCAL |
| `docs/roadmap/pcc/local_repo_mismatch_audit.md` | yes | Repository mismatch audit explaining why the claimed PCC stack was not present. | KEEP-LOCAL |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md` | yes | Blocked first external diff sampling attempt before retry/capture succeeded. | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md` | yes | Retry strategy for external diff sampling before captured evidence existed. | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL |
| `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md` | yes | Proposal for the first safe slice before the actual Linguist port. | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL |
| `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md` | yes | Pre-port plan for the five Linguist readiness template files. | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL |
| `docs/roadmap/pcc/pcc_stack_selective_port_plan.md` | yes | Selective-port planning doc for PR `#1301` layers before the slice was chosen. | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL |
| `docs/roadmap/pcc/untracked_audit_docs_disposition_plan.md` | yes | This disposition plan itself. | NEEDS-REVIEW |

## Disposition matrix

| File | Disposition | Reason | Proposed future action |
|---|---|---|---|
| `docs/roadmap/pcc/housekeeping_track_candidate_commit_plan.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | The plan was superseded by the actual evidence-trail commit and this disposition review. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/local_holdbacks_cleanup_plan.md` | KEEP-LOCAL | Records that the original holdback paths are absent in this checkout; useful as a local sanitation note. | Keep local. |
| `docs/roadmap/pcc/local_holdbacks_sanitation_audit.md` | KEEP-LOCAL | Local sanitation snapshot for the absent holdback set and temporary investigation context. | Keep local. |
| `docs/roadmap/pcc/local_practical_core_readiness_audit.md` | KEEP-LOCAL | Captures the original mismatch investigation; useful operator context even though the external trail is now tracked. | Keep local. |
| `docs/roadmap/pcc/local_repo_mismatch_audit.md` | KEEP-LOCAL | Captures the repo mismatch investigation; helpful locally, but not required in repo history. | Keep local. |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | Superseded by the retry plan and the captured sampling evidence. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | Superseded by the captured sampling evidence. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | Superseded by the actual Linguist slice port and merge. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | Superseded by the actual 5-file Linguist port and merge. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/pcc_stack_selective_port_plan.md` | DELETE-CANDIDATE-NEEDS-OWNER-APPROVAL | Superseded by the selected safe slice and later port evidence. | Delete later only if the owner approves pruning obsolete planning docs. |
| `docs/roadmap/pcc/untracked_audit_docs_disposition_plan.md` | NEEDS-REVIEW | This plan is the current working artifact and should be judged after it is read. | No immediate action. |

## Keep-local docs

The following files should remain untracked for now:

- `docs/roadmap/pcc/local_holdbacks_cleanup_plan.md`
- `docs/roadmap/pcc/local_holdbacks_sanitation_audit.md`
- `docs/roadmap/pcc/local_practical_core_readiness_audit.md`
- `docs/roadmap/pcc/local_repo_mismatch_audit.md`

## Obsolete after merge

The following planning docs were superseded by merged evidence and do not need
to remain as active work items:

- `docs/roadmap/pcc/housekeeping_track_candidate_commit_plan.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md`
- `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md`
- `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md`
- `docs/roadmap/pcc/pcc_stack_selective_port_plan.md`

## Delete candidates

The following files could be deleted later only after owner approval:

- `docs/roadmap/pcc/housekeeping_track_candidate_commit_plan.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md`
- `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md`
- `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md`
- `docs/roadmap/pcc/pcc_stack_selective_port_plan.md`

Do not delete them in this task.

## Track-later candidates

No additional track-later candidates are proposed in this pass.

## Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| deleting useful investigation logs | medium/high | owner approval required |
| tracking obsolete planning docs | medium | keep only tracked evidence trail |
| losing local context too early | medium | keep-local until owner decision |
| accidental `git clean` | high | explicitly forbidden |

## Recommended next step

Choose one:

- keep all remaining untracked audit docs local;
- owner approves deletion of obsolete docs;
- owner approves one more narrow commit;
- no further action.

## Final verdict

PASS-WITH-WARNINGS.

The source repo is clean relative to `origin/main`, the original holdbacks are
absent, and the remaining untracked files are local audit/planning docs. The
warning remains because several items are now obsolete planning artifacts that
should only be deleted after owner approval, while the investigation logs can
remain local.
