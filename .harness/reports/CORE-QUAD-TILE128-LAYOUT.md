# Harness Report: CORE-QUAD-TILE128-LAYOUT

* **Starting Main Commit:** `395641a0648e5e1a8524aed688b10512a14e9b84`
* **Branch:** `core-quad/quad-tile128-layout`
* **Existing Layout Inventory:** `QuadTile128` was previously defined simply as `#[repr(C)]`.
* **Exact Representation Change:** The attribute is now `#[repr(C, align(16))]`.
* **Size, Alignment and Offsets:**
  * Size: 32 bytes
  * Alignment: 16 bytes
  * `t` offset: 0
  * `f` offset: 16
* **Static Assertion Inventory:** Added static assertions inside `const _: ()` checking size, alignment, and both field offsets.
* **Test Inventory:**
  * `tile128_layout_contracts`: Confirms size, alignment, offsets, and array stride (`[QuadTile128; 2]`).
  * `tile128_plane_preservation`: Confirms `from_planes` retains asymmetric plane inputs.
  * `tile128_reg32_roundtrip_mixed`: Confirms complex deterministic lane patterns roundtrip correctly.
  * `tile128_semantic_stability`: Confirms accessors and mask evaluation logic continue to function optimally after the alignment change.
* **Compatibility Classification:**
  * Core semantic storage layout is fully qualified.
  * No serialized-data roundtrip qualification is claimed.
  * No GPU transport compatibility, byte-cast, Pod, or Zeroable safety is claimed.
* **Explicit Core/Visual Separation:** The core tile is strictly semantic storage. The visual adapter owns its distinct transport layout (e.g. `[u32; 4]`).
* **Exact Verification Results:**
  * `cargo fmt --all --check`: Passed
  * `cargo test -p semantic-core-quad --quiet`: Passed
  * `cargo check -p semantic-core-quad --no-default-features`: Passed
  * `cargo test -p semantic-core-quad --all-features --quiet`: Passed
  * `cargo test -p semantic-core-capsule --quiet`: Passed
  * `cargo test --test legacy_guards --quiet`: Passed
  * `cargo test --all-targets --quiet`: Passed
  * `git diff --check`: Passed
  * `scripts/harness-check.ps1`: Passed
* **Exact Changed-File List:**
  * `.harness/current.task.yaml`
  * `.harness/reports/CORE-QUAD-TILE128-LAYOUT.md`
  * `crates/semantic-core-quad/src/lib.rs`
  * `docs/roadmap/core_quad/quad_tile128_layout_boundary.md`
* **Commit SHA:** Pending
* **PR Number:** Pending

## Explicit Non-Changes
* Did not change mask semantics, lane numbering, or state logic.
* Did not add WGPU, WGSL, `GpuQuadTile128`, byte uploaders, or `bytemuck` support.
