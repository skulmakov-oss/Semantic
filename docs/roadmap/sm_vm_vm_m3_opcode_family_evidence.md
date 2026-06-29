# sm-vm VM-M3 Opcode-Family Profile Evidence

## Status

VM-M3 records opcode-family profiling evidence after VM-M2.

This document does not approve P5 runtime acceleration or any VM optimization.

## Method

- Source `.sm` fixtures compiled to SemCode.
- SemCode verified before execution.
- Execution profiled through `run_verified_entry_semcode_with_profile`.
- Opcode family summaries produced by the VM-M2 test-local helpers.
- No production telemetry.
- No runtime behavior changes.

## Commands

```bash
cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture
cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture
git diff --check
cargo fmt --check
```

## Workloads

| Fixture | Class | Default/Local | Purpose |
|---|---|---|---|
| `quad_logic_storm.sm` | micro-op | default | quad logical pressure |
| `quad_match_dispatch.sm` | micro-op | default | quad match/control-flow |
| `fact_merge_kernel.sm` | semantic kernel | default | evidence union |
| `fact_intersect_kernel.sm` | semantic kernel | default | evidence intersection |
| `delta_like_kernel.sm` | semantic kernel | default | state transition classification |
| `andromeda_fact_wave_64.sm` | Andromeda-shaped | default | small synthetic semantic wave |
| `andromeda_fact_wave_256.sm` | Andromeda-shaped | ignored/local | larger synthetic semantic wave |

## Per-Fixture Results

### `quad_logic_storm.sm`

| Family | Count | Ratio |
|---|---:|---:|
| `quad_logic` | `192 / 7320` | `2.62%` |
| `quad_family` | `1120 / 7320` | `15.30%` |
| `control_flow` | `1748 / 7320` | `23.88%` |
| `scalar_movement` | `3082 / 7320` | `42.10%` |
| `calls` | `257 / 7320` | `3.51%` |
| `integer_ops` | `1040 / 7320` | `14.21%` |

Top opcodes:

- `LoadVar: 1845`
- `StoreVar: 1237`
- `Jmp: 1010`
- `JmpIf: 738`
- `LoadI32: 603`
- `CmpEq: 480`
- `LoadQ: 320`
- `AddI32: 240`
- `Ret: 129`
- `ModI32: 128`
- `CmpNe: 128`
- `Call: 128`

Interpretation:

- `scalar_movement` dominates.
- `control_flow` is second.
- `quad_logic` is low relative to the total instruction mix.

### `quad_match_dispatch.sm`

| Family | Count | Ratio |
|---|---:|---:|
| `quad_logic` | `0 / 4042` | `0.00%` |
| `quad_family` | `636 / 4042` | `15.73%` |
| `control_flow` | `1048 / 4042` | `25.93%` |
| `scalar_movement` | `1596 / 4042` | `39.49%` |
| `calls` | `193 / 4042` | `4.77%` |
| `integer_ops` | `511 / 4042` | `12.64%` |

Top opcodes:

- `LoadVar: 906`
- `StoreVar: 690`
- `Jmp: 578`
- `JmpIf: 470`
- `CmpEq: 372`
- `LoadI32: 265`
- `LoadQ: 264`
- `AddI32: 144`
- `Ret: 97`
- `Call: 96`
- `CmpI32Lt: 54`
- `ModI32: 48`

Interpretation:

- `scalar_movement` dominates.
- `control_flow` is second.
- `quad_family` is present, but not as the primary pressure source.

### `fact_merge_kernel.sm`

| Family | Count | Ratio |
|---|---:|---:|
| `quad_logic` | `8 / 177` | `4.52%` |
| `quad_family` | `58 / 177` | `32.77%` |
| `control_flow` | `36 / 177` | `20.34%` |
| `scalar_movement` | `52 / 177` | `29.38%` |
| `calls` | `1 / 177` | `0.56%` |
| `integer_ops` | `28 / 177` | `15.82%` |

Top opcodes:

- `LoadQ: 32`
- `LoadVar: 30`
- `StoreVar: 22`
- `Jmp: 20`
- `LoadI32: 16`
- `JmpIf: 16`
- `AddI32: 12`
- `CmpEq: 10`
- `QOr: 8`
- `CmpNe: 8`
- `Assert: 2`
- `Ret: 1`

Interpretation:

- `quad_family` is strong for this kernel.
- `scalar_movement` and `control_flow` still carry substantial weight.

### `fact_intersect_kernel.sm`

| Family | Count | Ratio |
|---|---:|---:|
| `quad_logic` | `9 / 304` | `2.96%` |
| `quad_family` | `103 / 304` | `33.88%` |
| `control_flow` | `99 / 304` | `32.57%` |
| `scalar_movement` | `71 / 304` | `23.36%` |
| `calls` | `1 / 304` | `0.33%` |
| `integer_ops` | `26 / 304` | `8.55%` |

Top opcodes:

- `Jmp: 63`
- `LoadQ: 54`
- `LoadVar: 49`
- `CmpEq: 40`
- `JmpIf: 36`
- `StoreVar: 22`
- `LoadI32: 17`
- `AddI32: 9`
- `QAnd: 9`
- `Assert: 4`
- `Ret: 1`

Interpretation:

- `quad_family` and `control_flow` are both heavy.
- `scalar_movement` remains non-trivial.

### `delta_like_kernel.sm`

| Family | Count | Ratio |
|---|---:|---:|
| `quad_logic` | `0 / 2984` | `0.00%` |
| `quad_family` | `592 / 2984` | `19.84%` |
| `control_flow` | `908 / 2984` | `30.43%` |
| `scalar_movement` | `1034 / 2984` | `34.65%` |
| `calls` | `57 / 2984` | `1.91%` |
| `integer_ops` | `284 / 2984` | `9.52%` |

Top opcodes:

- `LoadVar: 691`
- `Jmp: 572`
- `StoreVar: 343`
- `JmpIf: 336`
- `CmpEq: 312`
- `LoadI32: 239`
- `LoadQ: 196`
- `BoolAnd: 98`
- `CmpNe: 84`
- `AddI32: 30`
- `Ret: 29`
- `Call: 28`

Interpretation:

- `scalar_movement` is the dominant cost.
- `control_flow` is the next largest family.
- `quad_family` is meaningful but not dominant.

### `andromeda_fact_wave_64.sm`

| Family | Count | Ratio |
|---|---:|---:|
| `quad_logic` | `128 / 8188` | `1.56%` |
| `quad_family` | `1209 / 8188` | `14.77%` |
| `control_flow` | `2092 / 8188` | `25.55%` |
| `scalar_movement` | `3390 / 8188` | `41.40%` |
| `calls` | `257 / 8188` | `3.14%` |
| `integer_ops` | `1165 / 8188` | `14.23%` |

Top opcodes:

- `LoadVar: 2023`
- `StoreVar: 1367`
- `Jmp: 1242`
- `JmpIf: 850`
- `LoadI32: 735`
- `CmpEq: 721`
- `LoadQ: 360`
- `AddI32: 232`
- `Ret: 129`
- `ModI32: 128`
- `Call: 128`
- `CmpI32Lt: 70`

Interpretation:

- `scalar_movement` dominates.
- `control_flow` is also substantial.
- `quad_family` is present but below the dominant non-quad families.

### `andromeda_fact_wave_256.sm`

| Family | Count | Ratio |
|---|---:|---:|
| `quad_logic` | `512 / 32596` | `1.57%` |
| `quad_family` | `4833 / 32596` | `14.83%` |
| `control_flow` | `8356 / 32596` | `25.64%` |
| `scalar_movement` | `13518 / 32596` | `41.47%` |
| `calls` | `1025 / 32596` | `3.14%` |
| `integer_ops` | `4597 / 32596` | `14.10%` |

Top opcodes:

- `LoadVar: 8071`
- `StoreVar: 5447`
- `Jmp: 4962`
- `JmpIf: 3394`
- `LoadI32: 2895`
- `CmpEq: 2881`
- `LoadQ: 1440`
- `AddI32: 928`
- `Ret: 513`
- `ModI32: 512`
- `Call: 512`
- `CmpI32Lt: 262`

Interpretation:

- This ignored/local workload continues the same pattern as the default corpus.
- `scalar_movement` remains dominant.
- `control_flow` remains the second-largest family.

## Aggregate Results

### Default Corpus

Fixtures included:

- `quad_logic_storm.sm`
- `quad_match_dispatch.sm`
- `fact_merge_kernel.sm`
- `fact_intersect_kernel.sm`
- `delta_like_kernel.sm`
- `andromeda_fact_wave_64.sm`

| Family | Count | Ratio |
|---|---:|---:|
| `quad_logic` | `337 / 23015` | `1.46%` |
| `quad_family` | `3718 / 23015` | `16.15%` |
| `control_flow` | `5931 / 23015` | `25.77%` |
| `scalar_movement` | `9225 / 23015` | `40.08%` |
| `calls` | `766 / 23015` | `3.33%` |
| `integer_ops` | `3054 / 23015` | `13.27%` |

Dominant families:

- `scalar_movement`
- `control_flow`
- `quad_family`

### Default Corpus + Local 256

Fixtures included:

- all default corpus fixtures
- `andromeda_fact_wave_256.sm`

| Family | Count | Ratio |
|---|---:|---:|
| `quad_logic` | `849 / 55611` | `1.53%` |
| `quad_family` | `8551 / 55611` | `15.38%` |
| `control_flow` | `14287 / 55611` | `25.69%` |
| `scalar_movement` | `22743 / 55611` | `40.90%` |
| `calls` | `1791 / 55611` | `3.22%` |
| `integer_ops` | `7651 / 55611` | `13.76%` |

Dominant families:

- `scalar_movement`
- `control_flow`
- `quad_family`

## Evidence Interpretation

The repeated pattern is consistent:

- `scalar_movement` is the dominant family.
- `control_flow` is the next largest family.
- `quad_family` is meaningful, but it does not dominate the corpus.
- `calls` and `integer_ops` are present but smaller than the two leading non-quad families.

No separate `QAnd + QOr` threshold reading is printed by the VM-M2 helper output, so this report does not infer a threshold crossing.

The family summaries do not suggest a new basis to reopen Pulsar P5-A review.

## P5 Gate

`P5-A candidate review: BLOCKED`

Reason:

- The current family mix still concentrates in `scalar_movement` and `control_flow`.
- `quad_family` is measurable but not dominant enough to imply a Pulsar-shaped runtime candidate.
- The ignored/local 256 workload reinforces the same shape rather than contradicting it.

## Recommended Next Slice

`VM-M4: docs(sm-vm): audit scalar movement in profiled workloads`

## Non-claims

This document does not claim:

- VM performance improved;
- Pulsar is obsolete;
- Pulsar runtime promotion is permanently blocked;
- P5 is cancelled;
- P5-A can never reopen;
- control-flow optimization is approved;
- scalar-movement optimization is approved;
- runtime behavior should change;
- SemCode format should change;
- verifier behavior should change.

## Validation

Commands run:

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture`
- `git diff --check`
- `cargo fmt --check`

Result:

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture` passed.
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture` passed.
- `git diff --check` passed.
- `cargo fmt --check` failed due to pre-existing unrelated formatting drift in `crates/prom-ui-backend-native/src/lib.rs`. This PR did not modify Rust source.
