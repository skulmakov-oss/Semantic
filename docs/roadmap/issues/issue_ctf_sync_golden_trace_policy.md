Title: ctf: record PCC canonical examples as golden trace candidates

## Description

Record the PCC canonical examples that may become future golden trace candidates.

This issue is an execution handle for the CTF sync golden trace policy slice.

The goal is to align Core Trust Freeze wording with the completed PCC practical contours without turning canonical examples into mandatory golden trace artifacts yet.

## Source Doc

- `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_golden_trace_policy.md`
- `docs/roadmap/language_maturity/core_trust_freeze/golden_trace_policy.md`
- `docs/roadmap/pcc/practical_core_phase_checkpoint.md`
- `docs/roadmap/pcc/pcc_ctf_sync_checkpoint.md`
- `docs/roadmap/pcc/pcc_ctf_sync_closeout.md`

## Scope

Record PCC canonical examples as future golden trace candidates:

- `examples/canonical/match_control_flow/src/main.sm`
- `examples/canonical/option_result_control_flow/src/main.sm`
- `examples/canonical/loop_control_flow/src/main.sm`
- `examples/canonical/text_core/src/main.sm`
- `examples/canonical/collections_core/src/main.sm`
- `examples/canonical/stdlib_v0_helpers/src/main.sm`

Supporting candidates:

- `examples/canonical/text_collections_toolbox/src/main.sm`
- `examples/canonical/cli_batch_core/src/main.sm`

## Acceptance Criteria

- PCC canonical examples are listed as trace candidates, not active golden requirements;
- trace policy remains future-facing and does not change current test expectations;
- `print(text)` output trace handling remains explicitly unresolved or future work;
- negative diagnostics remain broad-marker based for now;
- 7hell is not changed in this slice;
- no runtime behavior is changed;
- no verifier, VM, SemCode, or capability behavior is changed;
- the sync outcome remains `SYNC-PASS-WITH-FOLLOWUPS`.

## Out of Scope

- generating golden traces;
- adding trace files;
- changing canonical examples;
- changing 7hell;
- snapshotting full diagnostics;
- requiring exact diagnostic output;
- requiring `print(text)` output traces;
- changing VM execution;
- changing verifier admission;
- opening a new PCC feature contour.
