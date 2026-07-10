# CORE-QUAD-V1-COMPATIBILITY-CLOSEOUT

## Starting main commit

`77c7ac9b23169e2c66a71f098f0a87ff18e13c2f`

## Branch

`core-quad/v1-compatibility-closeout`

## Issue and parent

- Issue: #1413
- Parent: #1404

## Changed files

```text
.harness/current.task.yaml
.harness/reports/CORE-QUAD-V1-COMPATIBILITY-CLOSEOUT.md
docs/roadmap/core_quad/v1_compatibility_rollout.md
docs/roadmap/core_quad/v1_compatibility_closeout.md
```

## Stable public API registry

`QuadState`, `QuadroReg32`, qualified CPU/core `QuadTile128`, and the bank
container/indexing semantics remain stable under the documented v1 contract.
The N/F/T/S encoding remains `00/01/10/11`; no encoding migration is
authorized. Truth-map and lattice API families remain separate.

## Additive items landed

The mask bridge (`QuadLaneMask32` and validated `QuadPhysicalMask32`),
`PlaneDelta32`, `ExactStateDelta32`, tile truth maps, and in-place truth-map
helpers for `QuadroBank<N>` and `QuadTileBank<N>` are additive items.

## Legacy compatibility items retained

`QuadMask32`, `StateDelta32`, `StateDelta128`, `entered_true`, `left_true`, and
`raw_delta` remain compatibility surfaces and are not silently redefined. No
legacy public item was removed.

## Typed-mask status

Dense logical masks and validated packed physical masks are separate. Odd
physical bits are rejected and conversion is explicit.

## Delta-split status

`PlaneDelta32` owns plane membership transitions. `ExactStateDelta32` owns exact
state transitions. Compatibility delta structures retain documented mixed
semantics.

## Tile-layout status

`QuadTile128` is `repr(C, align(16))`, 32 bytes, with truth offset 0 and falsity
offset 16. This is qualified CPU/core storage only, not a GPU transport ABI.

## Tile/bank truth-map status

The seven default tile maps use deterministic four-register lifting. The same
seven operations are available as allocation-free, elementwise in-place helpers
for both bank types, with ordering and indexing preserved.

## EQUIV deferral

EQUIV is deferred and is not part of the qualified v1 public API.

## Feature qualification posture

`std` is tested; `no_std` is check-qualified through `--no-default-features`;
`serde` is compile/test-qualified under `--all-features`. Serialized
cross-version compatibility is not claimed.

## Core capsule result

`semantic-core-capsule` is the minimum downstream smoke consumer. A passing
test result is baseline integration evidence, not proof of total external
compatibility.

## Remaining issues

#1412 remains the qualification matrix and benchmark plan. #1417 remains GPU
transport representation. #1404 remains the umbrella closeout. None is claimed
complete here.

## Local verification commands and results

All requested checks passed on this branch:

```text
cargo +1.93.1 fmt --all --check                              PASS
cargo +1.93.1 clippy --workspace --all-targets -- -D warnings PASS
cargo test -p semantic-core-quad --quiet                     PASS (134 tests)
cargo check -p semantic-core-quad --no-default-features      PASS
cargo test -p semantic-core-quad --all-features --quiet      PASS (134 tests)
cargo test -p semantic-core-capsule --quiet                   PASS (8 tests)
cargo test --test legacy_guards --quiet                       PASS (10 tests)
cargo test --all-targets --quiet                              PASS
git diff --check                                              PASS
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1 PASS
```

## Explicit non-changes

No Rust source, tests, Cargo files, specs, workflows, other crates, public APIs,
EQUIV implementation, benchmark code, GPU transport, visual code, or unrelated
untracked files were modified or staged.
