# Pulsar P5-A Evidence Probe

## Status

This probe does not reopen P5-A.
This probe only adds local profiling evidence.
P5-A remains blocked unless the measured results show one narrow,
meaningful, batchable quad-state hot path and a future promotion review approves it.

- P4 shadow equivalence is closed and evidence-repaired.
- `#1237` completed CPU/features diagnostics and `QuadroBank` batch-path coverage.
- Current profiling evidence blocks P5-A.
- Fresh measured runtime evidence is required before candidate selection.

## Goal

This probe asks one narrow question:

Can current Semantic workloads produce a meaningful, batchable quad-state hot path
that could justify reopening Pulsar P5-A later?

The answer is based on local profiling evidence only.

## Workloads inspected

Existing profiling corpus context:

- `quad_logic_storm.sm`
- `quad_match_dispatch.sm`
- `fact_merge_kernel.sm`
- `fact_intersect_kernel.sm`
- `delta_like_kernel.sm`
- `andromeda_fact_wave_64.sm`
- `andromeda_fact_wave_256.sm`

New probe workloads:

- `p5a_probe/p5a_quad_batch_wave.sm`
- `p5a_probe/p5a_quad_helper_mix.sm`

## New probe workloads

### `p5a_quad_batch_wave.sm`

This workload exists to stress repeated quad logical operations in a direct,
batch-like loop shape without relying on a helper chain.

It uses:

- repeated `quad` state generation;
- `||`, `&&`, and `!` over a loop of 128 iterations;
- only enough scalar bookkeeping to keep the workload realistic.

This is the closest probe shape to a future batchable quad-state hot path.
It still must be treated as evidence, not as an approval.

### `p5a_quad_helper_mix.sm`

This workload exists to stress the same quad-state operations while adding helper
boundary pressure.

It uses:

- the same `quad` state generation shape;
- a helper function around repeated quad operations;
- the same 128-iteration loop scale;
- a checksum guard to keep execution honest.

This workload helps distinguish direct quad pressure from helper-induced
scalar/call overhead.

## Measurement method

Measured with the existing local `vm-profile` workload path:

```text
compile .sm source
  -> verify SemCode
  -> run verified entry with profile
  -> collect VmOpcodeProfile
  -> summarize opcode families
```

Relevant commands:

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads profile_pulsar_p5a_evidence_probe_pair -- --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads --no-run`
- `cargo test -p sm-vm`
- `cargo test -p sm-vm --all-features`

The probe reuses the existing `run_verified_entry_semcode_with_profile` and
`VmOpcodeProfile` path.

## Results

| Workload | Total | Quad logic | Quad family | Control flow | Scalar movement | Calls | Interpretation |
|---|---:|---:|---:|---:|---:|---:|---|
| `p5a_probe/p5a_quad_batch_wave.sm` | 14248 | 384 / 2.70% | 1985 / 13.93% | 3140 / 22.04% | 6346 / 44.54% | 513 / 3.60% | Direct batch-like quad pressure exists, but scalar movement still dominates. |
| `p5a_probe/p5a_quad_helper_mix.sm` | 17084 | 512 / 3.00% | 2433 / 14.24% | 3524 / 20.63% | 8167 / 47.80% | 769 / 4.50% | Helper boundary pressure increases call/scalar cost instead of producing a cleaner batch signal. |

Observed shape:

- both probe workloads produce real quad-state activity;
- both probe workloads are still dominated by scalar movement;
- helper pressure increases call count and scalar movement;
- the direct batch-like workload is the cleaner of the two, but it still does
  not show a clearly batchable hot path by itself.

## Interpretation

The probe shows that current Semantic workloads can generate quad-state pressure,
but the measured shape still looks structurally expensive in scalar movement.

The best direct batch-like workload is not yet strong enough to serve as a future
P5-A reopening signal on its own.

This is evidence for continued profiling and candidate narrowing, not for
runtime integration.

## P5-A gate decision

P5-A remains BLOCKED.

The probe did not show a narrow, meaningful, batchable quad-state hot path with
enough authority to reopen candidate review.

## Non-claims

This document does not claim:

- P5-A is open;
- P5-B is approved;
- Pulsar is runtime-integrated;
- Pulsar replaces `sm-vm`;
- VM performance improved;
- public VM API changed;
- SemCode format changed;
- verifier admission changed;
- production telemetry was added;
- PROMETHEUS or CTF boundaries widened.

## Validation

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads profile_pulsar_p5a_evidence_probe_pair -- --nocapture` passed
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads --no-run` passed
- `cargo test -p sm-vm` passed
- `cargo test -p sm-vm --all-features` passed

