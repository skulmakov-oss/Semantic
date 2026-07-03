# Housekeeping Audit Docs Classification

## Status

Result: PASS-WITH-WARNINGS

This is an audit-only housekeeping classification.

No files were deleted.
No files were moved.
No files were staged.
No files were committed.
No code/tests/examples/7hell files were changed.

## Purpose

Classify remaining untracked audit docs after the `#1302` and `#1303`
small-slice merges.

## Source repo state

- branch: `main`
- HEAD: `a708a4ab36d86adadfa226dc07bd5dc79287326e`
- origin/main: `a708a4ab36d86adadfa226dc07bd5dc79287326e`
- main == origin/main: `yes`
- dirty tree summary: 13 untracked audit docs remain
- untracked file count: `13`

## Context

- `#1302` Linguist readiness templates: merged
- `#1303` Semantic sugar track RFC: merged
- PR `#1301` remains external reference
- monolithic port remains prohibited
- the remaining files are housekeeping / audit docs only

## Untracked files inventory

| File | Exists | Size/short summary | Initial category |
|---|---:|---|---|
| `docs/roadmap/pcc/external/pr_1301_changed_files.txt` | yes | external PR file list evidence (119 lines) | TRACK-CANDIDATE |
| `docs/roadmap/pcc/local_holdbacks_sanitation_audit.md` | yes | local holdbacks sanitation snapshot | KEEP-LOCAL |
| `docs/roadmap/pcc/local_practical_core_readiness_audit.md` | yes | pre-merge Practical Core readiness audit | KEEP-LOCAL |
| `docs/roadmap/pcc/local_repo_mismatch_audit.md` | yes | pre-merge repo mismatch audit | KEEP-LOCAL |
| `docs/roadmap/pcc/pcc_stack_bridge_audit.md` | yes | bridge audit between local repo and PR `#1301` | TRACK-CANDIDATE |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md` | yes | initial blocked external diff sampling attempt | OBSOLETE-AFTER-MERGE |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling_captured.md` | yes | captured external diff sampling evidence | TRACK-CANDIDATE |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md` | yes | retry plan for external diff sampling | OBSOLETE-AFTER-MERGE |
| `docs/roadmap/pcc/pcc_stack_external_inventory.md` | yes | external PR inventory and layer decomposition | TRACK-CANDIDATE |
| `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md` | yes | first safe slice proposal for Linguist templates | OBSOLETE-AFTER-MERGE |
| `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md` | yes | port plan for the five Linguist files | OBSOLETE-AFTER-MERGE |
| `docs/roadmap/pcc/pcc_stack_linguist_wording_audit.md` | yes | wording gate audit for Linguist templates | TRACK-CANDIDATE |
| `docs/roadmap/pcc/pcc_stack_selective_port_plan.md` | yes | selective port plan for external PR `#1301` | OBSOLETE-AFTER-MERGE |

## Classification matrix

| File | Classification | Reason | Recommended action |
|---|---|---|---|
| `docs/roadmap/pcc/external/pr_1301_changed_files.txt` | TRACK-CANDIDATE | Captures the 119-file external evidence trail for PR `#1301`. | Track later, ideally together with the external diff sampling docs. |
| `docs/roadmap/pcc/local_holdbacks_sanitation_audit.md` | KEEP-LOCAL | Useful local housekeeping snapshot, but it is a transient working log and already reflected by later state. | Keep untracked for now. |
| `docs/roadmap/pcc/local_practical_core_readiness_audit.md` | KEEP-LOCAL | Records an early local mismatch snapshot; still useful locally, but it is a pre-merge working artifact. | Keep untracked for now. |
| `docs/roadmap/pcc/local_repo_mismatch_audit.md` | KEEP-LOCAL | Local mismatch log; useful for investigation, not required as repo history. | Keep untracked for now. |
| `docs/roadmap/pcc/pcc_stack_bridge_audit.md` | TRACK-CANDIDATE | Important audit trail explaining why the external PR stayed external instead of being imported monolithically. | Track later as core provenance. |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md` | OBSOLETE-AFTER-MERGE | The blocked sampling step was superseded by retry and captured sampling docs. | Leave untracked or retire later; do not delete here. |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling_captured.md` | TRACK-CANDIDATE | Strong evidence artifact: file list and representative diff sampling were actually captured. | Track later as core evidence. |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md` | OBSOLETE-AFTER-MERGE | Retry plan was superseded by captured evidence. | Leave untracked or retire later; do not delete here. |
| `docs/roadmap/pcc/pcc_stack_external_inventory.md` | TRACK-CANDIDATE | Core decomposition of PR `#1301` into layers; useful audit trail for future boundary work. | Track later, ideally with the bridge and sampling docs. |
| `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md` | OBSOLETE-AFTER-MERGE | The safe-slice proposal was superseded by the actual port and merge of the Linguist docs slice. | Keep only as historical trail if desired. |
| `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md` | OBSOLETE-AFTER-MERGE | The port plan was executed; the plan itself is now historical. | Keep only as historical trail if desired. |
| `docs/roadmap/pcc/pcc_stack_linguist_wording_audit.md` | TRACK-CANDIDATE | Documents the wording gates that made the Linguist slice safe to track and port. | Track later as provenance for the slice. |
| `docs/roadmap/pcc/pcc_stack_selective_port_plan.md` | OBSOLETE-AFTER-MERGE | The selective-port plan was overtaken by the actual small-slice port and merge. | Keep only as historical trail if desired. |

## Track candidates

Files that are safe and useful to track later:

- `docs/roadmap/pcc/external/pr_1301_changed_files.txt`
- `docs/roadmap/pcc/pcc_stack_bridge_audit.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling_captured.md`
- `docs/roadmap/pcc/pcc_stack_external_inventory.md`
- `docs/roadmap/pcc/pcc_stack_linguist_wording_audit.md`

These files preserve the evidence trail for why the external PR remained
external, how the small-slice port was selected, and how the wording gates were
validated.

## Local-only candidates

Files that should stay untracked for now:

- `docs/roadmap/pcc/local_holdbacks_sanitation_audit.md`
- `docs/roadmap/pcc/local_practical_core_readiness_audit.md`
- `docs/roadmap/pcc/local_repo_mismatch_audit.md`

These are useful investigation logs, but they are local working artifacts rather
than repository-level provenance that needs to be published.

## Obsolete candidates

Files that were needed before the `#1302` / `#1303` merges and now mostly serve
as historical context:

- `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md`
- `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md`
- `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md`
- `docs/roadmap/pcc/pcc_stack_selective_port_plan.md`

Do not delete them in this task.

## Move candidates

No file here is a strong move candidate based on the current evidence.

The better classification for the local audit logs is keep-local, not relocate.

## Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| committing stale audit trail | medium/high | verify wording before any future commit |
| losing useful mismatch evidence | medium | track key bridge and sampling docs |
| mixing housekeeping with feature work | medium | separate PR / commit groups |
| accidentally deleting docs | high | no delete without owner approval |

## Recommended next step

Choose one:

- create a small docs commit for selected `TRACK-CANDIDATE` audit docs;
- keep all local for now;
- ask owner to approve deletion of obsolete docs;
- split into multiple housekeeping commits.

## Final verdict

`PASS-WITH-WARNINGS`

Reason:

- the inventory is explicit and bounded;
- the evidence trail docs are worth preserving;
- several other files are clearly historical or local-only and should not be mixed into a feature commit without an owner decision.
