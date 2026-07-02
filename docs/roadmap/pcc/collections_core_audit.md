# PCC Collections Core Audit

Status: working audit note for the practical collections contour

## Purpose

This document records the current admitted collections surface in Semantic as
observed through the source specs, canonical examples, and collections-focused
fixtures.

It is intentionally conservative. It does not widen the language contract.
It captures the current practical contour so the next collections issues can be
scoped cleanly.

## Executive Verdict

Collections are already practical on the current admitted surface, but the
contract is split across layers:

- `docs/spec/syntax.md` still documents the collections surface conservatively.
- `docs/spec/types.md`, `docs/roadmap/v1_readiness.md`, benchmark fixtures, and
  canonical examples show a practical contour with `Sequence(T)`, `Map(K, V)`,
  indexing, iteration, and persistent helper operations.

Observed practical anchors:

- [examples/canonical/cli_batch_core/](C:\Users\said3\Desktop\EXOcode\EXOcode\examples\canonical\cli_batch_core\README.md)
- [examples/canonical/text_collections_toolbox/](C:\Users\said3\Desktop\EXOcode\EXOcode\examples\canonical\text_collections_toolbox\README.md)
- [examples/canonical/collections_core/](C:\Users\said3\Desktop\EXOcode\EXOcode\examples\canonical\collections_core\README.md)
- `tests/fixtures/snake_benchmark/README.md`
- `tests/fixtures/snake_benchmark/positive_sequence_len.sm`
- `tests/fixtures/snake_benchmark/positive_sequence_indexing.sm`
- `tests/fixtures/snake_benchmark/positive_sequence_iteration.sm`
- `tests/fixtures/snake_benchmark/positive_contains.sm`
- `tests/fixtures/snake_benchmark/positive_push_prepend.sm`
- `tests/fixtures/snake_benchmark/positive_pop.sm`
- `tests/fixtures/snake_benchmark/positive_map_basic.sm`
- `tests/fixtures/snake_benchmark/positive_map_empty_contextual.sm`
- `tests/fixtures/snake_benchmark/positive_map_persistent_update.sm`

Current verdict:

- `Sequence(T)` is a practical source type on the current contour.
- `Map(K, V)` is a practical source type on the current contour.
- ordered sequence literals are admitted.
- `expr[index]` on admitted ordered sequences is admitted.
- `for value in sequence { ... }` is admitted on the current practical contour.
- sequence helpers `len`, `is_empty`, `contains`, `push`, `prepend`, and `pop`
  are practical admitted helpers.
- map helpers `map_empty`, `map_set`, `map_get`, and `map_contains` are
  practical admitted helpers.
- collections are runtime-managed containers, not records.
- host-facing collection ABI widening remains out of scope.

## Observed Surface

### Sequence

Observed admitted practical forms:

- `Sequence(i32)` in declared type positions
- bracketed ordered sequence literals such as `[1, 2, 3]`
- sequence indexing with `expr[index]`
- iteration with `for value in sequence { ... }`
- `len(sequence)`
- `is_empty(sequence)`
- `contains(sequence, value)`
- persistent `push(sequence, value)`
- persistent `prepend(sequence, value)`
- persistent `pop(sequence)`

Not qualified here:

- mutation-heavy collection APIs beyond the current persistent helpers
- generic collection abstractions
- lazy pipelines / comprehensions
- collection formatting APIs

### Map

Observed admitted practical forms:

- `Map(K, V)` in declared type positions
- contextual `map_empty()`
- persistent `map_set(map, key, value)`
- lookup via `map_get(map, key, default)`
- membership checks via `map_contains(map, key)`

Observed boundary:

- `map_empty()` without contextual `Map(K, V)` type is rejected
- statement-form `map_empty();` is rejected
- `map_get` missing-key behavior currently uses a default rather than a trap
- map iteration policy remains bounded/open

Not qualified here:

- `remove`
- map comprehensions
- map formatting
- unordered iteration contracts beyond the current benchmarked surface

## Stable Quirks

- `docs/spec/syntax.md` still describes the collections surface conservatively
  and does not promote the full practical contour.
- sequence and map values are treated as runtime-managed containers, not
  records or host-ABI primitives.
- `map_get` default behavior is part of the current practical baseline.
- map iteration order is not yet separately closed as a standalone policy.

## Runtime / Stdlib Boundary

Current practical boundary:

- sequence and map helpers are source-level surface
- `Sequence(T)` and `Map(K, V)` are owner-layer container families
- `len`, `is_empty`, `contains`, `push`, `prepend`, `pop`, `map_empty`,
  `map_set`, `map_get`, and `map_contains` are helper calls, not general
  collection abstraction APIs
- collection helpers can compose with text and control flow, but that is a mixed
  contour and should not be confused with a collections-only contract

## Not Yet Canonicalized Here

- map iteration policy closeout
- missing-key trap semantics
- collection formatting API
- generic collection abstractions
- borrowed/shared collection models
- host-facing collection ABI widening

## Evidence Summary

Current collections evidence is distributed across:

- source type/spec documentation
- roadmap readiness documentation
- canonical mixed examples
- benchmark fixtures
- positive and negative collections fixtures

Practical summary:

- the admitted contour already supports ordinary small-program collection use
- the contour is still intentionally bounded
- collections should be treated as a practical core surface, not a general
  collection framework

## Follow-Up Issues

Recommended next issue pack:

- PCC-COLL-1: specify collections core contract
- PCC-COLL-2: add standalone canonical collections example
- PCC-COLL-3A: add collections negative diagnostics corpus
- PCC-COLL-3B: add collections negative diagnostics harness
- PCC-COLL-4: wire collections diagnostics into 7hell if needed
- PCC-COLL-5: close out the collections contour

## Closeout

See [`collections_core_closeout.md`](collections_core_closeout.md) for the final
contour summary once PCC-COLL-5 is completed.

## Non-Goals

- No language widening.
- No claim that `remove` is ready.
- No claim that map iteration is fully frozen.
- No claim that the collections contour is release-stable.
- No canonical promotion of any new collection operator until probe evidence
  exists.
