# Semantic Linguist Validation Workflow

Status: preparation workflow for local detection review

## Purpose

This note documents the local validation path for future GitHub Linguist work on
Semantic.

It does not open the external Linguist PR. It does not change identity, syntax,
or samples.

## Required Inputs

Before validation, the working copy should already have:

- frozen public identity from `docs/NAMING.md`
- syntax signature note from `docs/language/semantic_syntax_signature.md`
- Linguist entry draft from `docs/language/semantic_linguist_entry_draft.md`
- canonical `.sm` sample pack from `examples/canonical/README.md`

## Validation Workflow

1. Clone or fork `github-linguist/linguist` into a scratch directory.
2. Add the draft Semantic language entry to the local Linguist checkout.
3. Copy the representative canonical `.sm` samples into the local sample
   directory plan.
4. Run Linguist detection or test commands against the samples.
5. Record which files are classified as Semantic and which files are not.
6. Record any extension conflicts or false positives.
7. Decide whether the evidence is strong enough for the external PR.

## Recommended Sample Set

Use the existing canonical examples as the validation set:

- `cli_batch_core`
- `rule_state_decision`
- `data_audit_record_iterable`
- `wave2_local_helper_import`
- `positive_selected_import`

These are already the public sample surface in the repository.

## Decision Rule

Treat `.sm` as ready for external Linguist submission only if the local
detection results are stable and unambiguous across the representative samples.

If any of the following happens, block the external PR:

- the samples are not detected consistently;
- unrelated extensions are misclassified as Semantic;
- the local entry depends on unpromised syntax;
- the validation environment is incomplete.

## Current Workspace Note

In this workspace, local validation was completed after provisioning Ruby,
MSYS2 build tools, a temporary `charlock_holmes` encoding stub, and a scratch
Linguist checkout with a draft `Semantic` entry plus canonical `.sm` samples.

The validation results were:

- `rule_state_decision.sm` -> `Semantic`
- `data_audit_record_iterable.sm` -> `Semantic`
- `cli_batch_core.sm` -> `Semantic`
- `positive_selected_import.sm` -> `Semantic`
- `wave2_local_helper_import.sm` -> `Semantic`
- `README.md` -> `Markdown`

That means the workflow is no longer blocked in this workspace. The draft entry
and representative samples are sufficient for the external Linguist PR to be
prepared once the repo-side preparation branch is merged.

## Related Documents

- `docs/language/semantic_linguist_entry_draft.md`
- `docs/language/semantic_syntax_signature.md`
- `examples/canonical/README.md`
