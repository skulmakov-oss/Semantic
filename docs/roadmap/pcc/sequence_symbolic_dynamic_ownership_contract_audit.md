# Sequence Symbolic Dynamic Ownership Contract Audit

## 1. Executive Verdict

**Recommended position: delay `SequenceIndexDynamic` for now.**

SEQ-7b already gives the safe conservative fallback:

`seq[i] -> seq`

That is the correct contract until the project has a stable symbolic index
identity model that is verifier-first and deterministic.

Symbolic dynamic ownership is therefore **not yet justified as an
implementation slice**. It should remain a design question until the identity
contract is explicit.

## 2. Current Problem

Current sequence ownership support is split:

- static `seq[0]` / `seq[1]` is qualified;
- dynamic `seq[i]` currently falls back conservatively to `seq`.

That fallback is correct for safety, but it does not provide precision.

The missing question is not "can the runtime execute sequences?".
The missing question is "what makes a dynamic index the same path across the
compiler, SemCode, verifier, and VM?"

Without a stable identity model, any symbolic dynamic ownership path would be
guesswork.

## 3. Proposed Symbolic Ownership Shape

The future symbolic form could look like one of these:

- `seq.dynamic(i)`
- `SequenceIndexDynamic(token)`

The important part is not the syntax name; it is the token contract.

Candidate token meanings:

- local variable symbol;
- SSA value id;
- runtime register id;
- normalized expression id;
- iterator cursor id;
- verified symbolic index expression.

If the token is not stable and comparable across layers, symbolic ownership
cannot remain deterministic.

## 4. Equality Models

### Model 1 - Token identity only

`seq[i]` overlaps `seq[j]` only if `token(i) == token(j)`.

Pros:

- simple;
- deterministic;
- no arithmetic reasoning.

Cons:

- imprecise for equivalent expressions;
- token generation must be stable;
- different but equivalent forms may fail to match.

### Model 2 - Normalized expression identity

`seq[i + 0] == seq[i]` if normalization canonicalizes both to the same token.

Pros:

- more precise;
- better compiler reuse.

Cons:

- requires expression normalization;
- verifier must trust normalized identity;
- risks hidden symbolic complexity.

### Model 3 - Runtime value equality

`seq[i]` overlaps `seq[j]` only if runtime values are equal.

Pros:

- precise at runtime.

Cons:

- changes the ownership model materially;
- requires runtime tracking of index values;
- complicates verifier admission;
- makes deterministic overlap reasoning harder.

### Model 4 - Range / region fallback

Dynamic index is treated as an unknown region.

Pros:

- can evolve toward slices/iterators.

Cons:

- larger format and verifier design;
- not minimal;
- not the right next step for sequence precision.

## 5. Recommended Architecture Decision

**Recommended next position: delay `SequenceIndexDynamic` until a stable
symbolic identity contract exists.**

Reason:

- SEQ-7b already gives a safe conservative fallback;
- precision requires a token identity contract that is stable across layers;
- no new SemCode component should be added before that contract exists.

If the project later chooses to add symbolic precision, the contract must be
defined before implementation.

## 6. SemCode Questions for a Future Slice

If `SequenceIndexDynamic` is ever admitted, the project would need to define:

- ownership component tag and payload shape;
- symbol/token width and stability;
- validation rules for unknown or malformed tokens;
- version and capability implications;
- backward compatibility with the current static sequence vocabulary.

Possible future tag:

`OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX_DYNAMIC`

That is a design placeholder only. It is not an implementation request.

## 7. Runtime Comparison Questions

If symbolic sequence paths exist, overlap semantics would need to answer:

- `seq.dynamic(i)` vs `seq.dynamic(i)`;
- `seq.dynamic(i)` vs `seq.dynamic(j)`;
- `seq.dynamic(i)` vs `seq[0]`;
- `seq.dynamic(i)` vs `seq`.

The conservative default should be:

- unresolved equality => conflict.

That keeps the system safe when token identity cannot be proven.

## 8. Future Test Plan

These tests are deferred, not implemented:

- same dynamic token conflicts;
- different dynamic token behavior is specified conservatively;
- dynamic token vs static index behavior is specified;
- dynamic token vs parent sequence behavior is specified;
- static sibling behavior remains unchanged;
- malformed dynamic token is rejected by verifier.

If the project ever introduces `SequenceIndexDynamic`, these tests should be
the first qualification gate.

## 9. Explicit Non-Goals

- No `SequenceIndexDynamic` implementation.
- No iterator ownership.
- No range ownership.
- No symbolic equality implementation.
- No new SemCode path component.
- No new verifier checks.
- No new VM overlap semantics.
- No changes to `crates/sm-runtime-core`.
- No changes to `crates/sm-vm`.
- No changes to `crates/sm-format`.
- No changes to `crates/sm-verify`.
- No changes to `crates/sm-ir`.
- No changes to `tests/runtime_ownership_e2e.rs`.

## 10. Validation Evidence

Commands used for this audit:

- `Get-Location`
- `git status --short --untracked-files=all`
- `git branch --show-current`
- `git log --oneline -10`
- `Test-Path .\\docs\\roadmap\\pcc\\sequence_dynamic_ownership_contract_audit.md`
- `Test-Path .\\docs\\roadmap\\pcc\\sequence_ownership_contract_design.md`
- `Test-Path .\\docs\\dna\\SEMANTIC_UI_DNA.md`
- `Test-Path .\\tools\\7hell\\run.ps1`
- `Get-Content docs\\roadmap\\pcc\\sequence_dynamic_ownership_contract_audit.md`
- `Get-Content docs\\dna\\SEMANTIC_UI_DNA.md`
- `rg -n "SequenceIndexStatic|SequenceIndex|PathComponent|AccessPath|BorrowWriteConflict|write path overlaps active borrow|borrowed_paths|ownership path|OWNERSHIP_PATH_COMPONENT_SEQUENCE|SequenceOwnershipPath" crates/sm-runtime-core/src crates/sm-vm/src crates/sm-format/src crates/sm-verify/src crates/sm-ir/src tests/runtime_ownership_e2e.rs`
- `rg -n "dynamic sequence|dynamic index|symbolic|iterator ownership|range ownership|SequenceIndexDynamic|sequence ownership" docs tests crates`

Validation commands to run after writing this audit:

- `cargo fmt --check`
- `cargo test --workspace --all-features`
- `powershell -ExecutionPolicy Bypass -File .\\tools\\7hell\\run.ps1`
- `git diff --check`
- `git status --short --untracked-files=all`

## 11. Final Position

Sequence runtime support exists, but symbolic dynamic ownership is not proven.

The conservative fallback from SEQ-7b should remain the active contract until
the repository defines a stable symbolic index identity model.

Do not add `SequenceIndexDynamic` yet.
Keep `seq[i] -> seq` as the safe behavior.
