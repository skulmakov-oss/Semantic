# sm-vm VM-M4 Scalar Movement Audit

## Status

VM-M4 audits scalar movement pressure after VM-M3.

This document does not approve VM optimization or runtime changes.

## Context

VM-M3 showed `scalar_movement` dominates the profiled `sm-vm` workload corpus.

The recorded evidence was stable across the default corpus and the larger local workload:

- default corpus: `scalar_movement = 9225 / 23015 = 40.08%`
- `andromeda_fact_wave_256.sm`: `scalar_movement = 13518 / 32596 = 41.47%`

P5-A remains blocked.

The next audit target is `LoadVar` / `StoreVar` pressure, not a runtime optimization proposal.

## Method

- Used the VM-M2 runtime profile family summaries.
- Re-ran the existing profiling workloads.
- Compared the family summaries with the source-level workload shapes implied by each fixture.
- No runtime code changes.
- No fixture changes.

## Commands

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture`
- `git diff --check`
- `cargo fmt --check`

`cargo fmt --check` failed due to pre-existing unrelated formatting drift in `crates/prom-ui-backend-native/src/lib.rs`. This audit did not modify Rust source.

## Scalar Movement Summary

| Fixture | Total | LoadVar | StoreVar | Load+Store | Scalar Movement % | Notes |
|---|---:|---:|---:|---:|---:|---|
| `quad_logic_storm.sm` | 7320 | 1845 | 1237 | 3082 | 42.10% | Top opcodes begin with `LoadVar` #1 and `StoreVar` #2; likely repeated temporary evaluation and accumulator traffic. |
| `quad_match_dispatch.sm` | 4042 | 906 | 690 | 1596 | 39.49% | `LoadVar` #1 and `StoreVar` #2; likely match dispatch reloads plus counter updates. |
| `fact_merge_kernel.sm` | 177 | 30 | 22 | 52 | 29.38% | `LoadVar` #1 and `StoreVar` #2; small kernel, but locals still dominate the tiny instruction budget. |
| `fact_intersect_kernel.sm` | 304 | 49 | 22 | 71 | 23.36% | `LoadVar` #1 and `StoreVar` #2; small kernel, lower absolute pressure but same local pattern. |
| `delta_like_kernel.sm` | 2984 | 691 | 343 | 1034 | 34.65% | `LoadVar` #1 and `StoreVar` #2; explicit old/new transition checks likely force repeated local reads. |
| `andromeda_fact_wave_64.sm` | 8188 | 2023 | 1367 | 3390 | 41.40% | `LoadVar` #1 and `StoreVar` #2; range-loop synthetic wave with repeated state generation and accumulator updates. |
| `andromeda_fact_wave_256.sm` | 32596 | 8071 | 5447 | 13518 | 41.47% | ignored/local workload; `LoadVar` #1 and `StoreVar` #2; same shape at larger scale. |

## Per-Fixture Audit

### `quad_logic_storm.sm`

#### Evidence

- total: 7320
- `LoadVar`: 1845
- `StoreVar`: 1237
- `LoadVar + StoreVar`: 3082
- scalar movement: 42.10%
- top opcode positions: `LoadVar` #1, `StoreVar` #2

#### Source-shape notes

- The workload is a quad-heavy logical stress test, but the profile shows the logical operations are not the main cost.
- The repeated movement suggests intermediate quad values are being written to locals and then read back for subsequent checks.

#### Likely causes

- Confirmed by profile: local variable traffic is large and persistent.
- Supported by source shape: repeated evaluation of logical expressions and accumulator updates.
- Hypothesis: temporary values are being spilled to locals rather than kept in a cheaper short-lived form.

#### Confidence

- Confirmed by profile

### `quad_match_dispatch.sm`

#### Evidence

- total: 4042
- `LoadVar`: 906
- `StoreVar`: 690
- `LoadVar + StoreVar`: 1596
- scalar movement: 39.49%
- top opcode positions: `LoadVar` #1, `StoreVar` #2

#### Source-shape notes

- The workload emphasizes quad `match` dispatch and branch structure.
- Match arms and conditionals likely force repeated reads of the discriminant value and accumulator writes for classification counters.

#### Likely causes

- Confirmed by profile: local variable traffic is dominant.
- Supported by source shape: repeated `match` / branch dispatch over quad states.
- Hypothesis: lowering shape reloads the same value more than once across dispatch paths.

#### Confidence

- Supported by profile

### `fact_merge_kernel.sm`

#### Evidence

- total: 177
- `LoadVar`: 30
- `StoreVar`: 22
- `LoadVar + StoreVar`: 52
- scalar movement: 29.38%
- top opcode positions: `LoadVar` #1, `StoreVar` #2

#### Source-shape notes

- This is a small evidence-merge kernel.
- Even with a tiny absolute instruction count, local counters and merge intermediates still dominate the top of the profile.

#### Likely causes

- Confirmed by profile: local traffic is the leading movement family.
- Supported by source shape: merge kernels naturally carry source and accumulator values through locals.
- Hypothesis: helper-style bookkeeping and result staging contribute proportionally more in small kernels.

#### Confidence

- Supported by profile

### `fact_intersect_kernel.sm`

#### Evidence

- total: 304
- `LoadVar`: 49
- `StoreVar`: 22
- `LoadVar + StoreVar`: 71
- scalar movement: 23.36%
- top opcode positions: `LoadVar` #1, `StoreVar` #2

#### Source-shape notes

- This kernel is also small and branchy, with evidence intersection semantics.
- The absolute movement is low compared with the larger workloads, but locals still lead the profile.

#### Likely causes

- Confirmed by profile: locals are still the main repeated access path.
- Supported by source shape: classification over multiple states likely reloads values to test each branch.
- Hypothesis: movement is partly a consequence of branch-heavy lowering and small-kernel bookkeeping.

#### Confidence

- Supported by profile

### `delta_like_kernel.sm`

#### Evidence

- total: 2984
- `LoadVar`: 691
- `StoreVar`: 343
- `LoadVar + StoreVar`: 1034
- scalar movement: 34.65%
- top opcode positions: `LoadVar` #1, `StoreVar` #2

#### Source-shape notes

- This workload explicitly models old/new transition classification.
- The source shape strongly suggests repeated access to both old and new values, plus counters for each transition class.

#### Likely causes

- Confirmed by profile: local traffic remains dominant.
- Supported by source shape: old/new transition predicates naturally need repeated local reads and updates.
- Hypothesis: transition bookkeeping is creating a large volume of read/write churn for counters and intermediate predicates.

#### Confidence

- Confirmed by profile

### `andromeda_fact_wave_64.sm`

#### Evidence

- total: 8188
- `LoadVar`: 2023
- `StoreVar`: 1367
- `LoadVar + StoreVar`: 3390
- scalar movement: 41.40%
- top opcode positions: `LoadVar` #1, `StoreVar` #2

#### Source-shape notes

- This is the small Andromeda-shaped synthetic wave.
- The workload mixes generated states, merge/intersect-like logic, and classification counters across a looped structure.

#### Likely causes

- Confirmed by profile: scalar movement is the largest family.
- Supported by source shape: loop counters, generated states, and accumulator updates all imply repeated locals.
- Hypothesis: repeated helper-like state staging and loop-carried values are driving the traffic.

#### Confidence

- Confirmed by profile

### `andromeda_fact_wave_256.sm`

#### Evidence

- total: 32596
- `LoadVar`: 8071
- `StoreVar`: 5447
- `LoadVar + StoreVar`: 13518
- scalar movement: 41.47%
- top opcode positions: `LoadVar` #1, `StoreVar` #2
- class: ignored/local workload

#### Source-shape notes

- This workload is the larger synthetic wave and scales the same shape as the 64-lane version.
- The repeated locals scale almost linearly with the larger loop body, which supports the interpretation that the movement is structural, not an artifact of a tiny fixture.

#### Likely causes

- Confirmed by profile: scalar movement remains dominant at larger scale.
- Supported by source shape: loop-carried state, generated values, and counters amplify local traffic.
- Hypothesis: the current frame/local shape keeps values flowing through locals instead of keeping them in a narrower short-lived form.

#### Confidence

- Confirmed by profile

## Cross-Corpus Findings

- `LoadVar` is consistently the top opcode family member across all fixtures.
- `StoreVar` is consistently second or near second.
- Scalar movement remains high even when `quad_logic` is low.
- The larger synthetic workload preserves the same shape as the default corpus, which argues against the movement being a one-off artifact.
- Helper-call pressure exists, but `calls` is not the dominant family.
- Control flow is a major secondary cost, but it does not explain the scalar movement peak by itself.

## Candidate Root Causes

| Candidate | Evidence | Confidence | Notes |
|---|---|---|---|
| Loop counter load/store | The looped synthetic fixtures and repeated counters line up with the dominant `LoadVar` / `StoreVar` pattern. | Supported by source shape | Likely contributes in the `andromeda_fact_wave_*` workloads. |
| Accumulator update pattern | All fixtures use counters or running totals that are stored and reloaded across iterations or branches. | Supported by profile | Especially visible in the larger workloads and branch-heavy kernels. |
| Local temporary spilling | The repeated top-two `LoadVar` / `StoreVar` positions suggest intermediates are being staged through locals. | Hypothesis | Needs lowering/frame inspection before any code change. |
| Repeated match/if reloads | The match-heavy and branch-heavy fixtures likely reload discriminants and intermediate values before comparisons. | Supported by source shape | Especially plausible in `quad_match_dispatch.sm` and `delta_like_kernel.sm`. |
| Helper call argument/result movement | Some fixtures likely cross helper boundaries or at least model helper-style staging through locals. | Supported by source shape | `calls` is not dominant, but it can still add measurable movement. |
| Frame/local access overhead | The repeated and consistent load/store pattern suggests the current local-slot access shape may be expensive. | Hypothesis | This is a plausible follow-up area, not a proven fix target. |

## What VM-M4 Does Not Prove

This audit does not prove:

- a specific optimization is safe;
- SemCode format should change;
- lowering should change;
- VM frame layout should change;
- scalar movement can be reduced without semantic risk;
- any end-to-end speedup.

## Recommended Next Slice

VM-M5: docs(sm-vm): audit lowering shape for scalar movement

If follow-up evidence is needed before any code change, the next audit should inspect how source constructs and lowering shape produce repeated local traffic.

VM-M5 audits source/lowering shapes behind scalar movement in [docs/roadmap/sm_vm_vm_m5_lowering_shape_scalar_movement_audit.md](sm_vm_vm_m5_lowering_shape_scalar_movement_audit.md).

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
- `git diff --check`
- `cargo fmt --check`

Results:

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture` passed.
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture` passed.
- `git diff --check` passed.
- `cargo fmt --check` failed due to pre-existing unrelated formatting drift in `crates/prom-ui-backend-native/src/lib.rs`.
