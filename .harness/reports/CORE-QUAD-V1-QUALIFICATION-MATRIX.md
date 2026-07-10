# CORE-QUAD-V1-QUALIFICATION-MATRIX

## Starting main commit

`f563e39015db7d3a0ed0f727666b6481e5c44764`

## Branch

`core-quad/v1-qualification-matrix`

## Issue and parent

- Issue: #1412
- Parent: #1404

## Exact changed-file boundary

```text
.harness/current.task.yaml
.harness/reports/CORE-QUAD-V1-QUALIFICATION-MATRIX.md
crates/semantic-core-quad/tests/v1_qualification.rs
docs/roadmap/core_quad/v1_qualification_matrix.md
```

## Existing inline inventory reviewed

The reviewed inline inventory includes frozen state encoding, mask bridge
roundtrips and type distinction, odd physical-mask rejection, scalar/SWAR map
equivalence for all seven maps, 4x4 delta transitions, register/tile
roundtrips, tile map oracles, register and tile bank elementwise oracles, and
the repeated-state bank matrix.

## Downstream/public integration purpose

`v1_qualification.rs` imports `semantic_core_quad` as an external consumer and
uses only public API. It complements the inline tests by checking the actual
downstream surface instead of relying on implementation-local access.

## Integration test inventory

- `qualification_state_encoding_is_frozen`
- `qualification_register_scalar_swar_and_default_maps_agree`
- `qualification_truth_policy_invariants`
- `qualification_mask_bridge_roundtrip_and_rejection`
- `qualification_exact_and_plane_delta_matrix`
- `qualification_tile_maps_match_four_register_oracle`
- `qualification_reg_bank_maps_match_elementwise_oracle`
- `qualification_tile_bank_maps_match_elementwise_oracle`
- `qualification_truth_and_lattice_families_remain_distinct`

## Canonical vectors and matrix coverage

The exact seven canonical raw vectors from the task are used. Register binary
equivalence covers the complete 7x7 vector matrix. Delta coverage covers all
16 previous/current pairs in `[N, F, T, S]` with only lane 9 set. Tile and bank
fixtures use observably distinct elements. The suite is deterministic and uses
no randomness, wall-clock input, environment-specific CPU features, or
platform-dependent values.

## Qualification coverage

- **State encoding:** public `bits()` and `from_bits()` freeze `N/F/T/S` as `00/01/10/11`.
- **Register equivalence:** scalar, SWAR, and default unary plus six binary maps.
- **Policy invariants:** NOT involution, XOR identity, NAND/NOR derivation,
  IMPLIES policy, commutativity set, and directional witness.
- **Mask bridge:** dense roundtrips, alias passage, physical-bit validation,
  odd-bit rejection, and highest valid lane.
- **Delta matrix:** all exact-state fields and plane fields, with no unrelated
  lanes set.
- **Tile lifting:** public four-register construction, roundtrip, and seven
  default map oracles.
- **Register-bank lifting:** unary and six binary in-place maps, ordering, and
  right-bank preservation.
- **Tile-bank lifting:** unary and six binary in-place maps, ordering, and
  right-bank preservation.
- **Lattice boundary:** truth-map results are checked separately from meet,
  join, and inverse API-family results.

## EQUIV and benchmark deferral

EQUIV is excluded from the qualified v1 API and is not introduced here. No
benchmark implementation is included. The second #1412 slice is reserved for
benchmark inventory, execution method, relative-output policy, and issue
closeout without absolute performance promises.

## Feature posture and core capsule

- `std`: tested.
- `no_std`: check-qualified through `--no-default-features`.
- `serde`: all-features compile/test-qualified.
- `semantic-core-capsule`: retained as the minimum downstream smoke path.

## Exact local verification commands and results

```text
cargo +1.93.1 fmt --all --check                              PASS
cargo +1.93.1 clippy --workspace --all-targets -- -D warnings PASS
cargo test -p semantic-core-quad --test v1_qualification -- --nocapture PASS (9 tests)
cargo test -p semantic-core-quad --quiet                     PASS (134 inline, 9 integration)
cargo check -p semantic-core-quad --no-default-features      PASS
cargo test -p semantic-core-quad --all-features --quiet      PASS (134 inline, 9 integration)
cargo test -p semantic-core-capsule --quiet                   PASS (8 tests)
cargo test --test legacy_guards --quiet                       PASS (10 tests)
cargo test --all-targets --quiet                              PASS
git diff --check                                              PASS
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1 PASS
```

## Explicit non-changes

No production Rust source, Cargo file, public API, specification, other crate,
dependency, EQUIV surface, benchmark code, CPU-specific intrinsic, visual/GPU
code, runtime behavior, or unrelated untracked file was modified or staged.
This test-only slice does not close #1412.
