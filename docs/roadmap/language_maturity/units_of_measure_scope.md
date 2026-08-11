# Units Of Measure Scope

Status: completed first-wave baseline history
Related issue: `#118`

SSF-07 narrowing note: the original `+`/`-` inclusion below was written
generically over "the same numeric base type." SSF-07 (issue #1578) has since
selected a narrower boundary: binary and unary `+`/`-` are admitted only for
measured `f64`; measured `fx` reports an explicit narrow-slice gap, and
measured `i32`/`u32` are rejected as unsupported operators. `docs/spec/types.md`
and `docs/spec/source_semantics.md` are the current canonical source for this
narrower contract; this document's bullets below are preserved as first-wave
baseline history and are not the up-to-date operator boundary.

## Goal

Define a narrow, executable first-wave units-of-measure contract for `v0.2`
without opening a general dimensional-analysis system or widening runtime and
host boundaries.

This track is now completed on `main` as a narrow compile-time-only source
contract over the existing core numeric families.

## Decision Check

- [x] This is a new explicit post-stable track with its own scope decision
- [x] This remains compile-time-only and does not widen the VM/runtime contract
- [x] This is one stream, not a mixture of multiple tracks
- [x] This can close with a clear done-boundary

## Decision

`#118` will be implemented as a compile-time-only source contract over the
existing core numeric families.

First-wave syntax:

- `i32[m]`
- `u32[ms]`
- `f64[kg]`
- `fx[rpm]`

where the bracket payload is a single unit symbol.

## Included In First Wave

- unit annotations on `i32`, `u32`, `f64`, and `fx`
- unit-carrying local bindings, parameters, returns, tuple items, record
  fields, `Option(T)`, and `Result(T, E)` payload positions
- exact unit equality for assignment, call arguments, returns, and pattern
  bindings
- `+` and `-` only when both operands have the same numeric base type and the
  same unit annotation
- `==` and `!=` only when both operands have the same numeric base type and the
  same unit annotation
- explicit docs describing which operations preserve units and which ones reject
- lowering by erasing units after semantic validation, reusing the existing
  numeric carrier path

## Explicit Non-Goals

- implicit conversions between units
- conversion functions or conversion tables
- compound unit algebra such as `m/s`, `N*m`, or exponent notation
- unit inference from literals
- unit annotations on non-numeric families
- widening VM value carriers or public host ABI shapes
- reopening arithmetic policy beyond the documented first-wave subset

## Honest First-Wave Rules

- unit annotations are part of the source type contract, not part of the VM
  value representation
- unsuffixed numeric literals remain ordinary numeric literals and become
  unit-carrying only through typed positions
- `+`, `-`, `==`, and `!=` preserve the declared unit when both sides match
  exactly
- `*`, `/`, and `%` on unit-carrying values are rejected in the first wave
- unannotated numeric values do not implicitly coerce to annotated numeric types
- different unit symbols with the same numeric carrier are still mismatched
  types

## Done Boundary

`#118` is complete on `main` because:

1. parser accepts first-wave unit annotations in declared type positions,
2. sema reports compile-time mismatches on incompatible unit-annotated numeric
   values,
3. lowering remains unit-erased and reuses the existing numeric execution path,
4. docs define supported operations and honest non-conversion rules,
5. verified tests cover assignment, call, return, and operator mismatch cases.
