# Semantic Application Completeness PR Ledger

Status: proposed explicit post-readiness expansion program

Read this document using the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

Read this document together with:

- `docs/roadmap/final_readiness_verdict.md`
- `docs/roadmap/v1_readiness.md`
- `reports/g1_release_scope_statement.md`

## Purpose

This ledger defines the next honest expansion program after the completed
readiness cycle.

Its purpose is to make Semantic capable of authoring and validating
benchmark-class application experiments such as a self-learning snake.

This ledger is not:

- a claim that Semantic is already `public release`
- a silent widening of the current qualified contour
- a promise that browser UI becomes part of the language boundary

## Target Outcome

After the base-path PRs in this ledger land, the repository should be able to
support all of the following as real Semantic programs:

- a deterministic headless snake engine
- a training loop over that engine using a seeded pseudo-random source
- a policy/Q-table update loop over admitted collections and scalar operators
- a text trace or frame stream suitable for external inspection

The required benchmark family is:

- `examples/benchmarks/snake_core.sm`
- `examples/benchmarks/snake_learning.sm`
- `examples/benchmarks/snake_trace.sm`

The base path does **not** require:

- browser DOM ownership
- HTML/CSS as part of the language boundary
- a widget toolkit
- multi-window UI

An external HTML/TypeScript renderer may still be used as an adapter on top of
the experiment trace.

## Current Verified Baseline On `main`

Current `main` already admits these benchmark-relevant surfaces:

- enum declarations and enum-pattern `match`
- `text` literals, `text` type positions, and same-family text equality
- `Sequence(T)` declared types, literals, indexing, equality, and `for value in sequence`
- `len(sequence) -> i32` (landed PR #387)
- `is_empty(sequence) -> bool` (landed PR #395)
- `contains(sequence, value) -> bool` (landed PR #396)
- `push(sequence, value) -> Sequence(T)` (landed PR #397)
- `prepend(sequence, value) -> Sequence(T)` (landed PR #397)
- `pop(sequence) -> Sequence(T)` (landed PR #398)
- first-class closures with immutable capture
- the separate desktop UI boundary as a landed post-stable track
- `Map(K, V)` persistent lookup tables with scalar key families (landed PR #404)
  - `map_empty()` — contextual empty map; requires `let q: Map(K, V) = map_empty()`
  - `map_contains(m, k) -> bool`
  - `map_get(m, k, default) -> V`
  - `map_set(m, k, v) -> Map(K, V)` — functional update; returns new map
  - admitted key types: `i32`, `u32`, `bool`, `text`, `quad`
  - SemCode format: `SEMCOD14`, capability `CAP_MAP_VALUES = 1 << 15`
- deterministic seeded PRNG (landed PR #405)
  - `random_seed(seed: i32)` — seeds the VM PRNG; valid as a statement
  - `random_next_i32(lo: i32, hi: i32) -> i32` — deterministic bounded value in `[lo, hi)`
  - xorshift64 (period 2⁶⁴−1); range computed through `i64` to handle the full `i32` span
  - SemCode format: `SEMCOD15`, capability `CAP_PRNG = 1 << 16`
- same-family `i32` relational operators: `>`, `<`, `>=`, `<=` (pre-program, confirmed PR-B1 audit)
- same-family `i32` arithmetic: `+`, `-`, `*`, unary `-` (pre-program, confirmed PR-B1 audit)
- same-family `i32` division and modulo: `/`, `%` (landed PR-B2)
- `let mut` mutable locals (pre-program, confirmed PR-B1 audit)
- plain reassignment (pre-program, confirmed PR-B1 audit)
- statement `while` (pre-program, confirmed PR-B1 audit)
- statement `loop`, bare `break`, `continue` (pre-program, confirmed PR-B1 audit)
  - bare `break`/`continue` outside a loop correctly rejected with E0201
- first-wave text observation surface (landed PR-E1a)
  - `text + text -> text`
  - explicit `to_text(...)` for `text`, `bool`, `i32`, `u32`, and `quad`
  - no implicit scalar conversion through `+`
- narrow stdout experiment surface (landed PR-E2)
  - `print(msg: text)` — emits one line to stdout; no file I/O; no argv

Current `main` still fails this benchmark family at the following points:

- (none — all base-path surfaces admitted; benchmark pack work begins with PR-F1)

## Program Rules

- one PR = one logical step
- every behavioral PR must ship tests
- every docs/spec/runtime surface change must move together
- no silent widening of `published stable` or the current qualified contour
- merge only on green local validation where applicable and green GitHub CI
- keep one active expansion stream at a time

## PR Ledger

### A — Truth And Benchmark Baseline

- `PR-A1` [required]
  Title: `docs/spec: sync benchmark-critical current-main truth`
  Goal:
  - remove benchmark-relevant doc drift around already-landed `text`,
    `Sequence(T)`, closure capture, and sequence iteration behavior
  Scope:
  - truth-sync only
  Files:
  - benchmark-relevant `docs/spec/*` pages
  Gate:
  - docs-only
  - `git diff --check`
  - CI green

- `PR-A2` [required]
  Title: `tests: add snake benchmark gap matrix`
  Goal:
  - freeze the current pass/fail baseline before widening work starts
  Scope:
  - positive fixtures for already-landed benchmark-critical surfaces
  - negative fixtures for still-missing blockers
  Files:
  - `tests/fixtures/snake_benchmark/**`
  - one focused test target for the matrix
  Gate:
  - `cargo test -q`
  - `cargo test -q --test public_api_contracts`
  - focused benchmark-matrix test green
  - CI green

### B — Imperative Core Completion

- `PR-B1` [landed]
  Title: `docs/tests: audit same-family i32 relational baseline`
  Landed:
  - confirms same-family `i32` relational operators: `>`, `<`, `>=`, `<=`
  - adds `positive_i32_arithmetic.sm` for already-admitted `+`, `-`, `*`, unary `-`
  - adds `negative_i32_div.sm` to freeze the remaining division gap
  Goal:
  - remove stale application-completeness drift around already-admitted imperative surfaces
  Scope:
  - truth-sync and fixture coverage only
  - no parser/runtime/VM changes
  Gate:
  - `cargo test -q --test snake_benchmark_gap_matrix`
  - `git diff --check`
  - CI green

- `PR-B2` [landed]
  Title: `frontend/runtime: admit remaining i32 division and modulo`
  Landed:
  - `i32 / i32`
  - `i32 % i32`
  - divide-by-zero runtime contract
  - modulo-by-zero runtime contract
  Goal:
  - close the remaining same-family `i32` arithmetic gap
  Scope:
  - `i32 / i32`
  - `i32 % i32`
  - divide-by-zero runtime contract
  - modulo-by-zero runtime contract
  - do not widen `u32`, `f64`, `fx`, or measured numeric arithmetic in this PR
  Files:
  - frontend/lowering/VM/spec/tests layers as required
  Gate:
  - `cargo test -q`
  - `cargo test -q --test public_api_contracts`
  - targeted integer-arithmetic tests green
  - CI green

- `PR-B3` [pre-program landed]
  Title: `frontend/control: admit mutable locals and reassignment`
  Note:
  - confirmed by PR-B1 audit; no new implementation required in this program

- `PR-B4` [pre-program landed]
  Title: `frontend/control: admit statement while`
  Note:
  - confirmed by PR-B1 audit; no new implementation required in this program

- `PR-B4.5` [pre-program landed]
  Title: `frontend/control: admit statement loop and control exits`
  Note:
  - confirmed by PR-B1 audit; no new implementation required in this program

- `PR-B5` [landed]
  Title: `docs/spec/tests: freeze imperative core benchmark surface`
  Landed:
  - `docs/roadmap/b_wave_baseline.md` (PR-B5)
  Goal:
  - close the imperative-core wave as one explicit contract
  Scope:
  - docs/spec/tests freeze only
  Depends on:
  - `PR-B1` ✓ landed PR #407
  - `PR-B2` ✓ landed PR #408
  - `PR-B3` ✓ pre-program landed
  - `PR-B4` ✓ pre-program landed
  - `PR-B4.5` ✓ pre-program landed
  Gate:
  - docs-only
  - `git diff --check`
  - CI green

### C — Sequence Utility Layer

- `PR-C1` [landed]
  Title: `stdlib/sequence: admit len/is_empty/contains`
  Landed:
  - `len(sequence) -> i32` (PR #387, 2026-05-02)
  - `is_empty(sequence) -> bool` (PR #395, 2026-05-02)
  - `contains(sequence, value) -> bool` (PR #396, 2026-05-02)
  Goal:
  - provide the minimum observation helpers needed for snake state logic
  Scope:
  - no maps, no mutation-heavy API, no generic iterator framework
  Files:
  - sequence owner/spec/tests layers as required
  Gate:
  - `cargo test -q`
  - `cargo test -q --test public_api_contracts`
  - targeted sequence-helper tests green
  - CI green

- `PR-C2` [landed]
  Title: `stdlib/sequence: admit push/pop/prepend baseline`
  Landed:
  - `push(sequence, value) -> Sequence(T)` (PR #397, 2026-05-02)
  - `prepend(sequence, value) -> Sequence(T)` (PR #397, 2026-05-02)
  - `pop(sequence) -> Sequence(T)` (PR #398, 2026-05-02)
  Note:
  - these are persistent sequence helpers; they return new `Sequence(T)`
    values and rely on existing `let mut` + reassignment for evolving
    application state
  Goal:
  - make evolving snake bodies and traces practical without opening a broad
    collection redesign
  Scope:
  - the narrow sequence update operations needed by the benchmark pack
  Files:
  - source/runtime/spec/tests layers as required
  Gate:
  - `cargo test -q`
  - `cargo test -q --test public_api_contracts`
  - targeted sequence-update tests green
  - CI green

### D — Lookup State And Deterministic Randomness

- `PR-D1` [landed]
  Title: `docs/scope: define first-wave map surface`
  Scope doc:
  - `docs/roadmap/first_wave_map_surface.md`
  Goal:
  - open exactly one lookup-table family for benchmark-class application state
  Scope:
  - one deterministic `Map(K, V)` baseline only
  - no set family
  - no generic collection framework beyond the admitted map carrier
  Gate:
  - docs-only
  - `git diff --check`
  - CI green

- `PR-D2` [landed]
  Title: `frontend/runtime: admit first-wave Map(K, V)`
  Landed:
  - PR #404, 2026-05-03, merge SHA `0eca4d60` (squash)
  Goal:
  - support Q-tables and visit counters directly in Semantic
  Scope:
  - empty construction
  - `get`
  - `set`
  - `contains`
  - deterministic key behavior for the admitted key families
  Files:
  - source/runtime/spec/tests layers as required
  Depends on:
  - `PR-D1`
  Gate:
  - `cargo test -q`
  - `cargo test -q --test public_api_contracts`
  - targeted map tests green
  - CI green

- `PR-D3` [landed]
  Title: `stdlib/random: admit deterministic seeded PRNG`
  Landed:
  - PR #405, 2026-05-03, merge SHA `a586fc93` (squash)
  Goal:
  - support food spawning and exploration without host randomness
  Scope:
  - one seeded deterministic pseudo-random family only
  - no cryptographic or host-entropy claims
  Files:
  - std/random owner/spec/tests layers as required
  Gate:
  - `cargo test -q`
  - `cargo test -q --test public_api_contracts`
  - deterministic PRNG tests green
  - CI green

- `PR-D4` [landed]
  Title: `docs/spec/tests: freeze map and PRNG benchmark baseline`
  Landed:
  - `docs/roadmap/d_wave_baseline.md` (PR #406, 2026-05-03)
  Goal:
  - close the state/random wave explicitly
  Scope:
  - docs/spec/tests freeze only
  Depends on:
  - `PR-D2` ✓ landed PR #404
  - `PR-D3` ✓ landed PR #405
  Gate:
  - docs-only
  - `git diff --check`
  - CI green

### E — Observation Boundary

- `PR-E1` [scoped]
  Title: `docs/scope: define text observation contract`
  Scope doc:
  - `docs/roadmap/text_observation_contract.md`
  Goal:
  - define the minimal text surface needed for benchmark traces before
    implementation
  Scope:
  - `text + text`
  - explicit scalar `to_text(...)`
  - strict separation of user-facing `to_text` from internal `debug_render`
  - no stdout
  - no interpolation/templates/full formatting
  Gate:
  - docs-only
  - `git diff --check`
  - CI green

- `PR-E1a` [landed]
  Title: `stdlib/text: admit first-wave text concatenation and scalar to_text`
  Landed:
  - `text + text -> text`
  - explicit `to_text(...)` for `text`, `bool`, `i32`, `u32`, and `quad`
  - `text + scalar` remains rejected (E0201)
  - `to_text(record)` remains rejected (E0201)
  Depends on:
  - `PR-E1`
  Goal:
  - admit the first-wave text observation surface after the contract is frozen
  Scope:
  - `text + text`
  - explicit scalar `to_text(...)`
  - no stdout
  - no interpolation/templates/full formatting
  Files:
  - text owner/spec/tests layers as required
  Gate:
  - `cargo test -q`
  - `cargo test -q --test public_api_contracts`
  - targeted text-format tests green
  - CI green

- `PR-E2` [landed]
  Title: `cli/runtime: admit narrow stdout experiment surface`
  Landed:
  - `print(msg: text)` — emits msg + newline to stdout
  - `print` with non-text argument remains rejected (E0201)
  - new capability `CAP_STDOUT = 1 << 17`, format version `SEMCOD16`
  Goal:
  - provide one honest observation channel for headless experiments
  Scope:
  - stdout text emission only
  - no general file I/O
  - no argv story yet
  Files:
  - CLI/runtime/spec/tests layers as required
  Gate:
  - `cargo test -q`
  - `cargo test -q --test public_api_contracts`
  - targeted CLI/runtime tests green
  - CI green

### F — Benchmark Pack And Close-Out

- `PR-F1` [required]
  Title: `examples/tests: add snake_core benchmark`
  Goal:
  - prove deterministic game-state logic on the admitted application surface
  Scope:
  - headless snake engine only
  - no renderer
  Files:
  - `examples/benchmarks/snake_core.sm`
  - benchmark tests and fixtures
  Gate:
  - `cargo test -q`
  - focused benchmark tests green
  - CI green

- `PR-F2` [required]
  Title: `examples/tests: add snake_learning benchmark`
  Goal:
  - prove the self-learning loop is writable on the admitted surface
  Scope:
  - seeded deterministic training loop
  - Q-table and visit-count logic
  - no UI
  Files:
  - `examples/benchmarks/snake_learning.sm`
  - benchmark tests and fixtures
  Gate:
  - `cargo test -q`
  - focused learning-benchmark tests green
  - CI green

- `PR-F3` [required]
  Title: `docs/examples: add snake trace adapter contract`
  Goal:
  - define how Semantic experiments feed an external HTML/TypeScript renderer
    without making browser ownership part of the language boundary
  Scope:
  - one trace format
  - one adapter note
  - one benchmark-oriented workflow note
  Files:
  - benchmark docs/examples only
  Gate:
  - docs-only
  - `git diff --check`
  - CI green

- `PR-F4` [required]
  Title: `reports/tests: publish application-completeness benchmark verdict`
  Goal:
  - close this benchmark program with evidence rather than intuition
  Scope:
  - benchmark close-out report only
  Depends on:
  - `PR-F1`
  - `PR-F2`
  - `PR-F3`
  Gate:
  - benchmark evidence updated honestly
  - relevant benchmark tests green
  - CI green

## Contingency Slots

- `PR-CONT-1` [contingency]
  Trigger:
  - open only if stdout-only observation proves insufficient for the benchmark
    pack
  Title:
  - `cli/runtime: add narrow file export for experiment traces`

- `PR-CONT-2` [contingency]
  Trigger:
  - open only if a visual native demo is required after the headless benchmark
    pack is already green
  Title:
  - `examples/ui: add desktop snake demo over the existing UI boundary`

## Exit Condition

This program is complete only when:

- the snake benchmark pack is green on the admitted Semantic surface
- the trace/export story is explicit
- the benchmark does not depend on hidden author-only workarounds
- browser rendering, if used, remains an external adapter rather than an
  implicit language boundary
