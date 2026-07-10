# Housekeeping TRACK-CANDIDATE Commit Plan

## Status

Result: PASS-WITH-WARNINGS

This is a plan-only housekeeping document.

No files were staged.
No files were committed.
No files were deleted.
No files were moved.
No code/tests/examples/7hell files were changed.

## Purpose

Explain which untracked audit docs are safe candidates for a future
housekeeping commit and which should remain local or be treated as obsolete
after the `#1302` and `#1303` merges.

## Source repo state

- branch: `main`
- HEAD: `a708a4ab36d86adadfa226dc07bd5dc79287326e`
- origin/main: `a708a4ab36d86adadfa226dc07bd5dc79287326e`
- main == origin/main: `yes`
- dirty tree summary: 14 untracked audit docs remain
- untracked file count: `14`

## Input audit

- file: `docs/roadmap/pcc/housekeeping_audit_docs_classification.md`
- status: `PASS-WITH-WARNINGS`
- summary: classifies the 14 untracked docs into `TRACK-CANDIDATE`,
  `KEEP-LOCAL`, and `OBSOLETE-AFTER-MERGE`

## Candidate review matrix

| File | Prior classification | Refined decision | Reason | Future commit group |
|---|---|---|---|---|
| `docs/roadmap/pcc/external/pr_1301_changed_files.txt` | TRACK-CANDIDATE | COMMIT-IN-HOUSEKEEPING | Core external file-list evidence for PR `#1301`; useful provenance for future boundary work. | core evidence trail |
| `docs/roadmap/pcc/housekeeping_audit_docs_classification.md` | NEEDS-REVIEW | COMMIT-IN-HOUSEKEEPING | Records the audit result that separated core evidence from local logs and obsolete plans. | core evidence trail |
| `docs/roadmap/pcc/local_holdbacks_sanitation_audit.md` | KEEP-LOCAL | KEEP-LOCAL | Local operator-session sanitation snapshot; useful locally, but not necessary as published provenance. | none |
| `docs/roadmap/pcc/local_practical_core_readiness_audit.md` | KEEP-LOCAL | KEEP-LOCAL | Early mismatch investigation log; informative, but still a local working artifact. | none |
| `docs/roadmap/pcc/local_repo_mismatch_audit.md` | KEEP-LOCAL | KEEP-LOCAL | Local mismatch log; useful for the investigation history, not required in repo history. | none |
| `docs/roadmap/pcc/pcc_stack_bridge_audit.md` | TRACK-CANDIDATE | COMMIT-IN-HOUSEKEEPING | Explains why PR `#1301` remained an external reference instead of a monolithic import. | core evidence trail |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md` | OBSOLETE-AFTER-MERGE | OBSOLETE-AFTER-MERGE | Blocked sampling attempt was superseded by retry and captured sampling evidence. | none |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling_captured.md` | TRACK-CANDIDATE | COMMIT-IN-HOUSEKEEPING | Captured file list and representative diff sampling are strong evidence artifacts. | core evidence trail |
| `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md` | OBSOLETE-AFTER-MERGE | OBSOLETE-AFTER-MERGE | Retry plan was superseded by the captured evidence doc. | none |
| `docs/roadmap/pcc/pcc_stack_external_inventory.md` | TRACK-CANDIDATE | COMMIT-IN-HOUSEKEEPING | Decomposes PR `#1301` into layers and preserves the decomposition rationale. | core evidence trail |
| `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md` | OBSOLETE-AFTER-MERGE | OBSOLETE-AFTER-MERGE | Safe-slice proposal was superseded by the actual Linguist docs port and merge. | none |
| `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md` | OBSOLETE-AFTER-MERGE | OBSOLETE-AFTER-MERGE | Port plan was executed; the plan itself is now historical. | none |
| `docs/roadmap/pcc/pcc_stack_linguist_wording_audit.md` | TRACK-CANDIDATE | COMMIT-IN-HOUSEKEEPING | Records the wording gates that kept the Linguist slice conservative and safe. | core evidence trail |
| `docs/roadmap/pcc/pcc_stack_selective_port_plan.md` | OBSOLETE-AFTER-MERGE | OBSOLETE-AFTER-MERGE | Selective-port plan was overtaken by the actual small-slice port and merge. | none |

## Proposed housekeeping commit

If the future housekeeping commit is approved, keep it narrow.

Commit message:

```text
docs(pcc): track bridge and port audit trail
```

Include only:

```text
docs/roadmap/pcc/external/pr_1301_changed_files.txt
docs/roadmap/pcc/housekeeping_audit_docs_classification.md
docs/roadmap/pcc/pcc_stack_bridge_audit.md
docs/roadmap/pcc/pcc_stack_external_diff_sampling_captured.md
docs/roadmap/pcc/pcc_stack_external_inventory.md
docs/roadmap/pcc/pcc_stack_linguist_wording_audit.md
```

Do not include:

```text
docs/roadmap/pcc/local_holdbacks_sanitation_audit.md
docs/roadmap/pcc/local_practical_core_readiness_audit.md
docs/roadmap/pcc/local_repo_mismatch_audit.md
docs/roadmap/pcc/pcc_stack_external_diff_sampling.md
docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md
docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md
docs/roadmap/pcc/pcc_stack_linguist_port_plan.md
docs/roadmap/pcc/pcc_stack_selective_port_plan.md
```

## Commit gates

Before any future commit:

1. `git diff --cached --name-only` must match the approved file list exactly.
2. No code/tests/examples/7hell files.
3. No `docs/roadmap/issues/*` files unless explicitly approved.
4. No local tool state.
5. No obsolete planning docs unless owner explicitly approves.
6. No files that claim the missing PCC stack exists locally.
7. No monolithic PR `#1301` port claim.

## Recommended exclusions

Keep local or leave untracked:

- `docs/roadmap/pcc/local_holdbacks_sanitation_audit.md`
- `docs/roadmap/pcc/local_practical_core_readiness_audit.md`
- `docs/roadmap/pcc/local_repo_mismatch_audit.md`

These are still useful investigation logs, but they are not the core evidence
trail that needs to be published.

## Obsolete candidates

May be retained locally for history, but should not enter the housekeeping
commit without owner approval:

- `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md`
- `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md`
- `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md`
- `docs/roadmap/pcc/pcc_stack_linguist_port_plan.md`
- `docs/roadmap/pcc/pcc_stack_selective_port_plan.md`

## Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| tracking stale investigation logs | medium | commit only the core evidence trail |
| losing useful bridge history | medium | track the bridge and sampling evidence docs |
| mixing housekeeping with feature work | medium | keep this as a docs-only housekeeping commit |
| accidental stage of all untracked docs | high | explicit file list only |

## Recommended next step

Choose one:

- proceed to future housekeeping commit with approved list;
- keep all docs local for now;
- ask owner decision for ambiguous files;
- split into two housekeeping commits.

## Final verdict

`PASS-WITH-WARNINGS`

Reason:

- the core evidence trail is clear and small enough to commit later;
- several local audit logs are better left untracked;
- a handful of older planning docs are obsolete after the merges and should not be pulled into a housekeeping commit without owner approval.
