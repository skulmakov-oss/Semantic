# Runtime Ownership Conservative Contour Closeout

## 0. Status

Status:
  COMPLETE / CONSERVATIVE RUNTIME OWNERSHIP CONTOUR CLOSED

This document does **not** claim full symbolic alias precision.
This document does **not** claim range ownership or iterator ownership.
This document does **not** claim runtime value equality for dynamic indexes.

## 1. Qualified Ownership Families

The repository now has a conservative runtime ownership contour that is
qualified across the following families:

- record field paths;
- tuple index paths;
- ADT payload paths;
- static sequence index paths;
- dynamic sequence fallback to the parent sequence path.

Each family is qualified at the level currently supported by the existing
ownership tests, closeout docs, and runtime evidence. Precision is intentionally
limited where the repository has chosen a conservative fallback instead of a
symbolic identity model.

## 2. Ownership Path Matrix

| Family | Path form | Precision | Qualified behavior |
| --- | --- | --- | --- |
| Record field | `root.field(symbol)` | precise for direct named fields | same field conflicts; sibling fields can be disjoint; parent/child record-field overlap is rejected |
| Tuple index | `root.tuple[index]` | precise for direct tuple elements | same index conflicts; sibling indexes can be disjoint; parent/child overlap is rejected |
| ADT payload | `root.variant.payload[index]` | precise by variant + payload index | same variant/index conflicts; different variant or payload index can be disjoint |
| Static sequence index | `seq[0]`, `seq[1]` | precise for direct static indexes | same index conflicts; sibling static indexes can be disjoint; parent/child overlap is rejected |
| Dynamic sequence index | `seq[i] -> seq` | conservative | dynamic access falls back to the parent sequence path, so it conflicts with borrows/writes on the same sequence root |

This closeout does not claim any broader generalized alias analysis beyond the
qualified contour above.

## 3. Qualified Overlap Semantics

The common overlap rule across the qualified contour is:

- same root plus same full path => conflict;
- parent path overlaps child path;
- sibling precise components may be disjoint;
- dynamic sequence access falls back to the parent sequence path.

Examples backed by the existing ownership evidence:

- `record.a` vs `record.a` => conflict;
- `record.a` vs `record.b` => allowed when both are precise sibling fields;
- `tuple[0]` vs `tuple[1]` => allowed;
- `adt.Some.0` vs `adt.Some.1` => allowed;
- `adt.Some.0` vs `adt.Other.0` => allowed;
- `seq[0]` vs `seq[1]` => allowed;
- `seq[i]` vs `seq[0]` => conflict because `seq[i]` lowers to `seq`;
- `seq` vs `seq[i]` => conflict.

## 4. Active Conservative Contracts

The active conservative contracts are:

- dynamic sequence ownership:
  `seq[i] -> seq`;
- unresolved symbolic equality:
  not modeled;
- iterator ownership:
  not modeled;
- range ownership:
  not modeled.

## 5. Explicit Non-Claims

The project does **not** yet claim:

- global symbolic alias analysis;
- `SequenceIndexDynamic`;
- symbolic index equality;
- runtime value equality for dynamic indexes;
- range ownership;
- iterator ownership;
- advanced borrow splitting;
- cross-container alias precision;
- a new SemCode dynamic sequence component.

## 6. Deferred Work

Deferred future work remains:

- symbolic dynamic sequence identity contract;
- `SequenceIndexDynamic` SemCode design;
- range / region ownership contract;
- iterator cursor ownership contract;
- advanced alias reasoning;
- source-level negative golden tests for future precision slices;
- full Core Trust Freeze ownership matrix refresh.

## 7. Validation Evidence

This contour is backed by the existing runtime ownership evidence:

- runtime ownership tests in `tests/runtime_ownership_e2e.rs`;
- the record-field ownership matrix and closeout docs;
- the tuple ownership matrix and closeout docs;
- the ADT payload ownership docs and matrix;
- the sequence conservative contour closeout docs;
- workspace test pass;
- 7hell pass.

The evidence set distinguishes qualified families from deferred precision and
keeps the sequence fallback explicitly conservative.

## 8. Recommended Next Step

Recommended next step:

Stop expanding ownership precision here and use this contour as the PCC
conservative baseline.

If ownership work continues, do design-only audits first and do not implement
new precision before identity, range, or iterator contracts are explicit.

## 9. Final Verdict

Runtime ownership may now be claimed as a conservative qualified contour across:

- record fields;
- tuple indexes;
- ADT payloads;
- static sequence indexes;
- dynamic sequence parent fallback.

The project must not claim full symbolic alias precision, dynamic index
equality, range ownership, iterator ownership, or advanced alias reasoning yet.
