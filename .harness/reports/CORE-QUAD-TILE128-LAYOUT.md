# Harness Report: CORE-QUAD-TILE128-LAYOUT

## Status

- **Starting main commit:** `395641a0648e5e1a8524aed688b10512a14e9b84`
- **Branch:** `core-quad/quad-tile128-layout`
- **Implementation commit:** `9d31b72cdbb9ffa11c4790b3dc3eecaa44435a98`
- **First qualification correction:** `5736b5448bbe4d32b14cf173ee8751cbe18c9d79`
- **Evidence finalization commit:** `d8b0943adab2931b7b9833091110c3c60673cb6d`
- **Pull request:** `#1479`

## Layout Contract

`QuadTile128` previously used:

```rust
#[repr(C)]
```

It now uses:

```rust
#[repr(C, align(16))]
```

Qualified core layout:

* size: 32 bytes;
* alignment: 16 bytes;
* `t` offset: 0;
* `f` offset: 16;
* `[QuadTile128; 2]` size: 64 bytes.

Compile-time assertions cover size, alignment, and both field offsets.

## Test Inventory

* `tile128_layout_contracts` verifies size, alignment, field offsets, and array stride.
* `tile128_plane_preservation` verifies exact asymmetric truth-plane and falsity-plane preservation.
* `tile128_reg32_roundtrip_mixed` verifies four observably distinct registers survive `from_regs(...).to_regs()`.
* `tile128_semantic_stability` verifies lane access and exact truth, falsity, known, conflict, and null masks.

The semantic classification remains:

```text
T or F -> known
S      -> conflict
N      -> null
```

## Compatibility Classification

The core CPU semantic-storage layout is qualified by compile-time assertions and repository tests.

This task does not claim:

* serialized-data roundtrip compatibility;
* GPU transport ABI compatibility;
* general byte-cast safety;
* `Pod` or `Zeroable` safety;
* WGPU or WGSL compatibility.

The visual adapter remains the owner of any portable GPU transport representation.

## Verification History

The initial online CI run exposed:

* invalid test API names;
* an incorrect expectation that state `S` belonged to `known_mask`.

Those errors were corrected without changing production tile semantics.

Local fail-fast verification subsequently passed:

* `cargo +1.93.1 fmt --all --check`
* `cargo +1.93.1 clippy --workspace --all-targets -- -D warnings`
* `cargo test -p semantic-core-quad tile128_reg32_roundtrip_mixed -- --nocapture`
* `cargo test -p semantic-core-quad tile128_semantic_stability -- --nocapture`
* `cargo test -p semantic-core-quad --quiet`
* `cargo check -p semantic-core-quad --no-default-features`
* `cargo test -p semantic-core-quad --all-features --quiet`
* `cargo test -p semantic-core-capsule --quiet`
* `cargo test --test legacy_guards --quiet`
* `cargo test --all-targets --quiet`
* `git diff --check`
* `scripts/harness-check.ps1`

GitHub Actions run `#4346` passed all jobs at head:

```text
d8b0943adab2931b7b9833091110c3c60673cb6d
```

The forthcoming report-only repair commit will be revalidated by the PR CI.

## Changed Files in PR

* `.harness/current.task.yaml`
* `.harness/reports/CORE-QUAD-TILE128-LAYOUT.md`
* `crates/semantic-core-quad/src/lib.rs`
* `docs/roadmap/core_quad/quad_tile128_layout_boundary.md`

## Explicit Non-Changes

* No state encoding change.
* No lane numbering change.
* No mask semantic change.
* No register conversion semantic change.
* No GPU transport type.
* No WGPU or WGSL code.
* No byte-upload helper.
* No `bytemuck` dependency or derivation.