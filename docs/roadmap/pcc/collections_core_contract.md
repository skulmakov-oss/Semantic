# PCC Collections Core Contract

Status: PCC-COLL-1 contract draft

This document defines the currently admitted Practical Core collections surface
for Semantic.

It is based on:

- `docs/roadmap/pcc/collections_core_audit.md`
- `examples/canonical/cli_batch_core`
- `examples/canonical/text_collections_toolbox`
- current fixtures and smoke tests

## Scope

This contract covers the current PCC practical contour for:

- `Sequence(T)`
- `Map(K, V)`

It does not define a full generic collection framework.

## Sequence Contract

### Current Admitted Forms

The current practical contour admits:

- `Sequence(T)` type form;
- sequence literals where admitted by current syntax;
- indexing where admitted by current syntax;
- `len`;
- `is_empty`;
- `contains`;
- `push`;
- `prepend`;
- `pop`;
- `for` over admitted sequence forms.

### Canonical-Safe Sequence Operations

Canonical examples may use:

```semantic
let values: Sequence(i32) = [1, 2];
let grown: Sequence(i32) = push(values, 3);
let extended: Sequence(i32) = prepend(grown, 0);

assert(len(values) == 2);
assert(len(grown) == 3);
assert(contains(extended, 3));
```

If the actual admitted helper names differ in a future surface revision,
canonical examples must follow the current admitted names.

## Map Contract

### Current Admitted Forms

The current practical contour admits:

- `Map(K, V)` type form;
- `map_empty`;
- `map_set`;
- `map_get`;
- `map_contains`.

### Canonical-Safe Map Operations

Canonical examples may use:

```semantic
let flags: Map(i32, bool) = map_empty();
let next: Map(i32, bool) = map_set(flags, 1, true);

assert(map_contains(next, 1));
assert(map_get(next, 1, false) == true);
```

## Type Boundary

Collections are currently practical containers, not records.

This contract does not admit:

- arbitrary generic collection abstractions;
- user-defined collection traits;
- record-as-collection behavior;
- automatic collection formatting.

## Mutation Boundary

Current PCC examples may use admitted update helpers such as:

- `push`;
- `prepend`;
- `pop`;
- `map_set`.

This contract does not require in-place mutation semantics unless the current
admitted surface already defines them.

## Iteration Boundary

`for` over admitted sequence forms is part of the current practical contour.

This contract does not yet define:

- map iteration;
- iterator objects;
- lazy iterators;
- generator-like behavior;
- collection comprehensions.

## Missing-Key / Bounds Boundary

This contract records the current admitted surface but does not finalize all
trap semantics.

Open points:

- missing-key behavior for `map_get`;
- out-of-bounds behavior for indexing;
- `pop` from empty sequence;
- capacity / quota behavior.

These must be qualified by negative diagnostics or runtime trap fixtures before
being treated as fully stable.

## Text / Formatting Boundary

This contract does not admit:

- `to_text(collection)`;
- implicit collection formatting;
- printing raw collection values as a formatting API;
- host-facing collection ABI widening.

Use explicit scalar/text extraction where admitted.

## Out of Scope

- `remove`;
- map iteration;
- advanced generics;
- collection traits;
- ordered maps;
- sets;
- collection formatting;
- serialization;
- host ABI widening;
- persistent / immutable collection policy;
- iterator protocol design.

## Canonical Anchors

Current mixed anchors:

- `examples/canonical/cli_batch_core`
- `examples/canonical/text_collections_toolbox`
- `examples/canonical/collections_core`

## Follow-Up

- PCC-COLL-2: add standalone canonical `collections_core` example
- PCC-COLL-3A: add collections negative diagnostics corpus
- PCC-COLL-3B: add collections negative diagnostics harness
- PCC-COLL-4: wire collections diagnostics into 7hell
- PCC-COLL-5: collections closeout

## Closeout

See [`collections_core_closeout.md`](collections_core_closeout.md) for the final
contour summary once PCC-COLL-5 is completed.

## Explicit Non-Goals

- No new collection framework.
- No generic iterator protocol.
- No map iteration contract.
- No claim that `remove` is ready.
- No claim that collection formatting is stable.
- No host-facing collection ABI widening.
- No claim that the collections contour is release-stable.
