# PULSAR-Q4 — Quadro Hot Path Review

## Status

LOCAL DIAGNOSTIC REVIEW / NOT AN OPTIMIZATION CLAIM

## Reviewed Workloads

- `qreg_merge`
- `qreg_intersect`
- `qreg_inverse`
- `qreg_masks_all`
- `qreg_calc_delta`
- `qbank_merge_inplace`
- `qbank_intersect_inplace`
- `qbank_inverse_inplace`
- `qbank_calc_deltas_soa`
- `baseline_vec_u8_delta`

## Likely Hot Paths

Based on the benchmark baseline and the current `ton618-core` implementation, the likely hot paths are:

- `QuadroReg::masks_all`
- `QuadroReg::calc_delta`
- `QuadroBank::calc_deltas_soa`
- `QuadroBank::merge_inplace`
- `QuadroBank::intersect_inplace`
- `QuadroBank::inverse_inplace`
- the baseline `Vec<u8>` delta loop

The bank-level loops are the most stable timing band in the local runs and are the clearest candidates for future cleanup or backend work.

## Optimization Candidates

Possible future work only:

- scalar inlining review
- avoid repeated mask extraction if reused
- bank loop reset / copy discipline
- SIMD feature-gated batch backend
- aligned bank storage review
- criterion or iai-compatible formal bench later

## Rejected-for-now Optimizations

- No SIMD in this slice.
- No AVX2/NEON in this slice.
- No prefetch in this slice.
- No AVX-512 in this slice.
- No unsafe expansion in this slice.
- No runtime/VM integration in this slice.

## Safety Boundaries

This review does not change behavior.
It does not widen the active Core Trust Freeze contour.
It does not move Pulsar into VM execution authority.
It does not replace the checked scalar path.
It does not add benchmark dependencies or external measurement tooling.

Any future optimization must preserve the checked scalar path and keep the hot path review narrow enough to compare against the recorded local baseline.

## Recommended Next Slice

PULSAR-Q5 — Scalar Hot Path Cleanup

Scalar correctness and measurement should stay the reference before any SIMD discussion.
