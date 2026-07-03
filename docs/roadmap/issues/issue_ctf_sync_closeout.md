Title: ctf: close PCC sync follow-up wording pack

## Description

Close the compact PCC / CTF sync follow-up wording pack.

This issue is an execution handle for summarizing the five CTF sync slices created after the first Practical Core phase.

The goal is to confirm that the CTF follow-ups were documented as roadmap / trust-lane wording only, without introducing runtime, verifier, VM, SemCode, or capability behavior changes.

## Source Docs

- `docs/roadmap/pcc/pcc_ctf_sync_checkpoint.md`
- `docs/roadmap/pcc/pcc_ctf_sync_closeout.md`
- `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_closeout.md`
- `docs/roadmap/language_maturity/core_trust_freeze/index.md`

## Follow-up Slice Bodies

- `docs/roadmap/issues/issue_ctf_sync_runtime_value_registry.md`
- `docs/roadmap/issues/issue_ctf_sync_trap_taxonomy.md`
- `docs/roadmap/issues/issue_ctf_sync_determinism_matrix.md`
- `docs/roadmap/issues/issue_ctf_sync_capability_print_text.md`
- `docs/roadmap/issues/issue_ctf_sync_golden_trace_policy.md`

## Scope

Summarize and close the CTF sync follow-up wording pack covering:

- RuntimeValue registry alignment;
- trap taxonomy edge cases;
- determinism matrix PCC additions;
- `print(text)` capability wording;
- PCC golden trace candidates.

## Acceptance Criteria

- all five CTF sync issue bodies are listed;
- the sync outcome remains `SYNC-PASS-WITH-FOLLOWUPS`;
- wording confirms no runtime behavior was changed;
- wording confirms no verifier, VM, SemCode, or capability behavior was changed;
- PCC closeouts remain closed;
- CTF remains a separate trust lane;
- unresolved items remain explicit follow-ups, not hidden release claims;
- no new PCC feature contour is opened.

## Out of Scope

- implementing runtime changes;
- implementing verifier changes;
- implementing VM changes;
- implementing SemCode changes;
- changing capability behavior;
- generating golden traces;
- changing 7hell architecture;
- opening a new PCC practical contour;
- upgrading the outcome to `SYNC-PASS` without fresh evidence.
