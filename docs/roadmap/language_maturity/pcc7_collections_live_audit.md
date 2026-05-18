# PCC-7 Collections v0 Live Audit

Status: live audit
Owner: language maturity stream
Scope: Collections v0 readiness before PCC-7 implementation or fixture work
Non-goal: code changes

## 1. Purpose

This document audits current Collections v0 readiness on `main` after PCC-6
closeout.

It is docs-only. It does not add collection behavior.

## 2. Current Known Status

Current `main` already contains a benchmark-usable first-wave collection
surface:

- `Sequence<T>` is exercised through ordered indexing, iteration, length,
  emptiness, membership, push, prepend, and pop fixtures in the snake
  benchmark path.
- Dedicated PCC-7 positive Sequence fixture packaging now exists.
- `Map<K,V>` is exercised through contextual `map_empty()` construction and a
  basic persistent update / lookup baseline.
- Negative snake fixtures already lock the current `map_empty()` contextual
  typing boundary.
- The roadmap docs already classify `Sequence<T>` and `Map<K,V>` as
  runtime-managed dynamic containers, not records.
- The current docs already treat collections as a separate aggregate family
  with explicit carrier policy concerns.
- Deterministic iteration for `Sequence<T>` is evidenced by the benchmark
  surface and the dedicated PCC-7 Sequence suite; deterministic iteration for
  `Map<K,V>` is still policy-documented rather than separately closed.
- Memory / quota interaction is a required PCC-7 concern, but no dedicated
  closeout evidence exists yet.
- Dedicated PCC-7 fixture packaging is still missing for Map, negative
  diagnostics, and policy closeout.

Audit verdict:

```text
The Collections v0 baseline is partially present in main.
Sequence behavior is benchmark-evidenced and now backed by dedicated PCC-7
fixtures.
Map behavior is benchmark-evidenced only at the narrow contextual construction
/ basic lookup level.
PCC-7 still needs explicit fixture packaging and policy evidence for Map,
negative diagnostics, bounds / missing-key behavior, and memory / quota
interaction.
```

CTF touched: none
Reason: docs-only audit; no runtime value, trap, determinism, verifier,
SymbolId, capability, or trace change.

## 3. Readiness Matrix

| Layer | Required for PCC-7 | Current state | Ready? | Next action |
| --- | --- | --- | --- | --- |
| parser | `Sequence<T>` type syntax | confirmed-working | yes | keep dedicated Sequence fixtures stable |
| parser | `Map<K,V>` type syntax | confirmed-partial | partial | keep benchmark evidence, package PCC-7 fixtures |
| parser | sequence construction/literal | confirmed-working | yes | keep dedicated Sequence fixtures stable |
| parser | map construction/literal | confirmed-partial | partial | package PCC-7 fixtures and close policy gaps |
| frontend model | collection type representation | confirmed-working | yes | keep runtime-managed container boundary explicit |
| typecheck | Sequence element typing | confirmed-working | yes | keep dedicated Sequence fixtures stable |
| typecheck | Map key/value typing | confirmed-partial | partial | audit contextual typing and missing-key policy |
| sequence ops | indexing | confirmed-working | yes | keep dedicated Sequence fixtures stable |
| sequence ops | iteration | confirmed-working | yes | keep deterministic iteration evidence stable |
| sequence ops | len / is_empty / contains | confirmed-working | yes | keep dedicated Sequence fixtures stable |
| sequence ops | push / prepend / pop | confirmed-working | yes | keep persistent update evidence stable |
| map ops | empty map contextual typing | confirmed-partial | partial | keep current blocker fixture and policy wording stable |
| map ops | insert / lookup | confirmed-partial | partial | audit lookup / update policy evidence |
| map ops | missing-key behavior | confirmed-partial | partial | define and evidence the trap / default policy |
| determinism | Sequence iteration policy | confirmed-working | yes | keep deterministic iteration evidence stable |
| determinism | Map iteration policy | confirmed-partial | partial | audit whether iteration is admitted and stable |
| mutation | mutation semantics | confirmed-working | yes | keep persistent update evidence stable |
| assignment | assignment / aliasing policy | confirmed-partial | partial | audit aliasing limits before implementation |
| memory/quota | allocation and quota behavior | confirmed-partial | partial | collect explicit quota evidence |
| lowering | collection op lowering | confirmed-working | yes | keep current pipeline evidence stable |
| SemCode | stable representation | confirmed-working | yes | keep encoding stable and audited |
| verifier | validates emitted collection form | confirmed-working | yes | keep verifier coverage stable |
| VM/runtime | carrier behavior | confirmed-working | yes | keep runtime-managed container semantics explicit |
| diagnostics | clear collection errors | confirmed-working | yes | preserve current blocker diagnostics |
| tests | positive / negative coverage | confirmed-partial | partial | package PCC-7 fixtures explicitly |
| docs | collection boundary | confirmed-working | yes | keep `PCC-3.5` and PCC-7 boundary synced |

## 4. Risk List

Observed risks are narrow but real:

- Collections may silently become a general heap / GC story.
- Sequence / Map assignment may imply aliasing if not explicitly bounded.
- Mutation semantics may create nondeterminism if not policy-locked.
- Map iteration order must be deterministic if iteration is admitted.
- Missing-key and bounds behavior must be diagnostic / trap stable.
- Memory / quota interaction must not be ignored.
- Collections must not be confused with records.
- Collections must not widen host ABI.
- Stdlib helpers must not be smuggled into PCC-7 unless already required for
  the admitted surface.
- Dedicated PCC-7 fixture packaging is still missing for Map, negative
  diagnostics, and policy closeout even though the Sequence baseline is now
  fixture-backed.

## 5. Recommended PCC-7 Split

No new architecture seam is required for the current benchmark baseline.

Recommended split:

```text
PCC-7B — test(sequence): lock positive Sequence fixtures
PCC-7C — test(map): lock positive Map fixtures
PCC-7D — test(collections): lock negative diagnostics and trap fixtures
PCC-7E — docs(collections): close PCC-7 with evidence sync and roadmap status update
```

If audit work finds a missing seam, split it narrowly and do not widen the PR
into general collections, GC, or host ABI work.

Potential narrow follow-ups:

- PCC-7I1 sequence type / literal seam
- PCC-7I2 map contextual typing seam
- PCC-7I3 deterministic iteration policy seam
- PCC-7I4 bounds / missing-key trap policy seam
- PCC-7I5 memory / quota policy seam
- PCC-7I6 assignment / aliasing policy seam

## 6. Out of Scope

- general GC
- borrowed collections
- shared mutable collections
- persistent collections
- copy-on-write
- advanced ownership
- host ABI collection values
- records
- Option / Result
- broad stdlib expansion
- UI / Workbench

## 7. Acceptance Checklist

- [x] parser surface inspected
- [x] AST/frontend model inspected
- [x] typecheck inspected
- [x] Sequence support inspected
- [x] Map support inspected
- [x] iteration policy inspected
- [x] bounds behavior inspected
- [x] missing-key behavior inspected
- [x] mutation semantics inspected
- [x] assignment / aliasing policy inspected
- [x] memory / quota behavior inspected
- [x] lowering inspected
- [x] SemCode/verifier inspected
- [x] VM/runtime inspected
- [x] diagnostics inspected
- [x] tests inspected
- [x] docs inspected
- [x] risks documented
- [x] PCC-7 split proposed
- [x] PCC-7B positive Sequence fixtures added
- [x] no code changed

## 8. Evidence Notes

PCC-7 collections evidence is currently split between benchmark tests and
roadmap docs.

Covered positive cases:

- ordered `Sequence<T>` indexing and iteration
- `len(sequence) -> i32`
- `is_empty(sequence) -> bool`
- `contains(sequence, value) -> bool`
- `push(sequence, value) -> Sequence(T)`
- `prepend(sequence, value) -> Sequence(T)`
- `pop(sequence) -> Sequence(T)`
- contextual `map_empty()` plus basic map lookup/update baseline

Covered negative cases:

- `map_empty()` without contextual `Map(K, V)` type
- discarded statement-form `map_empty();`

Validation:

- `tests/snake_benchmark_gap_matrix.rs`
- `tests/fixtures/snake_benchmark/README.md`
- `docs/roadmap/language_maturity/collections_surface_full_scope.md`
- `docs/roadmap/first_wave_map_surface.md`

PCC-7 is not closed.
PCC-7 fixture packaging and policy closeout remain to be added.

## 9. PCC-7B Evidence

PCC-7B adds dedicated positive Sequence<T> acceptance fixtures.

Covered positive cases:

- `Sequence<T>` declared type position;
- sequence construction / admitted constructor surface;
- indexing;
- deterministic iteration for the current admitted sequence iteration surface;
- `len`;
- `is_empty`;
- `contains`;
- `push`;
- `prepend`;
- `pop`;
- function-boundary `Sequence<T>`.

Validation:

- `cargo test --test pcc7_sequence_acceptance`
- `cargo test -q --test snake_benchmark_gap_matrix snake_benchmark_positive_surface_passes_end_to_end`
- `git diff --check`

PCC-7B does not cover Map.
Map positive fixtures remain PCC-7C.
Negative diagnostics / traps remain PCC-7D.
Memory / quota and policy closeout remain PCC-7E or a narrow policy PR if
needed.
