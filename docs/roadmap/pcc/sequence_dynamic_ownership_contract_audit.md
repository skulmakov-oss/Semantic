# Sequence Dynamic Ownership Contract Audit

## 1. Executive Verdict

Recommended next implementation slice:

**SEQ-7b - conservative dynamic sequence ownership fallback**

Dynamic `seq[i]` should conservatively resolve to the parent sequence path
until the project has an explicit symbolic-index contract.

Recommended contract:

- static `seq[0]` / `seq[1]` stays on `SequenceIndexStatic(u32)`;
- dynamic `seq[i]` falls back to the whole-sequence path `seq`;
- iterator ownership stays deferred;
- range ownership stays deferred;
- no new SemCode ownership component is added for the first dynamic slice.

This is the safest next step because it preserves determinism, avoids a binary
format expansion, and matches the current code shape better than symbolic or
range precision.

## 2. Current Static Contract

SEQ-6b already proves the static contract:

| Borrow path | Write path | Expected |
| --- | --- | --- |
| `seq[0]` | `seq[0]` | conflict |
| `seq[0]` | `seq[1]` | allowed |
| `seq` | `seq[0]` | conflict |
| `seq[0]` | `seq` | conflict |

That slice is already covered by `tests/runtime_ownership_e2e.rs` and the
sequence E2E golden path. This audit does not reopen that work.

## 3. Current Problem

The dynamic case `seq[i]` is not equivalent to `seq[static_index]`.

The index may be:

- unknown at compile time;
- equal to another runtime index;
- constrained but unresolved;
- derived from iterator state;
- derived from user input or another computed expression.

The current repository has no qualified runtime identity contract for dynamic
element paths, so `seq[i]` cannot be treated as a precise ownership path yet.

## 4. Candidate Contract Models

### Model 1 - Conservative Whole-Sequence

Treat dynamic `seq[i]` as `seq`.

Pros:

- safe;
- simple;
- deterministic;
- no symbolic reasoning;
- no SemCode expansion.

Cons:

- overly restrictive;
- blocks disjoint dynamic indexes;
- not iterator-friendly.

### Model 2 - Runtime Symbolic Index Token

Represent dynamic access as `SequenceIndexDynamic(symbol/token)`.

Pros:

- more precise;
- can support iterator-style reasoning later.

Cons:

- requires runtime identity and equality rules;
- requires verifier/runtime contract;
- increases semantic surface area.

### Model 3 - Range / Region Ownership

Represent slices or ranges, such as `seq[start..end]`.

Pros:

- expressive for slices;
- future-friendly for bulk access.

Cons:

- larger SemCode change;
- more overlap logic;
- not minimal for the next slice.

### Model 4 - Iterator Cursor Ownership

Represent iterator state as a cursor or iterator token.

Pros:

- aligns with loop futures.

Cons:

- depends on an iterator ownership model;
- too broad for the next sequence slice.

## 5. Recommended Architecture Decision

Recommended next implementation option:

**Option A - no new component yet; dynamic `seq[i]` emits the parent sequence path**

This is the correct first step because:

- it preserves correctness;
- it avoids a new SemCode ownership kind;
- it keeps verifier and VM changes unnecessary for the first dynamic slice;
- it can be implemented and tested without symbolic equality machinery.

Options B and C are not appropriate as the next step because they add binary
format and overlap semantics before the dynamic contract itself is agreed.

## 6. Proposed SEQ-7b Follow-Up Tests

If SEQ-7b implements conservative fallback, the next tests should prove:

| Borrow path | Write path | Expected |
| --- | --- | --- |
| `seq[i]` | `seq[0]` | conflict |
| `seq[i]` | `seq[1]` | conflict |
| `seq[i]` | `seq` | conflict |
| `seq` | `seq[i]` | conflict |
| `seq[0]` | `seq[1]` | allowed |

Required test themes:

- dynamic index borrow conflicts with static writes;
- parent borrow conflicts with dynamic writes;
- static sibling behavior remains unchanged;
- no iterator claim;
- no range claim;
- no new SemCode component claim.

## 7. Explicit Non-Goals

- No `SequenceIndexDynamic`.
- No iterator ownership.
- No range ownership.
- No symbolic equality contract.
- No new VM path comparison semantics.
- No new verifier admission rules.
- No new SemCode ownership component.
- No changes to `crates/sm-runtime-core`.
- No changes to `crates/sm-vm`.
- No changes to `crates/sm-format`.
- No changes to `crates/sm-verify`.
- No changes to `crates/sm-ir`.
- No changes to `tests/runtime_ownership_e2e.rs` in this audit task.

## 8. Validation Evidence

Commands used for this audit:

- `Get-Location`
- `git status --short --untracked-files=all`
- `git branch --show-current`
- `git log --oneline -10`
- `Test-Path .\\tools\\7hell\\run.ps1`
- `Test-Path .\\tests\\sequence_ownership_golden.rs`
- `Test-Path .\\tests\\fixtures\\pcc_sequence_ownership\\positive_sequence_ownership.sm`
- `Test-Path .\\docs\\roadmap\\pcc\\sequence_ownership_contract_design.md`
- `Test-Path .\\docs\\dna\\SEMANTIC_UI_DNA.md`
- `rg -n "SequenceIndexStatic|SequenceIndex|PathComponent|AccessPath|BorrowWriteConflict|write path overlaps active borrow|borrowed_paths|ownership path|OWNERSHIP_PATH_COMPONENT_SEQUENCE" crates/sm-runtime-core/src crates/sm-vm/src crates/sm-format/src crates/sm-verify/src crates/sm-ir/src tests/runtime_ownership_e2e.rs`
- `rg -n "sequence ownership|dynamic index|iterator ownership|SequenceIndexStatic|OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX" docs tests crates`
- `Get-Content docs/roadmap/pcc/sequence_ownership_contract_design.md`
- `Get-Content docs/roadmap/pcc/sequence_ownership_contract_audit.md`
- `Get-Content docs/dna/SEMANTIC_UI_DNA.md`
- `Get-Content crates/sm-front/src/typecheck.rs | Select-Object -Skip 7660 -First 120`
- `Get-Content crates/sm-ir/src/legacy_lowering.rs | Select-Object -Skip 9140 -First 120`
- `Get-Content crates/sm-vm/src/semcode_vm.rs | Select-Object -Skip 2040 -First 40`
- `Get-Content crates/sm-verify/src/lib.rs | Select-Object -Skip 430 -First 30`

Validation commands for this audit:

- `cargo fmt --check`
- `cargo test --workspace --all-features`
- `powershell -ExecutionPolicy Bypass -File .\\tools\\7hell\\run.ps1`
- `git diff --check`

## 9. Final Position

Sequence runtime support exists, but dynamic sequence ownership is not proven.

The next implementation should be conservative fallback:

`seq[i]` -> parent sequence path `seq`

Dynamic token, range, and iterator models should wait for a later slice.

## SEQ-7b Implementation Result

SEQ-7b should record the conservative fallback once validated:

- dynamic sequence index ownership falls back to the parent sequence path;
- no new SemCode component was added;
- symbolic, range, and iterator ownership remain deferred.
