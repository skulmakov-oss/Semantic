# D-Wave Baseline — Frozen 2026-05-03

This document records the closed D-wave of the application-completeness program.

Read together with:

- `docs/roadmap/application_completeness_pr_ledger.md`
- `docs/roadmap/final_readiness_verdict.md`

## What Landed

### PR-D1 — Map surface scope doc [landed]

Scope document: `docs/roadmap/first_wave_map_surface.md`

Opened exactly one lookup-table family without making any runtime commitment.

### PR-D2 — `Map(K, V)` runtime [landed, PR #404, 2026-05-03]

Admitted `Map(K, V)` as a first-class persistent value type.

Admitted operations:

- `map_empty()` — contextual empty map; requires `let q: Map(K, V) = map_empty()`
- `map_contains(m, k) -> bool`
- `map_get(m, k, default) -> V`
- `map_set(m, k, v) -> Map(K, V)` — functional update; returns new map

Admitted key types: `i32`, `u32`, `bool`, `text`, `quad`

SemCode format: `SEMCOD14` (`MAGIC14`), capability `CAP_MAP_VALUES = 1 << 15`

Not admitted: mutable maps, set types, generic collection framework.

### PR-D3 — deterministic seeded PRNG [landed, PR #405, 2026-05-03]

Admitted two builtins:

- `random_seed(seed: i32)` — seeds the VM PRNG; valid as a statement
- `random_next_i32(lo: i32, hi: i32) -> i32` — deterministic bounded value in `[lo, hi)`

Algorithm: xorshift64, period 2⁶⁴−1. Zero seed is coerced to 1 to avoid the
fixed-point. Range is computed through `i64` arithmetic to handle spans wider
than `i32::MAX` (e.g. `random_next_i32(-2000000000, 2000000000)`).

Contract: `lo < hi` is enforced at runtime; `lo >= hi` produces a descriptive
`RuntimeError`.

SemCode format: `SEMCOD15` (`MAGIC15`), capability `CAP_PRNG = 1 << 16`

Not admitted: cryptographic randomness, host-entropy sources, multiple
independent PRNG streams.

## What Is Not Admitted By This Wave

- mutable map update in place
- set / multiset families
- `Map` iteration
- any host-entropy source
- multiple seeded PRNG streams

## Evidence

- All positive fixtures in `tests/fixtures/snake_benchmark/` pass `check`,
  `run`, `compile`, and `verify` end-to-end as of this freeze.
- `positive_map_basic.sm` covers empty construction, `set`, `get`, and
  `contains`.
- `positive_random_seeded.sm` covers determinism, range bounds, consecutive
  state advancement, and the full `i32` cross-zero span.
- `negative_random_invalid_range.sm` confirms the `lo >= hi` runtime contract.
- `snake_benchmark_gap_matrix` test target is green on `main`.
