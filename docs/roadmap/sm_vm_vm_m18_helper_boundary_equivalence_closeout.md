# sm-vm VM-M18 Helper-Boundary Equivalence Closeout

## Status

VM-M18 closes the helper-boundary result-equivalence cycle with a private test-only harness.

This document does not approve runtime changes, verifier changes, SemCode changes, fixture changes, VM optimization, or helper inlining.

## Closed Chain

- VM-M12 → helper-boundary lowering shape audit
- VM-M13 → result-equivalence evidence boundary
- VM-M14 → VM/test result surfaces audit
- VM-M15 → private test-only observation boundary
- VM-M16 → minimal test-only observation helper
- VM-M17 → pair-equivalence harness
- VM-M18 → closeout

## What Is Now Proven

- Helper-boundary fixture pairs are now compared at harness level.
- The harness uses a private test-only observation path.
- The observation boundary captures a terminal return-value and final-state snapshot from the verified execution path.
- The pair comparison normalizes away compiler-generated staging locals and compares the stable semantic locals for each current helper-boundary pair shape.
- The current helper-boundary shapes checked by the harness are:
  - VM-M9 helper boundary
  - VM-M11 G2 helper single-call
  - VM-M11 G2 helper call-chain

## What Remains Not Claimed

- No public VM API was widened.
- No production VM behavior changed.
- No verifier behavior changed.
- No SemCode format changed.
- No lowering or parser/typechecker behavior changed.
- No helper inlining was required.
- No optimization was approved.
- No claim is made about raw internal staging locals being identical between helper and inline variants.
- No claim is made beyond the current helper-boundary fixture pair shapes.

## Files Changed In Implementation

- `docs/roadmap/sm_vm_vm_m15_private_test_observation_boundary.md`
- `crates/sm-vm/src/semcode_vm.rs`
- `docs/roadmap/sm_vm_vm_m18_helper_boundary_equivalence_closeout.md`

## Validation

- `cargo test -p sm-vm helper_boundary -- --nocapture`
- `cargo test -p sm-vm equivalence -- --nocapture`
- `cargo fmt -- crates/sm-vm/src/semcode_vm.rs`

## Decision

PASS WITH LIMITATION — helper-vs-inline pair equivalence is now checked at harness level for the current helper-boundary fixture pair shapes, using canonicalized terminal semantic locals.

## Follow-Up, If Any

No immediate follow-up is required for the current cycle.

If new helper-boundary fixture shapes are added later, they should reuse the same private observation boundary and extend the pair list explicitly.

