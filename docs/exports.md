# Exports v0.2

This page is a companion guide to the current export and re-export surface.

The canonical public contract now lives in:

- `docs/spec/modules.md`
- `docs/spec/diagnostics.md`

This page should stay aligned with those source-contract documents and is
intended mainly as a compact guide with examples.

## Exportable Items

The current export set includes top-level Logos declarations:

1. `System`
2. `Entity`
3. `Law`

## Re-export

Re-export is supported through `Import pub ...`:

1. `Import pub "dep.sm"`
2. `Import pub "dep.sm" { Foo, Bar as Baz }`
3. `Import pub "dep.sm" *`

Each exported item stores provenance:

1. `Local { module }` — declared in the exporting module
2. `ReExport { chain }` — the complete hop chain (exporting module, every
   intermediate re-exporting module, the declaring module, then the symbol);
   a re-export of a re-export extends the chain and never truncates it

A plain (non-`pub`) import exports nothing, so there is no separate
"imported" export provenance state (PB-04, #1706). Every item also records
the declaring module id and declared name it ultimately denotes.

## Deterministic Export Surface

Export ordering is deterministic by declaration order (`decl_order` ascending).

Current clarification:

- local exports stay first in local declaration order
- re-exports append after locals in import declaration order
- within one re-exported dependency set, dependency export order is preserved

## Collision Policy

If two exports in one module publish the same public name, compilation fails with `E0242`.

The export namespace is flat (PB-04, #1686): the same public name across
different kinds (`Entity A` with `Law "A"` or `System A`) is also an `E0242`
collision. A kind qualifier in a selection (`{ Entity:A }`) asserts the kind of
that single item; it does not create a `(name, kind)` namespace.

## Symbol-level Cycle Policy

Re-export symbol cycles are detected and rejected with `E0243`, with a
deterministic chain trace that follows the current re-export recursion order.

## Examples

Collision (`E0242`):

```exo
Import pub "a.sm"
Import pub "b.sm"
```

Cycle (`E0243`):

```exo
// a.sm
Import pub "b.sm"
// b.sm
Import pub "a.sm"
```

## Related Errors

- `E0242`: see `docs/errors/E0242.md`
- `E0243`: see `docs/errors/E0243.md`
- `E0244`: see `docs/errors/E0244.md`
- `E0245`: see `docs/errors/E0245.md`
