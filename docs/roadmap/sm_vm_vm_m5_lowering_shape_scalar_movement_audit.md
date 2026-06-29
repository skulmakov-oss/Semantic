# sm-vm VM-M5 Lowering Shape Scalar Movement Audit

## Status

VM-M5 audits source/lowering shapes that appear to produce scalar movement.

This document does not approve VM optimization, lowering changes, or runtime changes.

## Context

- VM-M3 showed `scalar_movement` dominates.
- VM-M4 showed `LoadVar` / `StoreVar` dominate every profiled fixture.
- VM-M5 now audits lowering/source-shape causes behind that movement.

The profiling corpus remains the same as prior slices, and the local 256 workload continues to preserve the same movement shape as the default corpus.

VM-M4 documents the dominant scalar movement pressure in [docs/roadmap/sm_vm_vm_m4_scalar_movement_audit.md](sm_vm_vm_m4_scalar_movement_audit.md).

## Method

- Re-ran the existing profiling workloads.
- Inspected source fixtures directly.
- Used existing profile and top-opcode evidence.
- Did not modify fixtures, tests, VM, verifier, SemCode, parser, or lowering.

## Commands

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/quad_logic_storm.sm`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/quad_match_dispatch.sm`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/fact_merge_kernel.sm`
- `sed -n '1,260p' crates/sm-vm/tests/fixtures/profiling/fact_intersect_kernel.sm`
- `sed -n '1,300p' crates/sm-vm/tests/fixtures/profiling/delta_like_kernel.sm`
- `sed -n '1,360p' crates/sm-vm/tests/fixtures/profiling/andromeda_fact_wave_64.sm`
- `sed -n '1,420p' crates/sm-vm/tests/fixtures/profiling/andromeda_fact_wave_256.sm`
- `git diff --check`
- `cargo fmt --check`

`cargo fmt --check` failed due to pre-existing unrelated formatting drift in `crates/prom-ui-backend-native/src/lib.rs`. This audit did not modify Rust source.

## Fixture Movement Summary

| Fixture | Construct class | Total | LoadVar | StoreVar | Load+Store % | Primary suspected lowering source |
|---|---|---:|---:|---:|---:|---|
| `quad_logic_storm.sm` | `quad-expression-heavy` | 7320 | 1845 | 1237 | 42.10% | Repeated quad expression evaluation plus loop-carried accumulator updates and helper-boundary staging. |
| `quad_match_dispatch.sm` | `match-heavy` | 4042 | 906 | 690 | 39.49% | Match discriminant reloads, branch-local counter updates, and helper-style staging around dispatch. |
| `fact_merge_kernel.sm` | `if/branch-heavy` | 177 | 30 | 22 | 29.38% | Branch result staging and accumulator updates dominate the small kernel. |
| `fact_intersect_kernel.sm` | `if/branch-heavy` | 304 | 49 | 22 | 23.36% | Branch-discriminant reloads and classification counter updates. |
| `delta_like_kernel.sm` | `transition-classification` | 2984 | 691 | 343 | 34.65% | Old/new transition reloads, transition counters, and repeated staging through locals. |
| `andromeda_fact_wave_64.sm` | `Andromeda-shaped mixed` | 8188 | 2023 | 1367 | 41.40% | Loop-carried counters, generated states, and mixed classification staging. |
| `andromeda_fact_wave_256.sm` | `Andromeda-shaped mixed` | 32596 | 8071 | 5447 | 41.47% | Same shape as the 64-lane wave, scaled up; the local movement ratio remains stable. |

## Per-Fixture Lowering Shape Audit

### `quad_logic_storm.sm`

#### Source shape

- One quad-producing helper, `quad_cycle(index)`, maps a loop index into `N/F/T/S` via nested `if` expressions.
- The main loop runs 64 iterations and evaluates `left`, `right`, `merged`, `consensus`, and `inverted` values per iteration.
- The workload updates four counters with `if` branches against those values.

#### Scalar movement evidence

- `LoadVar`: 1845
- `StoreVar`: 1237
- `LoadVar + StoreVar`: 3082
- `LoadVar` is top opcode #1 and `StoreVar` is top opcode #2.
- Scalar movement is `42.10%`.

#### Suspected lowering pattern

- Loop counter local traffic.
- Temporary expression staging for quad values.
- Accumulator read-modify-write behavior for the counters.
- Helper-boundary-shaped movement around `quad_cycle`.

#### Confidence

- Confirmed by profile: the local traffic is dominant.
- Supported by source shape: repeated loop-carried values and temporary quad results clearly force locals.

### `quad_match_dispatch.sm`

#### Source shape

- One helper, `state_from_index(index)`, generates quad values from a loop index.
- A second helper, `dispatch_code(state)`, uses `match` on the quad state and returns an integer code.
- The main loop repeats both helper calls over 48 iterations and updates branch counters.

#### Scalar movement evidence

- `LoadVar`: 906
- `StoreVar`: 690
- `LoadVar + StoreVar`: 1596
- `LoadVar` is top opcode #1 and `StoreVar` is top opcode #2.
- Scalar movement is `39.49%`.

#### Suspected lowering pattern

- Match discriminant reload.
- Branch-local result staging.
- Accumulator updates for the state counters.
- Helper argument/result staging around `state_from_index` and `dispatch_code`.

#### Confidence

- Confirmed by profile: locals dominate the movement family.
- Supported by source shape: `match` plus nested `if` dispatch produces repeated reload/staging pressure.

### `fact_merge_kernel.sm`

#### Source shape

- A sequence of eight explicit merge cases compares `quad` results against `S` and `N`.
- Each case conditionally updates `conflict_count` and `known_count`.
- There is no helper abstraction here; the structure is branch-heavy and accumulator-heavy.

#### Scalar movement evidence

- `LoadVar`: 30
- `StoreVar`: 22
- `LoadVar + StoreVar`: 52
- `LoadVar` is top opcode #1 and `StoreVar` is top opcode #2.
- Scalar movement is `29.38%`.

#### Suspected lowering pattern

- Branch result staging.
- Accumulator read-modify-write.
- Repeated loading of the merge result for `== S` and `!= N` checks.

#### Confidence

- Supported by profile: local movement remains the dominant repeated access path.
- Supported by source shape: the repeated branch-and-counter structure explains the load/store churn.

### `fact_intersect_kernel.sm`

#### Source shape

- Nine explicit intersection cases cover all relevant quad pairings.
- Each case checks the resulting quad against `T`, `F`, `N`, and `S`, updating the corresponding counters.
- The shape is branch-heavy, with a large number of repeated comparisons.

#### Scalar movement evidence

- `LoadVar`: 49
- `StoreVar`: 22
- `LoadVar + StoreVar`: 71
- `LoadVar` is top opcode #1 and `StoreVar` is top opcode #2.
- Scalar movement is `23.36%`.

#### Suspected lowering pattern

- Branch-discriminant reload.
- Counter staging for the classification buckets.
- Temporary expression staging for the repeated equality checks.

#### Confidence

- Supported by profile: locals are still the leading movement pattern.
- Supported by source shape: repeated classification branches are the visible source-level cause.

### `delta_like_kernel.sm`

#### Source shape

- Two helper functions, `old_state_for` and `new_state_for`, provide explicit old/new quad states.
- The main loop walks 14 cases and applies six transition predicates per case.
- The workload is explicitly transition-classification shaped.

#### Scalar movement evidence

- `LoadVar`: 691
- `StoreVar`: 343
- `LoadVar + StoreVar`: 1034
- `LoadVar` is top opcode #1 and `StoreVar` is top opcode #2.
- Scalar movement is `34.65%`.

#### Suspected lowering pattern

- Old/new local reloads for transition predicates.
- Transition counter read-modify-write.
- Helper argument/result staging around `old_state_for` and `new_state_for`.

#### Confidence

- Confirmed by profile: the local movement is large and stable.
- Supported by source shape: transition classification naturally forces repeated locals.

### `andromeda_fact_wave_64.sm`

#### Source shape

- Two helper functions generate wave states and paired inference states from loop indices.
- The main loop runs 64 iterations, computes `merged` and `consensus`, and updates several counters.
- This is the synthetic mixed workload most closely resembling a larger inference wave.

#### Scalar movement evidence

- `LoadVar`: 2023
- `StoreVar`: 1367
- `LoadVar + StoreVar`: 3390
- `LoadVar` is top opcode #1 and `StoreVar` is top opcode #2.
- Scalar movement is `41.40%`.

#### Suspected lowering pattern

- Loop counter local traffic.
- Counter updates in the classification branches.
- Helper argument/result staging across `wave_state` and `wave_pair`.
- Temporary staging for `observed`, `inferred`, `merged`, and `consensus`.

#### Confidence

- Confirmed by profile: scalar movement is the dominant family.
- Supported by source shape: the workload uses loop-carried values and repeated local staging throughout.

### `andromeda_fact_wave_256.sm`

#### Source shape

- Same shape as the 64-lane wave, scaled to 256 iterations.
- The same helper-generated state paths and merged/consensus counters are exercised.
- This larger fixture is the best evidence that the movement pattern scales with the workload shape itself.

#### Scalar movement evidence

- `LoadVar`: 8071
- `StoreVar`: 5447
- `LoadVar + StoreVar`: 13518
- `LoadVar` is top opcode #1 and `StoreVar` is top opcode #2.
- Scalar movement is `41.47%`.
- This is the ignored/local workload.

#### Suspected lowering pattern

- Same as the 64-lane wave, but with more loop-carried repetitions.
- The stable ratio suggests structural source/lowering pressure rather than a tiny-fixture artifact.

#### Confidence

- Confirmed by profile.
- Supported by source shape.

## Cross-Corpus Lowering Patterns

| Pattern | Evidence | Affected fixtures | Confidence | Follow-up needed |
|---|---|---|---|---|
| Loop counter local traffic | `andromeda_fact_wave_64.sm` and `andromeda_fact_wave_256.sm` scale the same ratio while running explicit loops. | `quad_logic_storm.sm`, `quad_match_dispatch.sm`, `andromeda_fact_wave_64.sm`, `andromeda_fact_wave_256.sm` | Supported by profile | Compare loop shapes that differ only in counter/update style. |
| Accumulator read-modify-write | Every workload maintains counters or running totals; `StoreVar` stays near the top. | All fixtures | Confirmed by profile | Inspect whether accumulator staging is a lowering artifact or a necessary source shape. |
| Branch discriminant reload | Repeated `if`/`match` comparisons imply the same value is loaded for multiple tests. | `quad_match_dispatch.sm`, `fact_merge_kernel.sm`, `fact_intersect_kernel.sm`, `delta_like_kernel.sm` | Supported by source shape | Compare equivalent branch forms to see whether reloads are avoidable. |
| Match arm result staging | `match` dispatch and branch classification both stage results through locals. | `quad_match_dispatch.sm`, `fact_intersect_kernel.sm` | Supported by source shape | Inspect emitted shape for repeated result staging. |
| Temporary expression staging | Complex quad expressions and intermediate `let` bindings are stored before being reused. | `quad_logic_storm.sm`, `andromeda_fact_wave_64.sm`, `andromeda_fact_wave_256.sm` | Hypothesis | Add a source-shape comparison with fewer intermediate bindings. |
| Helper argument/result staging | Helper functions appear in multiple workloads and likely force values through locals. | `quad_logic_storm.sm`, `quad_match_dispatch.sm`, `delta_like_kernel.sm`, `andromeda_fact_wave_64.sm`, `andromeda_fact_wave_256.sm` | Supported by source shape | Compare helper-heavy and helper-free variants. |
| Old/new transition local reload | Transition predicates touch both old and new states repeatedly in the delta workload. | `delta_like_kernel.sm` | Supported by profile | Audit whether the transition shape itself forces the reloads. |

## What This Audit Suggests

- Loop-carried counters and accumulators explain a large fraction of `StoreVar`.
- Repeated local reads explain `LoadVar` dominance.
- `match` / `if` dispatch likely reloads discriminants and stages branch-local results.
- Temporary local staging is contributing to the pressure.
- Helper boundaries likely add movement, but they are not the dominant family.
- The repeated `LoadVar` / `StoreVar` shape looks structural and source-driven, not a one-off anomaly.

## What This Audit Does Not Prove

- No code change is safe yet.
- No speedup is guaranteed.
- No SemCode, lowering, or VM change is approved.
- No frame-layout change is justified yet.
- No exact optimization target is proven.

## Candidate Root Causes

| Candidate | Evidence | Confidence | Notes |
|---|---|---|---|
| Loop counter load/store | Loop-heavy fixtures and the scaled wave workloads preserve the same local-movement ratio. | Supported by profile | Likely a major contributor in the wave workloads. |
| Accumulator update pattern | All fixtures use counters or running totals that are repeatedly read and written. | Confirmed by profile | This is the clearest repeated pattern across the corpus. |
| Local temporary spilling | Complex expressions and intermediate values are staged through locals. | Hypothesis | Needs lowering or source-shape comparison before any code change. |
| Repeated match/if reloads | Branch-heavy fixtures show repeated discriminant tests and branch-local updates. | Supported by source shape | Especially plausible in the match and transition workloads. |
| Helper call argument/result staging | Helpers appear in several fixtures and plausibly force extra movement. | Supported by source shape | Not dominant, but measurable in the profile shape. |
| Frame/local access overhead | The consistent top-two `LoadVar` / `StoreVar` pattern suggests local-slot access may be a secondary cost. | Hypothesis | Requires deeper lowering/VM inspection before touching internals. |

## Recommended Next Slice

VM-M6: docs(sm-vm): compare equivalent source shapes for scalar movement

This is the best next step if the goal is to separate loop shape, branch shape, helper boundaries, and temporary staging without changing code. If that comparison stays broad, the next fallback would be a narrower lowering-implementation audit.

## Non-claims

This document does not claim:

- VM performance improved;
- runtime behavior changed;
- verifier behavior changed;
- SemCode format changed;
- parser/typechecker/lowering changed;
- Pulsar P5-A is reopened;
- scalar optimization is approved;
- any code change is safe without follow-up proof.

## Validation

Commands run:

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/quad_logic_storm.sm`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/quad_match_dispatch.sm`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/fact_merge_kernel.sm`
- `sed -n '1,260p' crates/sm-vm/tests/fixtures/profiling/fact_intersect_kernel.sm`
- `sed -n '1,300p' crates/sm-vm/tests/fixtures/profiling/delta_like_kernel.sm`
- `sed -n '1,360p' crates/sm-vm/tests/fixtures/profiling/andromeda_fact_wave_64.sm`
- `sed -n '1,420p' crates/sm-vm/tests/fixtures/profiling/andromeda_fact_wave_256.sm`
- `git diff --check`
- `cargo fmt --check`

Results:

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture` passed.
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture` passed.
- `git diff --check` passed.
- `cargo fmt --check` failed due to pre-existing unrelated formatting drift in `crates/prom-ui-backend-native/src/lib.rs`.
