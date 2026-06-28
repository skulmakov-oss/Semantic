# Sequence Ownership Contract Design

## 1. Executive Verdict

Recommended SEQ-1 contract:

**static sequence index ownership first**

Proposed initial vocabulary:

- `SequenceIndexStatic(u32)` for statically known element positions;
- `SequenceElementDynamic` deferred until a later slice;
- iteration ownership deferred until a later slice;
- whole-sequence ownership remains the parent path.

This is the smallest safe first contract because it mirrors the tuple slice
shape without pretending that dynamic indexing or iteration have the same
stability as fixed tuple positions.

## 2. Current Problem

Sequence runtime support exists, but runtime support is not ownership
qualification.

The current codebase already proves:

- sequence literals parse and typecheck;
- sequence indexing parses and typechecks;
- sequence iteration syntax exists;
- the VM executes `Value::Sequence`;
- runtime helpers such as `push`, `prepend`, `pop`, `len`, and `contains`
  exist and are tested.

What is missing is a sequence ownership contract with:

- a dedicated ownership vocabulary;
- a SemCode ownership component;
- verifier admission for that component;
- VM overlap semantics for sequence element paths;
- E2E and negative hardening evidence.

Without that contract, `seq[0]` is just a runtime access expression, not a
qualified ownership path.

## 3. Proposed Ownership Vocabulary

### Proposed path component

- Name: `SequenceIndexStatic`
- Meaning: a statically known sequence element position
- SemCode tag: proposed `OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX = 3`
- Decoded representation: `DecodedAccessPathComponent::SequenceIndexStatic(u32)`
- Runtime `AccessPath` component: `PathComponent::SequenceIndexStatic(u32)`
- Proposed `PatternPath` element: `PatternPathElem::SequenceIndexStatic(u32)`

### Conceptual meaning

- `seq` is the whole sequence path.
- `seq[0]` is a distinct owned element path.
- `seq[1]` is another distinct owned element path.
- `seq[i]` is deferred for this initial contract.
- iteration bindings are deferred for this initial contract.

This keeps the first version explicit and testable while avoiding false
precision for dynamic indices or iteration.

## 4. Static Index Semantics

Recommended overlap rules for the first version:

- `seq[0]` overlaps `seq[0]`
- `seq[0]` does not overlap `seq[1]`
- `seq` overlaps `seq[0]`
- `seq` overlaps `seq[1]`
- `seq[i]` is deferred and should not be claimed as supported in SEQ-1

If the project later chooses to represent unknown or dynamic element access,
that should be introduced as a separate conservative contract, not smuggled into
the static-index slice.

## 5. Dynamic Index / Iteration Policy

Dynamic index ownership is deferred.

Iteration ownership is also deferred.

Recommended policy:

- do not model `seq[i]` in the first ownership slice;
- do not claim ownership for iterator-bound elements;
- do not collapse dynamic element access into static element access;
- if the project later needs conservative behavior, define a separate
  `SequenceElementDynamic` path that overlaps all sequence elements.

This is deliberate. It keeps SEQ-1 narrow and prevents a static contract from
silently becoming a dynamic aliasing system.

## 6. Layer Impact Matrix

| Layer | Proposed change later | Risk | Notes |
| --- | --- | --- | --- |
| sm-front parser | No parser grammar change required for SEQ-1 if static indexing already parses | Low | The syntax already exists; the contract only needs to classify which source paths become ownership paths |
| sm-front typecheck / PatternPath | Add static sequence-index conflict planning | Medium | Must ensure `seq[0]` vs `seq[1]` is distinguished while `seq` remains the parent path |
| sm-ir lowering | Emit static sequence ownership events | Medium | Lowering needs a sequence element ownership component, not tuple reuse by assumption |
| sm-format | Add sequence ownership vocabulary and decode support | High | This is a public binary-format contract change and needs explicit versioning review |
| sm-runtime-core | Add runtime access-path vocabulary for sequence elements | Medium | Shared runtime path model must remain append-only and deterministic |
| sm-verify | Admit and structurally validate sequence ownership payloads | High | Verifier must reject malformed payloads deterministically |
| sm-vm | Enforce overlap semantics for sequence element ownership | High | VM needs explicit overlap rules for same vs different static indexes |
| tests | Add typecheck, lowering, E2E, and negative coverage | Low | Tests should prove the contract before gate expansion |
| 7hell | Add smoke only after evidence exists | Low | 7hell should remain the final qualification gate, not the first implementation target |
| docs | Keep the contract honest and narrow | Low | Design and audit docs must stay separate from implementation claims |

## 7. Proposed SEQ-2..SEQ-6 Plan

Recommended plan if the project approves the static contract:

- SEQ-2 - add sequence ownership vocabulary seed
- SEQ-3 - add sequence PatternPath / typecheck conflict tests
- SEQ-4 - add sequence lowering ownership tests
- SEQ-5 - add sequence E2E plus verifier / VM negative hardening
- SEQ-6 - add 7hell smoke plus closeout

This plan assumes the static-index vocabulary is accepted first. If that
vocabulary is rejected, the project should stop and revisit the sequence
ownership contract instead of forcing tuple semantics onto a different shape.

## 8. Explicit Non-Goals

- No Rust code in SEQ-1.
- No SemCode change in SEQ-1.
- No dynamic index support unless explicitly designed in a later slice.
- No iterator ownership implementation.
- No performance guarantees.
- No release qualification claim.
- No sequence ownership parity claim.
- No lowering emission in SEQ-1.
- No verifier changes in SEQ-1.
- No VM changes in SEQ-1.
- No 7hell changes in SEQ-1.

## 9. Validation Evidence

This design is grounded in the SEQ-0 audit and existing repository evidence.

Commands already used for the audit basis:

- `Get-Location`
- `git status --short --untracked-files=all`
- `git branch --show-current`
- `git log --oneline -10`
- `git branch --contains c2d4ae0`
- `Test-Path .\tools\7hell\run.ps1`
- `Test-Path .\docs\roadmap\pcc\sequence_ownership_contract_audit.md`
- `Test-Path .\docs\roadmap\pcc\tuple_ownership_matrix.md`
- `Get-Content docs/roadmap/pcc/sequence_ownership_contract_audit.md`
- `Get-Content crates/sm-runtime-core/src/lib.rs`
- `Get-Content crates/sm-ir/src/legacy_lowering.rs`
- `Get-Content crates/sm-vm/src/semcode_vm.rs`
- `Get-Content crates/sm-verify/src/lib.rs`
- `Get-Content crates/sm-front/src/typecheck.rs`

Validation commands for this design slice:

- `cargo fmt --check`
- `cargo test --workspace --all-features`
- `powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1`

## 10. Final Position

Sequence ownership should begin with static sequence-index ownership only.

Dynamic index ownership and iteration ownership remain deferred until they are
explicitly designed and separately qualified.
