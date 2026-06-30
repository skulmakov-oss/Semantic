# sm-vm VM-M9 Scalar Movement Source-Shape Fixture Evidence

## Status

VM-M9 records profiling evidence from VM-M8 scalar movement source-shape fixtures.

This document does not approve VM optimization, lowering changes, fixture changes, or runtime changes.

## Context

- VM-M7 specified the scalar movement micro-fixture matrix.
- VM-M8 added the controlled fixture pairs.
- VM-M9 records the measured evidence from those pairs.

## Method

- Re-ran `vm-profile` workload tests.
- Captured pair-level source-shape reports.
- Used the existing `VmOpcodeProfile`.
- Did not change tests, fixtures, VM, verifier, SemCode, parser, or lowering.

## Commands

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored`
- `git diff --check`
- `cargo fmt --check`
- `git status --short`

## Evidence Boundary

Pair equivalence is enforced by matching fixture-local assertions, not by a shared returned-value comparison at the Rust harness level.

This is the strongest equivalence boundary available for VM-M9 because no separate result-inspection API was introduced in VM-M8.

## Per-Fixture Profile Summary

| Fixture | Total | LoadVar | StoreVar | Load+Store | Scalar % | Quad family % | Control flow % | Integer ops % | Calls % |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `quad_logic_storm.sm` | 7320 | 1845 | 1237 | 3082 | 42.10% | 15.30% | 23.88% | 14.21% | 3.51% |
| `fact_merge_kernel.sm` | 177 | 30 | 22 | 52 | 29.38% | 32.77% | 20.34% | 15.82% | 0.56% |
| `fact_intersect_kernel.sm` | 304 | 49 | 22 | 71 | 23.36% | 33.88% | 32.57% | 8.55% | 0.33% |
| `delta_like_kernel.sm` | 2984 | 691 | 343 | 1034 | 34.65% | 19.84% | 30.43% | 9.52% | 1.91% |
| `quad_match_dispatch.sm` | 4042 | 906 | 690 | 1596 | 39.49% | 15.73% | 25.93% | 12.64% | 4.77% |
| `andromeda_fact_wave_64.sm` | 8188 | 2023 | 1367 | 3390 | 41.40% | 14.77% | 25.55% | 14.23% | 3.14% |
| `andromeda_fact_wave_256.sm` | 32596 | 8071 | 5447 | 13518 | 41.47% | 14.83% | 25.64% | 14.10% | 3.14% |
| `scalar_helper_boundary_helper.sm` | 1706 | 404 | 280 | 684 | 40.09% | 14.13% | 23.92% | 16.65% | 3.81% |
| `scalar_helper_boundary_inline.sm` | 1514 | 340 | 216 | 556 | 36.72% | 15.92% | 26.95% | 18.76% | 0.07% |
| `scalar_temp_staging_named.sm` | 4310 | 1092 | 700 | 1792 | 41.58% | 15.50% | 26.17% | 12.83% | 2.99% |
| `scalar_temp_staging_direct.sm` | 4222 | 1048 | 604 | 1652 | 39.13% | 17.05% | 26.72% | 13.10% | 3.06% |
| `scalar_dispatch_match.sm` | 4042 | 906 | 690 | 1596 | 39.49% | 15.73% | 25.93% | 12.64% | 4.77% |
| `scalar_dispatch_if_chain.sm` | 4282 | 1026 | 750 | 1776 | 41.48% | 14.85% | 25.88% | 11.93% | 4.51% |
| `scalar_loop_accumulator_looped.sm` | 256 | 51 | 35 | 86 | 33.59% | 7.03% | 25.00% | 28.12% | 0.39% |
| `scalar_loop_accumulator_explicit.sm` | 77 | 18 | 18 | 36 | 46.75% | 2.60% | 0.00% | 46.75% | 1.30% |
| `scalar_branch_counters_local.sm` | 1928 | 413 | 301 | 714 | 37.03% | 14.52% | 28.42% | 14.52% | 3.37% |
| `scalar_branch_counters_return_value.sm` | 2576 | 541 | 429 | 970 | 37.66% | 13.66% | 27.17% | 14.91% | 5.01% |
| `scalar_transition_old_new_repeated.sm` | 2984 | 691 | 343 | 1034 | 34.65% | 19.84% | 30.43% | 9.52% | 1.91% |
| `scalar_transition_old_new_packed_code.sm` | 4863 | 1101 | 503 | 1604 | 32.98% | 18.42% | 33.46% | 10.73% | 1.75% |

## Abbreviated Top Opcodes

The test output printed `summary_top_n(12)` for each fixture. The lists below are abbreviated to the opcodes most relevant to scalar movement diagnosis.

- `quad_logic_storm.sm`: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `LoadI32`, `CmpEq`, `LoadQ`, `AddI32`, `Ret`, `ModI32`, `CmpNe`, `Call`
- `fact_merge_kernel.sm`: `LoadQ`, `LoadVar`, `StoreVar`, `Jmp`, `LoadI32`, `JmpIf`, `AddI32`, `CmpEq`, `QOr`, `CmpNe`, `Assert`, `Ret`
- `fact_intersect_kernel.sm`: `Jmp`, `LoadQ`, `LoadVar`, `CmpEq`, `JmpIf`, `StoreVar`, `LoadI32`, `AddI32`, `QAnd`, `Assert`, `Ret`
- `delta_like_kernel.sm`: `LoadVar`, `Jmp`, `StoreVar`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `BoolAnd`, `CmpNe`, `AddI32`, `Ret`, `Call`
- `quad_match_dispatch.sm`: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `AddI32`, `Ret`, `Call`, `CmpI32Lt`, `ModI32`
- `andromeda_fact_wave_64.sm`: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `LoadI32`, `CmpEq`, `LoadQ`, `AddI32`, `Ret`, `ModI32`, `Call`, `CmpI32Lt`
- `andromeda_fact_wave_256.sm`: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `LoadI32`, `CmpEq`, `LoadQ`, `AddI32`, `Ret`, `ModI32`, `Call`, `CmpI32Lt`
- `scalar_helper_boundary_helper.sm`: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `LoadI32`, `CmpEq`, `LoadQ`, `AddI32`, `Ret`, `ModI32`, `Call`, `CmpI32Lt`
- `scalar_helper_boundary_inline.sm`: `LoadVar`, `Jmp`, `StoreVar`, `JmpIf`, `LoadI32`, `CmpEq`, `LoadQ`, `AddI32`, `ModI32`, `CmpI32Lt`, `QOr`, `BoolAnd`
- `scalar_temp_staging_named.sm`: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `AddI32`, `Ret`, `ModI32`, `Call`, `CmpI32Lt`
- `scalar_temp_staging_direct.sm`: `LoadVar`, `Jmp`, `StoreVar`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `AddI32`, `QAnd`, `Ret`, `ModI32`, `Call`
- `scalar_dispatch_match.sm`: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `AddI32`, `Ret`, `Call`, `CmpI32Lt`, `ModI32`
- `scalar_dispatch_if_chain.sm`: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `AddI32`, `Ret`, `Call`, `CmpI32Lt`, `ModI32`
- `scalar_loop_accumulator_looped.sm`: `LoadVar`, `StoreVar`, `JmpIf`, `LoadI32`, `Jmp`, `AddI32`, `CmpEq`, `CmpI32Lt`, `ModI32`, `BoolAnd`, `TupleGet`, `Assert`
- `scalar_loop_accumulator_explicit.sm`: `LoadI32`, `LoadVar`, `StoreVar`, `AddI32`, `CmpEq`, `Assert`, `Ret`
- `scalar_branch_counters_local.sm`: `LoadVar`, `Jmp`, `StoreVar`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `AddI32`, `CmpI32Lt`, `Ret`, `ModI32`, `BoolAnd`
- `scalar_branch_counters_return_value.sm`: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `LoadI32`, `CmpEq`, `LoadQ`, `Ret`, `AddI32`, `Call`, `CmpI32Lt`, `ModI32`
- `scalar_transition_old_new_repeated.sm`: `LoadVar`, `Jmp`, `StoreVar`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `BoolAnd`, `CmpNe`, `AddI32`, `Ret`, `Call`
- `scalar_transition_old_new_packed_code.sm`: `LoadVar`, `Jmp`, `CmpEq`, `JmpIf`, `StoreVar`, `LoadI32`, `LoadQ`, `BoolAnd`, `Ret`, `Call`, `AddI32`, `CmpI32Lt`

## Pair-Level Comparisons

| Pair | Fixture A | Fixture B | A Scalar % | B Scalar % | Delta Count | Delta Ratio | Direction |
|---|---|---|---:|---:|---:|---:|---|
| Helper boundary | `scalar_helper_boundary_helper.sm` | `scalar_helper_boundary_inline.sm` | 40.09% | 36.72% | 128 | 3.37% | Helper-heavy higher |
| Temporary staging | `scalar_temp_staging_named.sm` | `scalar_temp_staging_direct.sm` | 41.58% | 39.13% | 140 | 2.45% | Named staging higher |
| Dispatch shape | `scalar_dispatch_match.sm` | `scalar_dispatch_if_chain.sm` | 39.49% | 41.48% | -180 | -1.99% | If-chain higher |
| Looped accumulator | `scalar_loop_accumulator_looped.sm` | `scalar_loop_accumulator_explicit.sm` | 33.59% | 46.75% | 50 | -13.16% | Mixed |
| Branch counters | `scalar_branch_counters_local.sm` | `scalar_branch_counters_return_value.sm` | 37.03% | 37.66% | -256 | -0.62% | Return-value slightly higher |
| Old/new transition reload | `scalar_transition_old_new_repeated.sm` | `scalar_transition_old_new_packed_code.sm` | 34.65% | 32.98% | -570 | 1.67% | Mixed |

## Pair Notes

### A - Helper Boundary

- The helper-heavy form keeps `scalar_movement` above the inline form.
- `Call` / `Ret` pressure is also higher in the helper-heavy variant.
- The evidence strengthens the helper-boundary hypothesis, but it does not prove helper calls are intrinsically bad.
- Pair equivalence is enforced by fixture-local assertions, not by a shared returned-value comparison.

### B - Temporary Staging

- The named-staging variant carries a higher scalar-movement ratio than the direct-expression variant.
- The direct form still remains scalar-heavy, so the difference is real but not decisive.
- The evidence strengthens the temporary-staging hypothesis.
- Pair equivalence is enforced by fixture-local assertions, not by a shared returned-value comparison.

### C - Match vs If-Chain

- The if-chain variant has a slightly higher scalar-movement ratio than the match variant.
- The pressure shifts mostly into control flow and comparison opcodes rather than into a clean scalar win for either side.
- The evidence is mixed; it does not support a strong conclusion that match or if-chain is universally better.
- Pair equivalence is enforced by fixture-local assertions, not by a shared returned-value comparison.

### D - Looped Accumulator

- The looped variant has a lower scalar-movement ratio than the explicit variant, but a higher absolute scalar count than the explicit variant.
- The explicit fixture is much smaller overall, so the pair is useful but not cleanly isolating loop overhead by ratio alone.
- The evidence is mixed and only weakly supports the loop-carried-state hypothesis.
- Pair equivalence is enforced by fixture-local assertions, not by a shared returned-value comparison.

### E - Branch Counters

- The return-value classification variant has a slightly higher scalar-movement ratio and higher `StoreVar` count than the branch-local form.
- That weakens the simple claim that branch-local counter updates are the main driver by themselves.
- The evidence is mixed, with no strong basis for a source-style prescription.
- Pair equivalence is enforced by fixture-local assertions, not by a shared returned-value comparison.

### F - Old/New Transition Reload

- The repeated-old/new variant has the higher scalar-movement ratio, while the packed-code variant has the higher absolute scalar count because it performs more total work.
- This strengthens the repeated-reload hypothesis only modestly.
- The result is mixed overall because ratio and raw count point in different directions.
- Pair equivalence is enforced by fixture-local assertions, not by a shared returned-value comparison.

## Hypothesis Impact

| Hypothesis | Evidence | Impact | Confidence | Follow-up |
|---|---|---|---|---|
| Helper argument/result staging | Helper-heavy keeps scalar movement and call/return pressure above the inline form. | Strengthened | Strong | If this becomes a code candidate, compare equivalent helper-light shapes before touching lowering. |
| Named temporary expression staging | Named staging is consistently above the direct form on scalar-movement ratio. | Strengthened | Medium | A second-generation temporary-light comparison would make this sharper. |
| Match vs if-chain dispatch | The pair is close, and the direction differs between ratio and surrounding control-flow work. | Mixed | Medium | Needs a tighter source-equivalence comparison if this becomes an implementation question. |
| Looped accumulator movement | The looped variant is not a clean scalar-movement win over the explicit variant by ratio alone. | Inconclusive | Weak | A more controlled accumulator pair would help before any implementation audit. |
| Branch-local counter staging | The return-value variant is not lower on scalar movement, and `StoreVar` does not clearly improve. | Weakened | Weak | Needs a cleaner counter-staging pair before any lowering judgment. |
| Repeated old/new reloads | The repeated form is denser on scalar movement, though the packed form has more total work. | Strengthened | Medium | Compare a tighter transition pair or isolate the counter path if this becomes a candidate. |

## Interpretation

- The helper-boundary and temporary-staging hypotheses are the clearest wins in this matrix.
- The old/new transition result leans in the same direction, but not strongly enough to call it a code candidate.
- The dispatch, looped-accumulator, and branch-counter comparisons are mixed or weak.
- The evidence supports a next step that is still measurement-focused rather than implementation-focused.

## Recommended Next Slice

Recommended next PR:

`VM-M10: test(sm-vm): add second-generation scalar movement micro-fixtures`

Why:

- The matrix now has useful signal, but several comparisons remain mixed or ratio-sensitive.
- A second generation of fixtures can isolate the weak spots more cleanly before any lowering audit or implementation candidate selection.
- This is the safest next step because it preserves the evidence-first boundary.

Alternative options:

- `VM-M10: docs(sm-vm): select first scalar movement implementation audit candidate` if the project wants to make a narrower evidence decision before any new fixtures.
- `VM-M10: docs(sm-vm): audit lowering implementation for strongest scalar movement delta` only after a much cleaner controlled pair exists.
- `VM-M10: docs(sm-vm): specify result-equivalence checking for profiling fixtures` if the main limitation becomes fixture-level equivalence evidence rather than scalar movement signal.
- `VM-M11` records measured evidence from the VM-M10 G2 fixture set in `docs/roadmap/sm_vm_vm_m11_second_generation_scalar_movement_evidence.md`.

## Non-claims

This document does not claim:

- VM performance improved;
- runtime behavior changed;
- verifier behavior changed;
- SemCode format changed;
- parser/typechecker/lowering changed;
- fixtures changed;
- Pulsar P5-A is reopened;
- scalar optimization is approved;
- any code change is safe without follow-up proof.

## Validation

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture` passed
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture` passed
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads` passed
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored` passed
- `git diff --check` passed
- `cargo fmt --check` failed due to pre-existing unrelated formatting drift in `crates/prom-ui-backend-native/src/lib.rs`
- `git status --short` still shows unrelated pre-existing worktree changes outside this VM-M9 slice
