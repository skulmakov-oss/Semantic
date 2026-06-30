# Pulsar P4 Shadow Evidence Repair

## Status

This note records a small evidence repair for the existing Pulsar P4 shadow equivalence tests.

It does not reopen P5-A or claim runtime acceleration.

## Evidence Gap Repaired

- `ShadowMismatchReport` now records the CPU feature path and enabled Cargo features in mismatch diagnostics.
- The seeded shadow sweep now exercises `QuadroBank::merge_inplace` and `QuadroBank::intersect_inplace` under `alloc`, rather than only scalar `QuadroReg` merge/intersect paths.

## Boundary

- P4 remains shadow/test-only.
- The repair does not widen production runtime authority.
- The repair does not change VM, verifier, SemCode, or PROMETHEUS boundary behavior.

## P5 Status

P5-A remains blocked unless fresh measured runtime evidence later reopens it.

