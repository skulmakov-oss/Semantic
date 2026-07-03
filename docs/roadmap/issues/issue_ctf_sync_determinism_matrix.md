Title: ctf: update determinism matrix for PCC surfaces

## Description

Record the determinism implications of the closed PCC practical contours.

This issue is an execution handle for the CTF sync determinism matrix slice.

The goal is to keep Core Trust Freeze wording aligned with PCC-qualified behavior without changing runtime, verifier, VM, SemCode, or capability behavior.

## Source Doc

- `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_determinism_matrix.md`
- `docs/roadmap/language_maturity/core_trust_freeze/determinism_matrix.md`
- `docs/roadmap/pcc/pcc_ctf_sync_checkpoint.md`
- `docs/roadmap/pcc/pcc_ctf_sync_closeout.md`

## Scope

Document determinism expectations for PCC-qualified surfaces:

- `match` branch selection;
- `while` / `loop` execution;
- `break;` / `continue;`;
- terminal return paths;
- text equality;
- bounded `text + text`;
- `to_text(...)` for admitted scalar families;
- `Sequence(T)` iteration;
- `Sequence(T)` helpers;
- `Map(K, V)` helpers;
- `assert`;
- `print(text)` as capability-aware observable output.

## Acceptance Criteria

- PCC-qualified deterministic surfaces are listed;
- text concatenation is recorded as deterministic for admitted one-line `text` values;
- `to_text(...)` determinism is limited to admitted scalar families;
- `Sequence(T)` iteration is recorded as deterministic for current admitted sequence forms;
- `Map(K, V)` helper behavior is documented conservatively;
- map iteration remains explicitly out of scope;
- `print(text)` remains capability-aware and is not treated as unrestricted host ABI;
- no runtime behavior is changed;
- no verifier, VM, SemCode, or capability behavior is changed;
- the sync outcome remains `SYNC-PASS-WITH-FOLLOWUPS`.

## Out of Scope

- implementing determinism checks;
- changing map ordering behavior;
- admitting map iteration;
- changing `to_text(...)` formatting;
- changing text normalization policy;
- changing `print(text)` capability semantics;
- changing VM execution;
- changing verifier admission;
- opening a new PCC feature contour.
