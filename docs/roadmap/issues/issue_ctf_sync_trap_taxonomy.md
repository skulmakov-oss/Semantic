Title: ctf: record PCC trap taxonomy edge cases

## Description

Record the trap and diagnostic edge cases exposed by the closed PCC practical contours.

This issue is an execution handle for the CTF sync trap taxonomy slice.

The goal is to keep Core Trust Freeze wording aligned with PCC coverage without changing runtime, verifier, VM, or SemCode behavior.

## Source Doc

- `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_trap_taxonomy.md`
- `docs/roadmap/language_maturity/core_trust_freeze/trap_taxonomy.md`
- `docs/roadmap/pcc/pcc_ctf_sync_checkpoint.md`
- `docs/roadmap/pcc/pcc_ctf_sync_closeout.md`

## Scope

Document and classify PCC-exposed trap / diagnostic edges:

- non-bool `if` / `while` condition;
- `break` / `continue` outside loop;
- missing match fallback arm;
- missing return path;
- invalid text operation;
- unsupported `to_text(...)` target;
- invalid `print(...)` target;
- invalid sequence index type;
- invalid collection element / key / value type;
- unsupported map operation;
- unsupported collection formatting;
- missing map key behavior;
- empty `pop` behavior;
- out-of-bounds sequence access.

## Acceptance Criteria

- each edge is classified as one of:
  - compile-time diagnostic;
  - runtime trap candidate;
  - unresolved policy edge;
- current PCC negative harnesses are referenced:
  - `tests/pcc_control_flow_negative.rs`;
  - `tests/pcc_text_negative.rs`;
  - `tests/pcc_collections_negative.rs`;
  - `tests/pcc_stdlib_negative.rs`;
- unresolved semantics remain explicitly marked;
- no runtime behavior is changed;
- no verifier, VM, SemCode, or capability behavior is changed;
- the sync outcome remains `SYNC-PASS-WITH-FOLLOWUPS`.

## Out of Scope

- implementing new traps;
- changing diagnostic codes;
- changing verifier admission;
- changing VM execution;
- finalizing missing-key semantics;
- finalizing empty-pop semantics;
- finalizing out-of-bounds semantics;
- changing PCC fixtures or harnesses;
- opening a new PCC feature contour.
