# sm-vm VM-M7 Scalar Movement Micro-Fixture Matrix

## Status

VM-M7 specifies controlled micro-fixture pairs for isolating scalar movement pressure.

This document does not add fixtures or approve VM optimization.

## Context

- VM-M3 showed `scalar_movement` dominates.
- VM-M4 showed `LoadVar` / `StoreVar` dominate.
- VM-M5 mapped source/lowering suspects.
- VM-M6 identified missing controlled pairs.

VM-M7 defines the exact experimental matrix for a later test PR.

## Method

- Reviewed VM-M4, VM-M5, and VM-M6 evidence.
- Defined controlled source-shape fixture pairs.
- Did not modify code, tests, or fixtures.

## Matrix Overview

| Group | Pair | Fixture A | Fixture B | Isolates | Expected metric delta |
|---|---|---|---|---|---|
| A | Helper boundary pair | `scalar_helper_boundary_helper.sm` | `scalar_helper_boundary_inline.sm` | Helper argument/result staging | `LoadVar` / `StoreVar`, `Call` / `Ret`, total instructions |
| B | Temporary staging pair | `scalar_temp_staging_named.sm` | `scalar_temp_staging_direct.sm` | Temporary expression staging | `LoadVar` / `StoreVar`, `quad_family`, total instructions |
| C | Match vs if-chain pair | `scalar_dispatch_match.sm` | `scalar_dispatch_if_chain.sm` | Dispatch/discriminant reload shape | `LoadVar`, `StoreVar`, `CmpEq` / `CmpNe`, `Jmp` / `JmpIf`, `scalar_movement %` |
| D | Looped accumulator pair | `scalar_loop_accumulator_looped.sm` | `scalar_loop_accumulator_explicit.sm` | Loop counter and loop-carried accumulator movement | `LoadVar` / `StoreVar`, `Jmp` / `JmpIf`, integer ops, total instructions |
| E | Branch-local counter pair | `scalar_branch_counters_local.sm` | `scalar_branch_counters_return_value.sm` | Branch-local counter staging | `StoreVar`, `LoadVar`, branch/control-flow counts, integer ops |
| F | Old/new transition pair | `scalar_transition_old_new_repeated.sm` | `scalar_transition_old_new_packed_code.sm` | Repeated old/new state reloads | `LoadVar`, `StoreVar`, `CmpEq` / `CmpNe`, integer ops, total instructions |

## Metrics

| Metric | Why it matters |
|---|---|
| `total_instructions` | Normalizes all comparisons |
| `LoadVar` | Direct scalar read pressure |
| `StoreVar` | Direct scalar write pressure |
| `LoadVar + StoreVar %` | Primary scalar movement metric |
| `Call + Ret` | Helper boundary pressure |
| `Jmp + JmpIf` | Loop/control-flow pressure |
| `CmpEq + CmpNe` | Branch/discriminant reload proxy |
| `integer_ops` | Counter and loop arithmetic pressure |
| `quad_family` | Ensures quad logic shape remains comparable |

## Fixture Group Specifications

### Group A - Helper Boundary Pair

Purpose:

Isolate helper argument/result staging.

Semantic equivalence requirement:

The helper-heavy and inline variants must compute the same final counters and results, with the same loop count and the same value classes.

Expected comparison:

- `LoadVar` / `StoreVar` delta between helper and inline forms
- `Call` / `Ret` delta
- total instruction delta

Allowed conclusion:

Helper boundary likely contributes to scalar movement.

Forbidden conclusion:

- helper calls are bad
- lowering is wrong
- VM must inline functions

### Group B - Temporary Staging Pair

Purpose:

Isolate temporary expression staging.

Semantic equivalence requirement:

The named-intermediate and direct-expression variants must preserve the same final counters and results.

Expected comparison:

- `LoadVar` / `StoreVar` delta
- `quad_family` delta
- total instruction delta

Allowed conclusion:

Temporary staging likely contributes to local movement.

Forbidden conclusion:

- let bindings should be avoided
- compiler must eliminate temporaries immediately

### Group C - Match vs If-Chain Pair

Purpose:

Isolate dispatch/discriminant reload shape.

Semantic equivalence requirement:

Both variants must classify the same quad state and update the same counters.

Expected comparison:

- `LoadVar` count
- `StoreVar` count
- `CmpEq` / `CmpNe` count
- `Jmp` / `JmpIf` count
- `scalar_movement %`

Allowed conclusion:

Match or if-chain shape has different reload/control-flow pressure.

Forbidden conclusion:

- match is worse
- if-chain is better
- language syntax should change

### Group D - Looped Accumulator vs Explicit Cases Pair

Purpose:

Isolate loop counter and loop-carried accumulator movement.

Semantic equivalence requirement:

The looped and explicit variants must compute the same final result and cover the same semantic cases.

Expected comparison:

- `LoadVar` / `StoreVar` delta
- `Jmp` / `JmpIf` delta
- integer ops delta
- total instruction delta

Allowed conclusion:

Loop-carried state contributes to scalar movement.

Forbidden conclusion:

- loops should be avoided
- unrolling is an optimization recommendation

### Group E - Branch-Local Counter Pair

Purpose:

Isolate branch-local counter staging.

Semantic equivalence requirement:

The branch-local and return-value classification variants must compute the same final aggregate result.

Expected comparison:

- `StoreVar` count
- `LoadVar` count
- branch/control-flow count
- integer ops count

Allowed conclusion:

Branch-local counter staging likely contributes to `StoreVar` pressure.

Forbidden conclusion:

- branch-local updates are wrong
- language should prefer return-value classification

### Group F - Old/New Transition Pair

Purpose:

Isolate repeated old/new state reloads.

Semantic equivalence requirement:

The repeated-old/new and packed-code variants must compute the same final counters and transition classes.

Expected comparison:

- `LoadVar` count
- `StoreVar` count
- `CmpEq` / `CmpNe` count
- integer ops count
- total instruction count

Allowed conclusion:

Repeated old/new reloads likely contribute to scalar movement.

Forbidden conclusion:

- transition logic should be encoded differently in production
- SemCode should change

## Interpretation Rules

| Observation | Allowed interpretation | Forbidden interpretation |
|---|---|---|
| Helper-heavy has higher `Load+Store` | Helper boundary may contribute | helper calls are bad / must inline |
| Temporary-heavy has higher `Load+Store` | Temporary staging may contribute | let bindings should be avoided |
| Match-heavy differs from if-chain | Dispatch shape may matter | match is worse / if-chain is better |
| Looped variant differs from explicit cases | Loop-carried state may matter | loops should be avoided |
| Branch-local counters have higher `StoreVar` | Branch-local staging may contribute | branch-local updates are wrong |
| Transition-heavy has higher `LoadVar` | Repeated old/new reloads may matter | transition logic should change in production |

## Future Implementation Guardrails

| Guardrail | Requirement |
|---|---|
| Fixtures must compile and verify | Use current frontend syntax only |
| No parser/lowering changes | Keep this a docs-planned future test PR |
| No runtime changes | Use existing profiling harness only |
| No SemCode format changes | Compare only via existing compiled outputs |
| No thresholds as pass/fail | Treat metrics as evidence only |
| Results recorded as evidence only | No optimization approval from the matrix |

## VM-M8 Candidate

Future test PR:

`test(sm-vm): add scalar movement source-shape comparison fixtures`

VM-M8 should:

- add only the matrix fixtures accepted in VM-M7
- run them through existing `vm-profile` workload tests
- record counts through existing `VmOpcodeProfile`
- avoid exact count assertions unless intentionally snapshot-like
- keep P5 blocked unless separate evidence opens it

VM-M8 should not:

- change runtime
- change verifier
- change SemCode
- change parser/lowering
- change existing fixtures
- encode optimization thresholds as tests

VM-M9 records measured evidence from this matrix in [docs/roadmap/sm_vm_vm_m9_scalar_movement_source_shape_evidence.md](sm_vm_vm_m9_scalar_movement_source_shape_evidence.md).

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

Commands run:

- `git status --short`
- `sed -n '1,260p' docs/roadmap/sm_vm_vm_m6_equivalent_source_shape_scalar_movement.md`
- `sed -n '1,260p' docs/roadmap/sm_vm_vm_m5_lowering_shape_scalar_movement_audit.md`
- `sed -n '1,220p' docs/roadmap/sm_vm_vm_m4_scalar_movement_audit.md`
- `sed -n '1,180p' docs/roadmap/sm_vm_measured_improvement_path_after_p4h.md`
- `git diff --check`
- `cargo fmt --check`

Results:

- `git status --short` shows unrelated pre-existing dirty files only.
- `cargo fmt --check` was not run separately in this doc pass because the repository already has the known unrelated formatting drift in `crates/prom-ui-backend-native/src/lib.rs`.
- `git diff --check` has no new content issues for this doc task.
