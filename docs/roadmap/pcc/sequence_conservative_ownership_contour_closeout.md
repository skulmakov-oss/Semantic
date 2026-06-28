# Sequence Conservative Ownership Contour Closeout

## 0. Status

Status:
  COMPLETE / CONSERVATIVE CONTOUR CLOSED

This is **not** full symbolic, range, or iterator sequence ownership.

## 1. Closed Scope

The qualified sequence ownership contour is:

- static sequence index ownership is a precise path component;
- dynamic sequence index ownership falls back conservatively to the parent
  sequence path;
- runtime overlap rejects parent/child conflicts;
- same static index conflicts are rejected;
- static sibling indexes are allowed.

This means the repository now has an explicitly qualified conservative sequence
ownership contour, not a full dynamic precision model.

## 2. Qualified Behavior Matrix

| Borrow | Write | Expected | Status |
| --- | --- | --- | --- |
| `seq[0]` | `seq[0]` | conflict | qualified |
| `seq[0]` | `seq[1]` | allowed | qualified |
| `seq` | `seq[0]` | conflict | qualified |
| `seq[0]` | `seq` | conflict | qualified |
| `seq[i]` | `seq[0]` | conflict | qualified conservative |
| `seq[i]` | `seq[1]` | conflict | qualified conservative |
| `seq[i]` | `seq` | conflict | qualified conservative |
| `seq` | `seq[i]` | conflict | qualified conservative |

## 3. Active Contract

Current active dynamic contract:

`seq[i] -> seq`

This is safe but not precise. The fallback intentionally treats unresolved
dynamic element ownership as whole-sequence ownership.

## 4. Explicit Non-Claims

The project does **not** yet claim:

- precise dynamic index ownership;
- `SequenceIndexDynamic`;
- symbolic dynamic index equality;
- range ownership;
- iterator ownership;
- runtime value equality for dynamic indexes;
- a new SemCode dynamic sequence component.

## 5. Deferred Work

Deferred future work remains:

- symbolic dynamic sequence identity contract;
- `SequenceIndexDynamic` SemCode design;
- range / region ownership;
- iterator cursor ownership;
- dynamic index verifier rules;
- runtime symbolic equality rules.

## 6. Why Conservative First

The conservative fallback was chosen because it is:

- safe;
- deterministic;
- verifier-friendly;
- compatible with the current SemCode vocabulary;
- independent of a runtime identity contract.

The cost is that it is intentionally over-restrictive for potentially disjoint
dynamic indexes. Precision is deferred until the identity model is explicit.

## 7. Validation Evidence

The contour is backed by:

- runtime ownership E2E sequence tests;
- workspace test pass;
- 7hell pass.

The supporting sequence evidence is recorded in the earlier audit and
implementation slices, including the static sequence tests and the conservative
dynamic fallback result.

## 8. Next Recommended Step

Recommended next step:

Stop sequence implementation here for now and return to broader PCC work.

If precision is required later, start with another design-only contract for
symbolic identity before any new implementation slice.

## 9. Final Verdict

Conservative sequence ownership contour is qualified:

- static indexes are precise;
- dynamic indexes fall back to the parent sequence;
- symbolic / range / iterator precision remains deferred.

The project must not claim full dynamic sequence precision, symbolic index
equality, iterator ownership, or range ownership yet.
