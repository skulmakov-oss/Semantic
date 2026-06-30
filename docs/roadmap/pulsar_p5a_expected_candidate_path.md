# Pulsar P5-A Expected Candidate Path

## Status

This document describes the expected future path for reopening Pulsar P5-A.

It does not reopen P5-A.
It does not approve runtime integration.
It does not approve P5-B.
It does not change VM behavior, verifier behavior, SemCode format, public APIs, or production telemetry.

- P4 shadow equivalence is closed and evidence-repaired.
- `#1237` completed CPU/features diagnostics and `QuadroBank` batch-path coverage.
- Current profiling evidence blocks P5-A.
- Fresh measured runtime evidence is required before candidate selection.

## Current blocked state

P5-A is blocked because current measured workloads did not show a meaningful,
batchable quad-state hot path suitable for Pulsar runtime acceleration.

`P4-H`'s `15%` value was a local conservative review heuristic, not a canonical
promotion gate.

## P5-A Reopen Conditions

P5-A may reopen only when fresh measured runtime evidence shows:

- [ ] one narrow candidate hot path;
- [ ] meaningful opcode/runtime pressure;
- [ ] batchable quad-state operation shape;
- [ ] scalar authority path remains available;
- [ ] feature-gated Pulsar candidate path can be specified;
- [ ] runtime-level equivalence test plan exists;
- [ ] fallback to scalar is documented;
- [ ] no public VM API widening is required;
- [ ] no verifier or SemCode change is required;
- [ ] explicit promotion review is planned.

## Expected candidate path

```text
Measured VM Hot Path
  -> Scalar Authority Path
  -> Feature-Gated Pulsar Candidate
  -> Runtime Equivalence Check
  -> Fallback to Scalar
  -> Promotion Review
```

| Step | Meaning | Required evidence |
| --- | --- | --- |
| Measured VM Hot Path | Candidate must come from fresh profiling | workload, counts, ratio, reason hot |
| Scalar Authority Path | Scalar result remains source of truth | current scalar behavior documented |
| Feature-Gated Pulsar Candidate | Pulsar path is opt-in | feature name, disabled-by-default behavior |
| Runtime Equivalence Check | Pulsar must match scalar behavior | scalar-vs-Pulsar comparison plan |
| Fallback to Scalar | safe rollback path | fallback trigger and behavior |
| Promotion Review | explicit decision gate | P5-A approval record |

## Candidate examples, not approvals

Possible examples:

- repeated quad logical operations;
- quad mask extraction in runtime-heavy scenarios;
- batch-like quad state transitions;
- state delta calculation if measured hot;
- merge/intersect patterns if measured hot.

These examples are not approved candidates.
They become candidates only if fresh profiling evidence supports them.

## Required evidence package for future P5-A PR

Future P5-A PR must include:

- workload name and source;
- baseline scalar path;
- measured profile evidence;
- candidate hot-path reason;
- operation family;
- scalar authority comparison;
- Pulsar candidate path;
- feature gate;
- equivalence test plan;
- fallback behavior;
- non-claims;
- rollback posture.

## Non-claims

This document does not claim:

- P5-A is open;
- P5-B is approved;
- Pulsar is runtime-integrated;
- Pulsar replaces `sm-vm`;
- GPU/Vulkan backend exists;
- VM performance improved;
- public VM API changed;
- SemCode format changed;
- verifier admission changed;
- production telemetry was added;
- PROMETHEUS or CTF boundaries widened.

