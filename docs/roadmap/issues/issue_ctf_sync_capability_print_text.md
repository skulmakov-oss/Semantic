Title: ctf: clarify print text capability boundary

## Description

Clarify the Core Trust Freeze wording for `print(text)` after the closed PCC practical contours.

This issue is an execution handle for the CTF sync capability/effect boundary slice.

The goal is to keep `print(text)` canonical-safe for PCC examples while preserving the capability/effect boundary and avoiding any host ABI widening claim.

## Source Doc

- `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_capability_print_text.md`
- `docs/roadmap/language_maturity/core_trust_freeze/capability_effect_denial_matrix.md`
- `docs/roadmap/pcc/stdlib_v0_closeout.md`
- `docs/roadmap/pcc/pcc_ctf_sync_checkpoint.md`
- `docs/roadmap/pcc/pcc_ctf_sync_closeout.md`

## Scope

Document the current trust-lane status of `print(text)`:

- `print(text)` is canonical-safe practical PCC surface;
- `print(text)` remains capability-aware in principle;
- `print(text)` does not imply unrestricted host output;
- `print(text)` does not widen host ABI;
- `print(text)` does not define a debug / logging framework;
- future namespace or capability policy remains open.

## Acceptance Criteria

- `print(text)` is described as a capability-aware practical helper;
- wording explicitly denies host ABI widening;
- wording explicitly denies capability/effect boundary bypass;
- non-text print targets remain out of scope;
- `print(record)` and `print(collection)` remain rejected by current PCC negative diagnostics;
- future `debug` / `io` / capability-bound namespace policy remains open;
- no runtime behavior is changed;
- no verifier, VM, SemCode, or capability behavior is changed;
- the sync outcome remains `SYNC-PASS-WITH-FOLLOWUPS`.

## Out of Scope

- implementing capability declarations;
- changing `print(text)` behavior;
- adding `print(record)`;
- adding `print(collection)`;
- adding formatting APIs;
- adding debug / logging framework;
- widening host ABI;
- changing VM host-call behavior;
- changing verifier admission;
- opening a new PCC feature contour.
