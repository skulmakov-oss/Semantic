# sm-vm VM-M11 Second-Generation Scalar Movement Evidence

## Status

VM-M11 records profiling evidence from the VM-M10 second-generation scalar movement fixtures.

This document does not approve VM optimization, lowering changes, fixture changes, or runtime changes.

## Context

- VM-M8 added first-generation source-shape fixtures.
- VM-M9 recorded first-generation evidence.
- VM-M10 added second-generation G2 fixtures.
- VM-M11 records the measured G2 evidence.
- The ignored local `andromeda_fact_wave_256.sm` workload remained at `41.47%` scalar movement, matching VM-M9 and supporting corpus stability.

## Method

- Re-ran `vm-profile` workload tests.
- Captured G2 pair-level source-shape reports.
- Used the existing `VmOpcodeProfile` runtime profiling path.
- Did not change tests, fixtures, VM, verifier, SemCode, parser, or lowering.

## Commands

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads --no-run`
- `cargo clippy -p sm-vm --all-targets --all-features -- -D warnings`
- `cargo check -p sm-vm --no-default-features`
- `git diff --check`
- `cargo fmt --check`
- `git status --short`

## Evidence Boundary

Pair equivalence is enforced by matching fixture-local assertions, not by a shared returned-value comparison at the Rust harness level.

VM-M11 does not introduce result-inspection APIs.

## G2 Per-Fixture Profile Summary

| Fixture | Total | LoadVar | StoreVar | Load+Store | Scalar % | Quad family % | Control flow % | Integer ops % | Calls % |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `scalar_helper_boundary_single_call_helper.sm` | 592 | 132 | 88 | 220 | 37.16% | 15.03% | 27.36% | 14.86% | 2.87% |
| `scalar_helper_boundary_single_call_inline.sm` | 544 | 116 | 72 | 188 | 34.56% | 16.36% | 29.78% | 16.18% | 0.18% |
| `scalar_helper_boundary_call_chain_helper.sm` | 1898 | 468 | 344 | 812 | 42.78% | 12.75% | 21.50% | 14.91% | 6.80% |
| `scalar_helper_boundary_call_chain_inline.sm` | 1514 | 340 | 216 | 556 | 36.72% | 15.98% | 26.95% | 18.69% | 0.07% |
| `scalar_temp_staging_single_use_named.sm` | 1464 | 367 | 279 | 646 | 44.13% | 11.54% | 21.31% | 17.01% | 4.44% |
| `scalar_temp_staging_single_use_direct.sm` | 1368 | 319 | 231 | 550 | 40.20% | 12.35% | 22.81% | 18.20% | 4.75% |
| `scalar_temp_staging_multi_use_named.sm` | 1614 | 400 | 296 | 696 | 43.12% | 12.45% | 21.31% | 17.60% | 4.03% |
| `scalar_temp_staging_multi_use_direct.sm` | 2454 | 576 | 416 | 992 | 40.42% | 13.08% | 21.84% | 18.42% | 5.26% |
| `scalar_dispatch_equalized_match.sm` | 2710 | 606 | 462 | 1068 | 39.41% | 15.83% | 25.83% | 12.62% | 4.76% |
| `scalar_dispatch_equalized_if_chain.sm` | 2870 | 686 | 502 | 1188 | 41.39% | 14.95% | 25.78% | 11.92% | 4.49% |
| `scalar_loop_accumulator_equalized_looped.sm` | 554 | 115 | 79 | 194 | 35.02% | 15.88% | 29.24% | 14.08% | 3.07% |
| `scalar_loop_accumulator_equalized_explicit.sm` | 435 | 90 | 62 | 152 | 34.94% | 18.39% | 27.13% | 15.17% | 3.91% |
| `scalar_transition_equalized_repeated.sm` | 1377 | 311 | 157 | 468 | 33.99% | 21.35% | 27.23% | 10.09% | 2.40% |
| `scalar_transition_equalized_classified.sm` | 1280 | 314 | 206 | 520 | 40.62% | 14.37% | 26.25% | 12.11% | 3.83% |

## Pair-Level Comparisons

| Pair | Fixture A | Fixture B | A Scalar % | B Scalar % | Delta Count | Delta Ratio | Direction |
|---|---|---|---:|---:|---:|---:|---|
| G2-A helper single-call | `scalar_helper_boundary_single_call_helper.sm` | `scalar_helper_boundary_single_call_inline.sm` | 37.16% | 34.56% | 32 | 2.60% | Helper higher |
| G2-B helper call-chain | `scalar_helper_boundary_call_chain_helper.sm` | `scalar_helper_boundary_call_chain_inline.sm` | 42.78% | 36.72% | 256 | 6.06% | Helper higher |
| G2-C temporary single-use | `scalar_temp_staging_single_use_named.sm` | `scalar_temp_staging_single_use_direct.sm` | 44.13% | 40.20% | 96 | 3.92% | Named higher |
| G2-D temporary multi-use | `scalar_temp_staging_multi_use_named.sm` | `scalar_temp_staging_multi_use_direct.sm` | 43.12% | 40.42% | -296 | 2.70% | Named higher by ratio; direct higher in raw load/store count because total work is larger |
| G2-E equalized dispatch | `scalar_dispatch_equalized_match.sm` | `scalar_dispatch_equalized_if_chain.sm` | 39.41% | 41.39% | -120 | -1.98% | If-chain higher |
| G2-F equalized loop accumulator | `scalar_loop_accumulator_equalized_looped.sm` | `scalar_loop_accumulator_equalized_explicit.sm` | 35.02% | 34.94% | 42 | 0.08% | Essentially flat |
| G2-G equalized transition | `scalar_transition_equalized_repeated.sm` | `scalar_transition_equalized_classified.sm` | 33.99% | 40.62% | -52 | -6.64% | Classified higher |

## VM-M9 vs VM-M11 Signal Comparison

| Hypothesis | VM-M9 signal | VM-M11 signal | Direction stable? | Updated confidence |
|---|---|---|---|---|
| Helper boundary staging | Helper-heavy stayed above inline in VM-M9 (`40.09%` vs `36.72%`) | Helper stayed above inline in both G2 pairs (`37.16%` vs `34.56%`; `42.78%` vs `36.72%`) | Yes | Strong |
| Temporary staging | Named stayed above direct in VM-M9 (`41.58%` vs `39.13%`) | Named stayed above direct in both G2 pairs (`44.13%` vs `40.20%`; `43.12%` vs `40.42%`) | Yes | Strong |
| Dispatch shape | VM-M9 remained mixed (`39.49%` match vs `41.48%` if-chain) | VM-M11 equalized pair is still mixed (`39.41%` match vs `41.39%` if-chain) | Yes, but still mixed | Medium |
| Loop accumulator | VM-M9 was ratio-sensitive and inconclusive (`33.59%` looped vs `46.75%` explicit) | VM-M11 equalized pair is essentially flat (`35.02%` vs `34.94%`) | No | Inconclusive |
| Branch counters | VM-M9 slightly weakened the branch-local hypothesis (`37.03%` vs `37.66%`) | Not remeasured in G2 | Not available | Weak |
| Transition reload | VM-M9 modestly supported repeated old/new pressure (`34.65%` repeated vs `32.98%` packed-code) | VM-M11 equalized pair flips direction (`33.99%` repeated vs `40.62%` classified) | No | Inconclusive |

## Pair Notes

### G2-A — Helper Boundary Single-Call

The helper variant remains above the inline variant on scalar movement, but the delta is smaller than the G2-B call-chain pair.

Top opcodes:

- helper: `LoadVar`, `Jmp`, `StoreVar`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `AddI32`, `CmpI32Lt`, `Ret`, `ModI32`, `BoolAnd`
- inline: `LoadVar`, `Jmp`, `StoreVar`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `AddI32`, `CmpI32Lt`, `ModI32`, `BoolAnd`, `Assert`

### G2-B — Helper Boundary Call-Chain

The helper-chain variant amplifies the helper-boundary signal relative to the inline variant.

Top opcodes:

- helper: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `LoadI32`, `CmpEq`, `LoadQ`, `AddI32`, `Ret`, `Call`, `ModI32`, `CmpI32Lt`
- inline: `LoadVar`, `Jmp`, `StoreVar`, `JmpIf`, `LoadI32`, `CmpEq`, `LoadQ`, `AddI32`, `ModI32`, `CmpI32Lt`, `QOr`, `BoolAnd`

### G2-C — Temporary Staging Single-Use

Named single-use temporaries remain above the direct form on scalar movement.

Top opcodes:

- named: `LoadVar`, `StoreVar`, `Jmp`, `LoadI32`, `JmpIf`, `CmpEq`, `AddI32`, `LoadQ`, `Ret`, `ModI32`, `Call`, `CmpI32Lt`
- direct: `LoadVar`, `StoreVar`, `Jmp`, `LoadI32`, `JmpIf`, `CmpEq`, `AddI32`, `LoadQ`, `Ret`, `ModI32`, `Call`, `CmpI32Lt`

### G2-D — Temporary Staging Multi-Use

Named multi-use staging remains above the direct form by ratio, but the raw total-work comparison is less clean because the direct variant executes more instructions overall.

Top opcodes:

- named: `LoadVar`, `StoreVar`, `Jmp`, `LoadI32`, `JmpIf`, `CmpEq`, `AddI32`, `LoadQ`, `Ret`, `ModI32`, `Call`, `CmpI32Lt`
- direct: `LoadVar`, `StoreVar`, `Jmp`, `LoadI32`, `JmpIf`, `CmpEq`, `LoadQ`, `AddI32`, `Ret`, `ModI32`, `Call`, `QOr`

### G2-E — Equalized Dispatch

Equalization did not remove the mixed signal. The if-chain variant remains slightly higher on scalar movement.

Top opcodes:

- match: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `AddI32`, `Ret`, `Call`, `CmpI32Lt`, `ModI32`
- if-chain: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `AddI32`, `Ret`, `Call`, `CmpI32Lt`, `ModI32`

### G2-F — Equalized Loop Accumulator

Equalization removes most of the earlier ratio-vs-absolute-count ambiguity. The looped and explicit forms are now essentially flat.

Top opcodes:

- looped: `LoadVar`, `Jmp`, `StoreVar`, `JmpIf`, `CmpEq`, `LoadI32`, `LoadQ`, `AddI32`, `CmpI32Lt`, `Ret`, `ModI32`, `BoolAnd`
- explicit: `LoadVar`, `Jmp`, `StoreVar`, `LoadI32`, `CmpEq`, `JmpIf`, `LoadQ`, `AddI32`, `Ret`, `ModI32`, `Call`, `Assert`

### G2-G — Equalized Transition

The equalized pair does not preserve the VM-M9 direction. Classified now has the higher scalar movement ratio.

Top opcodes:

- repeated: `LoadVar`, `Jmp`, `StoreVar`, `JmpIf`, `CmpEq`, `LoadQ`, `LoadI32`, `BoolAnd`, `CmpNe`, `AddI32`, `Ret`, `Call`
- classified: `LoadVar`, `StoreVar`, `Jmp`, `JmpIf`, `LoadI32`, `CmpEq`, `LoadQ`, `BoolAnd`, `Ret`, `AddI32`, `Call`, `CmpNe`

## Hypothesis Impact After G2

| Hypothesis | Evidence | Impact | Confidence | Recommended follow-up |
|---|---|---|---|---|
| Helper argument/result staging | Stable across VM-M9 and both G2 helper pairs; call-chain pair strengthens the effect | Strengthened | Strong | `VM-M12: docs(sm-vm): audit helper-boundary lowering shape` |
| Named temporary expression staging | Stable across VM-M9 and both G2 temporary pairs | Strengthened | Strong | `VM-M12: docs(sm-vm): audit temporary-staging lowering shape` |
| Dispatch shape | Still mixed after equalization | Mixed | Medium | Keep on hold until a narrower controlled pair is needed |
| Loop-carried accumulator state | Equalized pair is flat, so the earlier large ratio gap is not stable | Weakened | Weak | No immediate audit candidate |
| Branch-local counter staging | Not remeasured in G2 | Inconclusive | Weak | Keep as baseline-only until a controlled pair exists |
| Repeated old/new transition reload | Direction flips under G2 equalization | Mixed | Medium | Consider a later transition-specific audit only if new evidence isolates it |

## Interpretation

The helper-boundary and temporary-staging hypotheses are now the clearest stable signals.

The dispatch and transition shapes remain mixed after equalization.

The loop accumulator signal is now much less distinctive than in VM-M9.

The evidence supports a docs-only lowering-shape audit next, not a code change.

## Recommended Next Slice

### Option 1

`VM-M12: docs(sm-vm): audit helper-boundary lowering shape`

Use this as the primary next step because helper-boundary pressure remained stable across VM-M9 and VM-M11 and the call-chain pair amplified the signal.

### Option 2

`VM-M12: docs(sm-vm): audit temporary-staging lowering shape`

Use this if the project wants to focus on the other stable signal before touching any implementation candidate list.

### Option 3

`VM-M12: docs(sm-vm): specify result-equivalence checking for profiling fixtures`

Use this if the remaining weakness is still the fixture-local assertion boundary rather than the scalar movement signal itself.

### Option 4

`VM-M12: test(sm-vm): add third-generation scalar movement fixtures`

Use this only if a later pass needs a more tightly controlled comparison matrix.

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
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads --no-run` passed
- `cargo clippy -p sm-vm --all-targets --all-features -- -D warnings` passed
- `cargo check -p sm-vm --no-default-features` passed
- `git diff --check` passed
- `cargo fmt --check` failed due to pre-existing unrelated formatting drift in `crates/prom-ui-backend-native/src/lib.rs`
- `git status --short` still shows unrelated pre-existing worktree changes outside this VM-M11 slice
