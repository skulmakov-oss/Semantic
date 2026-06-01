# PCC-7 Collections v0 Live Audit

Status: PCC-7 closed / evidence synced for current fixture-backed scope
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
- Dedicated PCC-7 positive Map fixture packaging now exists.
- Dedicated PCC-7 negative diagnostics / trap packaging now exists for map
  contextual typing, sequence bounds, sequence pop on empty, and map
  key/value mismatch.
- The roadmap docs already classify `Sequence<T>` and `Map<K,V>` as
  runtime-managed dynamic containers, not records.
- The current docs already treat collections as a separate aggregate family
  with explicit carrier policy concerns.
- Deterministic iteration for `Sequence<T>` is evidenced by the benchmark
  surface and the dedicated PCC-7 Sequence suite; deterministic iteration for
  `Map<K,V>` is still policy-documented rather than separately closed.
- PCC-7D now locks negative diagnostics / trap behavior for map contextual
  typing, sequence bounds, sequence pop on empty, and map key/value mismatch.
- Memory / quota interaction is a required PCC-7 concern, but no dedicated
  closeout evidence exists yet.
- Map missing-key behavior is still policy-documented rather than separately
  closed.

Audit verdict:

```text
The Collections v0 baseline is present in main for the current fixture-backed
scope.
Sequence behavior is benchmark-evidenced and backed by dedicated PCC-7
fixtures. Map behavior is benchmark-evidenced and now backed by dedicated
PCC-7 positive and negative fixtures at the narrow contextual construction /
basic insert-update / basic lookup level.
PCC-7 is closed for the current Practical Core fixture-backed scope.
Map missing-key behavior, Map iteration policy, assignment / aliasing policy,
and memory / quota evidence remain bounded open items.
```

CTF touched: none
Reason: docs-only audit; no runtime value, trap, determinism, verifier,
SymbolId, capability, or trace change.

## 3. Readiness Matrix

| Layer | Required for PCC-7 | Current state | Ready? | Next action |
| --- | --- | --- | --- | --- |
| parser | `Sequence<T>` type syntax | confirmed-working | yes | keep dedicated Sequence fixtures stable |
| parser | `Map<K,V>` type syntax | confirmed-working | yes | keep dedicated Map fixtures stable |
| parser | sequence construction/literal | confirmed-working | yes | keep dedicated Sequence fixtures stable |
| parser | map construction/literal | confirmed-working | yes | keep contextual `map_empty()` evidence and stable diagnostics |
| frontend model | collection type representation | confirmed-working | yes | keep runtime-managed container boundary explicit |
| typecheck | Sequence element typing | confirmed-working | yes | keep dedicated Sequence fixtures stable |
| typecheck | Map key/value typing | confirmed-working | yes | keep key/value mismatch diagnostics stable |
| sequence ops | indexing | confirmed-working | yes | keep dedicated Sequence fixtures stable |
| sequence ops | iteration | confirmed-working | yes | keep deterministic iteration evidence stable |
| sequence ops | len / is_empty / contains | confirmed-working | yes | keep dedicated Sequence fixtures stable |
| sequence ops | push / prepend / pop | confirmed-working | yes | keep persistent update evidence stable |
| map ops | empty map contextual typing | confirmed-working | yes | keep current blocker fixture and policy wording stable |
| map ops | insert / lookup | confirmed-working | yes | keep lookup / update evidence stable |
| map ops | missing-key behavior | confirmed-partial | partial | keep bounded-open policy note until explicit evidence lands |
| determinism | Sequence iteration policy | confirmed-working | yes | keep deterministic iteration evidence stable |
| determinism | Map iteration policy | confirmed-partial | partial | keep bounded-open policy note until explicit evidence lands |
| mutation | mutation semantics | confirmed-working | yes | keep persistent update evidence stable |
| assignment | assignment / aliasing policy | confirmed-partial | partial | keep bounded-open policy note until explicit evidence lands |
| memory/quota | allocation and quota behavior | confirmed-partial | partial | keep bounded-open policy note until explicit evidence lands |
| lowering | collection op lowering | confirmed-working | yes | keep current pipeline evidence stable |
| SemCode | stable representation | confirmed-working | yes | keep encoding stable and audited |
| verifier | validates emitted collection form | confirmed-working | yes | keep verifier coverage stable |
| VM/runtime | carrier behavior | confirmed-working | yes | keep runtime-managed container semantics explicit |
| diagnostics | clear collection errors | confirmed-working | yes | preserve current blocker diagnostics |
| tests | positive / negative coverage | confirmed-working | yes | keep dedicated PCC-7 fixtures stable |
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
- Dedicated PCC-7 fixture packaging now covers positive and negative
  Sequence / Map surfaces, but missing-key behavior, Map iteration policy, and
  memory / quota closeout are still open.

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
- [x] PCC-7C positive Map fixtures added
- [x] PCC-7D negative diagnostics / trap fixtures added
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

PCC-7 is closed for the current fixture-backed scope.
The following are not claimed complete by PCC-7E:

- Map missing-key behavior;
- Map iteration policy;
- assignment / aliasing policy;
- memory / quota closeout evidence.

These remain bounded policy / future-work items and must not be treated as
implemented by PCC-7E.

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
Negative diagnostics / traps now include PCC-7D.

## 10. PCC-7C Evidence

PCC-7C adds dedicated positive Map<K,V> acceptance fixtures.

Covered positive cases:

- `Map<K,V>` declared type position;
- contextual `map_empty()` construction;
- basic insert / update surface;
- basic lookup surface;
- persistent update behavior.

Validation:

- `cargo test --test pcc7_map_acceptance`
- `cargo test --test pcc7_sequence_acceptance`
- `cargo test -q --test snake_benchmark_gap_matrix snake_benchmark_positive_surface_passes_end_to_end`
- `git diff --check`

PCC-7C does not cover Sequence expansion.
Negative diagnostics / traps are covered by PCC-7D.
Missing-key behavior remains open because current `map_get` still returns a
default rather than trapping.
Memory / quota and policy closeout remain bounded open for later policy work.

## 11. PCC-7D Evidence

PCC-7D adds dedicated negative Collections diagnostics and trap fixtures.

Covered negative cases:

- `map_empty()` without contextual `Map(K,V)` type;
- statement-form `map_empty();`;
- Sequence bounds behavior via out-of-bounds indexing;
- empty Sequence pop behavior;
- Sequence element type mismatch;
- Map key type mismatch;
- Map value type mismatch.

Omitted from this phase:

- Map missing-key behavior, because current admitted `map_get` returns a
  default and does not trap.

Validation:

- `cargo test --test pcc7_collections_diagnostics`
- `cargo test --test pcc7_sequence_acceptance`
- `cargo test --test pcc7_map_acceptance`
- `cargo test -q --test snake_benchmark_gap_matrix snake_benchmark_positive_surface_passes_end_to_end`
- `git diff --check`

PCC-7D does not add new collection semantics.
PCC-7D does not implement missing-key behavior.
PCC-7D does not implement memory / quota policy.
PCC-7D does not change aliasing or ownership policy.
PCC-7 closeout remains PCC-7E.

## 12. PCC-7E Closeout

PCC-7A — docs audit / scope correction
PCC-7B — positive Sequence<T> fixtures
PCC-7C — positive Map<K,V> fixtures
PCC-7D — negative collection diagnostics and trap fixtures
PCC-7E — bounded closeout / roadmap sync

Final verdict:

```text
PCC-7 Collections v0 is closed for the current Practical Core fixture-backed
scope.
```

Evidence-backed statements:

- Sequence<T> positive path is evidence-backed.
- Map<K,V> contextual construction and basic update / lookup path are
  evidence-backed.
- Collection diagnostics / traps are evidence-backed for the admitted
  negative fixture set.
- Invalid collection programs covered by PCC-7D do not silently pass.
- Sequence bounds and empty-pop traps are deterministic under the current
  admitted surface.
- No new collection semantics were introduced.
- No GC policy was introduced.
- No host ABI collection widening was introduced.

Bounded-open items not claimed complete by PCC-7E:

- Map missing-key behavior;
- Map iteration policy;
- assignment / aliasing policy;
- memory / quota evidence.

These remain bounded policy / future-work items and must not be treated as
implemented by PCC-7E.

Validation:

- `cargo test --test pcc7_sequence_acceptance`
- `cargo test --test pcc7_map_acceptance`
- `cargo test --test pcc7_collections_diagnostics`
- `cargo test -q --test snake_benchmark_gap_matrix snake_benchmark_positive_surface_passes_end_to_end`
- `git diff --check`

## 13. PCC-7E Acceptance Checklist

- [x] PCC-7B positive Sequence fixtures added
- [x] PCC-7C positive Map fixtures added
- [x] PCC-7D negative diagnostics / trap fixtures added
- [x] PCC-7E bounded closeout evidence synced
- [x] PCC-7 roadmap status updated
- [x] open policy boundaries documented
- [x] no collection architecture redesign
- [x] no memory / quota implementation introduced
- [x] no host ABI widening introduced
