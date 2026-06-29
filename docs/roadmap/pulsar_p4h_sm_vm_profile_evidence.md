# Pulsar P4-H sm-vm Opcode Profile Evidence

## Status

P4-H records local profiling evidence from the P4-G Semantic workload corpus.

This document does not approve P5 runtime acceleration.

## Method

- Source fixtures were compiled from `.sm`.
- SemCode was verified before execution.
- Execution was profiled through `run_verified_entry_semcode_with_profile`.
- Opcode histograms were collected through `VmOpcodeProfile`.
- No production telemetry was used.
- No runtime behavior changed.

## Commands

```bash
cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture
cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture
```

## Workloads

| Fixture | Class | Default/Local | Purpose |
|---|---|---:|---|
| `quad_logic_storm.sm` | micro-op | default | quad logical pressure |
| `quad_match_dispatch.sm` | micro-op | default | quad match/control-flow |
| `fact_merge_kernel.sm` | semantic kernel | default | evidence union |
| `fact_intersect_kernel.sm` | semantic kernel | default | evidence intersection |
| `delta_like_kernel.sm` | semantic kernel | default | state transition classification |
| `andromeda_fact_wave_64.sm` | Andromeda-shaped | default | synthetic semantic wave |
| `andromeda_fact_wave_256.sm` | Andromeda-shaped | ignored/local | larger synthetic semantic wave |

## Results

### `fact_merge_kernel.sm`

- Compiled, verified, and executed successfully: yes
- Total instructions: `177`
- Top opcodes:
  - `LoadQ`: `32`
  - `LoadVar`: `30`
  - `StoreVar`: `22`
  - `Jmp`: `20`
  - `LoadI32`: `16`
  - `JmpIf`: `16`
  - `AddI32`: `12`
  - `CmpEq`: `10`
  - `QOr`: `8`
  - `CmpNe`: `8`
  - `Assert`: `2`
  - `Ret`: `1`

| Metric | Count | Ratio |
|---|---:|---:|
| `QAnd + QOr` | `8` | `4.52%` |
| quad logic | `8` | `4.52%` |
| quad family | `58` | `32.77%` |
| control flow | `36` | `20.34%` |

Interpretation:

- The workload is hot in `quad_family`, but `QAnd + QOr` is well below the P5-A trigger threshold.
- This is hot but not yet proven batchable.

### `fact_intersect_kernel.sm`

- Compiled, verified, and executed successfully: yes
- Total instructions: `304`
- Top opcodes:
  - `Jmp`: `63`
  - `LoadQ`: `54`
  - `LoadVar`: `49`
  - `CmpEq`: `40`
  - `JmpIf`: `36`
  - `StoreVar`: `22`
  - `LoadI32`: `17`
  - `AddI32`: `9`
  - `QAnd`: `9`
  - `Assert`: `4`
  - `Ret`: `1`

| Metric | Count | Ratio |
|---|---:|---:|
| `QAnd + QOr` | `9` | `2.96%` |
| quad logic | `9` | `2.96%` |
| quad family | `103` | `33.88%` |
| control flow | `99` | `32.57%` |

Interpretation:

- Strong control-flow and comparison pressure dominates.
- The workload is hot but not yet proven batchable.

### `quad_match_dispatch.sm`

- Compiled, verified, and executed successfully: yes
- Total instructions: `4042`
- Top opcodes:
  - `LoadVar`: `906`
  - `StoreVar`: `690`
  - `Jmp`: `578`
  - `JmpIf`: `470`
  - `CmpEq`: `372`
  - `LoadI32`: `265`
  - `LoadQ`: `264`
  - `AddI32`: `144`
  - `Ret`: `97`
  - `Call`: `96`
  - `CmpI32Lt`: `54`
  - `ModI32`: `48`

| Metric | Count | Ratio |
|---|---:|---:|
| `QAnd + QOr` | `0` | `0.00%` |
| quad logic | `0` | `0.00%` |
| quad family | `636` | `15.73%` |
| control flow | `1048` | `25.93%` |

Interpretation:

- This workload is control-flow heavy.
- It does not provide a P5-A quad-logic trigger.

### `delta_like_kernel.sm`

- Compiled, verified, and executed successfully: yes
- Total instructions: `2984`
- Top opcodes:
  - `LoadVar`: `691`
  - `Jmp`: `572`
  - `StoreVar`: `343`
  - `JmpIf`: `336`
  - `CmpEq`: `312`
  - `LoadI32`: `239`
  - `LoadQ`: `196`
  - `BoolAnd`: `98`
  - `CmpNe`: `84`
  - `AddI32`: `30`
  - `Ret`: `29`
  - `Call`: `28`

| Metric | Count | Ratio |
|---|---:|---:|
| `QAnd + QOr` | `0` | `0.00%` |
| quad logic | `0` | `0.00%` |
| quad family | `592` | `19.84%` |
| control flow | `908` | `30.43%` |

Interpretation:

- Transition classification is dominated by control flow and scalar movement.
- This is hot but not yet proven batchable.

### `quad_logic_storm.sm`

- Compiled, verified, and executed successfully: yes
- Total instructions: `7320`
- Top opcodes:
  - `LoadVar`: `1845`
  - `StoreVar`: `1237`
  - `Jmp`: `1010`
  - `JmpIf`: `738`
  - `LoadI32`: `603`
  - `CmpEq`: `480`
  - `LoadQ`: `320`
  - `AddI32`: `240`
  - `Ret`: `129`
  - `ModI32`: `128`
  - `CmpNe`: `128`
  - `Call`: `128`

| Metric | Count | Ratio |
|---|---:|---:|
| `QAnd + QOr` | `128` | `1.75%` |
| quad logic | `192` | `2.62%` |
| quad family | `1120` | `15.30%` |
| control flow | `1748` | `23.88%` |

Interpretation:

- The workload creates measurable quad-logic pressure, but the share is still small relative to total instructions.
- It is not enough on its own to justify a P5-A candidate review.

### `andromeda_fact_wave_64.sm`

- Compiled, verified, and executed successfully: yes
- Total instructions: `8188`
- Top opcodes:
  - `LoadVar`: `2023`
  - `StoreVar`: `1367`
  - `Jmp`: `1242`
  - `JmpIf`: `850`
  - `LoadI32`: `735`
  - `CmpEq`: `721`
  - `LoadQ`: `360`
  - `AddI32`: `232`
  - `Ret`: `129`
  - `ModI32`: `128`
  - `Call`: `128`
  - `CmpI32Lt`: `70`

| Metric | Count | Ratio |
|---|---:|---:|
| `QAnd + QOr` | `128` | `1.56%` |
| quad logic | `128` | `1.56%` |
| quad family | `1209` | `14.77%` |
| control flow | `2092` | `25.55%` |

Interpretation:

- This synthetic wave is control-flow heavy with modest quad-family pressure.
- It does not meet the quad-logic trigger for P5-A review.

### `andromeda_fact_wave_256.sm`

- Compiled, verified, and executed successfully: yes
- Total instructions: `32596`
- Top opcodes:
  - `LoadVar`: `8071`
  - `StoreVar`: `5447`
  - `Jmp`: `4962`
  - `JmpIf`: `3394`
  - `LoadI32`: `2895`
  - `CmpEq`: `2881`
  - `LoadQ`: `1440`
  - `AddI32`: `928`
  - `Ret`: `513`
  - `ModI32`: `512`
  - `Call`: `512`
  - `CmpI32Lt`: `262`

| Metric | Count | Ratio |
|---|---:|---:|
| `QAnd + QOr` | `512` | `1.57%` |
| quad logic | `512` | `1.57%` |
| quad family | `4833` | `14.83%` |
| control flow | `8356` | `25.64%` |

Interpretation:

- The larger wave confirms the same shape as the 64-sized variant.
- The workload is hot, but not yet proven batchable for a Pulsar runtime candidate.

## Aggregate interpretation

Across the recorded P4-G corpus:

- `QAnd + QOr`: `785 / 55611 = 1.41%`
- quad logic: `849 / 55611 = 1.53%`
- quad family: `8551 / 55611 = 15.37%`
- control flow: `14287 / 55611 = 25.69%`

Summary:

- `QAnd + QOr` did not reach the 15% P5-A review trigger in any workload.
- `quad_family` crossed 25% only in the fact merge / fact intersect kernels, but the signal is mixed with strong control-flow and scalar-movement pressure.
- The corpus shows measurable quad-family activity, but the dominant cost is still mostly outside pure quad logic.
- The data is hot but not yet proven batchable.

## P5 Gate Decision

`P5-A candidate review: BLOCKED`

Reason:

- The corpus does not meet the quad-logic trigger threshold.
- The strongest signals are in control flow and scalar movement, not in direct `QAnd + QOr` pressure.
- Batchability is not yet proven.

## Non-claims

This evidence does not claim:

- Pulsar runtime integration;
- default VM acceleration;
- production performance improvement;
- SemCode change;
- verifier change;
- CTF boundary change;
- PROMETHEUS boundary change;
- P5-B approval.

## Next Step

If a later corpus or profiling pass changes the measurement shape, reopen P5-A candidate review from fresh evidence.

Until then, the measured evidence supports more profiling or a different VM-improvement path rather than a Pulsar runtime promotion.
