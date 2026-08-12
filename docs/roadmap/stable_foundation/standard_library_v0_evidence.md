# SSF-03 Standard Library v0 Evidence

Status: candidate exit evidence; not Published Stable

Contract: `semantic.foundation.std/0.1`

Base: `0115b161f27d12aa877562b440a1cc5a84a05bcb`

This map binds each selected Standard Library v0 family to existing
implementation authority and executable evidence. SSF-03 adds no duplicate
runtime or frontend implementation because the selected behavior is already
owned end to end.

## Family qualification map

| Family | Positive evidence | Negative evidence | Compatibility baseline |
|---|---|---|---|
| `std.core` | `tests/pcc8_stdlib_acceptance.rs` | `tests/pcc8_stdlib_diagnostics.rs` | CTF-E1/CTF-E3 stdlib trace and trap fixtures |
| `std.quad` | qtruth frontend/lowering/VM unit matrices in `crates/sm-front`, `crates/sm-ir`, and `crates/sm-vm` | quad type mismatch and opcode truncation/rejection unit tests | distinct QTruth opcode-byte tests and canonical `semantic-core-quad` truth maps |
| `std.math` | `tests/pcc8_stdlib_acceptance.rs` (`positive_math_basic.sm`) | selected surface is `f64 -> f64` only; type/arity mismatches reject via the existing generic builtin-call diagnostics | `sqrt`/`abs` call straight into Rust `f64::sqrt`/`f64::abs`, unmetered and host-effect-free, per `crates/sm-vm/src/semcode_vm.rs` |
| `std.text` | `tests/pcc3_text_core_gate.rs`, `tests/pcc8_stdlib_acceptance.rs` | `tests/pcc_stdlib_negative.rs`, `tests/pcc8_stdlib_diagnostics.rs` | PCC3 lowering stability and CTF stdlib traces |
| `std.seq` | `tests/pcc7_sequence_acceptance.rs` | `tests/pcc7_collections_diagnostics.rs` | `tests/sequence_ownership_golden.rs` plus compile/verify paths |
| `std.map` | `tests/pcc7_map_acceptance.rs` | `tests/pcc7_collections_diagnostics.rs` | `tests/collections_map_surface_qualification.rs` plus compile/verify paths |
| `std.option` | `tests/pcc6_option_acceptance.rs` | `tests/pcc6_option_result_diagnostics.rs` | `tests/pcc6_option_result_ownership_golden.rs` |
| `std.result` | `tests/pcc6_result_acceptance.rs` | `tests/pcc6_option_result_diagnostics.rs` | `tests/pcc6_option_result_ownership_golden.rs` |
| `std.rand` | seeded cases in `tests/snake_benchmark_gap_matrix.rs` | invalid-range case in `tests/snake_benchmark_gap_matrix.rs` | exact xorshift64 fixture replay and SemCode compile/verify path |

`std.math` selects only `sqrt`/`abs`; its `sin`/`cos`/`tan`/`pow` remain
Deferred pending cross-platform transcendental determinism policy.
`std.serde` is fully Deferred, so it has no selected API to qualify. Both
absences (and `std.math`'s partial one) are protected by the contract drift
guard.

## Implementation authority

| Concern | Existing owner |
|---|---|
| builtin/type admission | `crates/sm-front` |
| source-to-instruction lowering | `crates/sm-ir` |
| artifact capabilities and structural admission | `crates/sm-verify` |
| deterministic execution | `crates/sm-vm` |
| quad truth maps | `crates/semantic-core-quad` |
| public command path | `crates/smc-cli` |

The selected surface remains verifier-first. A documentation family name does
not authorize a helper to bypass lowering, artifact capabilities, verifier
admission, quotas, or runtime traps.

## Boundary decisions

- `std.*` is a documentation identity, not an import namespace in Foundation
  Source 1.0.
- `print(text)` is excluded and routed into SSF-04.
- only `sqrt`/`abs` are promoted into `std.math`; `sin`/`cos`/`tan`/`pow`
  remain outside the compatibility surface.
- no Semantic source serialization API or encoding is claimed.
- map iteration/order and text indexing/normalization are not claimed.
- seeded PRNG is deterministic versioned VM state, not host entropy.
- `N` and `S` remain evidence states; no quad-to-bool normalization exists.

## Canonical example and guard

`examples/canonical/stdlib_v0_helpers/src/main.sm` exercises every selected
family in one pure, deterministic program. It is already covered by
`tests/canonical_examples.rs` through check, run, compile, and verify.

`tests/ssf_stdlib_v0_contract.rs` guards the contract ID, all ten target family
decisions, implementation anchors, evidence files, excluded effects, exact
PRNG algorithm, and canonical example surface.

## Validation contour

The phase PR must run at least:

- `cargo test -q --test ssf_stdlib_v0_contract`;
- `cargo test -q --test canonical_examples`;
- PCC3, PCC6, PCC7, PCC8, collection, qtruth, and seeded-random focused tests;
- the repository harness, PR-ready, boundary, public-API, release-bundle,
  no-std, all-target, and 7hell gates.

Skipped checks do not count as pass.

## Exit result

The selected Standard Library v0 contour is bounded by
`semantic.foundation.std/0.1` without inventing an import system or a second
implementation authority. SSF-04 remains blocked until the exact-head PR is
merged and issue #1574 records the merge.
